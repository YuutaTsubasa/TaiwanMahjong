//! 台數計算（台灣十六張常見規則）。

use super::rules::*;
use super::tiles::*;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeldKind {
    Chow,
    Pung,
    Kong,
    ConcealedKong,
}

#[derive(Clone, Debug, Serialize)]
pub struct Meld {
    pub kind: MeldKind,
    pub tiles: Vec<Tile>,
    /// 從哪個座位吃/碰/槓來的
    pub from: Option<usize>,
}

impl Meld {
    pub fn base(&self) -> Tile {
        self.tiles[0]
    }
    pub fn is_triplet(&self) -> bool {
        self.kind != MeldKind::Chow
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowerWin {
    EightImmortals,
    SevenRobOne,
}

pub struct WinCtx<'a> {
    /// 含胡的那張
    pub concealed: &'a [Tile],
    pub melds: &'a [Meld],
    pub flowers: &'a [Tile],
    pub win_tile: Option<Tile>,
    pub self_draw: bool,
    /// 0=東 1=南 2=西 3=北
    pub seat_wind: u8,
    pub round_wind: u8,
    pub after_kong: bool,
    pub rob_kong: bool,
    pub last_tile: bool,
    pub heaven: bool,
    pub earth: bool,
    pub flower_win: Option<FlowerWin>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScoreItem {
    pub name: String,
    pub tai: u32,
}

fn item(v: &mut Vec<ScoreItem>, name: &str, tai: u32) {
    v.push(ScoreItem { name: name.to_string(), tai });
}

/// 跟牌型無關的台（花、自摸、門清、海底…）
fn common_items(ctx: &WinCtx, v: &mut Vec<ScoreItem>) {
    let menqing = ctx.melds.iter().all(|m| m.kind == MeldKind::ConcealedKong);
    if ctx.heaven {
        item(v, "天胡", 24);
    } else if ctx.earth {
        item(v, "地胡", 16);
    }
    if ctx.flower_win.is_none() {
        if menqing && ctx.self_draw {
            item(v, "門清自摸", 3);
        } else if menqing {
            item(v, "門清", 1);
        } else if ctx.self_draw {
            item(v, "自摸", 1);
        }
    }
    // 花
    let has = |f: Tile| ctx.flowers.contains(&f);
    for group in 0..2u8 {
        let base = FLOWER_BASE + group * 4;
        if (0..4).all(|k| has(base + k)) {
            item(v, if group == 0 { "花槓(春夏秋冬)" } else { "花槓(梅蘭竹菊)" }, 2);
        } else if has(base + ctx.seat_wind) {
            item(v, &format!("正花({})", zh(base + ctx.seat_wind)), 1);
        }
    }
    if ctx.last_tile {
        item(v, if ctx.self_draw { "海底撈月" } else { "河底撈魚" }, 1);
    }
    if ctx.after_kong && ctx.self_draw {
        item(v, "槓上開花", 1);
    }
    if ctx.rob_kong {
        item(v, "搶槓", 1);
    }
}

/// 計算胡牌者的台數（不含莊家、連莊，那兩項依付款人另計）。
pub fn score(ctx: &WinCtx) -> Vec<ScoreItem> {
    let mut common = Vec::new();
    common_items(ctx, &mut common);

    if let Some(fw) = ctx.flower_win {
        let mut v = vec![ScoreItem {
            name: match fw {
                FlowerWin::EightImmortals => "八仙過海".into(),
                FlowerWin::SevenRobOne => "七搶一".into(),
            },
            tai: 8,
        }];
        v.extend(common.into_iter().filter(|i| !i.name.starts_with("正花") && !i.name.starts_with("花槓")));
        return v;
    }

    let c = counts(ctx.concealed);
    let mut best: Option<Vec<ScoreItem>> = None;
    let mut consider = |shape: Vec<ScoreItem>| {
        let total: u32 = shape.iter().map(|i| i.tai).sum();
        let cur: u32 = best.as_ref().map(|b| b.iter().map(|i| i.tai).sum()).unwrap_or(0);
        if best.is_none() || total > cur {
            best = Some(shape);
        }
    };

    // 聽幾張（拿掉胡的那張後）
    let wait_count = match ctx.win_tile {
        Some(w) => {
            let mut c2 = c;
            if c2[w as usize] > 0 {
                c2[w as usize] -= 1;
            }
            waits(&c2).len()
        }
        None => 0,
    };
    let single_wait = wait_count == 1;

    for d in decompositions(&c) {
        consider(shape_items(ctx, &d, single_wait));
    }
    if is_lilikuli(&c) {
        let mut v = Vec::new();
        item(&mut v, "嚦咕嚦咕", 8);
        if single_wait {
            item(&mut v, "獨聽", 1);
        }
        consider(v);
    }
    let mut v = best.unwrap_or_default();
    v.splice(0..0, common);
    v
}

fn shape_items(ctx: &WinCtx, d: &Decomp, single_wait: bool) -> Vec<ScoreItem> {
    let mut v = Vec::new();
    // 所有組：(是否刻子, 基準牌, 是否暗)
    let mut sets: Vec<(bool, Tile, bool)> = Vec::new();
    let win_in_chow_or_pair = ctx.win_tile.map_or(false, |w| {
        d.pair == w
            || d.sets.iter().any(|s| matches!(*s, Set::Chow(b) if w >= b && w <= b + 2))
    });
    for s in &d.sets {
        match *s {
            Set::Chow(b) => sets.push((false, b, true)),
            Set::Pung(t) => {
                // 放槍胡的那張若只能湊成這個刻子，就不算暗刻
                let concealed = ctx.self_draw || ctx.win_tile != Some(t) || win_in_chow_or_pair;
                sets.push((true, t, concealed));
            }
        }
    }
    for m in ctx.melds {
        sets.push((m.is_triplet(), m.base(), m.kind == MeldKind::ConcealedKong));
    }

    let all_tiles: Vec<Tile> = ctx
        .concealed
        .iter()
        .copied()
        .chain(ctx.melds.iter().flat_map(|m| m.tiles.iter().copied()))
        .collect();
    let has_honor = all_tiles.iter().any(|&t| is_honor(t));
    let suits: std::collections::BTreeSet<u8> =
        all_tiles.iter().filter(|&&t| is_suited(t)).map(|&t| t / 9).collect();

    // 一色
    if suits.is_empty() {
        item(&mut v, "字一色", 16);
    } else if suits.len() == 1 && !has_honor {
        item(&mut v, "清一色", 8);
    } else if suits.len() == 1 {
        item(&mut v, "混一色", 4);
    }

    let all_triplets = sets.iter().all(|s| s.0);
    if all_triplets && !suits.is_empty() {
        item(&mut v, "碰碰胡", 4);
    }

    // 三元、四喜
    let dragon_sets = sets.iter().filter(|s| s.0 && is_dragon(s.1)).count();
    let wind_sets = sets.iter().filter(|s| s.0 && is_wind(s.1)).count();
    let big_dragons = dragon_sets == 3;
    let small_dragons = dragon_sets == 2 && is_dragon(d.pair);
    let big_winds = wind_sets == 4;
    let small_winds = wind_sets == 3 && is_wind(d.pair);
    if big_dragons {
        item(&mut v, "大三元", 8);
    } else if small_dragons {
        item(&mut v, "小三元", 4);
    }
    if big_winds {
        item(&mut v, "大四喜", 16);
    } else if small_winds {
        item(&mut v, "小四喜", 8);
    }
    for s in &sets {
        if !s.0 || !is_honor(s.1) {
            continue;
        }
        if is_dragon(s.1) && !big_dragons && !small_dragons {
            item(&mut v, &format!("三元牌({})", zh(s.1)), 1);
        }
        if is_wind(s.1) && !big_winds && !small_winds {
            if s.1 == EAST + ctx.round_wind {
                item(&mut v, &format!("圈風({})", zh(s.1)), 1);
            }
            if s.1 == EAST + ctx.seat_wind {
                item(&mut v, &format!("門風({})", zh(s.1)), 1);
            }
        }
    }

    // 暗刻
    let concealed_triplets = sets.iter().filter(|s| s.0 && s.2).count();
    match concealed_triplets {
        3 => item(&mut v, "三暗刻", 2),
        4 => item(&mut v, "四暗刻", 5),
        5 => item(&mut v, "五暗刻", 8),
        _ => {}
    }

    // 全求人 / 半求人
    let all_exposed = ctx.melds.len() == 5 && ctx.melds.iter().all(|m| m.kind != MeldKind::ConcealedKong);
    if all_exposed {
        if ctx.self_draw {
            item(&mut v, "半求人", 1);
        } else {
            item(&mut v, "全求人", 2);
        }
    }

    // 平胡：無字無花、全順子、非自摸、聽兩面以上
    let ping = !has_honor
        && ctx.flowers.is_empty()
        && sets.iter().all(|s| !s.0)
        && !ctx.self_draw
        && !single_wait;
    if ping {
        item(&mut v, "平胡", 2);
    }

    if single_wait && !all_exposed {
        item(&mut v, "獨聽", 1);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Vec<Tile> {
        s.split_whitespace().map(|x| parse(x).unwrap()).collect()
    }

    fn ctx<'a>(concealed: &'a [Tile], melds: &'a [Meld], flowers: &'a [Tile], w: &str, self_draw: bool) -> WinCtx<'a> {
        WinCtx {
            concealed,
            melds,
            flowers,
            win_tile: parse(w),
            self_draw,
            seat_wind: 0,
            round_wind: 0,
            after_kong: false,
            rob_kong: false,
            last_tile: false,
            heaven: false,
            earth: false,
            flower_win: None,
        }
    }

    fn total(v: &[ScoreItem]) -> u32 {
        v.iter().map(|i| i.tai).sum()
    }

    fn names(v: &[ScoreItem]) -> Vec<String> {
        v.iter().map(|i| i.name.clone()).collect()
    }

    #[test]
    fn pinghu() {
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 2p 3p 2s 3s 4s 5s 5s");
        let s = score(&ctx(&h, &[], &[], "4s", false));
        assert!(names(&s).contains(&"平胡".to_string()), "{:?}", s);
        assert!(names(&s).contains(&"門清".to_string()));
    }

    #[test]
    fn qingyise_pengpeng() {
        let h = p("1m 1m 1m 3m 3m 3m 5m 5m 5m 7m 7m 7m 9m 9m 9m 2m 2m");
        let s = score(&ctx(&h, &[], &[], "2m", true));
        let n = names(&s);
        assert!(n.contains(&"清一色".to_string()));
        assert!(n.contains(&"碰碰胡".to_string()));
        assert!(n.contains(&"五暗刻".to_string()));
        assert!(n.contains(&"門清自摸".to_string()));
        // 也聽 1m（111+123+33），所以不是獨聽
        assert_eq!(total(&s), 3 + 8 + 4 + 8);
    }

    #[test]
    fn ron_on_pung_not_concealed() {
        // 胡 9m 只能成刻 → 四暗刻而非五暗刻
        let h = p("1m 1m 1m 3m 3m 3m 5p 5p 5p 7s 7s 7s 9m 9m 9m E E");
        let s = score(&ctx(&h, &[], &[], "9m", false));
        let n = names(&s);
        assert!(n.contains(&"四暗刻".to_string()), "{:?}", n);
    }

    #[test]
    fn dragons() {
        let h = p("C C C F F F P P 1m 2m 3m 4m 5m 6m 7s 8s 9s");
        let melds = [];
        let s = score(&ctx(&h, &melds, &[], "1m", false));
        let n = names(&s);
        assert!(n.contains(&"小三元".to_string()), "{:?}", n);
    }

    #[test]
    fn flowers() {
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 2p 3p 2s 3s 4s E E");
        let fl = [34, 35, 36, 37, 38];
        let s = score(&ctx(&h, &[], &fl, "4s", false));
        let n = names(&s);
        assert!(n.contains(&"花槓(春夏秋冬)".to_string()));
        assert!(n.contains(&"正花(梅)".to_string()));
    }
}
