<script>
  import { onDestroy } from 'svelte';
  let { compact = false } = $props();
  let trigger;
  let timer;
  let startPoint;
  function cancelHold() { clearTimeout(timer); startPoint = null; }
  function openSecret() {
    cancelHold();
    window.dispatchEvent(new CustomEvent('mahjong-open-cheat', { detail: { trigger } }));
  }
  function startHold(event) {
    if (event.button !== 0) return;
    cancelHold();
    startPoint = [event.clientX, event.clientY];
    timer = setTimeout(openSecret, 1500);
  }
  function move(event) {
    if (startPoint && Math.hypot(event.clientX - startPoint[0], event.clientY - startPoint[1]) > 12) cancelHold();
  }
  onDestroy(cancelHold);
</script>

<button type="button" class="brand" class:compact bind:this={trigger}
  aria-label="虛擬台麻館：長按開啟密技輸入"
  onpointerdown={startHold} onpointermove={move} onpointerup={cancelHold}
  onpointercancel={cancelHold} onpointerleave={cancelHold} onblur={cancelHold}
  oncontextmenu={(event) => event.preventDefault()}
  onclick={(event) => { if (event.detail === 0) openSecret(); }}>
  <div class="brand-name">虛擬台麻館<span aria-hidden="true" class="brand-tile">發</span></div>
  <div class="brand-english" lang="en">Virtual Taiwan Mahjong Parlor</div>
</button>

<style>
  .brand { display: inline-flex; flex-direction: column; align-items: center; color: #ffe4a1; filter: drop-shadow(0 3px 2px #171008); }
  .brand-name { display: flex; align-items: center; gap: .14em; font-family: 'DFKai-SB', '標楷體', 'Kaiti TC', serif; font-weight: 900; font-size: clamp(42px, 5.3vw, 82px); line-height: 1.25; letter-spacing: .025em; text-shadow: 1px 2px #87501b, -1px -1px #fff4d3, 0 4px 12px #000; white-space: nowrap; }
  .brand-tile { display: inline-grid; place-items: center; width: .68em; height: .92em; font-size: .62em; background: #fff1cd; color: #17664b; border: 2px solid #bc8c3d; border-radius: .08em; transform: rotate(12deg); text-shadow: none; box-shadow: 3px 3px #8c6b31; }
  .brand-english { font: 600 clamp(11px, 1.22vw, 19px)/1.6 Georgia, serif; letter-spacing: .055em; text-align: center; }
  .compact .brand-name { font-size: clamp(28px, 3vw, 44px); }
  .compact .brand-english { font-size: clamp(10px, 1vw, 13px); }
</style>
