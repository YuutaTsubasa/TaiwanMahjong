# Replace the painted theme with cinematic anime

The user requested a clear visual change from the 90s anime look to a Makoto Shinkai-inspired animated-film direction.

Direction: a Taiwanese mahjong parlor at dusk after rain, luminous blue/violet skies outside, warm indoor lighting, detailed backgrounds, fine linework and restrained facial proportions. Character colors, clothing, accessories and identities remain recognizable. All five expressions are redrawn, including Ayi's normal expression.

Deliverables: replace 35 portraits and four UI images in the existing `modern-paint` directories. Rename the visible choice to 動漫電影. Keep the internal style ID so saved preferences continue working. Update only this theme's tokens and decoration. Preserve the other five styles, gameplay and controls.

Parallel asset ownership: portraits-a owns Xiaocui/Yuehua/Aming/Tangtang; portraits-b owns Xiaochun/Lanjie/Ayi; UI owns the four scene/logo/table images. Root handles integration and verification. New provenance belongs in `docs/art/anime-film`; earlier artwork documentation remains historical.

Validation: inspect contact sheets and scenes for a meaningful style change; check alpha/dimensions, exact logo text and shared table geometry; run existing tests/build; inspect settings, title and actual gameplay at desktop/mobile sizes.
