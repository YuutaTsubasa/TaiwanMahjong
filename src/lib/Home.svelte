<script>
  import { onMount } from 'svelte';
  import { net, inTauri, connect, disconnect, startHost, stopHost, hostStatus, toast, setCharacter, savedCharacter } from './net.svelte.js';
  import Tile from './Tile.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import PortraitGrid from './ui/PortraitGrid.svelte';
  import FancyButton from './ui/FancyButton.svelte';

  const MENU = [
    ['#host', '開設牌局'],
    ['#join', '加入牌局'],
    ['#gallery', '角色圖鑑'],
    ['#settings', '遊戲設定'],
  ];
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import { getSelectableCharacters, getCharacter } from './characters.js';
  import { get } from 'svelte/store';
  import { ayiUnlocked } from './unlocks.js';

  let { page = 'title' } = $props();
  const isHostPage = $derived(page === 'host');
  function load(key, fallback) {
    try { return localStorage.getItem(key) ?? fallback; } catch { return fallback; }
  }
  function save(key, value) {
    try { localStorage.setItem(key, value); } catch {}
  }
  const servedByServer = !inTauri && location.port !== '1420' && location.protocol.startsWith('http');
  let name = $state(load('mj_name', ''));
  let port = $state(load('mj_port', '7777'));
  let address = $state(servedByServer ? location.host : load('mj_address', ''));
  let hostKey = $state('');
  let busy = $state(false);
  const available = getSelectableCharacters(get(ayiUnlocked));
  let character = $state(available.find(c => c.id === savedCharacter())?.id || available[Math.floor(Math.random() * available.length)].id);
  const selected = $derived(getCharacter(character));
  const connecting = $derived(!net.state && (net.status === 'connecting' || net.status === 'open'));
  onMount(() => { setCharacter(character); hostStatus().catch(() => {}); });
  const playerName = () => name.trim() || '玩家';

  async function host() {
    save('mj_name', name);
    save('mj_port', port);
    busy = true;
    try { await startHost(port, playerName()); }
    catch (error) { toast(String(error)); }
    finally { busy = false; }
  }
  function join(spectate = false) {
    if (!address.trim()) return toast('請輸入房主的位址，例如 192.168.1.5:7777');
    if (isHostPage && !hostKey.trim()) return toast('請輸入伺服器顯示的房主金鑰');
    save('mj_name', name);
    if (!servedByServer) save('mj_address', address);
    connect(address, { name: playerName(), hostKey: isHostPage ? hostKey.trim() : undefined, spectate });
  }
  function submit(event) {
    event.preventDefault();
    if (busy || connecting) return;
    if (isHostPage && inTauri) {
      if (net.hostInfo) {
        save('mj_name', name);
        connect(`localhost:${net.hostInfo.port}`, { name: playerName(), hostKey: net.hostInfo.host_key });
      } else host();
    } else join();
  }
  async function closeRoom() {
    busy = true;
    try { await stopHost(); } catch (error) { toast(String(error)); }
    finally { busy = false; }
  }
  function back() {
    if (connecting) disconnect();
  }
</script>

{#if page === 'title'}
  <main class="concept-page title-page">
    <section class="title-content" aria-label="虛擬台麻館主選單">
      <h1 class="title-logo"><Brand /></h1>
      <nav class="title-menu" aria-label="主選單">
        {#each MENU as [href, label]}
          <a class="menu-item" {href}>
            <span class="mark" aria-hidden="true"><Tile code="C" size="sm" /></span>
            <span class="label">{label}</span>
          </a>
        {/each}
      </nav>
    </section>
  </main>
{:else}
  <main class="concept-page room-page">
    <header class="page-header">
      <a class="back-link" href="#title" onclick={back}>← 返回標題</a>
      <div class="header-brand"><Brand compact /></div>
    </header>
    <div class="room-layout">
      <section class="art-panel room-panel">
        <OrnateHeading text={isHostPage ? '開設牌局' : '加入牌局'} sub={isHostPage ? '選好角色，準備入席。' : '找張好位子，今晚一起打牌。'} />
        <form id="room-entry" onsubmit={submit}>
          <label class="field name-field">
            <span>玩家名稱</span>
            <input bind:value={name} placeholder="玩家" maxlength="16" autocomplete="nickname" />
          </label>
        </form>
        <div class="field character-field">
          <span>選擇角色</span>
          <PortraitGrid bind:value={character} onpick={setCharacter} />
        </div>
        {#if connecting}
          <div class="connection-status" role="status">
            <span>連線到 {net.address} 中…</span>
            <button type="button" onclick={disconnect}>取消連線</button>
          </div>
        {:else}
          {#if isHostPage && inTauri}
            {#if net.hostInfo}
              <p class="hint">房間已在通訊埠 <strong>{net.hostInfo.port}</strong> 運作中。</p>
            {:else}
              <label class="field">
                <span>通訊埠</span>
                <input form="room-entry" class="port-input" bind:value={port} type="number" min="1" max="65535" required />
              </label>
              <p class="hint">建立後可在大廳邀請朋友與 AI。</p>
            {/if}
          {:else}
            <label class="field">
              <span>{isHostPage ? '伺服器位址' : '房間地址'}</span>
              <input form="room-entry" bind:value={address} placeholder="192.168.1.5:7777" required spellcheck="false" autocapitalize="none" />
            </label>
            {#if isHostPage}
              <label class="field">
                <span>房主金鑰</span>
                <input form="room-entry" bind:value={hostKey} type="password" placeholder="伺服器終端機顯示的金鑰" required autocomplete="off" />
              </label>
              <p class="hint">瀏覽器可管理已啟動的伺服器；在桌面版可直接建立房間。</p>
            {:else}
              <p class="hint">輸入房主提供的 IP 與通訊埠。</p>
            {/if}
          {/if}
          <div class="actions">
            {#if isHostPage && inTauri && net.hostInfo}
              <FancyButton variant="red" disabled={busy} onclick={closeRoom}>關閉房間</FancyButton>
            {:else if isHostPage}
              <FancyButton variant="red" href="#title">取消</FancyButton>
            {:else}
              <FancyButton variant="red" onclick={() => join(true)}>觀戰</FancyButton>
            {/if}
            <FancyButton variant="gold" tile="C" type="submit" form="room-entry" disabled={busy}>
              {busy ? '處理中…' : isHostPage ? (inTauri ? (net.hostInfo ? '回到房間' : '開設牌局') : '以房主連線') : '加入牌局'}
            </FancyButton>
          </div>
        {/if}
      </section>
      <aside class="character-stage" aria-label="所選角色">
        <div class="large-portrait"><Avatar id={character} size={540} /></div>
        <div class="art-panel portrait-caption">
          <strong>{selected.name}</strong>
          <p>{isHostPage ? '牌桌準備好了，一起來玩吧！' : '今晚也來摸幾圈吧。'}</p>
        </div>
      </aside>
    </div>
  </main>
{/if}

<style>
  /* ── 標題 ── */
  .title-page { background-image: linear-gradient(90deg, rgba(0, 21, 17, .55), transparent 55%), var(--scene-title); display: flex; align-items: center; padding: clamp(24px, 4vw, 68px); }
  .title-content { width: min(40%, 620px); min-width: 340px; display: flex; flex-direction: column; align-items: flex-start; gap: clamp(18px, 3.5vh, 44px); }
  h1 { margin: 0; }
  .title-logo :global(.logo) { width: clamp(340px, 34vw, 640px); }
  .title-menu { display: flex; flex-direction: column; gap: clamp(4px, 1vh, 12px); padding-left: clamp(0px, 1vw, 18px); }
  .menu-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 18px;
    width: clamp(280px, 22vw, 380px);
    padding: 8px 18px;
    color: #ffe9bc;
    text-decoration: none;
    font: 600 clamp(26px, 2.5vw, 40px)/1.35 var(--heading-font);
    letter-spacing: .3em;
    text-shadow: 0 2px 6px #000, 0 0 2px #000;
    border: 1px solid transparent;
  }
  .menu-item .mark { display: inline-flex; visibility: hidden; transform: rotate(-6deg); }
  .menu-item .mark :global(.tile) { --w: 34px; margin-bottom: 4px; }
  /* 選中的那一項（滑鼠、鍵盤，或預設第一項）有框和「中」 */
  .menu-item:hover, .menu-item:focus-visible,
  .title-menu:not(:hover):not(:focus-within) .menu-item:first-child {
    background: linear-gradient(90deg, #031f19f2, #042d1fd9);
    border-color: #d2ad5f;
    outline: 1px solid #7b5e28;
    outline-offset: 4px;
    box-shadow: 0 5px 14px #0008;
  }
  .menu-item:hover .mark, .menu-item:focus-visible .mark,
  .title-menu:not(:hover):not(:focus-within) .menu-item:first-child .mark { visibility: visible; }

  /* ── 開設／加入牌局 ── */
  .room-page { padding: clamp(14px, 1.8vw, 30px); background-image: linear-gradient(90deg, #021c19aa, transparent 60%), var(--scene-parlor); }
  .page-header { position: relative; display: flex; align-items: center; min-height: 64px; margin-bottom: 10px; }
  .header-brand { position: absolute; left: 0; width: 52%; display: flex; justify-content: center; pointer-events: none; }
  .header-brand :global(.brand) { pointer-events: auto; }
  .back-link { position: relative; z-index: 1; color: var(--gold); text-decoration: none; border: 1px solid var(--line); padding: 9px 18px; background: #071c18d9; white-space: nowrap; font-family: var(--heading-font); letter-spacing: .15em; }
  .room-layout { display: grid; grid-template-columns: minmax(0, 52%) minmax(0, 1fr); gap: clamp(12px, 2vw, 30px); align-items: stretch; }
  .room-panel { padding: clamp(16px, 2vw, 30px) clamp(18px, 2.4vw, 40px); }
  .field { display: grid; grid-template-columns: clamp(96px, 8vw, 132px) minmax(0, 1fr); gap: 14px; align-items: center; margin: 14px 0; }
  .field > span { color: var(--text); font: 600 clamp(17px, 1.45vw, 23px) var(--heading-font); letter-spacing: .2em; }
  .field input { width: 100%; min-width: 0; font-size: clamp(16px, 1.3vw, 20px); }
  .field .port-input { max-width: 220px; }
  .character-field { align-items: start; }
  .character-field > span { padding-top: 10px; }
  .hint { color: var(--muted); font-size: 13px; line-height: 1.6; margin: -4px 0 12px calc(clamp(96px, 8vw, 132px) + 14px); }
  .actions { display: grid; grid-template-columns: minmax(0, 0.55fr) minmax(0, 1fr); gap: clamp(12px, 2vw, 30px); margin-top: 18px; }
  .connection-status { display: flex; align-items: center; gap: 16px; flex-wrap: wrap; padding: 18px 0; overflow-wrap: anywhere; }
  .character-stage { display: flex; flex-direction: column; justify-content: flex-end; position: relative; min-width: 0; overflow: hidden; }
  .large-portrait { width: 100%; margin-bottom: -40px; filter: drop-shadow(0 6px 14px #0009); }
  .large-portrait :global(.avatar) { width: 100% !important; height: auto !important; aspect-ratio: 1; }
  .large-portrait :global(img) { object-fit: contain; }
  .portrait-caption { position: relative; padding: 18px 24px 16px; margin: 0 6% 12px 18%; }
  .portrait-caption strong {
    position: absolute;
    top: -20px;
    left: 18px;
    padding: 2px 18px;
    color: var(--gold);
    font: 600 24px var(--heading-font);
    letter-spacing: .2em;
    background: var(--art-panel);
    border: 1px solid var(--art-line);
  }
  .portrait-caption p { margin: 8px 0 0; font: 19px/1.6 var(--heading-font); letter-spacing: .08em; }
  @media (min-width: 1000px) and (max-height: 820px) {
    .room-panel { padding: 12px 24px; }
    .field { margin: 9px 0; }
  }
  @media (max-width: 900px) {
    .title-content { width: min(460px, 100%); min-width: 0; background: #021b1894; padding: 22px 14px; align-items: center; }
    .room-layout { grid-template-columns: 1fr; }
    .character-stage { display: none; }
    .header-brand { position: static; width: auto; margin-left: auto; }
  }
  @media (max-width: 480px) {
    .title-page { padding: 14px; }
    .field { grid-template-columns: 1fr; gap: 6px; }
    .hint { margin-left: 0; }
    .actions { grid-template-columns: 1fr; }
  }
</style>
