const MENU_PAGES = new Set(['host', 'join', 'gallery', 'settings']);

/** A live room owns the screen until the player disconnects. */
export function resolvePage(hash, gameState) {
  if (gameState) return gameState.phase === 'lobby' ? 'lobby' : 'table';
  const page = (hash || '').replace(/^#/, '');
  return MENU_PAGES.has(page) ? page : 'title';
}
