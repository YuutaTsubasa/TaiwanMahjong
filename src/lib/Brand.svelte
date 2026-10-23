<script>
  import { onDestroy } from 'svelte';
  import { artStyle } from './art-style.js';
  import { asset } from './site.js';
  import { t } from './i18n.svelte.js';
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
  aria-label={t('虛擬台麻館：長按開啟密技輸入')}
  onpointerdown={startHold} onpointermove={move} onpointerup={cancelHold}
  onpointercancel={cancelHold} onpointerleave={cancelHold} onblur={cancelHold}
  oncontextmenu={(event) => event.preventDefault()}
  onclick={(event) => { if (event.detail === 0) openSecret(); }}>
  <!-- 書法標誌（public/ui/<畫風>/logo.webp，1400×420 透明 WebP） -->
  <img class="logo" src={asset(`/ui/${$artStyle}/logo.webp`)} alt={t('虛擬台麻館 Virtual Taiwan Mahjong Parlor')} draggable="false" />
</button>

<style>
  .brand { display: inline-flex; align-items: center; filter: drop-shadow(0 4px 6px #000a); }
  .logo { display: block; width: clamp(320px, 36vw, 640px); height: auto; aspect-ratio: 1400 / 420; pointer-events: none; }
  .compact .logo { width: clamp(170px, 15vw, 260px); }
</style>
