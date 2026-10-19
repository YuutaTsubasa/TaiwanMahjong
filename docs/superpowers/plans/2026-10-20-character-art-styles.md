# Character art styles implementation plan

**Goal:** Keep the existing 1990s anime portraits and add complete PC-98 and MS-DOS sets with an immediately reactive, persistent local style selector.

**Architecture:** A Svelte store owns the local preference. Character metadata resolves the five expressions for each style. Every Avatar reads the store, so the picker, lobby, table, results and gallery share the same selection without changing server messages or character IDs.

**Tech Stack:** Svelte 5, Vite, Node test runner, transparent PNGs generated with built-in image_gen.

## Work

- [x] Generate twelve expression sheets (six characters per retro style), preserving identity and the normal/happy/surprised/sad/win order. Split equal cells into `public/characters/<id>/<pc98|msdos>/<expression>.png`; keep existing anime PNGs intact. Save prompts in `docs/art/retro-character-generation.json`.
- [x] Add `src/lib/art-style.js`: three named choices, a writable store initialized from `mj_art_style`, validated updates and safe storage access. Test default, valid preference, invalid preference, persistence and unavailable storage using the real store.
- [x] Extend `src/lib/characters.js` with style image sets and `getCharacterImage(id, expression, style)`. Test every character/expression/style against real PNG files, unknown values and normal-expression fallback.
- [x] Add `src/lib/ArtStylePicker.svelte` with accessible pressed-state buttons. Put it in `src/App.svelte` above a remaining-height content container, available on every screen.
- [x] Update `src/lib/Avatar.svelte` to resolve reactive style paths and apply pixelated sampling only to retro styles. Keep the existing SVG fallback. Keep Table's collapsed-sidebar button positioned inside the table rather than over the style toolbar.
- [x] Update README with selector behavior, paths and generation provenance. Run Node tests, `npm run build`, and `git diff --check`.
- [x] Verify all 90 unique portraits in the browser, compare both retro styles, reload to verify persistence, inspect the home and in-game layout, and review the finished code for regressions.

## Acceptance

All six existing identities and their five event expressions remain available in all three styles. Switching changes every displayed portrait immediately; reloading restores the choice. A bad/missing saved value defaults to 1990s anime, and unavailable storage does not prevent switching. All asset paths are local and included in the production build. The selector remains keyboard accessible and does not overlap game controls.
