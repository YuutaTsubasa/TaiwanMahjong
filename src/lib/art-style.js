import { writable } from 'svelte/store';

export const ART_STYLES = [
  { id: 'anime90', name: '90 年代', description: '賽璐璐動畫' },
  { id: 'pc98', name: 'PC-98', description: '細緻像素與抖色' },
  { id: 'msdos', name: 'MS-DOS', description: '鮮明 VGA 像素' },
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
