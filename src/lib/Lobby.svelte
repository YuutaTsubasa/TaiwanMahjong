<script>
  // 房間大廳，依 docs/art/concepts/lobby-host-anime90.png（房主）與 lobby-guest-anime90.png（訪客）排版。
  import { net, hostCmd, leave, disconnect, serverUrls, toast, send, setCharacter, inTauri } from './net.svelte.js';
  import { WEB_URL } from './site.js';
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import PortraitGrid from './ui/PortraitGrid.svelte';
  import FancyButton from './ui/FancyButton.svelte';
  import Modal from './ui/Modal.svelte';

  const s = $derived(net.state);
  const isHost = $derived(net.isHost);
  const seated = $derived(s.seats.filter((x) => x.name).length);
  const myCharacter = $derived(net.seat != null ? s.seats[net.seat]?.character : null);
  const takenCharacters = $derived(s.seats.filter((x) => x.character).map((x) => x.character));

  // 四個座位擺在桌子四角：左上、右上、左下、右下（照出牌順序繞一圈）
  const SLOT = ['tl', 'tr', 'br', 'bl'];

  const urls = $derived(serverUrls());
  let urlIdx = $state(0);
  const base = $derived(urls[Math.min(urlIdx, urls.length - 1)] || 'http://localhost:7777');
  const address = $derived(base.replace(/^https?:\/\//, ''));
  // 線上房間：分享房號與連結（App 裡沒有網址可用，改給網頁版的連結）
  const webBase = !inTauri && location.protocol.startsWith('http') ? location.origin + location.pathname : WEB_URL;
  const shareLink = $derived(net.room ? `${webBase}#join/${net.room}` : '');

  let tab = $state('friends');
  let aiTab = $state('claude');
  let showCharacter = $state(false);
  let showRules = $state(false);
  let settings = $state(null);
  $effect(() => {
    if (showRules) settings = { ...s.settings, skills: s.settings.skills ? 1 : 0 };
  });

  function applySettings() {
    hostCmd('settings', { settings: Object.fromEntries(Object.entries(settings).map(([k, v]) => [k, Number(v)])) });
    toast('規則已更新', 'info');
    showRules = false;
  }

  async function copy(t) {
    try {
      await navigator.clipboard.writeText(t);
      toast('已複製', 'info');
    } catch {
      toast('複製失敗，請手動選取');
    }
  }

  function roleOf(seat, i) {
    if (s.host_seat === i) return ['房主', 'host'];
    if (i === net.seat) return ['你', 'me'];
    if (seat.kind === 'bot') return ['電腦', 'bot'];
    if (seat.kind === 'agent') return ['AI', 'ai'];
    return ['玩家', 'human'];
  }
  function statusOf(seat) {
    if (seat.kind === 'bot') return '已入席';
    return seat.connected ? '在線' : '離線';
  }

  const roundsLabel = (n) => ({ 1: '東風圈', 2: '東南圈', 4: '一將' })[n] ?? `${n} 圈`;
  const rules = $derived([
    `底 ${s.settings.base}`,
    `每台 ${s.settings.per_tai}`,
    roundsLabel(s.settings.rounds),
    `出牌 ${s.settings.turn_timeout ? s.settings.turn_timeout + ' 秒' : '不限'}`,
    ...(s.settings.skills ? ['技能模式'] : []),
  ]);

  const fields = [
    ['base', '底', 10],
    ['per_tai', '每台', 10],
    ['rounds', '圈數（1=東風圈 4=一將）', 1],
    ['turn_timeout', '出牌秒數（0=不限）', 5],
    ['claim_timeout', '吃碰胡秒數（0=不限）', 5],
    ['next_hand_delay', '下一局自動開始秒數', 5],
    ['bot_delay_ms', '電腦思考毫秒', 100],
    ['skills', '技能模式（0=關 1=開）', 1],
  ];

  // 聊天：系統訊息（加入、離開…）與玩家聊天混在一起，依時間順序
  const chatLines = $derived([
    ...s.log.slice(-6).map((t) => ({ sys: true, text: t })),
    ...s.chat.slice(-20).map((c) => ({ sys: false, ...c })),
  ]);
  let chatText = $state('');
  let chatBox;
  $effect(() => {
    chatLines.length;
    if (chatBox) queueMicrotask(() => (chatBox.scrollTop = chatBox.scrollHeight));
  });
  function sendChat(e) {
    e.preventDefault();
    if (!chatText.trim()) return;
    send({ type: 'chat', text: chatText });
    chatText = '';
  }

  function exitRoom() {
    if (net.seat != null) leave();
    else disconnect();
  }

  const claudeCmd = $derived(`claude mcp add --transport http mahjong ${base}/mcp`);
  const aiPrompt = (who) =>
    `用 mahjong MCP 工具，以「${who}」的名字加入「虛擬台麻館」的台灣十六張麻將遊戲。之後不斷重複：呼叫 wait_for_turn，輪到你時用 take_action 選擇最好的 action_id，直到遊戲結束。打牌要快，每次決定都有時間限制。可以用 send_chat 跟大家聊天。`;
  const curlPrompt = $derived(`請讀 ${base}/llms.txt 的說明，用 curl 以「AI」的名字加入「虛擬台麻館」的台灣十六張麻將遊戲，然後一直玩到遊戲結束。`);
  const tunnelCmd = $derived(`cloudflared tunnel --url ${base.replace(/\/\/[^:]+:/, '//localhost:')}`);
</script>

<main class="concept-page lobby2">
  <header class="lobby-head">
    <div class="head-brand"><Brand compact /></div>
    <OrnateHeading text="房間大廳" size="md" />
    <div class="head-right">
      <FancyButton variant="red" onclick={exitRoom}>⇦ 離開房間</FancyButton>
    </div>
  </header>

  <div class="lobby-body">
    <!-- 左：座位 -->
    <section class="seats-area" aria-label="入席玩家">
      <div class="seat-count art-panel">
        <span class="label">入席玩家</span>
        <span class="count">{seated} / 4 已入席</span>
      </div>
      {#each s.seats as seat, i}
        {@const [role, roleClass] = seat.name ? roleOf(seat, i) : ['', '']}
        <div class="seat art-panel {SLOT[i]}" class:me={i === net.seat} class:empty={!seat.name}>
          <div class="seat-pic">
            {#if seat.name && seat.character}
              <Avatar id={seat.character} expression={i === net.seat ? 'happy' : 'normal'} size={200} />
            {:else}
              <svg class="chair" viewBox="0 0 100 110" aria-hidden="true">
                <path d="M24 8h52c4 0 6 3 6 7v44H18V15c0-4 2-7 6-7z" />
                <path d="M12 60h76v12H12z" />
                <path d="M18 72h8v34h-8zM74 72h8v34h-8zM34 22h32v28H34z" opacity=".55" />
              </svg>
            {/if}
          </div>
          <div class="seat-info">
            {#if seat.name}
              <b class="seat-name" title={seat.name}>{seat.name}</b>
              <span class="role {roleClass}">{role}</span>
              <span class="status"><i class="dot" class:on={seat.kind === 'bot' || seat.connected}></i>{statusOf(seat)}</span>
              {#if i === net.seat}
                <button type="button" class="seat-btn" onclick={() => (showCharacter = true)}>更換角色</button>
              {:else if isHost && s.host_seat !== i}
                <button type="button" class="seat-btn" onclick={() => hostCmd('remove', { seat: i })}>移除</button>
              {/if}
            {:else}
              <b class="seat-name wait">等待入席</b>
              {#if isHost}
                <button type="button" class="seat-btn" onclick={() => hostCmd('add_bot', { seat: i })}>加入電腦</button>
              {/if}
            {/if}
          </div>
        </div>
      {/each}
      <p class="fill-note">空位會在開始時補上電腦。</p>
    </section>

    <!-- 右：邀請／連線資訊＋聊天 -->
    <aside class="side">
      <section class="art-panel invite">
        <h2 class="panel-title">{isHost ? '邀請入席' : '連線資訊'}</h2>
        {#if isHost && !net.room}
          <div class="tabs" role="tablist">
            <button type="button" role="tab" class:sel={tab === 'friends'} aria-selected={tab === 'friends'} onclick={() => (tab = 'friends')}>朋友</button>
            <button type="button" role="tab" class:sel={tab === 'ai'} aria-selected={tab === 'ai'} onclick={() => (tab = 'ai')}>AI</button>
            <button type="button" role="tab" class:sel={tab === 'info'} aria-selected={tab === 'info'} onclick={() => (tab = 'info')}>連線資訊</button>
          </div>
        {/if}

        {#if !isHost || tab === 'friends'}
          {#if net.room}
            {#if isHost}<p>把房號告訴朋友，在「加入牌局」輸入就能入席。</p>{/if}
            <div class="copy-row">
              <code class="room-code">{net.room}</code>
              <button type="button" class="copy-btn" onclick={() => copy(net.room)}>複製</button>
            </div>
            <p class="small">或分享連結（不用裝 App）：</p>
            <div class="copy-row">
              <code>{shareLink}</code>
              <button type="button" class="copy-btn" onclick={() => copy(shareLink)}>複製</button>
            </div>
            {#if net.signalDown}
              <p class="warn">配對服務暫時連不上：已經在房間裡的人不受影響，但新的人暫時進不來。</p>
            {:else}
              <p class="ok"><i class="dot on"></i>{isHost ? '線上房間已開啟' : '已連線'}</p>
            {/if}
            {#if isHost}
              <p class="muted small">牌局在你的電腦上跑：開局後請別關掉或重新整理這個頁面。想跟 AI 一起打請用桌面版的區網房間。</p>
            {/if}
          {:else}
            {#if isHost}<p>分享房間位址，邀請朋友加入。</p>{/if}
            <div class="copy-row">
              <code>{address}</code>
              <button type="button" class="copy-btn" onclick={() => copy(address)}>複製</button>
            </div>
            <p class="ok"><i class="dot on"></i>{isHost ? '房間已開啟' : '已連線'}</p>
            {#if isHost}
              <p class="muted small">沒有 App 的朋友可直接用瀏覽器打開 <code>{base}/</code></p>
            {/if}
          {/if}
        {:else if tab === 'ai'}
          <div class="subtabs">
            <button type="button" class:sel={aiTab === 'claude'} onclick={() => (aiTab = 'claude')}>Claude Code</button>
            <button type="button" class:sel={aiTab === 'chatgpt'} onclick={() => (aiTab = 'chatgpt')}>ChatGPT</button>
            <button type="button" class:sel={aiTab === 'other'} onclick={() => (aiTab = 'other')}>其他 AI</button>
          </div>
          <div class="ai-body">
            {#if aiTab === 'claude'}
              <p>1. 把麻將伺服器加成 MCP：</p>
              <div class="copy-row"><code>{claudeCmd}</code><button type="button" class="copy-btn" onclick={() => copy(claudeCmd)}>複製</button></div>
              <p>2. 開一個<b>新的</b> claude 工作階段，貼上：</p>
              <div class="copy-row"><code class="wrap">{aiPrompt('Claude')}</code><button type="button" class="copy-btn" onclick={() => copy(aiPrompt('Claude'))}>複製</button></div>
            {:else if aiTab === 'chatgpt'}
              {#if net.room}
                <p>線上房間已經是 https 網址，不用開通道。</p>
                <div class="copy-row"><code>{base}/mcp</code><button type="button" class="copy-btn" onclick={() => copy(`${base}/mcp`)}>複製</button></div>
              {:else}
                <p>ChatGPT 需要 https 公開網址，先開通道：</p>
                <div class="copy-row"><code>{tunnelCmd}</code><button type="button" class="copy-btn" onclick={() => copy(tunnelCmd)}>複製</button></div>
              {/if}
              <p class="small">再把 <code>https://…/mcp</code> 加成 ChatGPT 的連接器（設定 → 應用程式與連接器 → 開發者模式），或在自訂 GPT 匯入 <code>/openapi.json</code>，然後貼上：</p>
              <div class="copy-row"><code class="wrap">{aiPrompt('ChatGPT').replace('mahjong MCP 工具', '麻將工具')}</code><button type="button" class="copy-btn" onclick={() => copy(aiPrompt('ChatGPT').replace('mahjong MCP 工具', '麻將工具'))}>複製</button></div>
            {:else}
              <p>任何能發 HTTP 請求的 AI 都能玩，給它這段話：</p>
              <div class="copy-row"><code class="wrap">{curlPrompt}</code><button type="button" class="copy-btn" onclick={() => copy(curlPrompt)}>複製</button></div>
            {/if}
          </div>
        {:else}
          {#if urls.length > 1}
            <div class="subtabs">
              {#each urls as u, i}
                <button type="button" class:sel={i === urlIdx} onclick={() => (urlIdx = i)}>{u.replace(/^https?:\/\//, '')}</button>
              {/each}
            </div>
          {/if}
          <dl class="info">
            <dt>網頁</dt><dd><code>{base}/</code></dd>
            <dt>MCP</dt><dd><code>{base}/mcp</code></dd>
            <dt>AI 說明</dt><dd><code>{base}/llms.txt</code></dd>
            <dt>OpenAPI</dt><dd><code>{base}/openapi.json</code></dd>
          </dl>
          {#if net.room}
            <p class="muted small">線上房間：大家都連到線上伺服器。房間沒人連線 15 分鐘後會自動關閉。</p>
          {:else}
            <p class="muted small">需在同一個區網，或房主的防火牆／路由器有開放這個埠。</p>
          {/if}
        {/if}
      </section>

      <section class="art-panel chat">
        <h2 class="panel-title">牌桌聊天</h2>
        <ol class="chat-list" bind:this={chatBox}>
          {#each chatLines as line}
            {#if line.sys}
              <li><span class="sys">[系統]</span> {line.text}</li>
            {:else}
              <li><span class="who" class:mine={line.seat === net.seat}>{line.name}</span>：{line.text}</li>
            {/if}
          {/each}
        </ol>
        {#if net.seat != null}
          <form onsubmit={sendChat}>
            <input bind:value={chatText} placeholder="說點什麼…" maxlength="300" />
            <button type="submit" class="send" disabled={!chatText.trim()} aria-label="送出">➤</button>
          </form>
        {/if}
      </section>
    </aside>

    <!-- 下：規則＋開始 -->
    <section class="art-panel rules-bar" aria-label="本局規則">
      <h2 class="rules-title">本局規則</h2>
      <ul class="rule-chips">
        {#each rules as r}<li>{r}</li>{/each}
      </ul>
      {#if isHost}
        <button type="button" class="seat-btn adjust" onclick={() => (showRules = true)}>調整規則</button>
      {/if}
    </section>
    <div class="start-area">
      {#if isHost}
        <FancyButton variant="gold" tile="F" onclick={() => hostCmd('start')} class="start-btn">開始遊戲</FancyButton>
      {:else}
        <div class="art-panel waiting"><span>等待房主開始遊戲…</span><span class="spinner" aria-hidden="true"></span></div>
      {/if}
    </div>
  </div>
</main>

<Modal bind:open={showCharacter} title="更換角色" width="760px">
  <PortraitGrid value={myCharacter} onpick={(id) => { setCharacter(id); showCharacter = false; }} taken={takenCharacters} />
</Modal>

<Modal bind:open={showRules} title="調整規則" width="620px">
  {#if settings}
    <div class="rule-form">
      {#each fields as [k, label, step]}
        <label>
          <span>{label}</span>
          <input type="number" min="0" {step} bind:value={settings[k]} />
        </label>
      {/each}
    </div>
    <div class="rule-actions">
      <FancyButton variant="red" onclick={() => (showRules = false)}>取消</FancyButton>
      <FancyButton variant="gold" onclick={applySettings}>套用</FancyButton>
    </div>
  {/if}
</Modal>

<style>
  .lobby2 {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 10px clamp(12px, 1.4vw, 24px) 14px;
    background-image: linear-gradient(#0003, #0003), var(--scene-parlor);
    background-size: cover;
    background-position: center;
  }
  .lobby-head {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 16px;
    min-height: 70px;
    margin: -10px calc(-1 * clamp(12px, 1.4vw, 24px)) 10px;
    padding: 4px clamp(12px, 1.4vw, 24px);
    background: linear-gradient(#021b16f2, #021b16d0);
    border-bottom: 1px solid var(--art-line);
  }
  .head-right { justify-self: end; }
  .head-right :global(.fancy) { min-height: 46px; font-size: clamp(16px, 1.3vw, 21px); padding: 6px 22px; }

  .lobby-body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) clamp(320px, 28vw, 470px);
    grid-template-rows: minmax(0, 1fr) auto;
    grid-template-areas: 'seats side' 'rules start';
    gap: 12px 16px;
  }

  /* ── 座位：四個角，中間露出背景的牌桌 ── */
  .seats-area { grid-area: seats; position: relative; min-height: 420px; }
  .seat-count { position: absolute; top: 0; left: 0; display: flex; align-items: center; gap: 14px; padding: 8px 20px; }
  .seat-count .label { font: 700 clamp(22px, 2vw, 32px) var(--heading-font); letter-spacing: 0.25em; color: var(--gold); }
  .seat-count .count { font: 600 clamp(14px, 1.2vw, 19px) var(--heading-font); letter-spacing: 0.1em; border: 1px solid var(--art-line); padding: 2px 14px; }
  .seat {
    position: absolute;
    display: flex;
    gap: 14px;
    width: min(46%, 390px);
    height: min(42%, 196px);
    padding: 10px;
  }
  .seat.tl { left: 2%; top: 14%; }
  .seat.tr { right: 2%; top: 14%; }
  .seat.bl { left: 0; bottom: 4%; }
  .seat.br { right: 0; bottom: 4%; }
  .seat.me { outline: 2px solid var(--gold); outline-offset: 2px; }
  .seat-pic {
    position: relative;
    flex: none;
    height: 100%;
    aspect-ratio: 1;
    overflow: hidden;
    border: 1px solid var(--art-line);
    background: radial-gradient(circle at 50% 30%, color-mix(in srgb, var(--gold) 20%, transparent), #0007 75%);
  }
  .seat-pic :global(.avatar) { position: absolute; left: 50%; top: 0; width: 118% !important; height: auto !important; aspect-ratio: 1; transform: translateX(-50%); }
  .chair { width: 100%; height: 100%; padding: 14%; fill: #0009; }
  .seat-info { display: flex; flex-direction: column; align-items: flex-start; justify-content: center; gap: 8px; min-width: 0; }
  /* flex: none：名字設了 overflow: hidden，不加的話會被直向排版壓扁、只剩半個字 */
  .seat-name { flex: none; font: 700 clamp(18px, 1.65vw, 28px)/1.4 var(--heading-font); letter-spacing: 0.08em; padding-bottom: 2px; color: var(--text); max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .seat-name.wait { color: var(--text); }
  .role { font: 700 15px var(--heading-font); letter-spacing: 0.3em; padding: 2px 18px; border-radius: var(--small-radius); border: 1px solid #0006; }
  .role.host { background: linear-gradient(#ffe7a4, #d5a34d); color: #3a2410; }
  .role.me { background: linear-gradient(#7fe0a0, #2e9a5a); color: #062614; }
  .role.bot { background: linear-gradient(#9ae6ff, #3a9fd0); color: #062033; }
  .role.ai { background: linear-gradient(#d0b4ff, #7a52c9); color: #fff; }
  .role.human { background: #ffffff22; color: var(--text); }
  .status { display: flex; align-items: center; gap: 8px; font-size: 15px; }
  .dot { display: inline-block; width: 11px; height: 11px; border-radius: 50%; background: #777; }
  .dot.on { background: #3ddc84; box-shadow: 0 0 6px #3ddc84; }
  .seat-btn { padding: 4px 16px; font-family: var(--heading-font); letter-spacing: 0.15em; }
  .fill-note {
    position: absolute;
    left: 50%;
    bottom: 1%;
    transform: translateX(-50%);
    margin: 0;
    padding: 4px 16px;
    font-size: 14px;
    letter-spacing: 0.1em;
    background: #000b;
    border: 1px solid var(--art-line);
    white-space: nowrap;
  }

  /* ── 右側 ── */
  .side { grid-area: side; display: grid; grid-template-rows: auto minmax(0, 1fr); gap: 12px; min-height: 0; }
  .panel-title { margin: 0 0 10px; font: 700 clamp(20px, 1.7vw, 28px) var(--heading-font); letter-spacing: 0.2em; color: var(--gold); }
  .panel-title::before { content: '❖ '; font-size: 0.7em; vertical-align: 3px; }
  .invite { padding: 14px 16px; min-height: 0; max-height: 52vh; overflow: auto; }
  .invite p { margin: 8px 0; font-size: 14px; line-height: 1.55; }
  .tabs { display: grid; grid-template-columns: repeat(3, 1fr); margin-bottom: 10px; border-bottom: 1px solid var(--art-line); }
  .tabs button { border-radius: 0; font: 600 17px var(--heading-font); letter-spacing: 0.15em; padding: 8px 0; }
  .tabs button.sel, .subtabs button.sel { background: linear-gradient(#6b2a2c, #3d1517); color: var(--gold); border-color: var(--gold); }
  .subtabs { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
  .subtabs button { padding: 4px 10px; font-size: 13px; }
  .copy-row { display: flex; align-items: stretch; gap: 8px; }
  .copy-row code {
    flex: 1;
    min-width: 0;
    padding: 8px 12px;
    font-size: 15px;
    word-break: break-all;
    background: #0008;
    border: 1px solid var(--art-line);
  }
  .copy-row code.wrap { white-space: pre-wrap; font-family: var(--ui-font); font-size: 13px; }
  .room-code { font: 700 clamp(24px, 2.4vw, 34px) var(--heading-font) !important; letter-spacing: 0.3em; text-align: center; color: var(--gold); }
  .copy-btn { flex: none; font: 700 17px var(--heading-font); letter-spacing: 0.15em; padding: 0 16px; }
  .ok { display: flex; align-items: center; gap: 8px; }
  .warn { color: #ffb36b; }
  .small { font-size: 12px !important; }
  .info { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 6px 12px; margin: 6px 0; font-size: 13px; }
  .info dt { color: var(--gold); }
  .info dd { margin: 0; word-break: break-all; }

  .chat { display: flex; flex-direction: column; min-height: 0; padding: 14px 16px; }
  .chat-list { flex: 1; min-height: 60px; overflow-y: auto; list-style: none; margin: 0; padding: 10px 12px; background: #0006; border: 1px solid var(--art-line); font-size: 15px; line-height: 1.7; }
  .sys { color: #ff9b6a; }
  .who { color: #7fd0ff; }
  .who.mine { color: var(--gold); }
  .chat form { display: flex; gap: 8px; margin-top: 10px; }
  .chat input { flex: 1; min-width: 0; }
  .send { width: 52px; padding: 0; font-size: 20px; }

  /* ── 下方 ── */
  .rules-bar { grid-area: rules; display: flex; align-items: center; gap: 18px; padding: 10px 20px; flex-wrap: wrap; }
  .rules-title { margin: 0; font: 700 clamp(22px, 2vw, 32px) var(--heading-font); letter-spacing: 0.2em; color: var(--gold); white-space: nowrap; }
  .rules-title::before { content: '❖ '; font-size: 0.6em; vertical-align: 4px; }
  .rule-chips { display: flex; flex-wrap: wrap; list-style: none; margin: 0; padding: 0; flex: 1; }
  .rule-chips li { padding: 2px clamp(10px, 1.4vw, 26px); font: 600 clamp(16px, 1.4vw, 22px) var(--heading-font); letter-spacing: 0.1em; border-left: 1px solid var(--art-line); }
  .adjust { font-size: 18px; padding: 8px 22px; }
  .start-area { grid-area: start; display: flex; }
  .start-area :global(.start-btn) { flex: 1; min-height: 72px; font-size: clamp(26px, 2.5vw, 40px); }
  .waiting { flex: 1; display: flex; align-items: center; justify-content: center; gap: 16px; font: 700 clamp(20px, 1.8vw, 28px) var(--heading-font); letter-spacing: 0.18em; color: var(--gold); }
  .spinner { width: 26px; height: 26px; border-radius: 50%; border: 3px dotted var(--gold); animation: spin 1.6s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .rule-form { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px 18px; }
  .rule-form label { display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 14px; }
  .rule-form input { width: 100px; }
  .rule-actions { display: flex; justify-content: center; gap: 16px; margin-top: 20px; }

  /* ── 窄螢幕：座位改成一般的兩欄，右側移到下面（1024 寬的橫向螢幕仍用四角排法，不用捲動） ── */
  @media (max-width: 900px) {
    .lobby2 { height: auto; min-height: 100%; overflow: auto; }
    .lobby-head { grid-template-columns: 1fr auto; }
    .head-brand { display: none; }
    /* 改成一欄時列高也要跟著改：沿用寬螢幕的 minmax(0, 1fr) 會把座位那一列壓成 0，名單被下面蓋住 */
    .lobby-body { flex: none; grid-template-columns: 1fr; grid-template-rows: auto; grid-template-areas: 'seats' 'rules' 'start' 'side'; }
    .seats-area { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; min-height: 0; padding-top: 58px; }
    .seat { position: static; width: auto; height: 150px; }
    .fill-note { position: static; transform: none; grid-column: 1 / -1; justify-self: center; }
    .invite { max-height: none; }
    .chat { min-height: 260px; }
  }
  @media (max-width: 600px) {
    .seats-area { grid-template-columns: 1fr; }
    .rule-form { grid-template-columns: 1fr; }
  }
  /* ── 手機橫向（畫面很矮）：座位 2×2、右側邀請與聊天、下方規則與開始，整頁不用捲動 ── */
  @media (max-height: 620px) and (orientation: landscape) {
    .lobby2 { height: 100%; min-height: 0; overflow: hidden; padding: 6px 10px 8px; }
    .lobby-head { min-height: 40px; margin: -6px -10px 6px; padding: 2px 10px; grid-template-columns: 1fr auto 1fr; }
    .head-brand { display: flex; }
    .head-brand :global(.logo) { max-height: 30px; width: auto; }
    .lobby-head :global(.ornate .text) { font-size: 18px; }
    .head-right :global(.fancy) { min-height: 32px; font-size: 14px; padding: 2px 12px; }
    .lobby-body { grid-template-columns: minmax(0, 1fr) clamp(220px, 34vw, 340px); grid-template-rows: minmax(0, 1fr) auto; grid-template-areas: 'seats side' 'rules start'; gap: 6px 8px; }
    .seats-area { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); grid-template-rows: repeat(2, minmax(0, 1fr)); gap: 6px; padding-top: 0; min-height: 0; }
    .seat-count, .fill-note { display: none; }
    .seat { position: static; width: auto; height: auto; min-height: 0; padding: 5px; gap: 8px; }
    .seat-info { gap: 3px; }
    .seat-name { font-size: 15px; }
    .role { font-size: 11px; padding: 0 8px; letter-spacing: .1em; }
    .status { font-size: 12px; gap: 4px; }
    .seat-btn { min-height: 26px !important; padding: 1px 8px; font-size: 12px; }
    .side { gap: 6px; grid-template-rows: minmax(0, 1fr) minmax(0, 1fr); grid-template-columns: minmax(0, 1fr); min-height: 0; min-width: 0; }
    .invite, .chat { min-width: 0; }
    .invite { max-height: none; min-height: 0; padding: 6px 10px; overflow: auto; }
    .tabs button { letter-spacing: 0; }
    .copy-btn { padding: 0 8px; font-size: 13px; letter-spacing: 0; }
    .panel-title { font-size: 15px; margin: 0 0 4px; }
    .tabs button { font-size: 13px; padding: 2px 0; min-height: 28px !important; }
    .chat { min-height: 0; padding: 6px 10px; }
    .chat-list { min-height: 30px; font-size: 12px; padding: 4px 8px; line-height: 1.5; }
    .chat form { margin-top: 4px; }
    .chat input { min-height: 30px !important; }
    .rules-bar { padding: 4px 10px; gap: 8px; flex-wrap: nowrap; min-width: 0; }
    .rules-title { display: none; }
    .rule-chips { flex-wrap: nowrap; min-width: 0; overflow: hidden; }
    .rule-chips li { font-size: 13px; padding: 0 8px; white-space: nowrap; }
    .rule-chips li:first-child { border-left: 0; padding-left: 0; }
    .adjust { font-size: 13px; padding: 2px 10px; min-height: 28px !important; }
    .start-area :global(.start-btn) { min-height: 40px; font-size: 20px; }
    .waiting { font-size: 15px; gap: 8px; }
  }
</style>
