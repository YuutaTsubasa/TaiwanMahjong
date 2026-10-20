import { writable } from 'svelte/store';

export const ART_STYLES = [
  { id: 'anime90', name: '90 年代', description: '動畫雀館 · 賽璐璐角色', edition: 'ANIMATION CLUB', mood: '燈亮了，今晚來一局。' },
  { id: 'pc98', name: 'PC-98', pixelated: true, description: '復古視窗 · 細緻抖色角色', edition: 'PC-98 MAHJONG CLUB', mood: '電腦雀莊，等你入席。' },
  { id: 'msdos', name: 'MS-DOS', pixelated: true, description: 'VGA 藍灰介面 · 鮮明像素角色', edition: 'TAIWAN MAHJONG / VGA', mood: '系統就緒。等待玩家加入。' },
  { id: 'taiwan-print', name: '台式印刷', description: '復古海報 · 紙感套色與油墨網點', edition: 'TAIWAN PRINT CLUB', mood: '老地方，茶泡好了。' },
  { id: 'gameboy', name: 'Game Boy', pixelated: true, description: '掌機四色 · 綠色點陣與俐落方框', edition: 'FOUR COLOR MAHJONG', mood: '四種顏色，一樣熱鬧。' },
  { id: 'modern-paint', name: '動漫電影', description: '雨後黃昏 · 通透天空與細緻逆光', edition: 'TWILIGHT MAHJONG PARLOR', mood: '雨停了，窗邊還有一個位置。' },
];

export const isPixelStyle = (id) => ART_STYLES.some((style) => style.id === id && style.pixelated);

const STORAGE_KEY = 'mj_art_style';
const isStyle = (id) => ART_STYLES.some((style) => style.id === id);

function loadStyle() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (isStyle(saved)) return saved;
  } catch {}
  return 'anime90';
}

const preference = writable(loadStyle());
export const artStyle = { subscribe: preference.subscribe };

export function setArtStyle(id) {
  if (!isStyle(id)) return;
  preference.set(id);
  try { localStorage.setItem(STORAGE_KEY, id); } catch {}
}
