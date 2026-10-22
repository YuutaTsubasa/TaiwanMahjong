const MENU_PAGES = new Set(['solo', 'host', 'join', 'gallery', 'settings']);

/** A live room owns the screen until the player disconnects. */
export function resolvePage(hash, gameState) {
  if (gameState) return gameState.phase === 'lobby' ? 'lobby' : 'table';
  // 分享連結可以帶房號，例如 #join/K7QX2
  const page = (hash || '').replace(/^#/, '').split('/')[0];
  return MENU_PAGES.has(page) ? page : 'title';
}

/** 分享連結裡的房號（#join/K7QX2 → K7QX2） */
export function roomCodeFromHash(hash) {
  const m = /^#join\/([A-Za-z0-9]{5})$/.exec(hash || '');
  return m ? m[1].toUpperCase() : null;
}
