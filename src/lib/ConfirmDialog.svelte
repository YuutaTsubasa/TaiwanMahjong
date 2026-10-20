<script>
  let dialog;
  let trigger;
  let pendingAction;
  let message = $state('');
  let confirmLabel = $state('確定');

  // All input methods use these same real buttons. Closing (including Escape or
  // gamepad B through Controls) cancels; only the explicit confirm button acts.
  export function ask(options) {
    if (dialog.open) return;
    trigger = document.activeElement;
    message = options.message;
    confirmLabel = options.confirmLabel ?? '確定';
    pendingAction = options.onconfirm;
    dialog.showModal();
    dialog.querySelector('[data-cancel]')?.focus();
  }
  function confirmAction() {
    const action = pendingAction;
    pendingAction = null;
    dialog.close('confirmed');
    action?.();
  }
  function closed() {
    pendingAction = null;
    if (trigger?.isConnected) trigger.focus({ preventScroll: true });
    trigger = null;
  }
</script>

<dialog bind:this={dialog} class="confirmation" aria-labelledby="game-confirm-title" aria-describedby="game-confirm-message" onclose={closed}>
  <h2 id="game-confirm-title">確認操作</h2>
  <p id="game-confirm-message">{message}</p>
  <div class="buttons">
    <button type="button" data-cancel onclick={() => dialog.close('cancel')}>取消</button>
    <button type="button" class="danger" onclick={confirmAction}>{confirmLabel}</button>
  </div>
</dialog>

<style>
  .confirmation { width: min(420px, calc(100vw - 32px)); padding: 24px; color: var(--text); background: var(--panel-solid); border: 2px solid var(--gold); border-radius: 14px; box-shadow: 0 16px 60px #0009; }
  .confirmation::backdrop { background: #000b; }
  h2 { margin: 0 0 16px; font-size: 21px; color: var(--gold); }
  p { line-height: 1.7; margin: 0 0 24px; }
  .buttons { display: flex; justify-content: flex-end; flex-wrap: wrap; gap: 12px; }
  button { min-height: 44px; min-width: 90px; }
  button:focus-visible { outline: 3px solid var(--gold); outline-offset: 3px; }
</style>
