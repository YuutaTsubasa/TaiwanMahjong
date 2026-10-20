//! 單機版：在瀏覽器（Web Worker）裡跑一整局。你坐一個位置，其他三家是電腦。
//!
//! JavaScript 端用法（見 src/lib/local/worker.js）：
//!   const game = new LocalGame('悠太', 'yuehua', '{"bot_delay_ms":700}');
//!   game.start();
//!   setInterval(() => { if (game.tick()) post(game.view()); }, 150);
//!   game.act('{"action_id":3}');
//! view() 回傳的 JSON 跟伺服器透過 WebSocket 送的狀態格式一樣，介面可以直接沿用。

use mahjong_core::room::{Room, SeatKind, Settings};
use serde_json::json;
use wasm_bindgen::prelude::*;

fn js_err(e: impl ToString) -> JsValue {
    JsValue::from_str(&e.to_string())
}

#[wasm_bindgen]
pub struct LocalGame {
    room: Room,
    seat: usize,
}

#[wasm_bindgen]
impl LocalGame {
    /// 建立單機房間並坐下。`settings_json` 可省略，格式同伺服器的 Settings。
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, character: Option<String>, settings_json: Option<String>) -> Result<LocalGame, JsValue> {
        let mut room = Room::new(Settings::default());
        if let Some(s) = settings_json.filter(|s| !s.trim().is_empty()) {
            room.update_settings(serde_json::from_str(&s).map_err(js_err)?);
        }
        let (seat, _token) = room.join(name, SeatKind::Human, character.as_deref()).map_err(js_err)?;
        room.connect(seat);
        Ok(LocalGame { room, seat })
    }

    /// 開始（或再來一場）：空位自動補電腦
    pub fn start(&mut self) -> Result<(), JsValue> {
        self.room.start_game().map_err(js_err)
    }

    /// 回到大廳（例如要換設定）
    pub fn reset(&mut self) {
        self.room.reset_to_lobby();
    }

    /// 推進時間：電腦出牌、逾時處理。狀態有變時回傳 true。
    pub fn tick(&mut self) -> bool {
        self.room.tick()
    }

    /// 執行動作：`{"action_id":3}` 或 `{"type":"discard","tile":"5m"}`
    pub fn act(&mut self, action_json: &str) -> Result<(), JsValue> {
        let v: serde_json::Value = serde_json::from_str(action_json).map_err(js_err)?;
        let a = self.room.parse_action(self.seat, &v).map_err(js_err)?;
        self.room.act(self.seat, a).map_err(js_err)
    }

    pub fn set_auto(&mut self, on: bool) {
        self.room.set_auto(self.seat, on);
    }

    pub fn set_character(&mut self, id: &str) -> Result<(), JsValue> {
        self.room.set_character(self.seat, id).map_err(js_err)
    }

    pub fn update_settings(&mut self, settings_json: &str) -> Result<(), JsValue> {
        let s: Settings = serde_json::from_str(settings_json).map_err(js_err)?;
        self.room.update_settings(s);
        Ok(())
    }

    pub fn chat(&mut self, text: &str) {
        self.room.push_chat(Some(self.seat), text);
    }

    pub fn seat(&self) -> usize {
        self.seat
    }

    /// 目前狀態（JSON 字串），格式同伺服器送給你的狀態
    pub fn view(&self) -> String {
        let mut v = self.room.view(Some(self.seat));
        v["is_host"] = json!(true);
        v["mode"] = json!("local");
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahjong_core::room::{Action, Phase};

    /// 用託管讓「你」也交給電腦，確認單機版能從頭打到結束
    #[test]
    fn local_game_runs_to_the_end() {
        let mut g = LocalGame::new("悠太", Some("yuehua".into()), Some(r#"{"bot_delay_ms":0}"#.into())).unwrap();
        g.start().unwrap();
        g.set_auto(true);
        let mut steps = 0;
        while g.room.phase != Phase::GameOver {
            // 託管時一局結束要自己按下一局
            if g.room.legal_actions(g.seat).contains(&Action::Next) {
                g.room.act(g.seat, Action::Next).unwrap();
                continue;
            }
            assert!(g.tick(), "stuck at {:?}", g.room.phase);
            steps += 1;
            assert!(steps < 200_000);
        }
        let v: serde_json::Value = serde_json::from_str(&g.view()).unwrap();
        assert_eq!(v["phase"], "game_over");
        assert_eq!(v["mode"], "local");
        assert_eq!(v["seats"][g.seat()]["character"], "yuehua");
    }

    #[test]
    fn rejects_illegal_actions() {
        let mut g = LocalGame::new("悠太", None, None).unwrap();
        // 大廳時沒有任何動作可做（這裡不呼叫 act，因為錯誤型別 JsValue 只能在 wasm 裡建立）
        assert!(g.room.legal_actions(g.seat()).is_empty());
        g.start().unwrap();
        let v: serde_json::Value = serde_json::from_str(&g.view()).unwrap();
        assert_eq!(v["phase"], "playing");
    }
}
