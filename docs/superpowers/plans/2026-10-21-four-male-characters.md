# Four regular male characters

**Goal:** Add approved 子衡 (ziheng), 承岳 (chengyue), 知白 (zhibai), 佑辰 (youchen) as regular characters with six complete art styles and five expressions per style. Ten regular characters plus hidden Ayi.

**Architecture:** Extend the existing frontend registry and the shared Rust core regular roster used by both server and WASM. Keep existing IDs, saved preferences and Ayi unlocking intact. Reuse expression routing and reaction events. Adjust existing portrait grid spacing/columns for eleven entries without new selection mechanics.

**Assets:** 120 new 512×512 RGBA portraits. Three independent asset tasks own two styles each. Use the approved concept board and design.json for identities; preserve exact prompts, generated sheets, normalization and visual QA under docs/art/male-roster-production. Ayi and existing artwork stay unchanged.

## Execution

- [x] Add frontend regression coverage for ten regular characters, four new IDs/names/lines and registry agreement with Rust; update expected expression routes to 330. Observe failures before registry edits.
- [x] Add Rust coverage for new regular character names and playable selections; observe missing-name failure, then extend the core registry to ten.
- [x] Add four frontend definitions and event dialogue. Correct obsolete registry comment path.
- [x] Adjust Home/Gallery portrait layout for eleven entries. Preserve semantic radio controls, keyboard/gamepad navigation and hidden-Ayi behavior.
- [x] Generate/cut/inspect all 120 portraits; validate alpha, sizes, unique expressions and Game Boy palette.
- [x] Run npm tests, Rust core tests, WASM/frontend build. Inspect own browser test tab for desktop/mobile selections, gallery expressions and a local match with a new character.
- [x] Save evidence and report results. Do not commit unless requested.
