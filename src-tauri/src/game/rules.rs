//! 牌型分析：胡牌判定、拆解、聽牌、向聽數。

use super::tiles::*;

pub type Counts = [u8; NUM_TYPES];

pub fn counts(tiles: &[Tile]) -> Counts {
    let mut c = [0u8; NUM_TYPES];
    for &t in tiles {
        if (t as usize) < NUM_TYPES {
            c[t as usize] += 1;
        }
    }
    c
}

fn total(c: &Counts) -> u32 {
    c.iter().map(|&x| x as u32).sum()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    /// 順子，存最小那張
    Chow(Tile),
    Pung(Tile),
}

#[derive(Clone, Debug)]
pub struct Decomp {
    pub pair: Tile,
    pub sets: Vec<Set>,
}

/// 列出手牌（3n+2 張）所有「n 組 + 1 對」的拆法。
pub fn decompositions(c: &Counts) -> Vec<Decomp> {
    let mut out = Vec::new();
    if total(c) % 3 != 2 {
        return out;
    }
    let mut c = *c;
    for p in 0..NUM_TYPES {
        if c[p] >= 2 {
            c[p] -= 2;
            let mut sets = Vec::new();
            decomp_rec(&mut c, &mut sets, p as Tile, &mut out);
            c[p] += 2;
        }
    }
    out
}

fn decomp_rec(c: &mut Counts, sets: &mut Vec<Set>, pair: Tile, out: &mut Vec<Decomp>) {
    let Some(i) = (0..NUM_TYPES).find(|&i| c[i] > 0) else {
        out.push(Decomp { pair, sets: sets.clone() });
        return;
    };
    if c[i] >= 3 {
        c[i] -= 3;
        sets.push(Set::Pung(i as Tile));
        decomp_rec(c, sets, pair, out);
        sets.pop();
        c[i] += 3;
    }
    if i < 27 && i % 9 <= 6 && c[i + 1] > 0 && c[i + 2] > 0 {
        c[i] -= 1;
        c[i + 1] -= 1;
        c[i + 2] -= 1;
        sets.push(Set::Chow(i as Tile));
        decomp_rec(c, sets, pair, out);
        sets.pop();
        c[i] += 1;
        c[i + 1] += 1;
        c[i + 2] += 1;
    }
}

fn can_form_sets(c: &mut Counts) -> bool {
    let Some(i) = (0..NUM_TYPES).find(|&i| c[i] > 0) else {
        return true;
    };
    if c[i] >= 3 {
        c[i] -= 3;
        let ok = can_form_sets(c);
        c[i] += 3;
        if ok {
            return true;
        }
    }
    if i < 27 && i % 9 <= 6 && c[i + 1] > 0 && c[i + 2] > 0 {
        c[i] -= 1;
        c[i + 1] -= 1;
        c[i + 2] -= 1;
        let ok = can_form_sets(c);
        c[i] += 1;
        c[i + 1] += 1;
        c[i + 2] += 1;
        if ok {
            return true;
        }
    }
    false
}

pub fn is_standard_win(c: &Counts) -> bool {
    if total(c) % 3 != 2 {
        return false;
    }
    let mut c = *c;
    for p in 0..NUM_TYPES {
        if c[p] >= 2 {
            c[p] -= 2;
            let ok = can_form_sets(&mut c);
            c[p] += 2;
            if ok {
                return true;
            }
        }
    }
    false
}

/// 嚦咕嚦咕：門清 17 張，七對 + 一刻。
pub fn is_lilikuli(c: &Counts) -> bool {
    if total(c) != 17 {
        return false;
    }
    let (mut pairs, mut trips) = (0, 0);
    for &x in c.iter() {
        match x {
            0 => {}
            2 => pairs += 1,
            3 => trips += 1,
            4 => pairs += 2,
            _ => return false,
        }
    }
    pairs == 7 && trips == 1
}

pub fn is_win(c: &Counts) -> bool {
    is_standard_win(c) || is_lilikuli(c)
}

/// 3n+1 張時，聽哪些牌。
pub fn waits(c: &Counts) -> Vec<Tile> {
    let mut c = *c;
    let mut v = Vec::new();
    for t in 0..NUM_TYPES {
        if c[t] < 4 {
            c[t] += 1;
            if is_win(&c) {
                v.push(t as Tile);
            }
            c[t] -= 1;
        }
    }
    v
}

/// 一般型的向聽數：`sets_needed` 為還需要湊幾組（5 - 已副露組數）。
/// -1 表示已胡，0 表示聽牌。
pub fn shanten(c: &Counts, sets_needed: i32) -> i32 {
    let mut c = *c;
    let mut best = 2 * sets_needed;
    sh_rec(&mut c, 0, 0, 0, false, sets_needed, &mut best);
    best
}

fn sh_rec(c: &mut Counts, i: usize, m: i32, t: i32, pair: bool, n: i32, best: &mut i32) {
    if *best <= -1 {
        return;
    }
    let mut i = i;
    while i < NUM_TYPES && c[i] == 0 {
        i += 1;
    }
    let cur = 2 * (n - m) - t.min(n - m) - pair as i32;
    if i == NUM_TYPES {
        if cur < *best {
            *best = cur;
        }
        return;
    }
    // 剪枝：剩下的牌就算全部組成面子，最多再減多少
    let remaining: i32 = c[i..].iter().map(|&x| x as i32).sum();
    let potential = (remaining * 2 + 2) / 3 + if pair { 0 } else { 1 };
    if cur - potential >= *best {
        return;
    }
    if c[i] >= 3 && m < n {
        c[i] -= 3;
        sh_rec(c, i, m + 1, t, pair, n, best);
        c[i] += 3;
    }
    let suited = i < 27;
    if suited && i % 9 <= 6 && c[i + 1] > 0 && c[i + 2] > 0 && m < n {
        c[i] -= 1;
        c[i + 1] -= 1;
        c[i + 2] -= 1;
        sh_rec(c, i, m + 1, t, pair, n, best);
        c[i] += 1;
        c[i + 1] += 1;
        c[i + 2] += 1;
    }
    if c[i] >= 2 && !pair {
        c[i] -= 2;
        sh_rec(c, i, m, t, true, n, best);
        c[i] += 2;
    }
    if m + t < n {
        if c[i] >= 2 {
            c[i] -= 2;
            sh_rec(c, i, m, t + 1, pair, n, best);
            c[i] += 2;
        }
        if suited && i % 9 <= 7 && c[i + 1] > 0 {
            c[i] -= 1;
            c[i + 1] -= 1;
            sh_rec(c, i, m, t + 1, pair, n, best);
            c[i] += 1;
            c[i + 1] += 1;
        }
        if suited && i % 9 <= 6 && c[i + 2] > 0 {
            c[i] -= 1;
            c[i + 2] -= 1;
            sh_rec(c, i, m, t + 1, pair, n, best);
            c[i] += 1;
            c[i + 2] += 1;
        }
    }
    let k = c[i];
    c[i] = 0;
    sh_rec(c, i + 1, m, t, pair, n, best);
    c[i] = k;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Vec<Tile> {
        s.split_whitespace().map(|x| parse(x).unwrap()).collect()
    }

    #[test]
    fn win_basic() {
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 1p 1p 2s 3s 4s E E");
        assert!(is_win(&counts(&h)));
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 1p 1p 2s 3s 4s E S");
        assert!(!is_win(&counts(&h)));
    }

    #[test]
    fn waits_test() {
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 1p 1p 2s 3s E E");
        assert_eq!(waits(&counts(&h)), vec![18, 21]);
    }

    #[test]
    fn lilikuli() {
        let h = p("1m 1m 2m 2m 3m 3m 5p 5p 7p 7p 9s 9s E E C C C");
        assert!(is_lilikuli(&counts(&h)));
    }

    #[test]
    fn shanten_test() {
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 1p 1p 2s 3s 4s E E");
        assert_eq!(shanten(&counts(&h), 5), -1);
        let h = p("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 1p 1p 2s 3s E E");
        assert_eq!(shanten(&counts(&h), 5), 0);
        let h = p("1m 4m 7m 1p 4p 7p 1s 4s 7s E S W N C F P 9m");
        assert!(shanten(&counts(&h), 5) >= 6);
    }
}
