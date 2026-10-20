<script>
  import { onMount } from 'svelte';
  import { net, clock, disconnect } from './lib/net.svelte.js';
  import SoundControl from './lib/SoundControl.svelte';
  import { onGameState, onCountdown } from './lib/audio/sound.svelte.js';
  import Home from './lib/Home.svelte';
  import Lobby from './lib/Lobby.svelte';
  import Table from './lib/Table.svelte';
  import Gallery from './lib/Gallery.svelte';
  import Settings from './lib/Settings.svelte';
  import Controls from './lib/Controls.svelte';
  import { resolvePage } from './lib/menu.js';
  import ArtStylePicker from './lib/ArtStylePicker.svelte';
  import Brand from './lib/Brand.svelte';
  import { artStyle } from './lib/art-style.js';
  import { processEvents, resetReactions } from './lib/reactions.svelte.js';

  $effect(() => {
    if (net.state) processEvents(net.state);
    else resetReactions();
  });

  $effect(() => {
    document.documentElement.dataset.uiStyle = $artStyle;
  });

  let hash = $state(location.hash);
  const page = $derived(resolvePage(hash, net.state));
  const inGame = $derived(page === 'lobby' || page === 'table');
  onMount(() => {
    const syncHash = () => (hash = location.hash);
    window.addEventListener('hashchange', syncHash);
    return () => window.removeEventListener('hashchange', syncHash);
  });

  // 音樂與音效跟著牌局走
  $effect(() => onGameState(net.state));
  $effect(() => {
    const s = net.state;
    onCountdown(s, s?.deadline_ms == null ? null : s.deadline_ms - (clock.now - net.receivedAt));
  });
</script>

<div class="app-shell" class:menu-shell={!inGame}>
  <div class="topbar">
    {#if page === 'table'}<div class="topbar-brand"><Brand compact /></div>{/if}
    <ArtStylePicker />
    <SoundControl />
  </div>
  <div class="screen">
    {#if page === 'lobby'}
      <Lobby />
    {:else if page === 'table'}
      <Table />
    {:else if page === 'gallery'}
      <Gallery />
    {:else if page === 'settings'}
      <Settings />
    {:else}
      <Home {page} />
    {/if}
  </div>
</div>

<Controls state={net.state} onback={() => {
  if (!net.state) {
    if (net.status === 'connecting' || net.status === 'open') disconnect();
    location.hash = '#title';
  }
}} />

{#if net.status === 'reconnecting'}
  <div class="banner">連線中斷，重新連線中…</div>
{/if}

<div class="toasts">
  {#each net.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.message}</div>
  {/each}
</div>

<style>
  .topbar { display: flex; flex-wrap: wrap; align-items: center; }
  .topbar-brand { display: flex; align-items: center; padding: 2px 4px 2px 14px; }
  /* 手機對局時把上方工具列收起來，空間留給牌桌（畫風、音量在「遊戲設定」可以調） */
  @media (max-width: 760px), (max-height: 520px) {
    .app-shell:not(.menu-shell) .topbar { display: none; }
    .app-shell:not(.menu-shell) { grid-template-rows: minmax(0, 1fr); }
  }
  .topbar > :global(.style-picker) { flex: 1; }
  .app-shell { height: 100%; display: grid; grid-template-rows: auto minmax(0, 1fr); }
  .menu-shell { grid-template-rows: minmax(0, 1fr) auto; }
  .menu-shell .screen { grid-row: 1; }
  .menu-shell .topbar { grid-row: 2; }
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
