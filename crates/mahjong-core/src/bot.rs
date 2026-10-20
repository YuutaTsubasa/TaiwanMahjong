//! 內建電腦：以向聽數 + 進張數選牌，簡單但不笨。

use super::room::{Action, Hand, Room, Step};
use super::rules::*;
use super::skills::Skill;
use super::tiles::*;

pub fn visible_counts(h: &Hand, seat: usize) -> Counts {
    let mut v = counts(&h.players[seat].concealed);
    for p in &h.players {
        for &t in &p.discards {
            v[t as usize] += 1;
        }
        for m in &p.melds {
            for &t in &m.tiles {
                v[t as usize] += 1;
            }
        }
    }
    v
}

/// 孤張程度，越高越該先丟
fn isolation(c: &Counts, t: usize) -> i32 {
    if t >= 27 {
        return if c[t] == 1 { 10 } else { 0 };
    }
    let r = t % 9;
    let near = |d: i32| -> u8 {
        let x = r as i32 + d;
        if (0..9).contains(&x) {
            c[(t as i32 + d) as usize]
        } else {
            0
        }
    };
    let neighbors = near(-2) + near(-1) + near(1) + near(2);
    let edge = if r == 0 || r == 8 { 2 } else if r == 1 || r == 7 { 1 } else { 0 };
    (if neighbors == 0 { 6 } else { 0 }) + edge
}

pub fn best_discard(concealed: &[Tile], sets_needed: i32, visible: &Counts) -> Tile {
    let c = counts(concealed);
    let mut scored: Vec<(Tile, i32)> = Vec::new();
    for t in 0..NUM_TYPES {
        if c[t] == 0 {
            continue;
        }
        let mut c2 = c;
        c2[t] -= 1;
        scored.push((t as Tile, shanten(&c2, sets_needed)));
    }
    let min_s = scored.iter().map(|x| x.1).min().unwrap_or(0);
    let mut best: Option<(Tile, i32, i32)> = None;
    for &(t, s) in scored.iter().filter(|x| x.1 == min_s) {
        let mut c2 = c;
        c2[t as usize] -= 1;
        let mut ukeire = 0;
        for u in 0..NUM_TYPES {
            if c2[u] >= 4 {
                continue;
            }
            c2[u] += 1;
            if shanten(&c2, sets_needed) < s {
                ukeire += (4 - visible[u] as i32).max(0);
            }
            c2[u] -= 1;
        }
        let iso = isolation(&c, t as usize);
        let key = (ukeire, iso);
        if best.map_or(true, |b| key > (b.1, b.2)) {
            best = Some((t, ukeire, iso));
        }
    }
    best.map(|b| b.0).unwrap_or(concealed[0])
}

/// 技能模式：電腦什麼時候發動自己的技能
fn wants_skill(room: &Room, h: &Hand, seat: usize, n: i32, visible: &Counts) -> bool {
    let p = &h.players[seat];
    let c = counts(&p.concealed);
    match room.skill_of(seat) {
        // 小翠：摸到的牌正好是最該丟的那張，而且還沒聽牌，就重摸
        Some(Skill::Redraw) => {
            let Step::Turn { drawn: Some(d), .. } = h.step else { return false };
            let out = best_discard(&p.concealed, n, visible);
            let mut c2 = c;
            c2[out as usize] -= 1;
            out == d && shanten(&c2, n) > 0
        }
        // 月華、蘭姐：牌局過了一半再用
        Some(Skill::SeeWaits) | Some(Skill::Peek) => h.drawable() < 45,
        // 糖糖：快聽牌時保護自己打出的牌
        Some(Skill::NoClaim) => shanten(&c, n) <= 1 && h.drawable() < 55,
        _ => false,
    }
}

/// 一般選最佳捨牌；月華看穿時避開別人聽的牌（向聽數最多差一）
fn safe_discard(h: &Hand, seat: usize, n: i32, visible: &Counts) -> Tile {
    let p = &h.players[seat];
    let best = best_discard(&p.concealed, n, visible);
    if p.see_waits_left == 0 {
        return best;
    }
    let mut danger = [false; NUM_TYPES];
    for (s, q) in h.players.iter().enumerate() {
        if s != seat && q.concealed.len() % 3 == 1 {
            for w in waits(&counts(&q.concealed)) {
                danger[w as usize] = true;
            }
        }
    }
    if !danger[best as usize] {
        return best;
    }
    let c = counts(&p.concealed);
    let s_best = {
        let mut c2 = c;
        c2[best as usize] -= 1;
        shanten(&c2, n)
    };
    (0..NUM_TYPES)
        .filter(|&t| c[t] > 0 && !danger[t])
        .map(|t| {
            let mut c2 = c;
            c2[t] -= 1;
            (shanten(&c2, n), t as Tile)
        })
        .filter(|&(s, _)| s <= s_best + 1)
        .min()
        .map(|(_, t)| t)
        .unwrap_or(best)
}

pub fn choose(room: &Room, seat: usize, legal: &[Action]) -> Action {
    for pref in [Action::Tsumo, Action::Ron, Action::Next] {
        if legal.contains(&pref) {
            return pref;
        }
    }
    let Some(h) = room.hand.as_ref() else {
        return legal[0].clone();
    };
    let p = &h.players[seat];
    let n = 5 - p.melds.len() as i32;
    let visible = visible_counts(h, seat);

    if legal.iter().any(|a| matches!(a, Action::Discard(_))) {
        if legal.contains(&Action::Skill) && wants_skill(room, h, seat, n, &visible) {
            return Action::Skill;
        }
        for a in legal {
            if matches!(a, Action::ConcealedKong(_) | Action::AddedKong(_)) {
                return a.clone();
            }
        }
        return Action::Discard(safe_discard(h, seat, n, &visible));
    }

    let tile = match &h.step {
        Step::Claims { tile, .. } | Step::RobKong { tile, .. } => *tile,
        _ => return Action::Pass,
    };
    let c = counts(&p.concealed);
    let s0 = shanten(&c, n);
    let honor_value = tile == RED
        || tile == RED + 1
        || tile == RED + 2
        || tile == EAST + room.seat_wind(seat)
        || tile == EAST + room.round_wind;
    // 打完一張後的最佳向聽數
    let after = |c2: &Counts, n2: i32| -> i32 {
        let mut best = 99;
        for t in 0..NUM_TYPES {
            if c2[t] > 0 {
                let mut c3 = *c2;
                c3[t] -= 1;
                best = best.min(shanten(&c3, n2));
            }
        }
        best
    };
    for a in legal {
        let ok = match a {
            Action::Kong => {
                let mut c2 = c;
                c2[tile as usize] -= 3;
                shanten(&c2, n - 1) <= s0
            }
            Action::Pung => {
                let mut c2 = c;
                c2[tile as usize] -= 2;
                let s1 = after(&c2, n - 1);
                s1 < s0 || (honor_value && s1 <= s0)
            }
            Action::Chow(st) => {
                let mut c2 = c;
                for x in *st..st + 3 {
                    if x != tile {
                        c2[x as usize] -= 1;
                    }
                }
                after(&c2, n - 1) < s0
            }
            _ => false,
        };
        if ok {
            return a.clone();
        }
    }
    Action::Pass
}
