//! 新手提示：輪到玩家做決定時，告訴他電腦會怎麼選、為什麼。
//!
//! 只給「自己」看（放在 view 的 `hint` 欄位），介面可以選擇要不要顯示。
//! 判斷方式跟內建電腦一樣：向聽數（離聽牌還差幾張）+ 進張數（有幾張牌能讓手牌變好）。

use super::bot::{best_discard, visible_counts};
use super::room::{Action, Hand, Room, Step};
use super::rules::*;
use super::tiles::*;
use serde_json::{json, Value};

/// 打完一張後最好的向聽數
fn after_discard(c: &Counts, n: i32) -> i32 {
    let mut best = 99;
    for t in 0..NUM_TYPES {
        if c[t] > 0 {
            let mut c2 = *c;
            c2[t] -= 1;
            best = best.min(shanten(&c2, n));
        }
    }
    best
}

/// 手上這些牌（3n+1 張）聽哪些、各還剩幾張沒出現
pub fn waits_left(h: &Hand, seat: usize) -> Vec<(Tile, u8)> {
    let c = counts(&h.players[seat].concealed);
    let visible = visible_counts(h, seat);
    waits(&c).into_iter().map(|w| (w, 4u8.saturating_sub(visible[w as usize]))).collect()
}

fn tiles_text(ts: &[Tile]) -> String {
    ts.iter().map(|&t| zh(t)).collect::<Vec<_>>().join("、")
}

/// 只有吃碰明槓會破門清（暗槓不算）
fn is_menqing(h: &Hand, seat: usize) -> bool {
    h.players[seat].melds.iter().all(|m| m.kind == super::scoring::MeldKind::ConcealedKong)
}

/// 「離聽牌還差 N 張」或「就聽牌了」
fn distance(s: i32) -> String {
    if s <= 0 {
        "就聽牌了".into()
    } else {
        format!("離聽牌還差 {} 張", s)
    }
}

fn discard_hint(room: &Room, h: &Hand, seat: usize, legal: &[Action]) -> Value {
    if legal.contains(&Action::Tsumo) {
        return json!({ "type": "tsumo", "text": "自摸了！按「自摸」胡牌。" });
    }
    let p = &h.players[seat];
    let n = 5 - p.melds.len() as i32;
    let visible = visible_counts(h, seat);
    let c = counts(&p.concealed);
    let out = best_discard(&p.concealed, n, &visible);
    let mut c2 = c;
    c2[out as usize] -= 1;
    let s = shanten(&c2, n);

    // 摸牌前就已經聽牌了嗎？（吃碰後要打牌時沒有「摸到的牌」）
    let drawn = match h.step {
        Step::Turn { drawn, .. } => drawn,
        _ => None,
    };
    let was_ready = drawn.map_or(false, |d| {
        let mut c0 = c;
        c0[d as usize] -= 1;
        !waits(&c0).is_empty()
    });
    let text = if s <= 0 {
        let w = waits(&c2);
        let left: u32 = w.iter().map(|&t| 4u32.saturating_sub(visible[t as usize] as u32)).sum();
        if w.is_empty() {
            format!("建議打{}。", zh(out))
        } else if was_ready && drawn == Some(out) {
            format!("繼續聽 {}（還剩 {} 張）：摸到的{}用不到，打掉就好。", tiles_text(&w), left, zh(out))
        } else if was_ready {
            format!("打{}，改聽 {}（還剩 {} 張）。", zh(out), tiles_text(&w), left)
        } else {
            format!("打{}就聽牌了，聽 {}（還剩 {} 張）。", zh(out), tiles_text(&w), left)
        }
    } else {
        // 進張：打完之後摸到哪些牌會更接近聽牌
        let mut kinds = 0;
        let mut total = 0;
        for u in 0..NUM_TYPES {
            if c2[u] >= 4 {
                continue;
            }
            let mut c3 = c2;
            c3[u] += 1;
            if shanten(&c3, n) < s {
                kinds += 1;
                total += 4u32.saturating_sub(visible[u] as u32);
            }
        }
        let lonely = {
            let t = out as usize;
            if t >= 27 {
                c[t] == 1
            } else {
                let r = (t % 9) as i32;
                (-2..=2).filter(|&d| d != 0).all(|d| {
                    let x = r + d;
                    !(0..9).contains(&x) || c[(t as i32 + d) as usize] == 0
                })
            }
        };
        let _ = kinds;
        let why = if lonely { "（孤張，用處不大）" } else { "" };
        format!("建議打{}{}：打完{}，有 {} 張牌能讓手牌變好。", zh(out), why, distance(s), total)
    };
    let mut v = json!({ "type": "discard", "tile": code(out), "text": text });
    if legal.contains(&Action::Skill) {
        if let Some(k) = room.skill_of(seat) {
            v["skill"] = json!(format!("也可以發動技能「{}」：{}", k.name(), k.desc()));
        }
    }
    v
}

fn claim_hint(room: &Room, h: &Hand, seat: usize, legal: &[Action]) -> Value {
    if legal.contains(&Action::Ron) {
        return json!({ "type": "claim", "advice": "ron", "text": "胡！這張就是你在等的牌。" });
    }
    let tile = match h.step {
        Step::Claims { tile, .. } | Step::RobKong { tile, .. } => tile,
        _ => return Value::Null,
    };
    let p = &h.players[seat];
    let n = 5 - p.melds.len() as i32;
    let c = counts(&p.concealed);
    let s0 = shanten(&c, n);
    let honor_value = tile == RED
        || tile == RED + 1
        || tile == RED + 2
        || tile == EAST + room.seat_wind(seat)
        || tile == EAST + room.round_wind;

    // 每個選項做下去之後的向聽數，挑最好的
    let mut best: Option<(&Action, i32)> = None;
    for a in legal {
        let s1 = match a {
            Action::Kong => {
                let mut c2 = c;
                c2[tile as usize] -= 3;
                shanten(&c2, n - 1)
            }
            Action::Pung => {
                let mut c2 = c;
                c2[tile as usize] -= 2;
                after_discard(&c2, n - 1)
            }
            Action::Chow(st) => {
                let mut c2 = c;
                for x in *st..st + 3 {
                    if x != tile {
                        c2[x as usize] -= 1;
                    }
                }
                after_discard(&c2, n - 1)
            }
            _ => continue,
        };
        if best.map_or(true, |b| s1 < b.1) {
            best = Some((a, s1));
        }
    }
    let name = |a: &Action| match a {
        Action::Kong => "槓",
        Action::Pung => "碰",
        _ => "吃",
    };
    let lose_menqing = if is_menqing(h, seat) { "不過會失去門清（1 台）。" } else { "" };
    match best {
        Some((a, s1)) if s1 < s0 || (matches!(a, Action::Pung | Action::Kong) && honor_value && s1 <= s0) => {
            let mut v = json!({
                "type": "claim",
                "advice": match a { Action::Kong => "kong", Action::Pung => "pung", _ => "chow" },
                "text": format!(
                    "建議{}：{}之後{}{}{}",
                    name(a),
                    name(a),
                    distance(s1),
                    if honor_value && !matches!(a, Action::Chow(_)) { format!("，而且{}是有台的字牌。", zh(tile)) } else { "。".into() },
                    lose_menqing
                ),
            });
            if let Action::Chow(st) = a {
                v["start"] = json!(code(*st));
            }
            v
        }
        Some((a, _)) => json!({
            "type": "claim",
            "advice": "pass",
            "text": format!("建議過：{}了也沒有比較快聽牌（現在{}）{}", name(a), distance(s0), if is_menqing(h, seat) { "，留著門清比較好。" } else { "。" }),
        }),
        None => json!({ "type": "claim", "advice": "pass", "text": "建議過。" }),
    }
}

/// 給這個座位的提示；沒有輪到他做決定時是 null。
pub fn hint(room: &Room, seat: usize) -> Value {
    let Some(h) = room.hand.as_ref() else { return Value::Null };
    let legal = room.legal_actions(seat);
    if legal.is_empty() {
        return Value::Null;
    }
    if legal.iter().any(|a| matches!(a, Action::Discard(_))) {
        discard_hint(room, h, seat, &legal)
    } else {
        claim_hint(room, h, seat, &legal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::{PlayerState, Settings};

    /// 座位 0 輪到出牌，四家的手牌由參數指定
    fn room_with(hands: [Vec<Tile>; 4]) -> Room {
        let mut room = Room::new(Settings { bot_delay_ms: 0, turn_timeout: 0, claim_timeout: 0, ..Default::default() });
        room.start_game().unwrap();
        let h = room.hand.as_mut().unwrap();
        for (s, hand) in hands.into_iter().enumerate() {
            h.players[s] = PlayerState { concealed: hand, ..Default::default() };
        }
        h.step = Step::Turn { seat: 0, drawn: None, after_kong: false, claimed: false };
        room
    }

    #[test]
    fn discard_hint_picks_lonely_honor() {
        // 1m2m3m 4p5p6p 7s8s9s 1p1p 2s3s 5m6m + 東、北（孤張）
        let room = room_with([
            vec![0, 1, 2, 12, 13, 14, 24, 25, 26, 9, 9, 19, 20, 4, 5, 27, 30],
            vec![],
            vec![],
            vec![],
        ]);
        let v = hint(&room, 0);
        assert_eq!(v["type"], "discard");
        let t = v["tile"].as_str().unwrap();
        assert!(t == code(27) || t == code(30), "應該打孤張字牌，結果是 {}", t);
        assert!(v["text"].as_str().unwrap().contains("孤張"), "{}", v["text"]);
        assert!(hint(&room, 1).is_null(), "沒輪到的人沒有提示");
    }

    #[test]
    fn claim_hint_pungs_dragon() {
        let mut room = room_with([
            vec![31, 0, 0, 0],
            vec![27, 28, 29, 30],
            vec![31, 31, 0, 1, 2, 12, 13, 14, 24, 25, 26, 9, 9, 19, 20, 33],
            vec![28, 29, 30, 33],
        ]);
        room.act(0, Action::Discard(31)).unwrap();
        let v = hint(&room, 2);
        assert_eq!(v["type"], "claim");
        assert_eq!(v["advice"], "pung");
        assert!(v["text"].as_str().unwrap().contains("門清"));
    }

    /// 已經聽牌，摸到用不到的牌：說「繼續聽」，不要再說「打這張就聽牌了」
    #[test]
    fn already_ready_says_keep_waiting() {
        // 一二三萬 四五六筒 七八九條 一一一筒 東東 三四萬（聽二萬、五萬），剛摸到北
        let mut room = room_with([
            vec![0, 1, 2, 12, 13, 14, 24, 25, 26, 9, 9, 9, 27, 27, 2, 3, 30],
            vec![],
            vec![],
            vec![],
        ]);
        room.hand.as_mut().unwrap().step = Step::Turn { seat: 0, drawn: Some(30), after_kong: false, claimed: false };
        let v = hint(&room, 0);
        let text = v["text"].as_str().unwrap();
        assert_eq!(v["tile"], code(30));
        assert!(text.starts_with("繼續聽"), "{}", text);
    }

    #[test]
    fn waits_left_counts_visible_tiles() {
        let mut room = room_with([vec![2, 30, 31, 32], vec![3, 4, 27, 27], vec![], vec![]]);
        room.hand.as_mut().unwrap().players[3].discards = vec![5, 5];
        let h = room.hand.as_ref().unwrap();
        // 聽三萬、六萬：三萬只在座位 0 手上（別人看不到，算還剩 4 張），六萬河裡已經有 2 張
        assert_eq!(waits_left(h, 1), vec![(2, 4), (5, 2)]);
    }
}
