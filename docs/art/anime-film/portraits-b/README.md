# Anime film portraits — group B

Completed 2026-10-21 with built-in `image_gen`. Replaces the modern-paint portraits of Xiaochun, Lanjie and Ayi, including all three normal expressions.

The original base portraits supplied identity and costume references. New art uses delicate film-anime linework, restrained faces, fine luminous hair, clean cel gradients, sunset rim light and cool reflected fill. The old thick-paint samples were not reused.

`prompts.json` contains exact generation and correction prompts, reference paths, generated source paths and output paths. `sheets/` contains final separated expression strips. The deterministic preparation script is `prepare.cjs`.

Verification:

- All 15 production images are 512×512 RGBA and have at least 16 pixels of transparent margin on all sides.
- All 15 SHA-256 hashes are different; `verification.json` records individual dimensions, bounds and hashes.
- Final `contact-sheet.png` was visually inspected. Rows: Xiaochun, Lanjie, Ayi. Columns: normal, happy, surprised, sad, win.
- Xiaochun retains orange ponytail, white ribbon and blue athletic shirt; Lanjie retains purple updo, gold glasses and red/gold qipao; Ayi retains brown hair, blue eyes, silver armor and black undersuit.
- Every expression is distinct. Full hair, glasses, ribbon and shoulders remain visible with separated silhouettes.

Only the assigned 15 production portraits and this provenance directory were changed for this replacement. Previous style provenance remains intact.
