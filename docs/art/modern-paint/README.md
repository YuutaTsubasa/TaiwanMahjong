# 現代厚塗動漫素材

> 歷史版本：使用者其後要求改成「動漫電影」。目前 `public/**/modern-paint` 已由 `docs/art/anime-film` 所記錄的新版取代；此資料夾保留舊版來源與處理程式，執行舊版程式會還原舊美術。

製作日期：2026-10-21。全部新圖使用內建 `image_gen`，未使用 API／CLI 生成。

## 交付內容

- `public/characters/{xiaocui,yuehua,aming,tangtang,xiaochun,lanjie,ayi}/modern-paint/{normal,happy,surprised,sad,win}.png`：共 35 張，512 × 512 RGBA。
- `public/ui/modern-paint/logo.png`：1400 × 420，直接複製核准樣張。
- `public/ui/modern-paint/table.png`：1600 × 1100，直接複製核准樣張。
- `public/ui/modern-paint/title-scene.png`：1920 × 1080，四位角色與麻將館主畫面。
- `public/ui/modern-paint/parlor.png`：1920 × 1080，無人物的同一麻將館。

阿翼的 `normal.png` 直接複製核准的 `docs/art/style-studies/2026-10-21/modern-paint/character.png`，原始位元組不變。保留棕色頭髮、藍眼、銀色盔甲及黑色內襯。其他角色依原始角色參考保留髮色、服裝、配件及眼睛顏色。

## 來源與處理

`prompts.json` 保存每次生成的完整提示詞、外部生成來源、專案內原圖、參考路徑、切割座標及成品路徑。`sources/` 保存七張五表情原圖與兩張場景原圖。已核准的圖另記錄為直接複製來源；其原始提示詞位於 `docs/art/style-studies/2026-10-21/prompts.json`。

`prepare-assets.cjs` 僅做已授權的確定性圖檔整理：在五等分附近的最低 alpha 欄位切割表情，移除透明外框，依比例放入 480 × 480 區域，外加四邊 16 像素透明邊界。保留生成的透明 alpha，沒有用程式重新畫五官、變更色彩或代替圖片生成。場景只縮放至 1920 × 1080。

## 驗證

執行：`node docs/art/modern-paint/prepare-assets.cjs`。

- 39 張交付 PNG 已存在，35 張角色圖均為 512 × 512 RGBA。
- 所有角色四邊 16 像素區域 alpha 均為零。
- 35 個角色表情 SHA-256 均不同，沒有重用同一表情。
- 阿翼普通表情、標誌與桌面和核准樣張逐位元相同。
- 已逐張視覺檢查生成原圖並檢查 `contact-sheet.png`：五表情順序為普通、開心、驚訝、難過、勝利；沒有背景方塊、跨格人物、文字標籤或遺失主要配件。
- 兩張場景保留既有房間鏡頭與家具位置。標題圖左側維持深色留白；空景無人物及 UI。

詳細尺寸、透明邊界及 SHA-256 存於 `verification.json`。本工作僅交付美術資產，遊戲載入與介面整合由主工作處理。
