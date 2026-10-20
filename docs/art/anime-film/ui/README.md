# 動漫電影 UI 素材

以內建 `image_gen` 重繪標題、雀館背景、透明字標與空白俯視桌面，覆蓋 `public/ui/modern-paint/` 對應檔案。

完整生成與修正提示詞保存在 [prompts.json](prompts.json)，原稿在 `sources/`，覆蓋前的厚塗版本在 `references/`。`prepare.cjs` 記錄尺寸正規化及桌面實體範圍裁切；[verification.json](verification.json) 記錄尺寸、透明度、可見邊界與雜湊。

標題與空景均為 1920×1080；Logo 為 1400×420 RGBA；牌桌為 1600×1100 RGBA，其實體正方形範圍是 (300,50)–(1299,1049)。Logo 初稿的大片光暈、牌桌初稿的外圍光影已經過針對性重繪修正。

預覽：[四圖接觸表](contact-sheet.png)。
