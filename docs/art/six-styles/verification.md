# Six selectable styles — 2026-10-21

This records the initial six-style release. The painted artwork was subsequently replaced by the cinematic anime revision documented in `../anime-film`; statements about matching the approved painted samples apply to the initial release only.

Implemented `taiwan-print`, `gameboy`, and `modern-paint` alongside the existing three styles. Each new set includes seven characters with five expressions (35 portraits) plus logo, table, title scene and parlor (four UI images).

Asset provenance, contact sheets and pixel/alpha checks are in the adjacent `taiwan-print`, `gameboy`, and `modern-paint` folders. Each approved Ayi normal portrait, logo and table was preserved byte-for-byte. All 105 new portraits are 512×512 RGBA with 16-pixel transparent margins. All Game Boy artwork uses the approved four opaque RGB colors and binary alpha.

Verification:

- `npm test`: 32 passed, 0 failed. Covers all 210 portrait paths and dimensions, 24 UI files, six-style persistence, fallbacks, pixel-art metadata, secret unlock and input controls.
- `npm run build`: WASM and production frontend succeeded. Final CSS/layout rebuild also succeeded. Existing unused-selector warnings in Gallery/Settings remain.
- Browser: all six selections changed the document theme, logo and character paths correctly.
- Browser at 1440×900: all six preview cards and completion button visible, zero broken images.
- Browser at 390×844: two-column cards, native style selector, no horizontal overflow; switching to modern paint survived a reload. Game Boy volume sliders retain a 28-pixel input height.
- Actual local Game Boy match: all portraits loaded; mouse selected one-wan and Enter discarded it, then three computer opponents played and the turn returned. Test match exited through its confirmation dialog.
- Independent code review found one hover-filter specificity issue, fixed. Physical gamepad and touchscreen hardware were not available for this pass; existing controller/navigation unit tests passed.

Screenshots: `settings-desktop.png`, `settings-mobile.png`. Browser viewport override reset after checks. No commit or push made.
