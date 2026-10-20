// 單機版：在背景執行緒跑 WebAssembly 牌局引擎（crates/mahjong-wasm）。
// 主執行緒送來的訊息格式跟 WebSocket 一樣（action、auto、character、chat、host），
// 回傳 { type: 'state', state } 也跟伺服器一樣，所以介面不用分兩套。
import init, { LocalGame } from './pkg/mahjong_wasm.js';

const ready = init();
let game = null;
let timer = null;

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
