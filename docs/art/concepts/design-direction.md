# 虛擬台麻館 — Virtual Taiwan Mahjong Parlor

2026-10-20：使用者指定遊戲名稱為「虛擬台麻館」，英文為「Virtual Taiwan Mahjong Parlor」。

目前為畫面概念探索，尚未將概念圖實作為遊戲介面。

## 標題畫面

- 已繪製提案：`title-virtual-taiwan-mahjong-parlor.png`。
- 90 年代賽璐璐動畫風，夜晚的台灣街坊麻將館，暖色吊燈、木質家具、綠色牌桌與窗外霓虹。
- 左側書法標題及直式選單：開設牌局、加入牌局、角色圖鑑、遊戲設定。
- 右側小翠邀請玩家入席，月華、阿明與蘭姐陪襯。
- 底部保留 90 年代／PC-98／MS-DOS 風格選項。
- 名稱已由使用者指定；其他版面細節仍屬提案。

## 下一張提案：開設牌局

使用者要求先繪製頁面。延續標題畫面的美術方向，左側整合玩家名稱、六位角色選擇、通訊埠及開設按鈕，右側呈現所選角色與空牌桌。開房後進大廳的既有流程保持不變，規則設定在大廳調整。

## 完整流程概念圖

以下皆為內建 image_gen 繪製的 90 年代版本概念圖，尚未實作成介面。使用者明確要求 subagent 分工完成其餘五頁，圖檔均已目視檢查。

| 畫面 | 圖檔 | 設計重點 |
| --- | --- | --- |
| 標題 | `title-virtual-taiwan-mahjong-parlor.png` | 品牌與主選單 |
| 開設牌局 | `host-game-anime90.png` | 名稱、六角色、通訊埠 |
| 房主大廳 | `lobby-host-anime90.png` | 座位、邀請、聊天、規則及開始遊戲 |
| 加入牌局 | `join-game-anime90.png` | 名稱、角色、房間位址、加入／觀戰 |
| 角色圖鑑 | `character-gallery-anime90.png` | 六角色導覽、大立繪、五表情及台詞 |
| 遊戲設定 | `settings-anime90.png` | 三畫風預覽、音樂／音效音量及靜音 |
| 訪客等待大廳 | `lobby-guest-anime90.png` | 三席入座、一席空位；等待房主、唯讀規則 |
| 正式對局 | `gameplay-anime90-v2.png` | 四方座位、棄牌區、17 張手牌、選牌標示及紀錄／聊天 |

`Lobby.svelte` 已有非房主的等待狀態；這張等待頁是同一大廳的客人視角，沒有新增準備按鈕或全員準備機制。房主才有移除／加入電腦、修改規則及開始遊戲等權限。

生成提示見 `join-wait-prompts.json`、`gallery-prompt.json`、`settings-prompt.json`、`gameplay-prompt.txt`。對局初稿 `gameplay-anime90.png` 保留，v2 修正部分筒子牌面。牌圖中的數量、花色、文字與細小控制圖示均屬概念示意，實作應使用既有 Tile 元件及真實遊戲狀態；圖鑑的喇叭圖示是美術示意，不代表已具備角色語音。
