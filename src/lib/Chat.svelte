<script>
  import { net, send } from './net.svelte.js';

  let { showLog = false } = $props();
  let text = $state('');
  let box;

  const items = $derived.by(() => {
    const s = net.state;
    if (!s) return [];
    const chat = s.chat.map((c) => ({ kind: 'chat', ...c }));
    if (!showLog) return chat;
    return [...s.log.map((l) => ({ kind: 'log', text: l })), ...chat.slice(-8)];
  });

  $effect(() => {
    items.length;
    if (box) queueMicrotask(() => (box.scrollTop = box.scrollHeight));
  });

  function submit(e) {
    e.preventDefault();
    if (!text.trim()) return;
    send({ type: 'chat', text });
    text = '';
  }
</script>

<div class="chat">
  <div class="list" bind:this={box}>
    {#each items as it}
      {#if it.kind === 'log'}
        <div class="log">{it.text}</div>
      {:else}
        <div class="msg"><b class:me={it.seat === net.seat}>{it.name}</b>：{it.text}</div>
      {/if}
    {/each}
  </div>
  {#if net.seat != null}
    <form onsubmit={submit}>
      <input bind:value={text} placeholder="說點什麼…" maxlength="300" />
    </form>
  {/if}
</div>

<style>
  .chat { display: flex; flex-direction: column; min-height: 0; flex: 1; }
  .list { flex: 1; overflow-y: auto; font-size: 13px; line-height: 1.55; padding-right: 4px; }
  .log { color: var(--muted); }
  .msg b { color: var(--accent); font-weight: 600; }
  .msg b.me { color: var(--gold); }
  form { margin-top: 8px; }
  input { width: 100%; }
</style>
