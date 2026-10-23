// 與遊戲伺服器的連線（全域狀態）：WebSocket（區網／自架伺服器）、WebRTC（線上房間）或單機引擎。
import { isTauri, invoke } from '@tauri-apps/api/core';
import { SIGNAL_SERVER, ROOM_CODE_CHARS } from './site.js';
import { P2PHost, P2PGuest, signalAvailable } from './p2p.js';
import { t } from './i18n.svelte.js';

export const inTauri = isTauri();

export const net = $state({
  status: 'idle', // idle | connecting | open | reconnecting
  address: '',
  seat: null,
  isHost: false,
  state: null,
  receivedAt: 0,
  hostInfo: null, // 本機開房時：{ port, host_key, ips }
  mode: 'online', // online：連到伺服器；local：單機；p2p：線上房間（房主的瀏覽器就是伺服器）
  room: null, // 線上房間的房號（區網／自架伺服器是 null）
  online: null, // 線上房間能不能用（配對服務在不在）；null = 還沒檢查
  signalDown: false, // 房主：配對服務暫時連不上（已經在房間裡的人不受影響，只是新的人進不來）
  toasts: [],
});

export const clock = $state({ now: performance.now() });
setInterval(() => (clock.now = performance.now()), 200);

let ws = null;
let wsUrl = '';
let opener = null; // 重連時呼叫：openWs 或 openGuest
let worker = null; // 單機版或 P2P 房主的 Web Worker（牌局引擎）
let p2pHost = null;
let p2pGuest = null;
let wantOpen = false;
let hello = null;
let retryTimer = null;
let toastId = 0;

const tokenKey = (addr) => `mj_token_${addr}`;
const hostKeyKey = (addr) => `mj_hostkey_${addr}`;

function store(key, value) {
  try {
    if (value == null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {}
}
function load(key) {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function toast(message, kind = 'error') {
  const id = ++toastId;
  net.toasts.push({ id, message, kind });
  setTimeout(() => {
    net.toasts = net.toasts.filter((t) => t.id !== id);
  }, 4000);
}

export function normalizeAddress(a) {
  a = (a || '').trim().replace(/^(https?|wss?):\/\//, '').replace(/\/.*$/, '');
  if (a && !a.includes(':') && !a.includes('.')) a = `localhost:${a}`;
  // https 頁面連的是 Cloudflare 之類的通道網址（wss 走 443），不要補通訊埠
  const tunnel = typeof location !== 'undefined' && location.protocol === 'https:' && !inTauri && /\.[a-z][a-z0-9-]*$/i.test(a);
  if (a && !tunnel && !/:\d+$/.test(a) && !a.endsWith(']')) a = `${a}:7777`;
  return a;
}

export function connect(address, { name, hostKey, spectate = false, character } = {}) {
  disconnect();
  address = normalizeAddress(address);
  if (!address) return;
  const secure = typeof location !== 'undefined' && location.protocol === 'https:' && !inTauri;
  wsUrl = `${secure ? 'wss' : 'ws'}://${address}/ws`;
  opener = openWs;
  openSession(address, { name, hostKey, spectate, character });
}

function openSession(address, { name, hostKey, spectate, character }) {
  if (hostKey) store(hostKeyKey(address), hostKey);
  net.address = address;
  net.status = 'connecting';
  wantOpen = true;
  hello = {
    type: 'hello',
    name,
    host_key: hostKey || load(hostKeyKey(address)) || undefined,
    spectate,
    character: character || load('mj_character') || undefined,
  };
  opener();
}

/** 收到伺服器（或 P2P 房主）傳來的訊息 */
function receive(m) {
  if (m.type === 'state') {
    net.state = m.state;
    net.receivedAt = performance.now();
    net.isHost = !!m.state.is_host;
    net.seat = m.state.you;
  } else if (m.type === 'welcome') {
    net.seat = m.seat;
    net.isHost = m.is_host;
    if (m.token) store(tokenKey(net.address), m.token);
  } else if (m.type === 'error') {
    toast(t(m.message));
  }
}

/** 斷線了：要的話過一下再重連 */
function scheduleRetry() {
  if (wantOpen) {
    net.status = net.state ? 'reconnecting' : 'connecting';
    retryTimer = setTimeout(opener, 1500);
  } else {
    net.status = 'idle';
  }
}

function openWs() {
  let socket;
  try {
    socket = new WebSocket(wsUrl);
  } catch (e) {
    toast(t('無法連線：{error}', { error: String(e) }));
    net.status = 'idle';
    return;
  }
  ws = socket;
  socket.onopen = () => {
    net.status = 'open';
    const token = hello.spectate ? undefined : load(tokenKey(net.address)) || undefined;
    socket.send(JSON.stringify({ ...hello, token }));
  };
  socket.onmessage = (e) => {
    try {
      receive(JSON.parse(e.data));
    } catch {}
  };
  socket.onclose = () => {
    if (ws !== socket) return;
    ws = null;
    scheduleRetry();
  };
}

// ───────────── 線上房間（P2P） ─────────────

/** 配對服務能不能用（介面用來決定要不要顯示線上房間）。每次進開房／加入畫面重新檢查。 */
export async function checkOnline() {
  net.online = await signalAvailable(SIGNAL_SERVER);
  return net.online;
}

function randomString(chars, n) {
  const buf = new Uint32Array(n);
  crypto.getRandomValues(buf);
  return Array.from(buf, (x) => chars[x % chars.length]).join('');
}

/** 開線上房間：這個瀏覽器就是伺服器（牌局引擎跑在 Web Worker），其他人用 WebRTC 連進來 */
export function hostRoom({ name, character, settings } = {}) {
  disconnect();
  const key = randomString('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789', 24);
  worker = new Worker(new URL('./local/worker.js', import.meta.url), { type: 'module' });
  net.mode = 'p2p';
  net.status = 'open';
  worker.onmessage = (e) => {
    const m = e.data;
    if (m.type === 'p2p-send') {
      if (m.id === 0) receive(m.msg);
      else p2pHost?.send(m.id, m.msg);
    } else if (m.type === 'error') {
      toast(t(m.message));
    }
  };
  worker.onerror = (e) => {
    toast(t('牌局引擎載入失敗：{error}', { error: e.message || String(e) }));
    disconnect();
  };
  worker.postMessage({ type: 'p2p-start', hostKey: key, settings });
  worker.postMessage({ type: 'p2p-connect', id: 0 });
  hello = { type: 'hello', name, host_key: key, character: character || load('mj_character') || undefined };
  worker.postMessage({ type: 'p2p-message', id: 0, msg: hello });
  registerRoom(key, 0);
}

function registerRoom(key, tries) {
  // 第一個字一定是英文字母，才不會跟「只打通訊埠號碼」的位址搞混
  const code = randomString('ABCDEFGHJKMNPQRSTUVWXYZ', 1) + randomString(ROOM_CODE_CHARS, 4);
  net.room = code;
  net.address = `p2p:${code}`;
  const h = new P2PHost({
    signal: SIGNAL_SERVER,
    code,
    key,
    onPeerOpen: (id) => worker?.postMessage({ type: 'p2p-connect', id }),
    onPeerMessage: (id, msg) => worker?.postMessage({ type: 'p2p-message', id, msg }),
    onPeerClose: (id) => worker?.postMessage({ type: 'p2p-disconnect', id }),
    // 房號剛好被別人用了：換一個
    onTaken: () => {
      if (p2pHost === h && tries < 5) registerRoom(key, tries + 1);
    },
    onSignalDown: (down) => {
      if (p2pHost === h) net.signalDown = down;
    },
  });
  p2pHost?.stop();
  p2pHost = h;
  h.start();
}

/** 加入線上房間：透過配對服務找到房主，用 WebRTC 直接連過去 */
export function joinRoom(code, { name, spectate = false, character } = {}) {
  disconnect();
  net.room = code.trim().toUpperCase();
  net.mode = 'p2p';
  opener = openGuest;
  openSession(`p2p:${net.room}`, { name, spectate, character });
}

function openGuest() {
  const g = new P2PGuest({
    signal: SIGNAL_SERVER,
    code: net.room,
    onOpen: () => {
      if (p2pGuest !== g) return;
      net.status = 'open';
      const token = hello.spectate ? undefined : load(tokenKey(net.address)) || undefined;
      g.send({ ...hello, token });
    },
    onMessage: (m) => {
      if (p2pGuest === g) receive(m);
    },
    onClose: () => {
      if (p2pGuest !== g) return;
      p2pGuest = null;
      scheduleRetry();
    },
    onFatal: (message, code) => {
      if (p2pGuest !== g) return;
      // 已經在房間裡卻找不到房主：房主關掉了頁面
      toast(net.state && code === 'no_host' ? t('房主離開了，房間已經關閉') : message);
      disconnect();
    },
  });
  p2pGuest = g;
  g.connect();
}

export function disconnect() {
  if (worker) {
    worker.terminate();
    worker = null;
  }
  p2pHost?.stop();
  p2pHost = null;
  p2pGuest?.close();
  p2pGuest = null;
  net.mode = 'online';
  net.room = null;
  net.signalDown = false;
  wantOpen = false;
  clearTimeout(retryTimer);
  if (ws) {
    const s = ws;
    ws = null;
    s.close();
  }
  net.state = null;
  net.seat = null;
  net.isHost = false;
  net.status = 'idle';
}

export function send(obj) {
  if (worker && net.mode === 'p2p') {
    worker.postMessage({ type: 'p2p-message', id: 0, msg: obj });
    return true;
  }
  if (worker) {
    worker.postMessage(obj);
    return true;
  }
  if (p2pGuest?.open) {
    p2pGuest.send(obj);
    return true;
  }
  if (ws && ws.readyState === 1) {
    ws.send(JSON.stringify(obj));
    return true;
  }
  toast(t('尚未連線'));
  return false;
}

export function act(actionId) {
  return send({ type: 'action', action_id: actionId, decision_id: net.state?.decision_id });
}

/** 換角色（並記住，下次加入沿用） */
export function setCharacter(id) {
  store('mj_character', id);
  if (hello) hello.character = id;
  if (net.seat != null) send({ type: 'character', id });
}

export function savedCharacter() {
  return load('mj_character');
}

/** 託管：電腦幫你打，但座位與手牌仍是你的 */
export function setAuto(on) {
  send({ type: 'auto', on });
}

export function hostCmd(cmd, extra = {}) {
  // 單機沒有大廳：「回大廳」「結束牌局」就是結束牌局，回到單機設定頁
  if (worker && net.mode === 'local' && cmd === 'reset') return disconnect();
  send({ type: 'host', cmd, ...extra });
}

/** 離開座位並回到首頁 */
export function leave() {
  if (worker) return disconnect();
  send({ type: 'leave' });
  store(tokenKey(net.address), null);
  setTimeout(disconnect, 100);
}

export async function startHost(port, name) {
  const info = await invoke('start_host', { port: Number(port) || 7777 });
  net.hostInfo = info;
  connect(`localhost:${info.port}`, { name, hostKey: info.host_key });
  return info;
}

export async function stopHost() {
  disconnect();
  await invoke('stop_host');
  net.hostInfo = null;
}

export async function hostStatus() {
  if (!inTauri) return null;
  const info = await invoke('host_status');
  net.hostInfo = info;
  return info;
}

/** 給其他人用的伺服器網址清單 */
export function serverUrls() {
  if (net.room) return [];
  if (net.hostInfo) {
    return net.hostInfo.ips.map((ip) => `http://${ip}:${net.hostInfo.port}`);
  }
  return net.address ? [`http://${net.address}`] : [];
}

/**
 * 單機遊玩：在 Web Worker 裡跑牌局引擎，你加三個電腦。
 * 畫面收到的狀態格式和連線時完全一樣。
 */
export function startLocal({ name, character, settings } = {}) {
  disconnect();
  worker = new Worker(new URL('./local/worker.js', import.meta.url), { type: 'module' });
  net.mode = 'local';
  net.address = '單機';
  net.status = 'open';
  net.isHost = true;
  worker.onmessage = (e) => {
    const m = e.data;
    if (m.type === 'state') {
      net.state = m.state;
      net.receivedAt = performance.now();
      net.seat = m.state.you;
    } else if (m.type === 'error') {
      toast(t(m.message));
    }
  };
  worker.onerror = (e) => {
    toast(t('單機引擎載入失敗：{error}', { error: e.message || String(e) }));
    disconnect();
  };
  worker.postMessage({ type: 'start', name, character, settings });
}
