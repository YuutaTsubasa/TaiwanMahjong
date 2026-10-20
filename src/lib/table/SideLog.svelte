<script>
  // 對局右側：牌桌紀錄（每行帶出手者頭像、牌名標紅）＋聊天＋離開按鈕
  import { net, send } from '../net.svelte.js';
  import Avatar from '../Avatar.svelte';

  let { onclose, onleave, onend } = $props();

  const s = $derived(net.state);
  let text = $state('');
  let logBox;
  let chatBox;

  // 依名字找出是誰（取最長的符合者，避免「小美」配到「小美美」之類）
  function seatOf(line) {
    let best = null;
    for (const seat of s.seats) {
      if (seat.name && line.startsWith(seat.name) && (!best || seat.name.length > best.name.length)) best = seat;
    }
    return best;
  }

  const ACTION = /^(打出|碰|槓|吃|加槓|補花|要加槓)\s+(.+?)(（.*）)?$/;

  const logItems = $derived(
    (s?.log ?? []).slice(-40).map((line) => {
      if (line.startsWith('──')) return { kind: 'sep', text: line.replace(/─/g, '').trim() };
      const seat = seatOf(line);
      const rest = seat ? line.slice(seat.name.length).trim() : line;
      const m = seat && ACTION.exec(rest);
      return m
        ? { kind: 'act', seat, verb: m[1], tile: m[2], note: m[3] ?? '' }
        : { kind: 'text', seat, text: seat ? rest : line };
    }),
  );

  const chatItems = $derived((s?.chat ?? []).slice(-30));

  $effect(() => {
    logItems.length;
    if (logBox) queueMicrotask(() => (logBox.scrollTop = logBox.scrollHeight));
  });
  $effect(() => {
    chatItems.length;
    if (chatBox) queueMicrotask(() => (chatBox.scrollTop = chatBox.scrollHeight));
  });

  function submit(e) {
    e.preventDefault();
    if (!text.trim()) return;
    send({ type: 'chat', text });
    text = '';
  }
</script>

<aside class="sidelog">
  <section class="log-sec">
    <header>
      <h2>牌桌紀錄</h2>
      <button type="button" class="small" onclick={onclose}>收起 ›</button>
    </header>
    <ol class="log" bind:this={logBox}>
      {#each logItems as it}
        {#if it.kind === 'sep'}
          <li class="sep">{it.text}</li>
        {:else}
          <li class="entry">
            {#if it.seat?.character}
              <span class="av"><Avatar id={it.seat.character} size={34} crop /></span>
            {:else}
              <span class="av sys">●</span>
            {/if}
            {#if it.kind === 'act'}
              <span class="who">{it.seat.name}</span>
              <span class="verb">{it.verb}</span>
              <b class="tname">{it.tile}</b>{#if it.note}<small>{it.note}</small>{/if}
            {:else}
              {#if it.seat}<span class="who">{it.seat.name}</span>{/if}
              <span class="txt">{it.text}</span>
            {/if}
          </li>
        {/if}
      {/each}
    </ol>
  </section>

  <section class="chat-sec">
    <header><h2>聊天</h2></header>
    <ol class="chat" bind:this={chatBox}>
      {#each chatItems as c}
        {@const seat = c.seat != null ? s.seats[c.seat] : null}
        <li class="entry" class:mine={c.seat === net.seat}>
          {#if seat?.character}
            <span class="av"><Avatar id={seat.character} size={30} crop /></span>
          {:else}
            <span class="av sys">●</span>
          {/if}
          <span class="who">{c.name}</span>：<span class="txt">{c.text}</span>
        </li>
      {:else}
        <li class="empty">還沒有人說話</li>
      {/each}
    </ol>
    {#if net.seat != null}
      <form onsubmit={submit}>
        <input bind:value={text} placeholder="說點什麼…" maxlength="300" />
        <button type="submit" disabled={!text.trim()} aria-label="送出">➤</button>
      </form>
    {/if}
  </section>

  <div class="leave">
    {#if onend}
      <button type="button" class="end" onclick={onend}>結束牌局</button>
    {/if}
    <button type="button" class="exit" onclick={onleave}>⇦ 離開房局</button>
  </div>
</aside>

<style>
  .sidelog {
    display: grid;
    grid-template-rows: minmax(0, 1.25fr) minmax(0, 1fr) auto;
    /* 欄寬固定為面板寬度，不讓較長的紀錄把按鈕撐出框外 */
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
    min-height: 0;
    height: 100%;
    padding: 12px;
    background: var(--art-panel);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
    box-shadow: inset 0 0 0 4px #0007, inset 0 0 0 5px color-mix(in srgb, var(--art-line) 50%, transparent), 0 8px 26px #0008;
  }
  section { display: flex; flex-direction: column; min-height: 0; }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-bottom: 6px;
    margin-bottom: 6px;
    border-bottom: 1px solid var(--art-line);
  }
  h2 {
    margin: 0;
    font-family: var(--heading-font);
    font-size: 21px;
    letter-spacing: 0.14em;
    color: var(--gold);
  }
  h2::before { content: '❖ '; font-size: 0.75em; vertical-align: 2px; }
  ol { list-style: none; margin: 0; padding: 0 4px 0 0; overflow-y: auto; flex: 1; min-height: 0; }
  .entry {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 5px 2px;
    border-bottom: 1px solid color-mix(in srgb, var(--art-line) 28%, transparent);
    font-size: 14px;
    line-height: 1.4;
  }
  .av { flex: none; display: grid; place-items: center; width: 34px; }
  .av.sys { color: var(--muted); font-size: 10px; }
  .who { color: var(--text); font-weight: 600; white-space: nowrap; }
  .verb { color: var(--muted); }
  .tname { color: #ff6b6b; font-family: var(--heading-font); font-size: 16px; }
  .txt { color: var(--muted); overflow-wrap: anywhere; }
  .sep {
    text-align: center;
    color: var(--gold);
    font-size: 12px;
    padding: 6px 0 4px;
    letter-spacing: 0.06em;
  }
  .chat .entry { align-items: flex-start; }
  .chat .txt { color: var(--text); }
  .chat .mine .who { color: var(--gold); }
  .empty { color: var(--muted); font-size: 13px; padding: 8px 2px; }
  form { display: flex; gap: 6px; margin-top: 8px; }
  form input { flex: 1; min-width: 0; }
  form button { flex: none; width: 46px; padding: 0; font-size: 18px; }
  .leave { display: flex; flex-direction: column; gap: 8px; }
  .exit, .end {
    width: 100%;
    padding: 10px;
    font-family: var(--heading-font);
    font-size: 20px;
    letter-spacing: 0.18em;
  }
  .exit {
    color: #ffe9c7;
    background: linear-gradient(#7a1c22, #4a0f14);
    border: 1px solid var(--art-line);
    box-shadow: inset 0 0 0 3px #2a080b, inset 0 0 0 4px color-mix(in srgb, var(--art-line) 70%, transparent), 0 4px 10px #0008;
  }
  .end { font-size: 15px; padding: 6px; }
</style>
