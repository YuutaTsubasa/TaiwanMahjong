//! 牌的編碼。
//!
//! 0..=8   一萬..九萬 (1m..9m)
//! 9..=17  一筒..九筒 (1p..9p)
//! 18..=26 一條..九條 (1s..9s)
//! 27..=30 東南西北 (E S W N)
//! 31..=33 中發白 (C F P)
//! 34..=41 春夏秋冬梅蘭竹菊 (f1..f8)

pub type Tile = u8;

pub const NUM_TYPES: usize = 34;
pub const EAST: Tile = 27;
pub const RED: Tile = 31;
pub const FLOWER_BASE: Tile = 34;

const NUM_ZH: [&str; 9] = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];
const SUIT_ZH: [&str; 3] = ["萬", "筒", "條"];
const SUIT_CODE: [&str; 3] = ["m", "p", "s"];
const HONOR_ZH: [&str; 7] = ["東", "南", "西", "北", "中", "發", "白"];
const HONOR_CODE: [&str; 7] = ["E", "S", "W", "N", "C", "F", "P"];
const FLOWER_ZH: [&str; 8] = ["春", "夏", "秋", "冬", "梅", "蘭", "竹", "菊"];
pub const WIND_ZH: [&str; 4] = ["東", "南", "西", "北"];

pub fn is_flower(t: Tile) -> bool {
    (34..42).contains(&t)
}

pub fn is_honor(t: Tile) -> bool {
    (27..34).contains(&t)
}

pub fn is_suited(t: Tile) -> bool {
    t < 27
}

pub fn is_dragon(t: Tile) -> bool {
    (31..34).contains(&t)
}

pub fn is_wind(t: Tile) -> bool {
    (27..31).contains(&t)
}

pub fn code(t: Tile) -> String {
    match t {
        0..=26 => format!("{}{}", t % 9 + 1, SUIT_CODE[(t / 9) as usize]),
        27..=33 => HONOR_CODE[(t - 27) as usize].to_string(),
        34..=41 => format!("f{}", t - 33),
        _ => "?".into(),
    }
}

pub fn zh(t: Tile) -> String {
    match t {
        0..=26 => format!("{}{}", NUM_ZH[(t % 9) as usize], SUIT_ZH[(t / 9) as usize]),
        27..=33 => HONOR_ZH[(t - 27) as usize].to_string(),
        34..=41 => FLOWER_ZH[(t - 34) as usize].to_string(),
        _ => "?".into(),
    }
}

/// 代碼加中文，例如 `5m(五萬)`，給 AI 看的文字用。
pub fn label(t: Tile) -> String {
    format!("{}({})", code(t), zh(t))
}

/// 接受 `5m`、`E`、`f1`、`1z`、`五萬`、`5萬`、`東`、`紅中` 等寫法。
pub fn parse(s: &str) -> Option<Tile> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // 去掉 "5m(五萬)" 這種括號附註
    let s = s.split('(').next().unwrap_or(s).trim();
    let chars: Vec<char> = s.chars().collect();
    if chars.len() == 1 {
        let c = chars[0].to_ascii_uppercase();
        if let Some(i) = HONOR_CODE.iter().position(|h| h.starts_with(c)) {
            return Some(27 + i as Tile);
        }
    }
    if chars.len() == 2 && chars[0].is_ascii_digit() {
        let n = chars[0].to_digit(10)? as Tile;
        match chars[1].to_ascii_lowercase() {
            'm' if (1..=9).contains(&n) => return Some(n - 1),
            'p' if (1..=9).contains(&n) => return Some(9 + n - 1),
            's' if (1..=9).contains(&n) => return Some(18 + n - 1),
            'z' if (1..=7).contains(&n) => return Some(27 + n - 1),
            _ => {}
        }
    }
    if chars.len() == 2 && chars[0].to_ascii_lowercase() == 'f' {
        let n = chars[1].to_digit(10)? as Tile;
        if (1..=8).contains(&n) {
            return Some(33 + n);
        }
    }
    // 中文
    let aliases: [(&str, Tile); 8] = [
        ("紅中", 31),
        ("青發", 32),
        ("發財", 32),
        ("白板", 33),
        ("東風", 27),
        ("南風", 28),
        ("西風", 29),
        ("北風", 30),
    ];
    for (a, t) in aliases {
        if s == a {
            return Some(t);
        }
    }
    for t in 0..42u8 {
        if zh(t) == s {
            return Some(t);
        }
    }
    if chars.len() == 2 {
        if let (Some(n), Some(suit)) = (
            chars[0].to_digit(10),
            SUIT_ZH.iter().position(|x| x.starts_with(chars[1])),
        ) {
            if (1..=9).contains(&n) {
                return Some(suit as Tile * 9 + n as Tile - 1);
            }
        }
        // 「索」也當條子
        if chars[1] == '索' {
            if let Some(n) = chars[0].to_digit(10) {
                return Some(18 + n as Tile - 1);
            }
        }
    }
    None
}

/// 144 張：每種 4 張 + 8 張花。
pub fn full_set() -> Vec<Tile> {
    let mut v = Vec::with_capacity(144);
    for t in 0..NUM_TYPES as Tile {
        for _ in 0..4 {
            v.push(t);
        }
    }
    for f in 34..42 {
        v.push(f);
    }
    v
}

pub fn list_codes(ts: &[Tile]) -> Vec<String> {
    ts.iter().map(|&t| code(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for t in 0..42u8 {
            assert_eq!(parse(&code(t)), Some(t), "code {}", code(t));
            assert_eq!(parse(&zh(t)), Some(t), "zh {}", zh(t));
            assert_eq!(parse(&label(t)), Some(t));
        }
        assert_eq!(parse("1z"), Some(27));
        assert_eq!(parse("紅中"), Some(31));
        assert_eq!(parse("5萬"), Some(4));
        assert_eq!(parse("s"), Some(28));
    }
}
