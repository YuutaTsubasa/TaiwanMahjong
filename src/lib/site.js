// 網站部署相關：GitHub Pages 版放在子路徑（/TaiwanMahjong/）下，
// App 與 headless 伺服器則從根路徑提供。Node 測試時沒有 import.meta.env，視為根路徑。
const env = import.meta.env || {};

export const BASE_URL = env.BASE_URL || '/';

// 純靜態網站（GitHub Pages）：沒有伺服器，https 頁面也連不到區網的 ws://
export const staticSite = !!env.VITE_STATIC_SITE;

// public/ 底下的檔案路徑，例如 asset('/ui/anime90/logo.png')
export function asset(path) {
  return BASE_URL + path.replace(/^\//, '');
}
