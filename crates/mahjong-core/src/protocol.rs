//! 玩家連線的訊息協定（WebSocket 或 WebRTC 都一樣）。
//!
//! 玩家送：hello（加入／重連）、action、character、auto、chat、leave、host（房主指令）
//! 回給玩家：welcome、error，以及 { type: "state", state } 牌桌狀態。
//! 伺服器（crates/mahjong-server）和瀏覽器裡的 P2P 房主（crates/mahjong-wasm）共用這份，
//! 兩邊的行為才會一模一樣。

use crate::room::{Room, SeatKind, Settings};
use serde_json::{json, Value};

/// 一個連線的狀態：坐在哪個位置、是不是房主
#[derive(Default, Clone, Debug)]
pub struct ClientCtx {
    pub seat: Option<usize>,
    pub is_host: bool,
}

/// 這個連線看到的牌桌狀態
pub fn client_view(room: &Room, ctx: &ClientCtx) -> Value {
    let mut v = room.view(ctx.seat);
    v["is_host"] = json!(ctx.is_host);
    v
}

/// 連線中斷（座位保留，之後可以用 token 重連）
pub fn client_disconnected(room: &mut Room, ctx: &ClientCtx) {
    if let Some(s) = ctx.seat {
        room.disconnect(s);
    }
}

pub fn err(msg: impl Into<String>) -> Value {
    json!({"type": "error", "message": msg.into()})
}

/// 處理一則玩家訊息。回傳（要回給這個玩家的訊息, 牌桌狀態有沒有改變）。
/// 改變時呼叫端要把新狀態送給所有人；不管有沒有改變，都要把狀態回給送訊息的這個玩家。
pub fn handle_client_msg(room: &mut Room, host_key: &str, ctx: &mut ClientCtx, v: &Value) -> (Vec<Value>, bool) {
    let ty = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
    match ty {
        "hello" => {
            if let Some(k) = v.get("host_key").and_then(|x| x.as_str()) {
                ctx.is_host = !k.is_empty() && k == host_key;
            }
            if let Some(s) = ctx.seat.take() {
                room.disconnect(s);
            }
            let token = v.get("token").and_then(|x| x.as_str()).unwrap_or("");
            if let Some(s) = room.seat_of_token(token) {
                ctx.seat = Some(s);
                room.connect(s);
                if ctx.is_host {
                    room.host_seat = Some(s);
                }
                return (vec![json!({"type": "welcome", "seat": s, "token": token, "is_host": ctx.is_host})], true);
            }
            if v.get("spectate").and_then(|x| x.as_bool()).unwrap_or(false) {
                return (vec![json!({"type": "welcome", "seat": null, "token": null, "is_host": ctx.is_host})], false);
            }
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
            let character = v.get("character").and_then(|x| x.as_str());
            match room.join(name, SeatKind::Human, character) {
                Ok((s, tok)) => {
                    ctx.seat = Some(s);
                    room.connect(s);
                    if ctx.is_host {
                        room.host_seat = Some(s);
                    }
                    (vec![json!({"type": "welcome", "seat": s, "token": tok, "is_host": ctx.is_host})], true)
                }
                Err(e) => (
                    vec![
                        err(format!("{}（以觀戰模式進入）", e)),
                        json!({"type": "welcome", "seat": null, "token": null, "is_host": ctx.is_host}),
                    ],
                    false,
                ),
            }
        }
        "action" => {
            let Some(s) = ctx.seat else { return (vec![err("你沒有座位")], false) };
            match room.parse_action(s, v).and_then(|a| room.act(s, a)) {
                Ok(()) => (vec![], true),
                Err(e) => (vec![err(e)], false),
            }
        }
        "character" => {
            let Some(s) = ctx.seat else { return (vec![err("你沒有座位")], false) };
            let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("");
            match room.set_character(s, id) {
                Ok(()) => (vec![], true),
                Err(e) => (vec![err(e)], false),
            }
        }
        "auto" => {
            let Some(s) = ctx.seat else { return (vec![err("你沒有座位")], false) };
            let on = v.get("on").and_then(|x| x.as_bool()).unwrap_or(false);
            room.set_auto(s, on);
            (vec![], true)
        }
        "chat" => {
            let text = v.get("text").and_then(|x| x.as_str()).unwrap_or("");
            room.push_chat(ctx.seat, text);
            (vec![], true)
        }
        "leave" => {
            if let Some(s) = ctx.seat.take() {
                let _ = room.remove_seat(s);
            }
            (vec![json!({"type": "welcome", "seat": null, "token": null, "is_host": ctx.is_host})], true)
        }
        "host" => {
            if !ctx.is_host {
                return (vec![err("只有房主可以這樣做")], false);
            }
            let seat = v.get("seat").and_then(|x| x.as_u64()).map(|x| x as usize);
            let r = match v.get("cmd").and_then(|x| x.as_str()).unwrap_or("") {
                "add_bot" => room.add_bot(seat).map(|_| ()),
                "remove" => room.remove_seat(seat.unwrap_or(9)),
                "start" => room.start_game(),
                "reset" => {
                    room.reset_to_lobby();
                    Ok(())
                }
                "settings" => match serde_json::from_value::<Settings>(v.get("settings").cloned().unwrap_or(Value::Null)) {
                    Ok(s) => {
                        room.update_settings(s);
                        Ok(())
                    }
                    Err(e) => Err(format!("設定格式錯誤：{}", e)),
                },
                other => Err(format!("未知指令 {}", other)),
            };
            match r {
                Ok(()) => (vec![], true),
                Err(e) => (vec![err(e)], false),
            }
        }
        _ => (vec![err("未知訊息")], false),
    }
}
