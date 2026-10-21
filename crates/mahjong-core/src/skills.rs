//! 技能模式：每個角色有一個技能（房主在規則裡開啟，預設關閉）。
//!
//! 主動技能在自己的回合多一個「技能」動作；被動技能條件符合時自動發動。
//! 除了阿翼的銀之盾一場一次，其他每局一次。

use serde::{Deserialize, Deserializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skill {
    /// 小翠：摸到的牌放回牌尾，重摸一張（主動）
    Redraw,
    /// 月華：看穿其他三家聽什麼牌，維持到自己打出第三張牌（主動）
    SeeWaits,
    /// 阿明：分數最低時，開局自動把配牌換好一點（被動）
    Comeback,
    /// 糖糖：這一回合打出的牌別人不能吃、碰、槓（胡不擋）（主動）
    NoClaim,
    /// 小春：可以吃任何人打的牌，不限上家（每局一次，吃的時候發動）
    QuickChow,
    /// 蘭姐：偷看牌牆接下來的 3 張（主動）
    Peek,
    /// 阿翼：放槍時不用付錢（被動，一場一次）
    Shield,
    /// 子衡：胡牌時多算 1 台（被動）
    Foreshadow,
    /// 承岳：別人自摸時不用付錢（被動，一場一次）
    Sturdy,
    /// 知白：這一回合打出的牌別人不能胡（吃碰槓不擋）（主動）
    Rest,
    /// 佑辰：看到分數最高那家的整副手牌，維持到自己打出三張牌（主動）
    ReadMind,
}

impl Skill {
    pub fn of(character: &str) -> Option<Skill> {
        Some(match character {
            "xiaocui" => Skill::Redraw,
            "yuehua" => Skill::SeeWaits,
            "aming" => Skill::Comeback,
            "tangtang" => Skill::NoClaim,
            "xiaochun" => Skill::QuickChow,
            "lanjie" => Skill::Peek,
            "ayi" => Skill::Shield,
            "ziheng" => Skill::Foreshadow,
            "chengyue" => Skill::Sturdy,
            "zhibai" => Skill::Rest,
            "youchen" => Skill::ReadMind,
            _ => return None,
        })
    }

    pub fn id(self) -> &'static str {
        match self {
            Skill::Redraw => "redraw",
            Skill::SeeWaits => "see_waits",
            Skill::Comeback => "comeback",
            Skill::NoClaim => "no_claim",
            Skill::QuickChow => "quick_chow",
            Skill::Peek => "peek",
            Skill::Shield => "shield",
            Skill::Foreshadow => "foreshadow",
            Skill::Sturdy => "sturdy",
            Skill::Rest => "rest",
            Skill::ReadMind => "read_mind",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Skill::Redraw => "元氣再來！",
            Skill::SeeWaits => "看穿",
            Skill::Comeback => "逆轉之魂",
            Skill::NoClaim => "拜託嘛～",
            Skill::QuickChow => "快攻",
            Skill::Peek => "老闆娘的眼力",
            Skill::Shield => "銀之盾",
            Skill::Foreshadow => "伏筆回收",
            Skill::Sturdy => "穩固",
            Skill::Rest => "休止符",
            Skill::ReadMind => "讀心術",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Skill::Redraw => "摸到不想要的牌時，放回牌尾重摸一張。每局一次。",
            Skill::SeeWaits => "看穿其他三家聽什麼牌，維持到自己再打出三張牌。每局一次。",
            Skill::Comeback => "分數最低時，開局自動把配牌換好一點。",
            Skill::NoClaim => "這一回合打出的牌，別人不能吃、碰、槓（胡牌擋不住）。每局一次。",
            Skill::QuickChow => "可以吃任何人打出的牌，不限上家。每局一次。",
            Skill::Peek => "偷看牌牆接下來會被摸走的 3 張牌。每局一次。",
            Skill::Shield => "放槍時不用付錢。一場一次。",
            Skill::Foreshadow => "胡牌時多算 1 台。",
            Skill::Sturdy => "別人自摸時不用付錢。一場一次。",
            Skill::Rest => "這一回合打出的牌，別人不能胡（吃、碰、槓擋不住）。每局一次。",
            Skill::ReadMind => "看穿分數最高那家的整副手牌，維持到自己再打出三張牌。每局一次。",
        }
    }

    /// 要自己按「技能」才會發動的
    pub fn is_active(self) -> bool {
        matches!(self, Skill::Redraw | Skill::SeeWaits | Skill::NoClaim | Skill::Peek | Skill::Rest | Skill::ReadMind)
    }

    /// 一場只能用一次的（用過與否記在 Room::shield_used，跨局保留）
    pub fn once_per_game(self) -> bool {
        matches!(self, Skill::Shield | Skill::Sturdy)
    }
}

/// 設定裡的開關：介面的數字輸入會送 0／1，也接受 true／false。
pub fn bool_or_number<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum B {
        Bool(bool),
        Num(f64),
    }
    Ok(match B::deserialize(d)? {
        B::Bool(b) => b,
        B::Num(n) => n != 0.0,
    })
}
