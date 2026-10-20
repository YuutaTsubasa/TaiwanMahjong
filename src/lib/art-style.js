import { writable } from 'svelte/store';

export const ART_STYLES = [
  { id: 'anime90', name: '90 年代', description: '動畫雀館 · 賽璐璐角色', edition: 'ANIMATION CLUB', mood: '燈亮了，今晚來一局。' },
  { id: 'pc98', name: 'PC-98', description: '復古視窗 · 細緻抖色角色', edition: 'PC-98 MAHJONG CLUB', mood: '電腦雀莊，等你入席。' },
  { id: 'msdos', name: 'MS-DOS', description: 'VGA 藍灰介面 · 鮮明像素角色', edition: 'TAIWAN MAHJONG / VGA', mood: '系統就緒。等待玩家加入。' },
];

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
