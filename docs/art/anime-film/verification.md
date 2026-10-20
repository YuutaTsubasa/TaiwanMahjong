# Verification

- All 35 current portrait SHA-256 hashes differ from the previous painted release.
- Independent Sharp inspection: all 35 are 512×512 RGBA and every pixel in the outer 16-pixel margin is transparent.
- Both character contact sheets visually inspected: five distinct expressions, consistent identities, finer outlines and new warm/cool lighting.
- `npm test`: 32 passed, 0 failed. Existing style preferences, unknown-style fallbacks, Ayi unlock and input-control tests pass.
- Independent static review: retained `modern-paint` ID and storage key; movie-specific CSS scoped to that theme; heading/panel rules outrank shared ornament styles.
- Browser at its existing 540×884 viewport: 動漫電影 appears in settings; switching loads the new scene and interface colors. Gallery has zero broken images and no horizontal overflow. Character images use `image-rendering: auto`.
- Independent UI inspection: all four files differ from the former painted assets; dimensions match the specification. Logo and table have alpha; backgrounds are opaque. Table visible bounds are exactly (300,50)–(1299,1049), with a blank felt surface. Logo Chinese and English text visually checked.
- `npm run build` (WASM + frontend) passed. Final title-menu CSS rebuild passed. Existing unused-selector warnings remain.
- New title scene and mist-blue menu were inspected in the running preview and saved as `title-preview.png`. This pass did not complete a new live match; gameplay code is unchanged.
- No commit, push or deployment performed.
