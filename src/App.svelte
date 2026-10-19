<script>
  import { net } from './lib/net.svelte.js';
  import Home from './lib/Home.svelte';
  import Lobby from './lib/Lobby.svelte';
  import Table from './lib/Table.svelte';
</script>

{#if !net.state}
  <Home />
{:else if net.state.phase === 'lobby'}
  <Lobby />
{:else}
  <Table />
{/if}

{#if net.status === 'reconnecting'}
  <div class="banner">連線中斷，重新連線中…</div>
{/if}

<div class="toasts">
  {#each net.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.message}</div>
  {/each}
</div>

<style>
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
