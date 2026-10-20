# 阿翼角色圖像來源與交付

2026-10-20；使用內建 `image_gen` 生成，未使用 API／CLI fallback。

## 身分來源

- 組合：**H01-B01（頭 01 小版 × 身體 01 初盔甲）small-v38**。
- ProjectVtuber 實際模型子專案：`C:/Users/User/Repo/ProjectVtuber/projects/vrm-model/`。
- 現行檔案：`output/current/H01-B01/H01-B01_small-v38_VRM1.vrm`。
- 本次實讀 SHA-256：`0b1ad4f411723a2451cea7df9d23b1503e162c694e9a48cc7fed8c47ce7b0496`，與 `output/current.json` 相符。
- 製作來源：`output/trials/armor-soles-2026-10-13/small-v38/`。
- 已閱讀根目錄與子專案 AGENTS、子專案 README、`docs/model-catalog.md`、`output/current.json`、small-v38 README。AGENTS 的 small-v37 說明為歷史快照；current.json 指向較新的 small-v38。

辨識保留：短棕亂髮、彎曲翹起的呆毛、斜瀏海、大藍眼、少年男性臉型、無金邊的銀色初騎士盔甲、圓弧肩甲與內側凸緣、黑色高領內搭。

## 實際視覺參考

使用既有模型渲染，沒有猜測模型外觀，也沒有重新渲染或修改 VRM。

| 本地副本 | 原檔（相對模型子專案） | 用途 |
|---|---|---|
| `references/h01-face-v33.png` | `output/current/_source/new-vtuber-v33/face_vrm1_front.png` | H01 臉部、眼睛、髮型 |
| `references/h01-b01-armor-small-v37.png` | `output/trials/armor-soles-2026-10-13/small-v37/vrm1-front.png` | B01 銀甲與黑內搭 |

small-v38 README 記錄該版本從 small-v37 以 GLB 增量方式只更動肩甲 JOINTS／WEIGHTS accessor；因此沿用既有同身分渲染作插畫參考。這些插畫不是 VRM 幾何、動畫或實際追蹤驗證。

風格參考使用本遊戲既有 `public/characters/xiaocui/normal.png`、`docs/art/previews/xiaocui-pc98.png`、`docs/art/previews/xiaocui-dos.png`，只取渲染風格，不取小翠角色身分。

## 生成與切圖

完整原始提示詞分別保存於 `anime90-prompt.txt`、`pc98-prompt.txt`、`msdos-prompt.txt`。

| 風格 | 原始生成檔 | 本地表情稿 |
|---|---|---|
| 90 年代動畫 | `exec-8b197d17-aaeb-4595-b564-96875f959337.png` | `sheets/anime90-initial.png` |
| PC-98 | `exec-578c4228-894a-44fb-8dd8-85fdb220538c.png` | `sheets/pc98.png` |
| MS-DOS | `exec-894ace4b-8ad4-488d-927b-7b8ba4c94098.png` | `sheets/msdos.png` |

生成工具原始目錄為 `C:/Users/User/.codex/generated_images/01a15744-0847-7db0-9bfb-c62ec795ce4e/`，已複製進本專案，不依賴該目錄運作。

各稿 1536×1024，三欄兩列，讀序為 normal、happy、surprised、sad、win，最後一格不用。`slice_assets.py` 僅裁切、等比例縮放、透明補邊；未重繪、去背、調色或修改生成 alpha。動畫稿上排衣甲延伸至 y=514，利用 y=514～520 空隙分切，避免前排衣甲混進下排。每張成品 512×512 RGBA，至少 16px 外部透明邊界。動畫採 Lanczos，兩種像素風格採 nearest-neighbor。像素風格是生成的視覺風格，不宣稱精確符合歷史硬體色數或固定像素網格。

## 成品與檢查

成品共 15 張：

- `public/characters/ayi/{normal,happy,surprised,sad,win}.png`
- `public/characters/ayi/pc98/{normal,happy,surprised,sad,win}.png`
- `public/characters/ayi/msdos/{normal,happy,surprised,sad,win}.png`

執行 `python docs/art/ayi/slice_assets.py` 通過 15 張尺寸、RGBA、透明像素比例、可見內容比例及邊界檢查；逐檔 SHA-256、裁切框、透明像素數存於 `asset-verification.json`。人工檢視三張原稿、透明合成檢查及 `contact-sheet.png`，確認三風格皆保留棕髮藍眼銀甲、五種表情順序正確、無額外手部或文字。

`contact-sheet.png` 為方便檢視的淺灰底合成圖，上至下依序為動畫、PC-98、MS-DOS；成品本身保留透明背景。生成原稿的非可見區有低 alpha 殘值，保留原始資料；最外圍透明補邊為 alpha 0。
