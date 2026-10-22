// 網站部署相關：GitHub Pages 版放在子路徑（/TaiwanMahjong/）下，
// App 與 headless 伺服器則從根路徑提供。Node 測試時沒有 import.meta.env，視為根路徑。
const env = import.meta.env || {};

export const BASE_URL = env.BASE_URL || '/';

// 純靜態網站（GitHub Pages）：沒有伺服器，https 頁面也連不到區網的 ws://
export const staticSite = !!env.VITE_STATIC_SITE;

// 線上房間的配對服務（workers/signal，Cloudflare）。空字串 = 不提供線上房間，介面上相關的選項都會藏起來。
// 開發時可以在 .env.development.local 寫 VITE_SIGNAL_SERVER=http://localhost:8787 用本機的 wrangler dev。
export const SIGNAL_SERVER = (env.VITE_SIGNAL_SERVER ?? 'https://taiwan-mahjong-signal.taiwan-mahjong-signal.workers.dev').replace(
  /\/$/,
  '',
);

// 網頁版的網址：分享房間連結用（https://…/#join/房號）
export const WEB_URL = env.VITE_WEB_URL || 'https://yuuta-tsubasa.studio/TaiwanMahjong/';

// 房號用的字元（去掉容易看錯的 0/O、1/I/L）
export const ROOM_CODE_CHARS = 'ABCDEFGHJKMNPQRSTUVWXYZ23456789';

// 房號：5 個英文或數字，至少有一個英文字母（只打數字視為通訊埠）
export function isRoomCode(text) {
  return /^[A-Za-z0-9]{5}$/.test(text || '') && /[A-Za-z]/.test(text);
}

// public/ 底下的檔案路徑，例如 asset('/ui/anime90/logo.png')
export function asset(path) {
  return BASE_URL + path.replace(/^\//, '');
}
