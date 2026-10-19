<script>
  import { net, clock } from './lib/net.svelte.js';
  import SoundControl from './lib/SoundControl.svelte';
  import { onGameState, onCountdown } from './lib/audio/sound.svelte.js';
  import Home from './lib/Home.svelte';
  import Lobby from './lib/Lobby.svelte';
  import Table from './lib/Table.svelte';
  import Gallery from './lib/Gallery.svelte';
  import ArtStylePicker from './lib/ArtStylePicker.svelte';
  import { processEvents, resetReactions } from './lib/reactions.svelte.js';

  $effect(() => {
    if (net.state) processEvents(net.state);
    else resetReactions();
  });

  let hash = $state(location.hash);
  window.addEventListener('hashchange', () => (hash = location.hash));

  // 音樂與音效跟著牌局走
  $effect(() => onGameState(net.state));
  $effect(() => {
    const s = net.state;
    onCountdown(s, s?.deadline_ms == null ? null : s.deadline_ms - (clock.now - net.receivedAt));
  });
</script>

<div class="app-shell">
  <div class="topbar">
    <ArtStylePicker />
    <SoundControl />
  </div>
  <div class="screen">
    {#if hash === '#gallery'}
      <Gallery />
    {:else if !net.state}
      <Home />
    {:else if net.state.phase === 'lobby'}
      <Lobby />
    {:else}
      <Table />
    {/if}
  </div>
</div>

{#if net.status === 'reconnecting'}
  <div class="banner">連線中斷，重新連線中…</div>
{/if}

<div class="toasts">
  {#each net.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.message}</div>
  {/each}
</div>

<style>
  .topbar { display: flex; flex-wrap: wrap; }
  .topbar > :global(.style-picker) { flex: 1; }
  .app-shell { height: 100%; display: grid; grid-template-rows: auto minmax(0, 1fr); }
  .screen { min-width: 0; min-height: 0; overflow: hidden; }
  .banner {
    position: fixed;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    background: var(--red);
    padding: 6px 16px;
    border-radius: 0 0 10px 10px;
    z-index: 100;
  }
  .toasts {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 100;
    pointer-events: none;
  }
  .toast {
    background: rgba(20, 20, 20, 0.92);
    border-left: 4px solid var(--red);
    padding: 10px 16px;
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.4);
  }
  .toast.info { border-left-color: var(--accent); }
</style>
