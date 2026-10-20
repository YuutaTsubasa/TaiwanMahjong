# Integration verification — 2026-10-20

- `npm test`: 30 passed, 0 failed.
- `npm run build`: successful, no compiler warnings.
- Independent code review: initial reconnect discard lock, native confirmation/gamepad conflict and short-height title overflow findings fixed; follow-up review clean.
- Browser: title / host / join / gallery / settings navigation; PC-98 preference survives reload; MS-DOS setting updates background and all preview images load; audio slider updates.
- Ayi: initially absent, keyboard Konami code unlocks, reload retains unlock, gallery shows correct PC-98 portraits. Another origin remained locked yet could render another player's Ayi portrait. On-screen code buttons also successfully unlock.
- Temporary localhost server: host entered with Ayi; guest entered and saw waiting state without host controls; host started with auto-filled bots. Enter selected a tile, Escape deselected it, then two Enter presses discarded once (confirmed by server-fed log).
- In-app leave confirmation opens, Escape cancels, explicit confirmation exits.
- Responsive checks: 1280×720 desktop, 390×844 guest board with collapsible sidebar and horizontally scrollable hand; narrow settings without document overflow; 844×390 title begins below scroll origin and scrolls vertically.
- Saved screenshots: `gameplay-pc98.png`, `mobile-secret-pad.png`, `title-anime90.png`.

Limits: no physical gamepad or touch device was connected. Gamepad mapping/repeat, navigation math, control stepping, virtual keyboard edits, code tracking and unlock conditions are covered by automated tests; physical stick/button and long-press touch behavior still need device QA. Full-match winning criteria were tested with state fixtures, not a manually completed full match. Tauri native host creation was preserved but this verification used browser + local standalone server.

The temporary server was stopped and temporary guest tab closed. Production preview remains open on port 1421. Browser UI verification unlocked Ayi in that local preview profile.
