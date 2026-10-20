# 動漫電影：角色素材 A

依使用者要求，將原 `modern-paint` 選項的四位角色改為明顯不同的動漫電影畫風：細緻輪廓、自然較小的眼睛、修長臉部比例、通透髮絲、冷紫／青反光與暖色夕照輪廓光。保留原角色髮色、服裝、眼睛顏色與配件。

本批包含小翠、月華、阿明、糖糖，各五種獨立表情，總計 20 張。內部路徑維持 `public/characters/<id>/modern-paint/{normal,happy,surprised,sad,win}.png`。

所有原畫均由內建 `image_gen` 新生成。參考圖片僅提供角色身分，沒有沿用上一版厚塗參考；完整提示詞、生成來源、專案內來源圖及切割座標見 `prompts.json`。原始五表情圖保留於 `sources/`。沒有修改舊版來源紀錄。

執行 `node docs/art/anime-film/portraits-a/prepare-assets.cjs` 會整理本批素材。程式僅切割、裁掉透明外框、按比例縮放至 480 × 480 並加四邊 16 像素透明邊界，不繪製角色或改色。

驗證：20 張皆為 512 × 512 RGBA；四邊 16 像素全透明；20 個 SHA-256 皆不同。已視覺檢查 `contact-sheet.png`，四行依序為小翠、月華、阿明、糖糖；五欄依序為普通、開心、驚訝、難過、勝利。詳細數值記錄於 `verification.json`。
