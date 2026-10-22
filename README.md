# 🀄 虛擬台麻館 Virtual Taiwan Mahjong Parlor

一間線上的台灣十六張麻將館（Tauri v2 + Svelte）。「虛擬」有三層意思：坐在對面的可能是 AI、館裡有二次元的虛擬角色、而且整間館開在線上。

🎮 **馬上玩**：<https://yuuta-tsubasa.studio/TaiwanMahjong/> ——網頁版免安裝，單機跟電腦打（牌局引擎以 WebAssembly 在瀏覽器裡跑）。網頁版不能開房，只能用「加入牌局」連到 https 通道網址（例如 cloudflared）；要開房請用桌面 App。

同一張牌桌可以坐：

- 👤 **人類**：用桌面 App，或直接用瀏覽器打開房主的網址
- 🤖 **Claude Code**：透過 MCP（`/mcp`）
- 💬 **ChatGPT**：透過 MCP 連接器或自訂 GPT Actions（`/openapi.json`）
- 🛠 **任何 AI / 程式**：HTTP API，curl 就能玩（說明在 `/llms.txt`）
- 💻 **內建電腦**：補空位，或接手斷線玩家

房主的 App 內建一個伺服器（預設埠 7777），所有牌局邏輯都在伺服器端判定，玩家只送出動作。

## 下載 App

各平台的安裝檔都由 GitHub Actions 打包（[`.github/workflows/build-apps.yml`](.github/workflows/build-apps.yml)）：

| 平台 | 檔案 | 備註 |
| --- | --- | --- |
| Windows | `.exe`（安裝程式）、`.msi` | SmartScreen 會警告「未知的發行者」，點「其他資訊 → 仍要執行」 |
| macOS | `.dmg`（`aarch64` = Apple Silicon、`x64` = Intel） | 沒有公證，第一次開啟要在 Finder 按右鍵 →「打開」 |
| Linux | `.deb`、`.rpm`、`.AppImage` | |
| Android | `.apk` | 允許「安裝不明來源的應用程式」後直接安裝 |
| iOS | 模擬器用的 `.app` | 實機需要 Apple 開發者帳號簽章，見下方 |

- 推一個 `v*` 標籤（例如 `v0.2.0`）會打包全部平台並附到同名的 Release。
- 也可以在 Actions →「Build apps」→「Run workflow」手動打包，檔案在該次執行的 Artifacts。

**Android 固定簽章**（選用）：沒設定的話每次用臨時金鑰簽章，能安裝，但升級前要先移除舊版。想要能直接覆蓋升級，就在 repo 的 Settings → Secrets 加上 `ANDROID_KEYSTORE_BASE64`（`base64 -w0 release.jks` 的結果）、`ANDROID_KEYSTORE_PASSWORD`、`ANDROID_KEY_ALIAS`。

**iOS 實機**：需要 Apple Developer Program（年費 US$99）。有帳號之後，在 macOS 上執行 `npx tauri ios init`、在 Xcode 設定 Team，再 `npx tauri ios build` 就能裝到自己的 iPhone；要在 CI 上打包 `.ipa`，需要把簽章憑證與 provisioning profile 放進 Secrets。

## 開發

需求：Node 18+、Rust 1.89+（Windows 需 WebView2，Win10/11 已內建）。

```bash
npm install
```

```bash
npm run tauri dev
```

打包安裝檔：

```bash
npm run tauri build
```

只跑伺服器、不開視窗（放在主機上，或給 AI 測試用）：

```bash
npm run build
```

```bash
npm run server -- --port 7777 --dist dist
```

會印出網址與「房主金鑰」。用瀏覽器開網址、首頁點「我是房主」輸入金鑰，就能當房主。
其他參數：`--bots 3`（先放 3 個電腦）、`--start`（直接開打）、`--settings '{"turn_timeout":60}'`。

線上公開伺服器（任何人都能開房，每個房間有 5 碼房號；部署在 Fly.io，設定在 `fly.toml`）：

```bash
npm run server -- --public --port 7799
```

開發時在專案根目錄放一個 `.env.development.local`，寫 `VITE_ONLINE_SERVER=http://localhost:7799`，`npm run dev` 的「線上」房間就會連到本機。

部署（房間都存在記憶體裡，所以只能有一台機器）：

```bash
fly deploy --ha=false
```

測試：

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

## 怎麼玩

**線上房間（最簡單）**：在「開設牌局」選「線上」，會拿到一個 5 碼房號，例如 `K7QX2`。
朋友在「加入牌局」輸入房號，或直接打開分享連結 `https://yuuta-tsubasa.studio/TaiwanMahjong/#join/K7QX2`（不用裝 App）。
AI 也能加入：MCP 網址是 `https://taiwan-mahjong.fly.dev/r/房號/mcp`，已經是 https，ChatGPT 不用開通道。
房間沒人連線 15 分鐘後會自動關閉。

**區網房間（桌面版）**：

1. 房主在 App 的「開設牌局」選「區網」再按「開設牌局」。
2. 大廳右邊的「邀請玩家與 AI」有現成的指令可以複製：
   - **朋友**：App 選「加入房間」輸入 `IP:7777`，或瀏覽器開 `http://IP:7777/`
   - **Claude Code**：
     ```bash
     claude mcp add --transport http mahjong http://IP:7777/mcp
     ```
     然後跟 Claude 說：「用 mahjong MCP 工具加入虛擬台麻館的麻將遊戲，一直玩到結束」
   - **ChatGPT**：ChatGPT 在雲端，需要 https 公開網址。可用 Cloudflare 臨時通道：
     ```bash
     cloudflared tunnel --url http://localhost:7777
     ```
     把得到的 `https://xxxx.trycloudflare.com/mcp` 加到 ChatGPT「設定 → 應用程式與連接器 → 進階 → 開發者模式」的自訂連接器，
     或在自訂 GPT 的「動作」匯入 `https://xxxx.trycloudflare.com/openapi.json`。
3. 房主按「開始遊戲」，空位會自動補電腦。遊戲中有人加入會接手電腦的座位。
4. 點一下選牌、再點一下（或雙擊）打出；吃碰槓胡會跳出按鈕。

> 朋友連不進來的話，通常是 Windows 防火牆擋住了 7777 埠，第一次開房時允許存取即可。

## 二次元角色

內建 6 個原創角色（小翠、月華、阿明、糖糖、小春、蘭姐），每個都有 normal / happy / surprised / sad / win 五種表情和自己的台詞。
遊戲上方的「整體風格」可切換 **90 年代、PC-98、MS-DOS**，首頁、大廳、牌桌、結算與圖鑑的角色與介面會立即一起更新。選擇會記在這台裝置的瀏覽器／App 中，重新開啟仍會沿用；每位玩家可自行選擇喜歡的畫風。
90 年代版採暖金綠色雀館與圓角；PC-98 版採紫藍抖色背景與雙線視窗；MS-DOS 版採 VGA 藍底、青色標題與灰色立體按鈕。麻將牌、牌背、對話泡泡與結算視窗同步換色，選牌與最新棄牌仍保留清楚的標示。整體配色集中在 `src/themes.css`。
三套共 90 張 512 × 512 透明 PNG：90 年代版在 `public/characters/<id>/`，PC-98 和 MS-DOS 版分別在其 `pc98/`、`msdos/` 子目錄。復古版使用像素顯示，保留抖色與像素邊緣。
素材由內建 imagegen 繪製；提示與表情排列記錄在 `docs/art/character-generation.json`、`docs/art/retro-character-generation.json`。
首頁和大廳可以選角色；電腦會自動分配不重複的角色；AI 加入時可以用 `character` 參數挑。
碰、槓、吃、胡、放槍時，角色會換表情並冒出對話泡泡；胡牌時結算畫面會有贏家立繪。

- 角色圖鑑：網址後面加 `#gallery`（例如 `http://localhost:7777/#gallery`）可以看到所有角色與表情
- 換成自己的圖：把 90 年代版 PNG 放到 `public/characters/<id>/`，在 `src/lib/characters.js` 該角色加上
  `images: { normal: '/characters/<id>/normal.png', happy: …, surprised: …, sad: …, win: … }`，沒給的表情會用 normal。
  另外兩種畫風請放入對應的 `pc98/`、`msdos/` 子目錄，五張檔名均為 `normal.png`、`happy.png`、`surprised.png`、`sad.png`、`win.png`。
- 新增角色：在 `src/lib/characters.js` 加一筆，並在 `src-tauri/src/game/room.rs` 的 `CHARACTERS` 加上同樣的 id（給電腦分配與 AI 選用）

## 音樂與音效（PC-98 風）

全部用 Web Audio 即時合成，沒有任何音檔：模仿 PC-98 的 YM2608（OPNA）音源——FM 音色、SSG 方波、節奏音源。
曲子用 MML 寫在 `src/lib/audio/music.js`，全部原創：大廳、對局、海底（剩 16 張以內）三首循環曲，
加上胡牌、別人胡、放槍、流局、遊戲結束五段短曲；打牌、吃碰、槓、補花、輪到你、倒數也都有音效。
上方工具列可以調整音樂／音效音量或靜音（瀏覽器規定要先點一下畫面才會出聲）。
`npm test` 會檢查每首循環曲的各音軌小節數是否對齊。

## AI 介面

| 端點 | 用途 |
| --- | --- |
| `POST /mcp` | MCP（Streamable HTTP）。工具：`join_game`、`wait_for_turn`、`take_action`、`get_state`、`send_chat`、`get_rules` |
| `POST /api/join` | `{"name":"Claude","character":"yuehua"}` → `{token, seat, character}`（character 可省略） |
| `GET /api/wait?token=…&timeout=60&format=text` | 長輪詢，輪到你才回傳（文字版牌桌 + 編號動作） |
| `GET /api/state?token=…&format=text` | 立即取得狀態 |
| `POST /api/action?format=text` | `{"token":…, "action_id":3}`，或 `{"type":"discard","tile":"5m"}`、`{"type":"chow","start":"3m"}` |
| `POST /api/chat` | `{"token":…, "text":"安安"}` |
| `GET /llms.txt` | 給 AI 讀的完整說明 |
| `GET /openapi.json` | GPT Actions 用 |

牌代碼：`1m..9m` 萬、`1p..9p` 筒、`1s..9s` 條、`E S W N` 東南西北、`C F P` 中發白、`f1..f8` 春夏秋冬梅蘭竹菊。也接受中文牌名。

每個決定都有時間限制（房主可調）。逾時能胡就自動胡，不然自動「過」或由電腦代打一張，所以 AI 想太久也不會卡住牌局。

## 規則

- 144 張（含 8 花），每人 16 張、莊家 17 張，胡牌為 5 組 + 1 對
- 只能吃上家；胡 > 碰/槓 > 吃；一砲多響時截胡（離放槍者最近者優先）
- 過水、搶槓、海底留 16 張、流局連莊
- 台數：莊家、連N拉N、門清、自摸、門清自摸、三元牌、圈風、門風、正花、花槓、平胡、全求人、半求人、獨聽、海底撈月、河底撈魚、槓上開花、搶槓、三/四/五暗刻、碰碰胡、混一色、清一色、字一色、小/大三元、小/大四喜、嚦咕嚦咕、八仙過海、七搶一、地胡、天胡
- 計分：底 + 台數 × 台錢，放槍一家付、自摸三家付；牽涉莊家的那筆另加莊家與連莊台

### 技能模式（預設關閉）

房主在大廳的「調整規則」或單機設定裡開啟。每個角色有一個技能，除了銀之盾一場一次，其他每局一次：

| 角色 | 技能 | 效果 |
| --- | --- | --- |
| 小翠 | 元氣再來！ | 摸到不想要的牌時，放回牌尾重摸一張 |
| 月華 | 看穿 | 看穿其他三家聽什麼牌，維持到自己再打出三張牌 |
| 阿明 | 逆轉之魂 | 分數最低時，開局自動把配牌換好一點（被動） |
| 糖糖 | 拜託嘛～ | 這一回合打出的牌，別人不能吃、碰、槓（胡牌擋不住） |
| 小春 | 快攻 | 可以吃任何人打出的牌，不限上家 |
| 蘭姐 | 老闆娘的眼力 | 偷看牌牆接下來會被摸走的 3 張牌 |
| 蒼牙 | 鎖定獵物 | 看穿分數最高那家的整副手牌，維持到自己再打出三張牌 |
| 金爺 | 加注 | 胡牌時多算 1 台（被動） |
| 雷歐 | 王命 | 這一回合打出的牌，別人不能胡（吃、碰、槓擋不住） |
| 阿烈 | 身經百戰 | 別人自摸時不用付錢（被動，一場一次） |
| 阿翼 | 銀之盾 | 放槍時不用付錢（被動，一場一次） |

技能定義在 `crates/mahjong-core/src/skills.rs`，介面的名稱與台詞在 `src/lib/characters.js`。

## 程式結構

```
src-tauri/src/
  game/tiles.rs     牌的編碼與解析
  game/rules.rs     胡牌判定、拆解、聽牌、向聽數
  game/scoring.rs   台數計算
  game/room.rs      房間與牌局狀態機（權威）、給 UI / AI 的狀態輸出
  game/bot.rs       內建電腦（向聽數 + 進張數）
  lib.rs            Tauri 指令：start_host / stop_host / host_status
  bin/mahjong-server.rs  無介面伺服器（只是呼叫 crates/mahjong-server）
crates/mahjong-server/src/
  server.rs         axum：WebSocket、HTTP API、MCP、llms.txt、OpenAPI、靜態檔；單一房間或公開多房間
  cli.rs            mahjong-server 的參數（--public 是線上公開伺服器）
src/
  lib/net.svelte.js 連線狀態
  lib/Home / Lobby / Table / Result …  介面
```

註：最新版 Tauri 需要 Rust 1.90，`src-tauri/.cargo/config.toml` 設了 `incompatible-rust-versions = "fallback"`，讓 Rust 1.89 自動選用相容的版本（目前是 Tauri 2.11）。npm 的 `@tauri-apps/api` 也固定在 2.11，跟 Rust 的 `tauri` 版本一致。
