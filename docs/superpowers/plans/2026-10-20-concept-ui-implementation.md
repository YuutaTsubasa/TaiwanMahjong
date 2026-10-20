# Concept UI Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development for isolated implementation and review. Keep edits within assigned files; preserve unrelated local work. No commit or deployment requested.

**Goal:** Turn the approved visual direction into a working shared interface, with independently replaceable 90s, PC-98 and MS-DOS assets.

**Architecture:** Keep the existing Svelte game state and network authority. Use hash destinations for title/host/join/gallery/settings when disconnected, and prioritize an active game over stale menu destinations. One style store drives character portraits, CSS tokens and scene images. Text, controls, seats, tiles, timers and messages remain live components, never baked into screenshots.

**Tech Stack:** Svelte 5, Vite, existing Tauri host commands and WebSocket connection, native CSS, built-in image_gen.

## Asset / layout contract

- Scene files: `public/ui/<anime90|pc98|msdos>/title-scene.png` and `parlor.png`.
- Both are landscape 16:9; render `cover` with center/right focal point. Title reserves left 40% for a real text logo and menu; right 60% has the illustrated cast. Parlor has no people or controls; table is centered in the lower half.
- No baked UI text, frames, buttons, labels or form values in scene images.
- Existing `public/characters` portraits remain shared and switch by the same `artStyle`.
- DOM brand wordmark avoids raster typography scaling and preserves accessible game name.
- Title: left 42% content, right illustration. Form pages: left 56% functional form, right selected portrait. Gallery: selector / portrait / detail. Settings: form / character. Under 800px stack panels and reduce or hide purely decorative artwork. All menu pages scroll at short heights.
- Desktop game: existing live hand/discard geometry retained, sidebar collapsible. Board receives visual trim only; scene must not reduce tile contrast.
- Lobby: four live seats, invitation/chat column, host-only controls, read-only guest rules and waiting message. Empty seats auto-fill via existing server.
- PC-98 and MS-DOS are generated only after the shared layout and scene geometry are established. Until files exist, fall back to anime90 scene with existing era palettes; no broken image requests.

## Tasks

- [x] Add a pure menu route/asset resolver with behavior tests: unknown route → title; active session → lobby/table; valid menu routes; invalid theme → anime90. Run tests red then green.
- [x] Replace Home's mixed landing/form screen with title, host and join states. Preserve Tauri-only host creation and browser host-key workflow; clearly explain browser cannot launch a server. Keep disconnect cancellation, saved names/characters, reconnect and running-host actions.
- [x] Build interactive character gallery and settings from existing data/stores, without invented unlocks/voice or new gameplay settings. Later user-approved scope adds hidden character Ayi and the documented match/menu-code unlock.
- [x] Create the two clean anime90 scene assets, copy into public/ui, document prompts and reference originals.
- [x] Integrate common art frames, layout, brand and live lobby/board styling. Final viewport and narrow-flow checks remain in the verification task.
- [x] Generate geometry-matched PC-98 and MS-DOS scene pairs via subagents and document the complete asset inventory in `docs/art/ui/README.md`. Final style switching/persistence checks remain in the verification task.
- [x] Run existing and new tests, Vite production build, actual browser navigation, join/host-key lobby, real gameplay and review. Save screenshots and asset inventory. Results and device-test limitations: `docs/art/ui/verification/README.md`.

## Verification

Run `npm test` and `npm run build`. Test routing / assets using Node's test runner. Use browser UI for title → each page → back, character/expression switching, audio/style settings, saved preference reload, join/cancel, host/guest permissions, board sidebar. Use temporary localhost server for actual lobby/game. Review spec first then code. Do not require pixel-identical generated layouts: generated art supplies scenery; actual controls are responsive components. Preserve existing game/backend behavior.
