<script>
  import { onMount } from 'svelte';
  import { net, inTauri, connect, startHost, stopHost, hostStatus, toast } from './net.svelte.js';
  import Tile from './Tile.svelte';

  function load(k, d) {
    try { return localStorage.getItem(k) ?? d; } catch { return d; }
  }
  function save(k, v) {
    try { localStorage.setItem(k, v); } catch {}
  }

  // 由遊戲伺服器直接提供網頁時（瀏覽器加入），預設連回同一台
  const servedByServer = !inTauri && location.port !== '1420' && location.protocol.startsWith('http');

  let name = $state(load('mj_name', ''));
  let port = $state(load('mj_port', '7777'));
  let address = $state(servedByServer ? location.host : load('mj_address', ''));
  let hostKey = $state('');
  let showHostKey = $state(false);
  let busy = $state(false);

  onMount(() => {
    hostStatus().catch(() => {});
  });

  const playerName = () => name.trim() || '玩家';

  async function host() {
    save('mj_name', name);
    save('mj_port', port);
    busy = true;
    try {
      await startHost(port, playerName());
    } catch (e) {
      toast(String(e));
    } finally {
      busy = false;
    }
  }

  function join(spectate = false) {
    if (!address.trim()) return toast('請輸入房主的位址，例如 192.168.1.5:7777');
    save('mj_name', name);
    if (!servedByServer) save('mj_address', address);
    connect(address, { name: playerName(), hostKey: hostKey.trim() || undefined, spectate });
  }

  const demo = ['1m', '9p', '5s', 'E', 'C', 'F', 'P', 'f1'];
</script>

<main>
  <div class="hero">
    <div class="tiles">
      {#each demo as c, i}
        <div style:transform="rotate({(i - 3.5) * 4}deg) translateY({Math.abs(i - 3.5) * 4}px)">
          <Tile code={c} size="lg" />
        </div>
      {/each}
    </div>
    <h1>台灣十六張麻將</h1>
    <p class="muted">區網連線對戰 · 人類、Claude Code、ChatGPT 都能上桌</p>
  </div>

  <div class="name card">
    <label>
      <span>你的名字</span>
      <input bind:value={name} placeholder="玩家" maxlength="16" />
    </label>
  </div>

  {#if net.status === 'connecting'}
    <div class="card status">
      <div class="spinner"></div>
      連線到 {net.address} 中…
      <button class="small" onclick={() => location.reload()}>取消</button>
    </div>
  {:else}
    <div class="grid">
      {#if inTauri}
        <section class="card">
          <h2>🏠 開房間</h2>
          <p class="muted">在這台電腦開一個房間，其他人（或 AI）連到你的 IP 加入。</p>
          {#if net.hostInfo}
            <p>房間已在埠 <b>{net.hostInfo.port}</b> 運作中。</p>
            <div class="row">
              <button class="primary" onclick={() => connect(`localhost:${net.hostInfo.port}`, { name: playerName(), hostKey: net.hostInfo.host_key })}>
                回到我的房間
              </button>
              <button class="danger" onclick={() => stopHost()}>關閉房間</button>
            </div>
          {:else}
            <label class="inline">
              <span>埠號</span>
              <input bind:value={port} style="width: 90px" />
            </label>
            <button class="primary" disabled={busy} onclick={host}>開房</button>
          {/if}
        </section>
      {/if}

      <section class="card">
        <h2>🔗 加入房間</h2>
        <p class="muted">輸入房主給你的位址（IP:埠號）。</p>
        <input class="addr" bind:value={address} placeholder="192.168.1.5:7777" onkeydown={(e) => e.key === 'Enter' && join()} />
        {#if showHostKey}
          <input class="addr" bind:value={hostKey} placeholder="房主金鑰（伺服器終端機上有印）" />
        {/if}
        <div class="row">
          <button class="primary" onclick={() => join(false)}>加入</button>
          <button onclick={() => join(true)}>觀戰</button>
          {#if !showHostKey}
            <button class="small link" onclick={() => (showHostKey = true)}>我是房主（無介面伺服器）</button>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</main>

<style>
  main {
    height: 100%;
    overflow: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px 16px;
    gap: 20px;
  }
  .hero { text-align: center; }
  .tiles { display: flex; gap: 4px; justify-content: center; margin-bottom: 18px; }
  h1 {
    font-size: 44px;
    margin: 0;
    letter-spacing: 6px;
    font-family: 'DFKai-SB', '標楷體', 'Kaiti TC', serif;
    color: var(--gold);
    text-shadow: 0 3px 0 #7a4b00, 0 6px 18px rgba(0, 0, 0, 0.5);
  }
  .hero p { margin: 8px 0 0; }
  .name { width: min(720px, 100%); padding: 14px 20px; }
  .name label { display: flex; align-items: center; gap: 12px; }
  .name input { flex: 1; }
  .grid {
    width: min(720px, 100%);
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
  }
  h2 { margin: 0 0 6px; font-size: 20px; }
  section p { margin: 0 0 14px; font-size: 14px; }
  .row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
  .inline { display: inline-flex; align-items: center; gap: 8px; margin-right: 10px; }
  .addr { width: 100%; margin-bottom: 10px; }
  .link { background: none; border: none; color: var(--muted); text-decoration: underline; }
  .status { display: flex; align-items: center; gap: 14px; }
  .spinner {
    width: 18px;
    height: 18px;
    border: 3px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--gold);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
