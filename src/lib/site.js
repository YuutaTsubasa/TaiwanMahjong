// 網站部署相關：GitHub Pages 版放在子路徑（/TaiwanMahjong/）下，
// App 與 headless 伺服器則從根路徑提供。Node 測試時沒有 import.meta.env，視為根路徑。
const env = import.meta.env || {};

export const BASE_URL = env.BASE_URL || '/';

// 純靜態網站（GitHub Pages）：沒有伺服器，https 頁面也連不到區網的 ws://
export const staticSite = !!env.VITE_STATIC_SITE;

// 線上對戰的公開伺服器（Fly.io）；開發時可以用 VITE_ONLINE_SERVER=http://localhost:7799 指到本機
export const ONLINE_SERVER = (env.VITE_ONLINE_SERVER || 'https://taiwan-mahjong.fly.dev').replace(/\/$/, '');

// 網頁版的網址：分享房間連結用（https://…/#join/房號）
export const WEB_URL = env.VITE_WEB_URL || 'https://yuuta-tsubasa.studio/TaiwanMahjong/';

// 房號：5 個英文或數字，至少有一個英文字母（只打數字視為通訊埠）
export function isRoomCode(text) {
  return /^[A-Za-z0-9]{5}$/.test(text || '') && /[A-Za-z]/.test(text);
}

// public/ 底下的檔案路徑，例如 asset('/ui/anime90/logo.png')
export function asset(path) {
  return BASE_URL + path.replace(/^\//, '');
}
