// 單機版：在背景執行緒跑 WebAssembly 牌局引擎（crates/mahjong-wasm）。
// 主執行緒送來的訊息格式跟 WebSocket 一樣（action、auto、character、chat、host），
// 回傳 { type: 'state', state } 也跟伺服器一樣，所以介面不用分兩套。
//
// P2P 房主模式（p2p-start）：這個瀏覽器就是伺服器。每個連線（房主自己是 0，其他人是 WebRTC 連進來的）
// 用 p2p-connect / p2p-message / p2p-disconnect 交給引擎；要送給某個連線的訊息用 { type: 'p2p-send', id, msg } 回傳。
import init, { LocalGame, HostGame } from './pkg/mahjong_wasm.js';

const ready = init();
let game = null;
let timer = null;
let host = null; // P2P 房主模式的 HostGame

function send(id, msg) {
  self.postMessage({ type: 'p2p-send', id, msg });
}

function sendState(id) {
  send(id, { type: 'state', state: JSON.parse(host.view(id)) });
}

function broadcast() {
  for (const id of host.client_ids()) sendState(id);
}

function hostMessage(id, msg) {
  const r = JSON.parse(host.message(id, JSON.stringify(msg)));
  for (const reply of r.replies) send(id, reply);
  if (r.changed) broadcast();
  else sendState(id);
}

function post() {
  if (game) self.postMessage({ type: 'state', state: JSON.parse(game.view()) });
}

function startLoop() {
  clearInterval(timer);
  // 電腦出牌、逾時處理都在 tick 裡；有變化才回傳畫面
  timer = setInterval(() => {
    try {
      if (game && game.tick()) post();
    } catch (err) {
      self.postMessage({ type: 'error', message: String(err) });
    }
  }, 150);
}

self.onmessage = async (event) => {
  await ready;
  const m = event.data;
  try {
    switch (m.type) {
      case 'p2p-start':
        host?.free();
        host = new HostGame(m.hostKey, JSON.stringify(m.settings || {}));
        clearInterval(timer);
        timer = setInterval(() => {
          try {
            if (host.tick()) broadcast();
          } catch (err) {
            self.postMessage({ type: 'error', message: String(err) });
          }
        }, 150);
        break;
      case 'p2p-connect':
        host.connect(m.id);
        break;
      case 'p2p-message':
        hostMessage(m.id, m.msg);
        break;
      case 'p2p-disconnect':
        if (host.disconnect(m.id)) broadcast();
        break;
      case 'start':
        game?.free();
        game = new LocalGame(m.name || '玩家', m.character || undefined, JSON.stringify(m.settings || {}));
        game.start();
        post();
        startLoop();
        break;
      case 'action':
        game.act(JSON.stringify({ action_id: m.action_id, decision_id: m.decision_id }));
        post();
        break;
      case 'auto':
        game.set_auto(!!m.on);
        post();
        break;
      case 'character':
        game.set_character(m.id);
        post();
        break;
      case 'chat':
        game.chat(m.text || '');
        post();
        break;
      case 'host':
        if (m.cmd === 'start') game.start();
        else if (m.cmd === 'settings') game.update_settings(JSON.stringify(m.settings || {}));
        post();
        break;
    }
  } catch (err) {
    self.postMessage({ type: 'error', message: String(err) });
  }
};
