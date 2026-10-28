// 網站部署相關：GitHub Pages 版放在子路徑（/TaiwanMahjong/）下，
// App 與 headless 伺服器則從根路徑提供。Node 測試時沒有 import.meta.env，視為根路徑。
// 房號的規則與配對服務網址的處理在 VirtualCore（virtual-core/net/site.js），這裡只放台麻自己的設定。
import { signalServer } from 'virtual-core/net/site.js';

export { ROOM_CODE_CHARS, isRoomCode } from 'virtual-core/net/site.js';

const env = import.meta.env || {};

export const BASE_URL = env.BASE_URL || '/';

// 純靜態網站（GitHub Pages）：沒有伺服器，https 頁面也連不到區網的 ws://
export const staticSite = !!env.VITE_STATIC_SITE;

// 線上房間的配對服務（VirtualCore 的 workers/signal，部署成 taiwan-mahjong-signal），預設是已部署的那一個。
// 開發時可以在 .env.development.local 寫 VITE_SIGNAL_SERVER=http://localhost:8787 用本機的 wrangler dev；
// 寫 VITE_SIGNAL_SERVER=off 表示不提供線上房間（介面上相關的選項都會藏起來）。
const DEFAULT_SIGNAL_SERVER = 'https://taiwan-mahjong-signal.taiwan-mahjong-signal.workers.dev';
export const SIGNAL_SERVER = signalServer(env.VITE_SIGNAL_SERVER, DEFAULT_SIGNAL_SERVER);

// 網頁版的網址：分享房間連結用（https://…/#join/房號）
export const WEB_URL = env.VITE_WEB_URL || 'https://yuuta-tsubasa.studio/TaiwanMahjong/';

// public/ 底下的檔案路徑，例如 asset('/ui/anime90/logo.webp')
export function asset(path) {
  return BASE_URL + path.replace(/^\//, '');
}
