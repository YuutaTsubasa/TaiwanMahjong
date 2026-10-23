// 先把圖載好、解碼好，進牌桌時立繪和牌桌不會一張一張慢慢冒出來。
// 只載「這桌會用到的」：四家角色在目前畫風的五種表情、牌桌與背景；其他畫風等切換時再載。
import { getCharacterImage } from './characters.js';
import { asset } from './site.js';

const EXPRESSIONS = ['normal', 'happy', 'surprised', 'sad', 'win'];
// url → Promise；Image 物件留著，瀏覽器才不會把解碼好的圖丟掉
const loading = new Map();
const keep = [];
const done = new Set();

export function preloadImage(url) {
  if (!url) return Promise.resolve();
  if (!loading.has(url)) {
    const img = new Image();
    img.decoding = 'async';
    keep.push(img);
    loading.set(
      url,
      new Promise((resolve) => {
        const finish = () => {
          done.add(url);
          resolve();
        };
        img.onload = () => (img.decode ? img.decode().catch(() => {}) : Promise.resolve()).then(finish);
        img.onerror = finish; // 載不到也不卡住遊戲
        img.src = url;
      }),
    );
  }
  return loading.get(url);
}

/** 全部載好，或等到 timeoutMs 為止（網路很慢時也不會一直等） */
export function preloadAll(urls, timeoutMs = 3000) {
  const all = Promise.all(urls.map(preloadImage));
  return Promise.race([all, new Promise((resolve) => setTimeout(resolve, timeoutMs))]);
}

/** 這桌會用到的圖 */
export function tableImages(state, style) {
  const urls = [asset(`/ui/${style}/table.webp`), asset(`/ui/${style}/parlor.webp`)];
  for (const seat of state?.seats ?? []) {
    if (!seat?.character) continue;
    for (const e of EXPRESSIONS) urls.push(getCharacterImage(seat.character, e, style));
  }
  return [...new Set(urls)];
}

/** 這些圖是不是都已經載好了（用來決定要不要顯示「準備中」） */
export function allLoaded(urls) {
  return urls.every((url) => done.has(url));
}
