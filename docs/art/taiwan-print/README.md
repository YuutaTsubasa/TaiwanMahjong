# 台式復古印刷海報素材

Built-in `image_gen` production assets, completed 2026-10-21.

- 35 portraits at `public/characters/<id>/taiwan-print/<expression>.png`.
- Four UI assets at `public/ui/taiwan-print/`.
- Exact prompts, authoritative references, generated source paths and correction prompts are recorded in `prompts.json`.
- Approved Ayi normal, logo and table are byte-identical copies of the selected samples. Their original generation prompts are in `../style-studies/2026-10-21/prompts.json`.

All seven corrected expression sheets were visually inspected, then sliced deterministically with `prepare.cjs`. The final `contact-sheet.png` was inspected across all 35 portraits. Rows: Xiaocui, Yuehua, Aming, Tangtang, Xiaochun, Lanjie, Ayi. Columns: normal, happy, surprised, sad, win.

Verification:

- All 35 portraits are 512×512 RGBA with at least 16 fully transparent pixels on every canvas edge. `verification.json` records each nonzero-alpha bound.
- All 35 portrait SHA-256 hashes are distinct. Expressions are visually distinct and retain each character's hairstyle, accessories and costume.
- Corrected sheets provide clear separation between silhouettes. Cropping excludes distant alpha=1 noise, retaining the alpha values within each crop.
- Ayi normal retains the exact approved sample; its wider shoulder crop therefore differs slightly from the other expressions.
- Title scene and parlor are 1920×1080. Both were visually inspected: the title preserves the four original character identities and quiet left area, and the unoccupied parlor preserves its furniture, lighting and camera.
- Logo and table match the approved sample hashes exactly.

Run `node docs/art/taiwan-print/prepare.cjs` to reproduce production files and the contact sheet from the recorded generated sources.
