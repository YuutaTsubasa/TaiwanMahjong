# 虛擬台麻館：交接說明（2026-10-26）

之前的工作視窗開在 `C:\Users\User\Repo\TaiwanMahjong`（以及 worktree `TaiwanMahjong-chars`），
現在專案搬到 `C:\Users\User\Repo\OtherRetroGames\TaiwanMahjong`（從 GitHub 重新 clone，是最新版）。
這份文件是給接手的視窗看的：目前狀態、做過的決定、怎麼測、還沒做完的事。

## 目前狀態

- 最新版本 **v0.7.5**（master `3224826`），六個平台的安裝檔都在 Release 上；網頁版是同一份。
- 測試：`npm test`（前端 40 個）、`cargo test --manifest-path crates/mahjong-core/Cargo.toml`（引擎 36 個）、
  `cargo test --manifest-path crates/mahjong-server/Cargo.toml`（MCP 工具 2 個）。全部通過。
- 沒有未完成的程式改動；`docs/art/` 底下沒進版控的是 ChatGPT 的美術草稿（約 600 MB），從舊資料夾複製過來的。

## 舊資料夾（等使用者確認後再刪）

- `C:\Users\User\Repo\TaiwanMahjong`：舊的 checkout，落後 GitHub 60 幾個 commit，還有 ChatGPT 很久以前沒 commit 的改動（都已經被後來的版本取代）。
  有三個 worktree 掛在它底下：`..\TaiwanMahjong-chars`（之前所有工作都在這裡做，已經全部推上 GitHub）、`.claude\worktrees\character-skills`、`.claude\worktrees\single-player`。
- `C:\Users\User\Repo\Volleyball`：排球的舊資料夾，新的在 `OtherRetroGames\Volleyball`（重新 clone，`art-inbox` 的 zip 已複製過去）。有一個 worktree `.claude\worktrees\zealous-faraday-636e60`。
- 刪除方式（使用者確認新資料夾都正常、不再需要舊的之後）：先關掉開在舊資料夾的視窗，再整個資料夾刪掉即可（worktree 也一起）。

## 這段時間做過的事（新到舊，重點）

- **v0.7.5**：手把相容性（非標準手把的十字鍵 `padDpad`、接上／拔掉提示、鍵盤／手把選到的按鈕閃爍外框＋放大）、MCP 工具的四個 annotations 與測試、授權（程式碼 MIT、美術角色台詞 CC BY-NC 4.0）。
- **v0.7.4**：圖片全部轉 WebP（159→39 MB）、進牌桌前預先載入這桌的圖（`src/lib/preload.js`）、立繪淡入。
- **v0.7.3**：手把 Start 打開遊戲選單、LB／RB 在區塊（`data-gamepad-region`）之間跳、第一次按方向鍵的起點規則（`Controls.svelte` 的 `moveFocus`）。
- **v0.7.2**：提示泡泡不再把手把焦點關住（只有 `aria-modal`、`<dialog>`、`.overlay` 才是 modal）。
- **v0.7.x**：提示與牌桌紀錄的牌畫成小牌（`MsgText.svelte`）；日文角色名雷歐・阿明・阿烈・糖糖改片假名。
- **v0.6.x–0.7.0**：多國語言（中／英／日）。原文是 key（`t('原文')`）；引擎訊息改成「中文樣板＋參數」（`crates/mahjong-core/src/msg.rs` → 前端 `src/lib/msg.js`）；台詞翻譯在 `src/lib/i18n/{en,ja}/dialogue.js`，`tests/dialogue-i18n.test.js` 檢查漏翻。
- 更早：四位新角色與技能、手機排版、最終排名、牌河風車排法、GitHub Pages、各平台 CI 打包……見 git log。

## 做過的決定（不要再改回去）

- 角色名：英文 Xiaocui、Yuehua、Ah-Ming、Tangtang、Xiaochun、Sister Lan、Cangya、Uncle Jin、Leo、Ah-Lieh、**Tsubasa**（阿翼）；日文大多照漢字，雷歐→レオ、阿明→アーミン、阿烈→アーリエ、糖糖→タンタン、阿翼→翼，圖鑑顯示中文讀音。
- 台數名稱保留中文、英日文介面加括號說明（`hints.js` 的 `taiLabel`）。
- 單機最終排名的按鈕是「回單機設定」（不是回標題）。
- 不換成 MyGo（Go 的桌面框架）：會失去 Android／iOS、Rust 引擎要重寫；體積問題已用 WebP 解決。
- M8ven（第三方 MCP 評分網站）的信件：不需要回應、不要裝它的 GitHub App。

## 怎麼在本機測

- 建置：`npm run build`（會先用 wasm-pack 編引擎）→ `dist/`。
- 伺服器：`./src-tauri/target/debug/mahjong-server.exe --port 7797 --dist dist`（要先 `cargo build --manifest-path src-tauri/Cargo.toml`）。開著的時候不能重新編譯伺服器（exe 被鎖住），用 `cargo check` 檢查即可。
- 房主測試：伺服器啟動時會印出房主金鑰，在網頁「開設牌局」→「自架伺服器」輸入。
- **手把測試**：Claude 的瀏覽器窗格被藏起來時 `requestAnimationFrame` 不會跑。做法：把 `dist/index.html` 複製成 `dist/padtest.html`，在 `<head>` 最前面插一段 script：
  把 `requestAnimationFrame` 換成 `setTimeout`、`navigator.getGamepads` 換成回傳自己做的假手把物件，再用 `__press('A')` 之類的函式改按鈕狀態（非標準手把把十字鍵放在 `axes[9]`）。
  `:focus` 樣式在窗格沒有系統焦點時不會成立：把 `data-input` 的 CSS 規則複製到測試用的 class 上看計算後的樣式。
- 牌河塞滿測試：在瀏覽器裡複製 `.river` 的子元素到 N 張，量各區塊有沒有重疊。

## 還可以做的事（使用者沒有要求，提過但未決定）

- Android APK 仍有 212 MB：可以依 CPU 架構分開打包（各 < 100 MB）。
- `SECURITY.md`、README 說明 MCP 能做什麼（讓想接 AI 的人比較放心）。
- 簡體中文可以用 OpenCC 自動轉。
- 真實手機手把（Razer Kishi 等）還沒實測；有提示但不能用時，請使用者回報提示裡的手把名稱。
