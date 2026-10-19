<script>
  import { net, hostCmd, leave, disconnect, serverUrls, toast } from './net.svelte.js';
  import { SEAT_KIND_ICON, SEAT_KIND_LABEL } from './tiles.js';
  import Chat from './Chat.svelte';

  const s = $derived(net.state);
  const urls = $derived(serverUrls());
  let urlIdx = $state(0);
  const base = $derived(urls[Math.min(urlIdx, urls.length - 1)] || 'http://localhost:7777');

  let tab = $state('claude');
  let settings = $state(null);
  $effect(() => {
    if (s && !settings) settings = { ...s.settings };
  });

  function applySettings() {
    hostCmd('settings', { settings: Object.fromEntries(Object.entries(settings).map(([k, v]) => [k, Number(v)])) });
    toast('設定已更新', 'info');
  }

  async function copy(t) {
    try {
      await navigator.clipboard.writeText(t);
      toast('已複製', 'info');
    } catch {
      toast('複製失敗，請手動選取');
    }
  }

  const claudeCmd = $derived(`claude mcp add --transport http mahjong ${base}/mcp`);
  const claudePrompt = '用 mahjong MCP 工具，以「Claude」的名字加入台灣麻將遊戲。之後不斷重複：呼叫 wait_for_turn，輪到你時用 take_action 選擇最好的 action_id，直到遊戲結束。打牌要快，每次決定都有時間限制。可以用 send_chat 跟大家聊天。';
  const curlPrompt = $derived(`請讀 ${base}/llms.txt 的說明，用 curl 以「AI」的名字加入台灣十六張麻將遊戲，然後一直玩到遊戲結束。`);
  const tunnelCmd = $derived(`cloudflared tunnel --url ${base.replace(/\/\/[^:]+:/, '//localhost:')}`);

  const fields = [
    ['base', '底', 10],
    ['per_tai', '每台', 10],
    ['rounds', '圈數（1=東風圈 4=一將）', 1],
    ['turn_timeout', '出牌秒數（0=不限）', 5],
    ['claim_timeout', '吃碰胡秒數（0=不限）', 5],
    ['next_hand_delay', '下一局自動開始秒數', 5],
    ['bot_delay_ms', '電腦思考毫秒', 100],
  ];
</script>

<main>
  <header>
    <h1>🀄 房間大廳</h1>
    <div class="spacer"></div>
    {#if net.seat != null}
      <button class="small" onclick={leave}>離開座位</button>
    {/if}
    <button class="small" onclick={disconnect}>回首頁</button>
  </header>

  <div class="layout">
    <section class="card seats">
      <h2>座位</h2>
      <div class="seat-grid">
        {#each s.seats as seat, i}
          <div class="seat" class:me={i === net.seat} class:empty={!seat.name}>
            <div class="idx">座位 {i}</div>
            {#if seat.name}
              <div class="who">
                <span class="icon">{SEAT_KIND_ICON[seat.kind]}</span>
                <span class="nm">{seat.name}</span>
                {#if i === net.seat}<span class="badge">你</span>{/if}
              </div>
              <div class="meta">
                <span class="dot" class:on={seat.connected}></span>
                {SEAT_KIND_LABEL[seat.kind]} · {seat.connected ? '在線' : '離線'}
              </div>
              {#if net.isHost}
                <button class="small" onclick={() => hostCmd('remove', { seat: i })}>移除</button>
              {/if}
            {:else}
              <div class="who muted">空位</div>
              {#if net.isHost}
                <button class="small" onclick={() => hostCmd('add_bot', { seat: i })}>＋ 電腦</button>
              {/if}
            {/if}
          </div>
        {/each}
      </div>

      {#if net.isHost}
        <div class="start">
          <button class="primary big" onclick={() => hostCmd('start')}>開始遊戲</button>
          <span class="muted">空位會自動補上電腦。座位隨時可讓 AI / 玩家接手電腦。</span>
        </div>
      {:else}
        <p class="muted wait">等待房主開始遊戲…</p>
      {/if}

      <h2 class="sub">規則設定</h2>
      {#if settings}
        <div class="settings">
          {#each fields as [k, label, step]}
            <label>
              <span>{label}</span>
              <input type="number" min="0" {step} bind:value={settings[k]} disabled={!net.isHost} />
            </label>
          {/each}
        </div>
        {#if net.isHost}
          <button class="small" onclick={applySettings}>套用設定</button>
        {/if}
      {/if}
    </section>

    <section class="card invite">
      <h2>邀請玩家與 AI</h2>
      {#if urls.length > 1}
        <div class="urls">
          {#each urls as u, i}
            <button class="small" class:sel={i === urlIdx} onclick={() => (urlIdx = i)}>{u}</button>
          {/each}
        </div>
      {/if}
      <div class="tabs">
        <button class:sel={tab === 'human'} onclick={() => (tab = 'human')}>👤 朋友</button>
        <button class:sel={tab === 'claude'} onclick={() => (tab = 'claude')}>🤖 Claude Code</button>
        <button class:sel={tab === 'chatgpt'} onclick={() => (tab = 'chatgpt')}>💬 ChatGPT</button>
        <button class:sel={tab === 'other'} onclick={() => (tab = 'other')}>🛠 其他 AI</button>
      </div>

      {#if tab === 'human'}
        <p>朋友可以用本 App 選「加入房間」輸入：</p>
        <div class="copy"><code>{base.replace('http://', '')}</code><button class="small" onclick={() => copy(base.replace('http://', ''))}>複製</button></div>
        <p>沒有 App 的話，直接用瀏覽器打開：</p>
        <div class="copy"><code>{base}/</code><button class="small" onclick={() => copy(base + '/')}>複製</button></div>
        <p class="muted">需在同一個區網，或房主的防火牆 / 路由器有開放這個埠。</p>
      {:else if tab === 'claude'}
        <p>1. 在終端機把麻將伺服器加成 MCP：</p>
        <div class="copy"><code>{claudeCmd}</code><button class="small" onclick={() => copy(claudeCmd)}>複製</button></div>
        <p>2. <b>開一個新的</b> <code>claude</code> 工作階段（MCP 工具只在啟動時載入），貼上這段話：</p>
        <div class="copy"><code class="wrap">{claudePrompt}</code><button class="small" onclick={() => copy(claudePrompt)}>複製</button></div>
        <p class="muted">Claude 會用 join_game → wait_for_turn → take_action 自己打。就算沒載入 MCP，Claude 也會自己讀 /llms.txt 改用 HTTP API 玩。</p>
      {:else if tab === 'chatgpt'}
        <p>ChatGPT 在雲端，需要一個 <b>https 公開網址</b> 才連得到你的電腦。最簡單是用 Cloudflare 臨時通道：</p>
        <div class="copy"><code>{tunnelCmd}</code><button class="small" onclick={() => copy(tunnelCmd)}>複製</button></div>
        <p>它會給你一個 <code>https://xxxx.trycloudflare.com</code>，接著擇一：</p>
        <ul>
          <li><b>MCP 連接器</b>：ChatGPT 設定 → 應用程式與連接器 → 進階 → 開啟開發者模式 → 建立連接器，網址填 <code>https://xxxx.trycloudflare.com/mcp</code>，驗證選「無」。</li>
          <li><b>自訂 GPT Actions</b>：建立 GPT → 動作 → 從網址匯入 <code>https://xxxx.trycloudflare.com/openapi.json</code>。</li>
        </ul>
        <p>然後對 ChatGPT 說：</p>
        <div class="copy"><code class="wrap">{claudePrompt.replace('Claude', 'ChatGPT').replace('mahjong MCP 工具', '麻將工具')}</code><button class="small" onclick={() => copy(claudePrompt.replace('Claude', 'ChatGPT').replace('mahjong MCP 工具', '麻將工具'))}>複製</button></div>
      {:else}
        <p>任何能發 HTTP 請求的 AI 或程式都能玩。說明文件：</p>
        <div class="copy"><code>{base}/llms.txt</code><button class="small" onclick={() => copy(base + '/llms.txt')}>複製</button></div>
        <p>可以直接給 AI 這段話：</p>
        <div class="copy"><code class="wrap">{curlPrompt}</code><button class="small" onclick={() => copy(curlPrompt)}>複製</button></div>
        <p class="muted">MCP 端點 <code>{base}/mcp</code> · OpenAPI <code>{base}/openapi.json</code></p>
      {/if}

      <h2 class="sub">聊天</h2>
      <div class="chatbox"><Chat /></div>
    </section>
  </div>
</main>

<style>
  main { height: 100%; display: flex; flex-direction: column; padding: 16px 24px; gap: 14px; overflow: auto; }
  header { display: flex; align-items: center; gap: 10px; }
  header h1 { margin: 0; font-size: 24px; }
  .spacer { flex: 1; }
  .layout { display: grid; grid-template-columns: 1.1fr 1fr; gap: 18px; min-height: 0; }
  @media (max-width: 980px) { .layout { grid-template-columns: 1fr; } }
  h2 { margin: 0 0 12px; font-size: 18px; }
  h2.sub { margin-top: 22px; }
  .seat-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; }
  .seat {
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 12px 14px;
    background: rgba(255, 255, 255, 0.04);
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: flex-start;
  }
  .seat.me { border-color: var(--gold); box-shadow: 0 0 0 1px var(--gold) inset; }
  .seat.empty { border-style: dashed; }
  .idx { font-size: 12px; color: var(--muted); }
  .who { font-size: 17px; display: flex; gap: 6px; align-items: center; }
  .badge { background: var(--gold); color: #3d2200; font-size: 11px; padding: 1px 6px; border-radius: 6px; font-weight: 700; }
  .meta { font-size: 12px; color: var(--muted); display: flex; align-items: center; gap: 6px; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: #777; }
  .dot.on { background: var(--accent); box-shadow: 0 0 6px var(--accent); }
  .start { display: flex; align-items: center; gap: 14px; margin-top: 18px; flex-wrap: wrap; }
  .start span { font-size: 13px; }
  .big { font-size: 18px; padding: 12px 28px; }
  .wait { margin-top: 18px; }
  .settings { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px 14px; margin-bottom: 10px; }
  .settings label { display: flex; justify-content: space-between; align-items: center; gap: 8px; font-size: 13px; }
  .settings input { width: 90px; }
  .invite p { margin: 10px 0 6px; font-size: 14px; }
  .invite ul { margin: 6px 0; padding-left: 20px; font-size: 14px; line-height: 1.6; }
  .tabs { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 6px; }
  .tabs button, .urls button { padding: 6px 12px; font-size: 14px; }
  .sel { background: rgba(255, 209, 102, 0.22) !important; border-color: var(--gold) !important; }
  .urls { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 10px; }
  .copy {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 8px 10px;
  }
  .copy code { flex: 1; word-break: break-all; color: #d8f3dc; }
  .copy code.wrap { white-space: pre-wrap; font-family: inherit; font-size: 13px; }
  .chatbox { height: 180px; display: flex; flex-direction: column; }
</style>
