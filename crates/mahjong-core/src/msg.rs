//! 給介面翻譯用的訊息：一段中文樣板（同時也是翻譯的 key）加上參數。
//!
//! 例如 `Msg::new("{name} 打出 {tile}").player("name", 2, "小翠(電腦)").tile("tile", t)`：
//! - `zh()` 產生原本的中文句子（給牌局紀錄的文字版、AI 介面用）
//! - `json()` 產生 `{"k": "{name} 打出 {tile}", "a": {"name": {"p": 2, "n": "小翠(電腦)"}, "tile": {"tile": "3m"}}}`，
//!   前端用 k 查翻譯，再把參數換成該語言的玩家名字、牌名。
//!
//! 參數的 JSON 形式：
//! - 數字、字串：原樣顯示（不翻譯）
//! - `{"t": "碰"}`：要翻譯的詞（介面用 t() 查）
//! - `{"p": 座位, "n": 名字}`：玩家（介面依角色換成該語言的名字）
//! - `{"tile": "3m"}`、`{"tiles": ["3m", "4m"]}`：牌
//! - `{"m": {...}}`：另一則訊息（巢狀）
//! - `{"list": [訊息...], "sep": "、"}`：多則訊息用分隔符號串起來

use super::tiles::{code, zh, Tile};
use serde_json::{json, Map, Value};

#[derive(Clone, Debug)]
pub enum Arg {
    Raw(String),
    Text(String),
    Player(usize, String),
    Tile(Tile),
    Tiles(Vec<Tile>),
    Msg(Box<Msg>),
    List(Vec<Msg>, &'static str),
}

#[derive(Clone, Debug)]
pub struct Msg {
    pub key: String,
    pub args: Vec<(&'static str, Arg)>,
}

impl Msg {
    pub fn new(key: impl Into<String>) -> Self {
        Msg { key: key.into(), args: Vec::new() }
    }
    fn arg(mut self, name: &'static str, a: Arg) -> Self {
        self.args.push((name, a));
        self
    }
    /// 不翻譯的值（數字、分數）
    pub fn raw(self, name: &'static str, v: impl ToString) -> Self {
        self.arg(name, Arg::Raw(v.to_string()))
    }
    /// 要翻譯的詞（技能名稱、「碰」「槓」之類）
    pub fn text(self, name: &'static str, v: impl Into<String>) -> Self {
        self.arg(name, Arg::Text(v.into()))
    }
    pub fn player(self, name: &'static str, seat: usize, n: impl Into<String>) -> Self {
        self.arg(name, Arg::Player(seat, n.into()))
    }
    pub fn tile(self, name: &'static str, t: Tile) -> Self {
        self.arg(name, Arg::Tile(t))
    }
    pub fn tiles(self, name: &'static str, ts: &[Tile]) -> Self {
        self.arg(name, Arg::Tiles(ts.to_vec()))
    }
    pub fn msg(self, name: &'static str, m: Msg) -> Self {
        self.arg(name, Arg::Msg(Box::new(m)))
    }
    pub fn list(self, name: &'static str, ms: Vec<Msg>, sep: &'static str) -> Self {
        self.arg(name, Arg::List(ms, sep))
    }

    /// 中文句子
    pub fn zh(&self) -> String {
        let mut out = self.key.clone();
        for (name, a) in &self.args {
            let v = match a {
                Arg::Raw(s) | Arg::Text(s) | Arg::Player(_, s) => s.clone(),
                Arg::Tile(t) => zh(*t),
                Arg::Tiles(ts) => ts.iter().map(|&t| zh(t)).collect::<Vec<_>>().join("、"),
                Arg::Msg(m) => m.zh(),
                Arg::List(ms, sep) => ms.iter().map(|m| m.zh()).collect::<Vec<_>>().join(sep),
            };
            out = out.replace(&format!("{{{}}}", name), &v);
        }
        out
    }

    pub fn json(&self) -> Value {
        let mut a = Map::new();
        for (name, arg) in &self.args {
            let v = match arg {
                Arg::Raw(s) => match s.parse::<i64>() {
                    Ok(n) => json!(n),
                    Err(_) => json!(s),
                },
                Arg::Text(s) => json!({ "t": s }),
                Arg::Player(p, n) => json!({ "p": p, "n": n }),
                Arg::Tile(t) => json!({ "tile": code(*t) }),
                Arg::Tiles(ts) => json!({ "tiles": ts.iter().map(|&t| code(t)).collect::<Vec<_>>() }),
                Arg::Msg(m) => json!({ "m": m.json() }),
                Arg::List(ms, sep) => json!({ "list": ms.iter().map(|m| m.json()).collect::<Vec<_>>(), "sep": sep }),
            };
            a.insert(name.to_string(), v);
        }
        json!({ "k": self.key, "a": a })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_chinese_and_json() {
        let m = Msg::new("{name} 打出 {tile}").player("name", 2, "小翠(電腦)").tile("tile", 2);
        assert_eq!(m.zh(), "小翠(電腦) 打出 三萬");
        let j = m.json();
        assert_eq!(j["k"], "{name} 打出 {tile}");
        assert_eq!(j["a"]["name"]["p"], 2);
        assert_eq!(j["a"]["tile"]["tile"], code(2));
        let nested = Msg::new("建議打{tile}{why}").tile("tile", 27).msg("why", Msg::new("（孤張，用處不大）"));
        assert_eq!(nested.zh(), "建議打東（孤張，用處不大）");
    }
}
