<script>
  import { onMount } from 'svelte';
  import { ayiUnlocked, isWinningMatch, unlockAyi } from './unlocks.js';
  import { createCheatTracker, gamepadCommands, isEditableTarget, keyToControl, chooseDirectionalTarget, steppedValue, setControlValue, nextSelectIndex } from './input-controls.js';
  import ControlsKeyboard from './ControlsKeyboard.svelte';

  let { state: gameState = null, onback = () => {} } = $props();
  let pad;
  let keyboard;
  let returnFocus;
  let notice = $state('');
  let entered = $state('');
  let noticeTimer;
  const cheat = createCheatTracker();
  const symbols = { ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→', B: 'B', A: 'A' };

  function announceUnlock() {
    if (unlockAyi()) {
      notice = '隱藏角色「阿翼」已解鎖！可在角色選擇與角色圖鑑中查看。';
      clearTimeout(noticeTimer);
      noticeTimer = setTimeout(() => (notice = ''), 6500);
    }
  }

  $effect(() => {
    if (isWinningMatch(gameState)) announceUnlock();
    if (gameState) {
      cheat.reset();
      if (pad?.open) pad.close();
    }
  });

  function feedSecret(token) {
    if (gameState) return false;
    const complete = cheat.feed(token);
    if (pad?.open) entered = complete ? '秘技成功！' : Array.from({ length: cheat.progress }, (_, i) => symbols[['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'B', 'A'][i]]).join(' ');
    if (complete) announceUnlock();
    return complete;
  }

  function openPad(event) {
    if (gameState || pad?.open) return;
    returnFocus = event?.detail?.trigger ?? document.activeElement;
    cheat.reset();
    entered = '';
    pad.showModal();
  }

  function restoreFocus() {
    cheat.reset();
    if (returnFocus?.isConnected) returnFocus.focus();
  }

  function visible(element) {
    const closedDetails = element.closest('details:not([open])');
    if (closedDetails && !closedDetails.querySelector(':scope > summary')?.contains(element)) return false;
    return element.getClientRects().length > 0 &&
      !element.closest('[hidden], [inert], [aria-hidden="true"]') &&
      getComputedStyle(element).visibility !== 'hidden';
  }

  // Buttons and links share their existing click handlers: pointer/touch, native Enter/Space,
  // and gamepad A all follow the same action path. No alternate discard shortcut is used.
  function controls() {
    const dialogs = [...document.querySelectorAll('dialog[open]')].filter(visible);
    const overlays = [...document.querySelectorAll('[role="dialog"], [aria-modal="true"], .overlay')].filter(visible);
    // Native modal dialogs take precedence over page overlays regardless of DOM order.
    // The regular scope spans the whole app so focus can leave the audio/style footer.
    const scope = dialogs.at(-1) ?? overlays.at(-1) ?? document.querySelector('.app-shell') ?? document;
    return [...scope.querySelectorAll('button, a[href], summary, select, textarea, input:not([type="hidden"]), [role="button"][tabindex="0"]')]
      .filter(element => !element.matches(':disabled, [disabled], [aria-disabled="true"], [tabindex="-1"], [data-gamepad-skip]') && visible(element));
  }

  function moveFocus(direction) {
    let candidates = controls();
    const active = document.activeElement;
    // Horizontal movement stays inside the player's hand; up/down can reach action buttons.
    if (active?.matches('[data-hand-tile]') && (direction === 'ArrowLeft' || direction === 'ArrowRight')) {
      candidates = candidates.filter(element => element.matches('[data-hand-tile]'));
    }
    const next = chooseDirectionalTarget(candidates.map(element => element.getBoundingClientRect()), candidates.indexOf(active), direction);
    if (next < 0) return false;
    candidates[next].focus({ preventScroll: true });
    candidates[next].scrollIntoView({ block: 'nearest', inline: 'nearest' });
    return true;
  }

  function back() {
    const dialog = [...document.querySelectorAll('dialog[open]')].filter(visible).at(-1);
    if (dialog) { dialog.close('cancel'); return; }
    if (gameState) window.dispatchEvent(new CustomEvent('mahjong-cancel-selection'));
    else onback();
  }

  function keydown(event) {
    if (event.defaultPrevented || isEditableTarget(event.target) || event.isComposing) {
      cheat.reset();
      return;
    }
    if (event.key === 'Escape') {
      if (!event.repeat) back();
      event.preventDefault();
      return;
    }
    const token = keyToControl(event);
    if (document.querySelector('dialog[open]')) {
      if (token?.startsWith('Arrow') && moveFocus(token)) event.preventDefault();
      // Tab and native Enter/Space let keyboard users operate the same on-screen pad.
      return;
    }
    if (!token) {
      if (!['Shift', 'Control', 'Alt', 'Meta'].includes(event.key)) cheat.reset();
      return;
    }
    if (!event.repeat && feedSecret(token)) { event.preventDefault(); return; }
    if (token.startsWith('Arrow') && moveFocus(token)) event.preventDefault();
    // Keyboard A/B enter the menu code only; Enter and Space keep native button semantics.
  }

  function gamepadCommand({ token, repeat }) {
    const active = document.activeElement;
    const editing = isEditableTarget(active);
    const modal = !!document.querySelector('dialog[open]');
    if (editing || keyboard?.isOpen()) cheat.reset();
    // Inside the touch pad, the gamepad navigates its real buttons; B closes the dialog.
    // Outside it, the raw D-pad / B / A sequence can unlock directly from the menu.
    if (!modal && !editing && !repeat && feedSecret(token)) return;
    // The code's penultimate B must not leave the menu before the final A.
    if (!modal && !gameState && token === 'B' && cheat.progress === 9) return;
    if (token.startsWith('Arrow')) {
      const horizontal = token === 'ArrowLeft' || token === 'ArrowRight';
      const direction = token === 'ArrowLeft' ? -1 : 1;
      if (horizontal && active?.matches('input[type="range"], input[type="number"]') && !active.disabled && !active.readOnly) {
        setControlValue(active, steppedValue(active, direction));
      } else if (horizontal && active?.matches('select') && !active.disabled) {
        const next = nextSelectIndex(active.options, active.selectedIndex, direction);
        if (next >= 0) setControlValue(active, active.options[next].value);
      } else moveFocus(token);
    }
    else if (token === 'B') back();
    else if (token === 'A') {
      const candidates = controls();
      if (!candidates.includes(active)) { moveFocus('ArrowDown'); return; }
      if (active.matches('textarea, input:not([type="range"]):not([type="checkbox"]):not([type="radio"]):not([type="button"]):not([type="submit"]):not([type="reset"])')) {
        keyboard.openFor(active);
      } else if (active.matches('select')) {
        const next = nextSelectIndex(active.options, active.selectedIndex, 1);
        if (next >= 0) setControlValue(active, active.options[next].value);
      } else if (!active.matches('input[type="range"]')) active.click();
    }
  }

  onMount(() => {
    let frame;
    const snapshots = new Map();
    let disposed = false;
    function poll(now) {
      if (disposed) return;
      let pads = [];
      try { pads = navigator.getGamepads?.() ?? []; } catch {}
      const connected = new Set();
      for (const gamepad of pads) {
        if (!gamepad?.connected || (gamepad.mapping && gamepad.mapping !== 'standard')) continue;
        connected.add(gamepad.index);
        const result = gamepadCommands(gamepad, snapshots.get(gamepad.index), now);
        snapshots.set(gamepad.index, result.snapshot);
        if (!document.hidden) result.commands.forEach(gamepadCommand);
      }
      for (const index of snapshots.keys()) if (!connected.has(index)) snapshots.delete(index);
      frame = requestAnimationFrame(poll);
    }
    window.addEventListener('keydown', keydown);
    window.addEventListener('mahjong-open-cheat', openPad);
    frame = requestAnimationFrame(poll);
    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
      clearTimeout(noticeTimer);
      window.removeEventListener('keydown', keydown);
      window.removeEventListener('mahjong-open-cheat', openPad);
    };
  });
</script>

<dialog bind:this={pad} class="secret-pad" aria-labelledby="secret-title" onclose={restoreFocus}>
  <div class="pad-header">
    <h2 id="secret-title">復古秘技手把</h2>
    <button type="button" aria-label="關閉秘技手把" onclick={() => pad.close()}>✕</button>
  </div>
  <p>點選方向與 B、A 按鈕，輸入你記得的那串秘技。</p>
  <div class="pad-buttons" role="group" aria-label="秘技輸入">
    <button type="button" class="up" aria-label="上" onclick={() => feedSecret('ArrowUp')}>↑</button>
    <button type="button" class="left" aria-label="左" onclick={() => feedSecret('ArrowLeft')}>←</button>
    <button type="button" class="down" aria-label="下" onclick={() => feedSecret('ArrowDown')}>↓</button>
    <button type="button" class="right" aria-label="右" onclick={() => feedSecret('ArrowRight')}>→</button>
    <button type="button" class="b" aria-label="B" onclick={() => feedSecret('B')}>B</button>
    <button type="button" class="a" aria-label="A" onclick={() => feedSecret('A')}>A</button>
  </div>
  <output class="entered" aria-live="polite">{entered || '等待輸入…'}</output>
  {#if $ayiUnlocked}<p class="unlocked">阿翼已解鎖，歡迎來到雀館！</p>{/if}
</dialog>
<ControlsKeyboard bind:this={keyboard} />
<div class="unlock-notice" class:shown={!!notice} role="status" aria-live="polite">{notice}</div>

<style>
  .secret-pad {
    color: var(--text); background: var(--panel-solid); border: 2px solid var(--gold);
    border-radius: 16px; padding: 24px; width: min(420px, calc(100vw - 32px));
    max-height: calc(100dvh - 32px); overflow: auto; box-shadow: 0 16px 60px #0009;
  }
  .secret-pad::backdrop { background: #000b; }
  .pad-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  h2 { font-size: 20px; margin: 0; }
  p { color: var(--muted); line-height: 1.6; font-size: 14px; }
  .pad-buttons { display: grid; grid-template-columns: repeat(5, 1fr); grid-template-rows: repeat(2, 52px); gap: 8px; margin: 24px 0; }
  .pad-buttons button { padding: 0; font-size: 24px; touch-action: manipulation; }
  .up { grid-column: 2; grid-row: 1; }
  .left { grid-column: 1; grid-row: 2; }
  .down { grid-column: 2; grid-row: 2; }
  .right { grid-column: 3; grid-row: 2; }
  .b { grid-column: 4; grid-row: 2; }
  .a { grid-column: 5; grid-row: 1; }
  .entered { display: block; min-height: 28px; letter-spacing: 2px; color: var(--gold); text-align: center; }
  p.unlocked { color: var(--accent); text-align: center; }
  button:focus-visible { outline: 3px solid var(--gold); outline-offset: 3px; }
  .unlock-notice {
    position: fixed; left: 50%; bottom: 24px; transform: translateX(-50%); z-index: 1000;
    width: max-content; max-width: calc(100vw - 32px); pointer-events: none; opacity: 0;
    padding: 14px 20px; border-radius: 12px; border: 1px solid var(--gold);
    background: var(--panel-solid); color: var(--gold); box-shadow: 0 8px 30px #0006;
  }
  .unlock-notice.shown { opacity: 1; }
</style>
