// 與遊戲伺服器的 WebSocket 連線（全域狀態）。
import { isTauri, invoke } from '@tauri-apps/api/core';

export const inTauri = isTauri();

export const net = $state({
  status: 'idle', // idle | connecting | open | reconnecting
  address: '',
  seat: null,
  isHost: false,
  state: null,
  receivedAt: 0,
  hostInfo: null, // 本機開房時：{ port, host_key, ips }
  mode: 'online', // online：連到伺服器；local：單機（瀏覽器裡的 WebAssembly 引擎）
  toasts: [],
});

export const clock = $state({ now: performance.now() });
setInterval(() => (clock.now = performance.now()), 200);

let ws = null;
let worker = null; // 單機版的 Web Worker
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
  if (a && !/:\d+$/.test(a) && !a.endsWith(']')) a = `${a}:7777`;
  return a;
}

export function connect(address, { name, hostKey, spectate = false, character } = {}) {
  disconnect();
  address = normalizeAddress(address);
  if (!address) return;
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
  open();
}

function open() {
  const secure = typeof location !== 'undefined' && location.protocol === 'https:' && !inTauri;
  let socket;
  try {
    socket = new WebSocket(`${secure ? 'wss' : 'ws'}://${net.address}/ws`);
  } catch (e) {
    toast(`無法連線：${e}`);
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
    let m;
    try {
      m = JSON.parse(e.data);
    } catch {
      return;
    }
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
      toast(m.message);
    }
  };
  socket.onclose = () => {
    if (ws !== socket) return;
    ws = null;
    if (wantOpen) {
      net.status = net.state ? 'reconnecting' : 'connecting';
      retryTimer = setTimeout(open, 1500);
    } else {
      net.status = 'idle';
    }
  };
}

export function disconnect() {
  if (worker) {
    worker.terminate();
    worker = null;
  }
  net.mode = 'online';
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
  if (worker) {
    worker.postMessage(obj);
    return true;
  }
  if (ws && ws.readyState === 1) {
    ws.send(JSON.stringify(obj));
    return true;
  }
  toast('尚未連線');
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
  // 單機沒有大廳：「回大廳」「結束牌局」就是回到標題
  if (worker && cmd === 'reset') return disconnect();
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
      toast(m.message);
    }
  };
  worker.onerror = (e) => {
    toast(`單機引擎載入失敗：${e.message || e}`);
    disconnect();
  };
  worker.postMessage({ type: 'start', name, character, settings });
}
