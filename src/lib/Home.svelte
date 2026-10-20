<script>
  import { onMount } from 'svelte';
  import { net, inTauri, connect, disconnect, startHost, stopHost, hostStatus, toast, setCharacter, savedCharacter } from './net.svelte.js';
  import CharacterPicker from './CharacterPicker.svelte';
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import { getSelectableCharacters, getCharacter } from './characters.js';
  import { get } from 'svelte/store';
  import { ayiUnlocked } from './unlocks.js';
  import { ART_STYLES, artStyle } from './art-style.js';

  let { page = 'title' } = $props();
  const theme = $derived(ART_STYLES.find((style) => style.id === $artStyle));
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
      <div class="edition">{theme.edition}</div>
      <h1><Brand /></h1>
      <p class="mood">{theme.mood}</p>
      <nav class="title-menu" aria-label="主選單">
        <a class="menu-link" href="#host"><span aria-hidden="true">◆</span>開設牌局<span aria-hidden="true">›</span></a>
        <a class="menu-link" href="#join"><span aria-hidden="true">◆</span>加入牌局<span aria-hidden="true">›</span></a>
        <a class="menu-link" href="#gallery"><span aria-hidden="true">◆</span>角色圖鑑<span aria-hidden="true">›</span></a>
        <a class="menu-link" href="#settings"><span aria-hidden="true">◆</span>遊戲設定<span aria-hidden="true">›</span></a>
      </nav>
      <p class="welcome">台灣十六張麻將 · 今晚，一起入席。</p>
    </section>
  </main>
{:else}
  <main class="concept-page room-page">
    <header class="page-header">
      <a class="back-link" href="#title" onclick={back}>← 返回標題</a>
      <Brand compact />
    </header>
    <div class="room-layout">
      <section class="art-panel room-panel">
        <h1 class="concept-heading">{isHostPage ? '開設牌局' : '加入牌局'}</h1>
        <p class="panel-subtitle">{isHostPage ? '選好角色，準備入席。' : '找張好位子，今晚一起打牌。'}</p>
        <form id="room-entry" onsubmit={submit}>
          <label class="field name-field">
            <span>玩家名稱</span>
            <input bind:value={name} placeholder="玩家" maxlength="16" autocomplete="nickname" />
          </label>
        </form>
        <fieldset class="character-field">
          <legend>選擇角色</legend>
          <CharacterPicker bind:value={character} onpick={setCharacter} size={68} />
        </fieldset>
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
              <p class="hint">建立後可在大廳邀請朋友與電腦，並調整規則。</p>
            {/if}
          {:else}
            <label class="field">
              <span>{isHostPage ? '伺服器位址' : '房間位址'}</span>
              <input form="room-entry" bind:value={address} placeholder="192.168.1.5:7777" required spellcheck="false" autocapitalize="none" />
            </label>
            {#if isHostPage}
              <label class="field">
                <span>房主金鑰</span>
                <input form="room-entry" bind:value={hostKey} type="password" placeholder="伺服器終端機顯示的金鑰" required autocomplete="off" />
              </label>
              <p class="hint">瀏覽器可管理已啟動的伺服器；在桌面版可直接建立房間。</p>
            {:else}
              <p class="hint">輸入房主提供的 IP 與通訊埠，或完整房間網址。</p>
            {/if}
          {/if}
          <div class="actions">
            <a href="#title" class="cancel-link">取消</a>
            {#if isHostPage && inTauri && net.hostInfo}
              <button type="button" class="danger" disabled={busy} onclick={closeRoom}>關閉房間</button>
            {:else if !isHostPage}
              <button type="button" onclick={() => join(true)}>觀戰</button>
            {/if}
            <button form="room-entry" type="submit" class="primary" disabled={busy}>
              {busy ? '處理中…' : isHostPage ? (inTauri ? (net.hostInfo ? '回到我的房間' : '開設牌局') : '以房主身分連線') : '加入牌局'}
              <span aria-hidden="true"> ›</span>
            </button>
          </div>
        {/if}
      </section>
      <aside class="character-stage" aria-label="所選角色">
        <div class="large-portrait"><Avatar id={character} size={540} /></div>
        <div class="art-panel portrait-caption">
          <strong>{selected.name}</strong>
          <p>{selected.intro}，等你入席。</p>
        </div>
      </aside>
    </div>
  </main>
{/if}

<style>
  .title-page { background-image: linear-gradient(90deg, rgba(0, 21, 17, .38), transparent 65%), var(--scene-title); display: flex; align-items: center; padding: clamp(24px, 4vw, 68px); }
  .title-content { width: 43%; min-width: 340px; text-align: center; padding-bottom: 12px; }
  h1 { margin: 0; }
  .edition { color: var(--gold); font-size: 10px; letter-spacing: .3em; margin-bottom: 14px; }
  .mood { color: #ffe9bc; font-size: clamp(14px, 1.4vw, 20px); letter-spacing: .12em; text-shadow: 0 2px 6px #000; margin: 16px 0 28px; }
  .title-menu { display: flex; flex-direction: column; gap: 12px; max-width: 360px; margin: auto; }
  .menu-link { display: flex; align-items: center; justify-content: space-between; padding: 13px 28px; color: #ffe7ac; background: linear-gradient(100deg, rgba(2, 31, 24, .96), rgba(4, 45, 31, .9)); border: 1px solid #c39b50; outline: 1px solid #7b5e28; outline-offset: 3px; font: 600 clamp(21px, 2vw, 30px)/1.3 'DFKai-SB', '標楷體', serif; letter-spacing: .2em; text-decoration: none; box-shadow: 0 5px 14px #0008, inset 0 0 16px #a9842b1a; transition: background .15s, transform .15s; }
  .menu-link:hover, .menu-link:focus-visible { background: #745229; color: #fff4d4; transform: translateX(4px); }
  .menu-link span { font-size: 15px; letter-spacing: 0; }
  .welcome { color: #e1cda6; font-size: 12px; letter-spacing: .06em; margin-top: 28px; text-shadow: 0 1px 4px #000; }
  .room-page { padding: clamp(16px, 2vw, 32px); background-image: linear-gradient(90deg, #021c1999, transparent), var(--scene-parlor); }
  .page-header { display: flex; align-items: center; gap: 28px; min-height: 62px; margin-bottom: 16px; }
  .back-link, .cancel-link { color: var(--gold); text-decoration: none; border: 1px solid var(--line); padding: 10px 18px; background: #071c18d9; white-space: nowrap; }
  .room-layout { display: grid; grid-template-columns: minmax(0, 56%) minmax(0, 1fr); gap: clamp(12px, 2vw, 30px); align-items: stretch; }
  .room-panel { padding: clamp(18px, 2.2vw, 34px); }
  .panel-subtitle { text-align: center; margin: 6px 0 20px; color: var(--muted); letter-spacing: .1em; }
  .field { display: grid; grid-template-columns: 92px minmax(0, 1fr); gap: 12px; align-items: center; margin: 14px 0; }
  .field > span, legend { color: var(--gold); font-weight: 600; letter-spacing: .08em; }
  .field input { width: 100%; min-width: 0; }
  .field .port-input { max-width: 180px; }
  .character-field { border: 0; padding: 0; margin: 18px 0; min-width: 0; }
  legend { margin-bottom: 12px; padding: 0; }
  .character-field :global(.picker) { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
  .character-field :global(.opt) { border-radius: 2px; padding: 8px 6px; gap: 4px; }
  .character-field :global(.avatar.crop) { width: 100% !important; max-width: 104px; height: auto !important; aspect-ratio: 1.45; border-radius: 1px; box-shadow: none; }
  .character-field :global(.opt.sel) { box-shadow: 0 0 10px #eaca5970, inset 0 0 0 1px var(--gold); }
  .character-field :global(.opt span) { font-size: 15px; letter-spacing: .15em; }
  .hint { color: var(--muted); font-size: 12px; line-height: 1.6; margin: 10px 0 18px; }
  .actions { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; margin-top: 18px; }
  .actions .primary { flex: 1; font-size: 19px; white-space: nowrap; }
  .connection-status { display: flex; align-items: center; gap: 16px; flex-wrap: wrap; padding: 18px 0; overflow-wrap: anywhere; }
  .character-stage { display: flex; flex-direction: column; justify-content: flex-end; position: relative; min-width: 0; overflow: hidden; }
  .large-portrait { width: 100%; margin-bottom: -20px; filter: drop-shadow(0 6px 14px #0009); }
  .large-portrait :global(.avatar) { width: 100% !important; height: auto !important; aspect-ratio: 1; }
  .large-portrait :global(img) { object-fit: contain; }
  .portrait-caption { position: relative; padding: 16px 20px; margin: 0 16px 12px; }
  .portrait-caption strong { color: var(--gold); font: 600 25px 'DFKai-SB', '標楷體', serif; letter-spacing: .15em; }
  .portrait-caption p { margin: 8px 0 0; line-height: 1.5; }
  @media (min-width: 1000px) and (max-height: 800px) {
    .room-panel { padding: 16px 24px; }
    .panel-subtitle { margin-bottom: 12px; }
    .character-field { margin: 12px 0; }
    .character-field :global(.avatar.crop) { max-width: 90px; }
    .field { margin: 10px 0; }
  }
  @media (max-width: 800px) {
    .title-page { background-position: 60% center; }
    .title-content { width: min(440px, 100%); min-width: 0; background: #021b1894; padding: 25px 14px; }
    .room-layout { grid-template-columns: 1fr; }
    .character-stage { display: none; }
    .page-header { justify-content: space-between; gap: 10px; }
    .back-link { padding: 8px; font-size: 13px; }
  }
  @media (max-width: 420px) {
    .title-page { padding: 14px; }
    .field { grid-template-columns: 1fr; gap: 6px; }
    .room-panel { padding: 16px; }
    .page-header { flex-wrap: wrap; }
  }
</style>
