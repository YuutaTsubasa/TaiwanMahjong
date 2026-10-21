//! 房間與牌局狀態機。伺服器端權威：所有動作都經過 `Room::act` 驗證。

use super::bot;
use super::hints;
use super::rules::*;
use super::scoring::*;
use super::skills::{bool_or_number, Skill};
use super::tiles::*;
use rand::distributions::Alphanumeric;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::time::Duration;
use web_time::Instant;

/// 留牌（海底保留）張數
pub const RESERVE: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatKind {
    Human,
    Agent,
    Bot,
}

/// 內建角色（id, 名字, 介紹）。前端 src/lib/characters.js 的外觀與台詞要跟這裡同步。
pub const CHARACTERS: [(&str, &str, &str); 10] = [
    ("xiaocui", "小翠", "綠色雙馬尾的元氣少女"),
    ("yuehua", "月華", "銀髮的冷靜大小姐"),
    ("aming", "阿明", "熱血的刺蝟頭少年"),
    ("tangtang", "糖糖", "愛撒嬌的粉紅包包頭"),
    ("xiaochun", "小春", "橘色馬尾的運動少女"),
    ("lanjie", "蘭姐", "戴眼鏡的麻將館老闆娘"),
    ("ziheng", "子衡", "溫柔有禮、牌風卻毫不留情的書店老闆"),
    ("chengyue", "承岳", "沉穩可靠、常來幫忙修桌椅的家具修復師"),
    ("zhibai", "知白", "話少卻細心、銀藍髮的大提琴手"),
    ("youchen", "佑辰", "笑容親切、擅長讀心的活動主持人"),
];

pub fn character_name(id: &str) -> Option<&'static str> {
    CHARACTERS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// 也允許自訂角色 id（前端有對應設定即可）
fn valid_character_id(id: &str) -> bool {
    (1..=32).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// 給畫面播角色反應用的事件（碰、胡、放槍…）
#[derive(Clone, Debug, Serialize)]
pub struct GameEvent {
    pub seq: u64,
    pub seat: usize,
    pub kind: &'static str,
    pub tile: Option<String>,
}

pub struct SeatInfo {
    pub name: String,
    pub character: String,
    pub kind: SeatKind,
    pub token: String,
    pub connections: u32,
    pub last_seen: Instant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 底
    pub base: i64,
    /// 每台
    pub per_tai: i64,
    /// 打幾圈（1 = 東風圈，4 = 一將）
    pub rounds: u8,
    /// 吃碰胡等待秒數，0 = 不限
    pub claim_timeout: u64,
    /// 出牌等待秒數，0 = 不限
    pub turn_timeout: u64,
    /// 一局結束後自動開下一局的秒數，0 = 等所有人按下一局
    pub next_hand_delay: u64,
    /// 電腦思考時間（毫秒）
    pub bot_delay_ms: u64,
    /// 技能模式：每個角色有一個技能（見 skills.rs）
    #[serde(deserialize_with = "bool_or_number")]
    pub skills: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            base: 100,
            per_tai: 20,
            rounds: 1,
            claim_timeout: 30,
            turn_timeout: 90,
            next_hand_delay: 30,
            bot_delay_ms: 700,
            skills: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Discard(Tile),
    Tsumo,
    ConcealedKong(Tile),
    AddedKong(Tile),
    Ron,
    Pung,
    Kong,
    /// 順子起始牌
    Chow(Tile),
    Pass,
    Next,
    /// 發動角色技能（技能模式）
    Skill,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerState {
    pub concealed: Vec<Tile>,
    pub melds: Vec<Meld>,
    pub flowers: Vec<Tile>,
    pub discards: Vec<Tile>,
    /// 過水：放過胡之後到下次摸牌前不能胡別人打的牌
    pub skip_ron: bool,
    pub draws: u32,
    /// 技能模式：這一局的技能用過了
    pub skill_used: bool,
    /// 月華的看穿：還能看幾張（自己每打一張減一）
    pub see_waits_left: u8,
    /// 糖糖：下一張打出的牌別人不能吃碰槓
    pub no_claim_next: bool,
    /// 蘭姐偷看到的牌，以及偷看時牌牆前面已經被摸走幾張
    pub peek: Vec<Tile>,
    pub peek_at: usize,
    /// 知白：下一張打出的牌別人不能胡
    pub no_ron_next: bool,
    /// 佑辰的讀心術：看誰的手牌、還能看幾張（自己每打一張減一）
    pub read_mind: Option<usize>,
    pub read_mind_left: u8,
}

#[derive(Clone, Debug)]
pub enum Step {
    Turn {
        seat: usize,
        drawn: Option<Tile>,
        after_kong: bool,
        claimed: bool,
    },
    Claims {
        discarder: usize,
        tile: Tile,
        options: [Vec<Action>; 4],
        responses: [Option<Action>; 4],
    },
    RobKong {
        seat: usize,
        tile: Tile,
        options: [Vec<Action>; 4],
        responses: [Option<Action>; 4],
    },
}

pub struct Hand {
    pub wall: VecDeque<Tile>,
    pub players: [PlayerState; 4],
    pub step: Step,
    pub any_claim: bool,
    pub last_discard: Option<(usize, Tile)>,
    /// 從牌牆前面摸走的張數（蘭姐偷看用）
    pub front_drawn: usize,
}

impl Hand {
    pub fn drawable(&self) -> usize {
        self.wall.len().saturating_sub(RESERVE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Lobby,
    Playing,
    HandOver,
    GameOver,
}

#[derive(Clone, Debug, Serialize)]
pub struct Payment {
    pub from: usize,
    pub to: usize,
    pub tai: u32,
    pub amount: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct HandResult {
    pub draw: bool,
    pub winner: Option<usize>,
    pub loser: Option<usize>,
    pub self_draw: bool,
    pub win_tile: Option<String>,
    pub items: Vec<ScoreItem>,
    pub hand_tai: u32,
    /// 莊家 + 連莊（只在莊家有關的付款中計算）
    pub dealer_items: Vec<ScoreItem>,
    pub payments: Vec<Payment>,
    pub deltas: [i64; 4],
    pub dealer_stays: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatMsg {
    pub seat: Option<usize>,
    pub name: String,
    pub text: String,
}

#[derive(Default, Clone, Copy)]
struct WinFlags {
    self_draw: bool,
    after_kong: bool,
    rob_kong: bool,
    last_tile: bool,
    heaven: bool,
    earth: bool,
    flower_win: Option<FlowerWin>,
}

pub struct Room {
    pub seats: [Option<SeatInfo>; 4],
    pub settings: Settings,
    pub phase: Phase,
    pub scores: [i64; 4],
    pub round_wind: u8,
    pub dealer: usize,
    pub start_dealer: usize,
    pub streak: u32,
    pub hand_no: u32,
    pub hand: Option<Hand>,
    pub result: Option<HandResult>,
    pub ready: [bool; 4],
    /// 房主坐在哪個座位（大廳顯示「房主」用；房主權限仍由連線時的金鑰決定）
    pub host_seat: Option<usize>,
    /// 託管：座位仍屬於玩家，但由電腦代為出手
    pub auto: [bool; 4],
    pub decision_id: u64,
    pub decision_started: Instant,
    pub deadline: Option<Instant>,
    pub log: VecDeque<String>,
    pub chat: VecDeque<ChatMsg>,
    pub events: VecDeque<GameEvent>,
    event_seq: u64,
    /// 阿翼的銀之盾用過了（一場一次）
    pub shield_used: [bool; 4],
}

fn gen_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}

impl Room {
    pub fn new(settings: Settings) -> Self {
        Room {
            seats: Default::default(),
            settings,
            phase: Phase::Lobby,
            scores: [0; 4],
            round_wind: 0,
            dealer: 0,
            start_dealer: 0,
            streak: 0,
            hand_no: 0,
            hand: None,
            result: None,
            ready: [false; 4],
            host_seat: None,
            auto: [false; 4],
            decision_id: 0,
            decision_started: Instant::now(),
            deadline: None,
            log: VecDeque::new(),
            chat: VecDeque::new(),
            events: VecDeque::new(),
            event_seq: 0,
            shield_used: [false; 4],
        }
    }

    // ───────────── 大廳 ─────────────

    pub fn name(&self, seat: usize) -> String {
        self.seats[seat]
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| format!("座位{}", seat))
    }

    fn push_event(&mut self, seat: usize, kind: &'static str, tile: Option<Tile>) {
        self.event_seq += 1;
        self.events.push_back(GameEvent { seq: self.event_seq, seat, kind, tile: tile.map(code) });
        while self.events.len() > 32 {
            self.events.pop_front();
        }
    }

    /// 選角色：指定的有效就用，否則挑一個還沒人用的內建角色。`free_seat` 是即將被取代的座位。
    fn pick_character(&self, wanted: Option<&str>, free_seat: Option<usize>) -> String {
        if let Some(w) = wanted.map(str::trim).filter(|w| valid_character_id(w)) {
            return w.to_string();
        }
        let used: Vec<&str> = (0..4)
            .filter(|&s| Some(s) != free_seat)
            .filter_map(|s| self.seats[s].as_ref().map(|i| i.character.as_str()))
            .collect();
        let mut free: Vec<&str> = CHARACTERS.iter().map(|c| c.0).filter(|c| !used.contains(c)).collect();
        if free.is_empty() {
            free = CHARACTERS.iter().map(|c| c.0).collect();
        }
        free.choose(&mut rand::thread_rng()).unwrap().to_string()
    }

    pub fn set_character(&mut self, seat: usize, id: &str) -> Result<(), String> {
        if !valid_character_id(id) {
            return Err("角色 id 不合法".into());
        }
        let Some(i) = self.seats.get_mut(seat).and_then(|s| s.as_mut()) else {
            return Err("你沒有座位".into());
        };
        i.character = id.to_string();
        Ok(())
    }

    fn push_log(&mut self, s: String) {
        self.log.push_back(s);
        while self.log.len() > 60 {
            self.log.pop_front();
        }
    }

    pub fn push_chat(&mut self, seat: Option<usize>, text: &str) {
        let text: String = text.trim().chars().take(300).collect();
        if text.is_empty() {
            return;
        }
        let name = match seat {
            Some(s) => self.name(s),
            None => "系統".into(),
        };
        self.chat.push_back(ChatMsg { seat, name, text });
        while self.chat.len() > 50 {
            self.chat.pop_front();
        }
    }

    pub fn seat_of_token(&self, token: &str) -> Option<usize> {
        if token.is_empty() {
            return None;
        }
        (0..4).find(|&s| self.seats[s].as_ref().map_or(false, |i| i.token == token))
    }

    /// 電腦或託管中的座位，由內建電腦出手
    pub fn auto_plays(&self, seat: usize) -> bool {
        self.is_bot(seat) || self.auto[seat]
    }

    pub fn set_auto(&mut self, seat: usize, on: bool) {
        if seat >= 4 || self.seats[seat].is_none() || self.auto[seat] == on {
            return;
        }
        self.auto[seat] = on;
        let name = self.name(seat);
        self.push_log(format!("{} {}", name, if on { "開啟託管" } else { "取消託管" }));
    }

    pub fn is_bot(&self, seat: usize) -> bool {
        self.seats[seat].as_ref().map_or(false, |i| i.kind == SeatKind::Bot)
    }

    pub fn is_connected(&self, seat: usize) -> bool {
        match &self.seats[seat] {
            None => false,
            Some(i) => {
                i.kind == SeatKind::Bot
                    || i.connections > 0
                    || i.last_seen.elapsed() < Duration::from_secs(120)
            }
        }
    }

    pub fn touch(&mut self, seat: usize) {
        if let Some(i) = self.seats[seat].as_mut() {
            i.last_seen = Instant::now();
        }
    }

    pub fn connect(&mut self, seat: usize) {
        if let Some(i) = self.seats[seat].as_mut() {
            i.connections += 1;
            i.last_seen = Instant::now();
        }
    }

    pub fn disconnect(&mut self, seat: usize) {
        if let Some(i) = self.seats[seat].as_mut() {
            i.connections = i.connections.saturating_sub(1);
            i.last_seen = Instant::now();
        }
    }

    /// 加入房間：大廳時坐空位；遊戲中則接手電腦的位置。
    pub fn join(&mut self, name: &str, kind: SeatKind, character: Option<&str>) -> Result<(usize, String), String> {
        let mut name: String = name.trim().chars().take(16).collect();
        if name.is_empty() {
            name = if kind == SeatKind::Agent { "AI".into() } else { "玩家".into() };
        }
        let seat = if self.phase == Phase::Lobby {
            (0..4).find(|&s| self.seats[s].is_none())
        } else {
            (0..4).find(|&s| self.is_bot(s))
        };
        let Some(seat) = seat else {
            return Err("房間已滿（遊戲進行中只能接手電腦的座位）".into());
        };
        let token = gen_token();
        let replaced = self.seats[seat].as_ref().map(|i| i.name.clone());
        let character = self.pick_character(character, Some(seat));
        self.auto[seat] = false;
        self.seats[seat] = Some(SeatInfo {
            name: name.clone(),
            character,
            kind,
            token: token.clone(),
            connections: 0,
            last_seen: Instant::now(),
        });
        match replaced {
            Some(r) => self.push_log(format!("{} 接手了 {} 的座位", name, r)),
            None => self.push_log(format!("{} 加入了座位 {}", name, seat)),
        }
        Ok((seat, token))
    }

    pub fn add_bot(&mut self, seat: Option<usize>) -> Result<usize, String> {
        let seat = match seat {
            Some(s) if s < 4 => s,
            Some(_) => return Err("座位號碼錯誤".into()),
            None => (0..4).find(|&s| self.seats[s].is_none()).ok_or("沒有空位")?,
        };
        if self.seats[seat].is_some() {
            return Err("該座位已有人".into());
        }
        let character = self.pick_character(None, None);
        let name = format!("{}(電腦)", character_name(&character).unwrap_or("電腦"));
        self.auto[seat] = false;
        self.seats[seat] = Some(SeatInfo {
            name,
            character,
            kind: SeatKind::Bot,
            token: gen_token(),
            connections: 0,
            last_seen: Instant::now(),
        });
        Ok(seat)
    }

    /// 大廳時移除座位；遊戲中改由電腦代打。
    pub fn remove_seat(&mut self, seat: usize) -> Result<(), String> {
        if seat >= 4 || self.seats[seat].is_none() {
            return Err("座位是空的".into());
        }
        if self.phase == Phase::Lobby || self.phase == Phase::GameOver {
            let n = self.name(seat);
            self.seats[seat] = None;
            if self.host_seat == Some(seat) {
                self.host_seat = None;
            }
            self.push_log(format!("{} 離開了座位", n));
        } else if let Some(i) = self.seats[seat].as_mut() {
            if i.kind != SeatKind::Bot {
                i.kind = SeatKind::Bot;
                self.auto[seat] = false;
                i.token = gen_token();
                i.name = format!("{}(電腦代打)", i.name);
                let n = i.name.clone();
                self.push_log(format!("{} 改由電腦代打", n));
            }
        }
        Ok(())
    }

    pub fn update_settings(&mut self, mut s: Settings) {
        s.rounds = s.rounds.clamp(1, 4);
        s.base = s.base.clamp(0, 1_000_000);
        s.per_tai = s.per_tai.clamp(0, 1_000_000);
        s.bot_delay_ms = s.bot_delay_ms.min(10_000);
        self.settings = s;
    }

    pub fn start_game(&mut self) -> Result<(), String> {
        if self.phase == Phase::Playing || self.phase == Phase::HandOver {
            return Err("遊戲已經在進行".into());
        }
        for s in 0..4 {
            if self.seats[s].is_none() {
                self.add_bot(Some(s))?;
            }
        }
        self.scores = [0; 4];
        self.shield_used = [false; 4];
        self.round_wind = 0;
        self.streak = 0;
        self.hand_no = 0;
        self.dealer = rand::thread_rng().gen_range(0..4);
        self.start_dealer = self.dealer;
        self.push_log(format!("遊戲開始！起莊：{}", self.name(self.dealer)));
        self.start_hand();
        Ok(())
    }

    pub fn reset_to_lobby(&mut self) {
        self.phase = Phase::Lobby;
        self.auto = [false; 4];
        self.hand = None;
        self.result = None;
        self.deadline = None;
        self.scores = [0; 4];
        self.new_decision(0);
        self.push_log("回到大廳".into());
    }

    fn new_decision(&mut self, timeout_secs: u64) {
        self.decision_id += 1;
        self.decision_started = Instant::now();
        self.deadline = if timeout_secs > 0 {
            Some(Instant::now() + Duration::from_secs(timeout_secs))
        } else {
            None
        };
    }

    pub fn seat_wind(&self, seat: usize) -> u8 {
        ((seat + 4 - self.dealer) % 4) as u8
    }

    // ───────────── 牌局 ─────────────

    fn start_hand(&mut self) {
        self.hand_no += 1;
        self.result = None;
        self.ready = [false; 4];
        let mut tiles = full_set();
        tiles.shuffle(&mut rand::thread_rng());
        let mut wall: VecDeque<Tile> = tiles.into();
        let mut players: [PlayerState; 4] = Default::default();
        for _ in 0..4 {
            for k in 0..4 {
                let s = (self.dealer + k) % 4;
                for _ in 0..4 {
                    players[s].concealed.push(wall.pop_front().unwrap());
                }
            }
        }
        players[self.dealer].concealed.push(wall.pop_front().unwrap());
        self.hand = Some(Hand {
            wall,
            players,
            step: Step::Turn {
                seat: self.dealer,
                drawn: None,
                after_kong: false,
                claimed: false,
            },
            any_claim: false,
            last_discard: None,
            front_drawn: 0,
        });
        self.phase = Phase::Playing;
        self.push_log(format!(
            "── 第 {} 局 {}風圈 莊家 {}{} ──",
            self.hand_no,
            WIND_ZH[self.round_wind as usize],
            self.name(self.dealer),
            if self.streak > 0 { format!(" 連{}", self.streak) } else { String::new() }
        ));
        for k in 0..4 {
            let s = (self.dealer + k) % 4;
            self.replace_flowers(s);
        }
        self.apply_comeback();
        if self.check_flower_win() {
            return;
        }
        self.new_decision(self.settings.turn_timeout);
    }

    fn replace_flowers(&mut self, seat: usize) {
        let h = self.hand.as_mut().unwrap();
        let p = &mut h.players[seat];
        loop {
            let fl: Vec<Tile> = p.concealed.iter().copied().filter(|&t| is_flower(t)).collect();
            if fl.is_empty() {
                break;
            }
            p.concealed.retain(|&t| !is_flower(t));
            p.flowers.extend(&fl);
            for _ in &fl {
                if let Some(t) = h.wall.pop_back() {
                    p.concealed.push(t);
                }
            }
        }
    }

    /// 八仙過海 / 七搶一
    fn check_flower_win(&mut self) -> bool {
        let h = self.hand.as_ref().unwrap();
        let fc: Vec<usize> = h.players.iter().map(|p| p.flowers.len()).collect();
        if let Some(w) = (0..4).find(|&s| fc[s] == 8) {
            self.finish_win(
                w,
                None,
                None,
                WinFlags {
                    self_draw: true,
                    flower_win: Some(FlowerWin::EightImmortals),
                    ..Default::default()
                },
            );
            return true;
        }
        if let Some(w) = (0..4).find(|&s| fc[s] == 7) {
            if let Some(l) = (0..4).find(|&s| fc[s] == 1) {
                self.finish_win(
                    w,
                    Some(l),
                    None,
                    WinFlags {
                        flower_win: Some(FlowerWin::SevenRobOne),
                        ..Default::default()
                    },
                );
                return true;
            }
        }
        false
    }

    fn draw(&mut self, seat: usize, mut from_back: bool, after_kong: bool) {
        let name = self.name(seat);
        loop {
            let h = self.hand.as_mut().unwrap();
            if h.wall.len() <= RESERVE {
                self.exhaustive_draw();
                return;
            }
            let t = if from_back { h.wall.pop_back() } else { h.wall.pop_front() }.unwrap();
            if !from_back {
                h.front_drawn += 1;
            }
            if is_flower(t) {
                h.players[seat].flowers.push(t);
                self.push_log(format!("{} 補花 {}", name, zh(t)));
                self.push_event(seat, "flower", Some(t));
                if self.check_flower_win() {
                    return;
                }
                from_back = true;
                continue;
            }
            let p = &mut h.players[seat];
            p.concealed.push(t);
            p.draws += 1;
            p.skip_ron = false;
            h.step = Step::Turn {
                seat,
                drawn: Some(t),
                after_kong,
                claimed: false,
            };
            self.new_decision(self.settings.turn_timeout);
            return;
        }
    }

    fn exhaustive_draw(&mut self) {
        self.push_log("流局（海底）".into());
        for s in 0..4 {
            self.push_event(s, "draw", None);
        }
        self.result = Some(HandResult {
            draw: true,
            winner: None,
            loser: None,
            self_draw: false,
            win_tile: None,
            items: vec![],
            hand_tai: 0,
            dealer_items: vec![],
            payments: vec![],
            deltas: [0; 4],
            dealer_stays: true,
            summary: "流局，莊家連莊".into(),
        });
        self.phase = Phase::HandOver;
        self.new_decision(self.settings.next_hand_delay);
    }

    pub fn legal_actions(&self, seat: usize) -> Vec<Action> {
        if seat >= 4 {
            return vec![];
        }
        match self.phase {
            Phase::HandOver => {
                if self.ready[seat] {
                    vec![]
                } else {
                    vec![Action::Next]
                }
            }
            Phase::Playing => {
                let Some(h) = &self.hand else { return vec![] };
                match &h.step {
                    Step::Turn { seat: s, claimed, drawn, .. } if *s == seat => {
                        let p = &h.players[seat];
                        let c = counts(&p.concealed);
                        let mut v = Vec::new();
                        if !claimed && is_win(&c) {
                            v.push(Action::Tsumo);
                        }
                        if self.can_use_skill(seat, h, *drawn, *claimed) {
                            v.push(Action::Skill);
                        }
                        if !claimed && h.wall.len() > RESERVE {
                            for t in 0..NUM_TYPES {
                                if c[t] == 4 {
                                    v.push(Action::ConcealedKong(t as Tile));
                                }
                            }
                            for m in &p.melds {
                                if m.kind == MeldKind::Pung && c[m.base() as usize] > 0 {
                                    v.push(Action::AddedKong(m.base()));
                                }
                            }
                        }
                        for t in 0..NUM_TYPES {
                            if c[t] > 0 {
                                v.push(Action::Discard(t as Tile));
                            }
                        }
                        v
                    }
                    Step::Claims { discarder: origin, options, responses, .. }
                    | Step::RobKong { seat: origin, options, responses, .. } => {
                        // 依優先權輪流問：胡（近的先）→ 碰／槓 → 吃。
                        // 優先權比較高、還沒決定的人先選，選「過」之後才輪到下一位，
                        // 免得吃的人先按下去，才發現被別人碰走。
                        let max_prio = |s: usize| options[s].iter().map(|a| Self::claim_prio(*origin, s, a)).max().unwrap_or(0);
                        let mine = max_prio(seat);
                        let someone_first = (0..4).any(|o| {
                            o != seat && !options[o].is_empty() && responses[o].is_none() && max_prio(o) > mine
                        });
                        if responses[seat].is_none() && !someone_first {
                            options[seat].clone()
                        } else {
                            vec![]
                        }
                    }
                    _ => vec![],
                }
            }
            _ => vec![],
        }
    }

    pub fn has_actions(&self, seat: usize) -> bool {
        !self.legal_actions(seat).is_empty()
    }

    pub fn act(&mut self, seat: usize, a: Action) -> Result<(), String> {
        let legal = self.legal_actions(seat);
        if !legal.contains(&a) {
            return Err(format!("不合法的動作：{}", self.action_label(&a)));
        }
        if self.phase == Phase::HandOver {
            self.ready[seat] = true;
            // 託管中的真人也要等他（或 tick 幫他）按下一局，才不會一胡牌就跳過結果
            if (0..4).all(|s| self.ready[s] || self.is_bot(s)) {
                self.next_hand();
            }
            return Ok(());
        }
        match a {
            Action::Discard(t) => self.do_discard(seat, t),
            Action::Tsumo => self.do_tsumo(seat),
            Action::ConcealedKong(t) => self.do_concealed_kong(seat, t),
            Action::AddedKong(t) => self.do_added_kong(seat, t),
            Action::Skill => self.use_skill(seat),
            Action::Ron | Action::Pung | Action::Kong | Action::Chow(_) | Action::Pass => {
                let h = self.hand.as_mut().unwrap();
                match &mut h.step {
                    Step::Claims { responses, .. } | Step::RobKong { responses, .. } => {
                        responses[seat] = Some(a);
                    }
                    _ => {}
                }
                self.try_resolve();
                // 還沒結束：輪到下一位（優先權較低）的人選，重新計時
                if matches!(self.hand.as_ref().map(|h| &h.step), Some(Step::Claims { .. } | Step::RobKong { .. })) {
                    self.new_decision(self.settings.claim_timeout);
                }
            }
            Action::Next => {}
        }
        Ok(())
    }

    fn remove_tiles(v: &mut Vec<Tile>, t: Tile, n: usize) {
        for _ in 0..n {
            if let Some(i) = v.iter().position(|&x| x == t) {
                v.remove(i);
            }
        }
    }

    fn do_discard(&mut self, seat: usize, t: Tile) {
        let name = self.name(seat);
        let h = self.hand.as_mut().unwrap();
        Self::remove_tiles(&mut h.players[seat].concealed, t, 1);
        h.players[seat].discards.push(t);
        h.last_discard = Some((seat, t));
        // 糖糖的「拜託嘛～」：這張牌只能被胡
        let no_claim = std::mem::take(&mut h.players[seat].no_claim_next);
        // 知白的「休止符」：這張牌不能被胡
        let no_ron = std::mem::take(&mut h.players[seat].no_ron_next);
        let sw = &mut h.players[seat].see_waits_left;
        *sw = sw.saturating_sub(1);
        let rm = &mut h.players[seat].read_mind_left;
        *rm = rm.saturating_sub(1);
        let quick_chow: [bool; 4] = std::array::from_fn(|o| {
            self.settings.skills
                && self.seats[o].as_ref().and_then(|i| Skill::of(&i.character)) == Some(Skill::QuickChow)
                && !h.players[o].skill_used
        });
        let last = h.wall.len() <= RESERVE || no_claim;
        let mut options: [Vec<Action>; 4] = Default::default();
        for k in 1..4 {
            let o = (seat + k) % 4;
            let q = &h.players[o];
            let mut c = counts(&q.concealed);
            let mut opts = Vec::new();
            c[t as usize] += 1;
            if !q.skip_ron && !no_ron && is_win(&c) {
                opts.push(Action::Ron);
            }
            c[t as usize] -= 1;
            if !last {
                if c[t as usize] >= 3 {
                    opts.push(Action::Kong);
                }
                if c[t as usize] >= 2 {
                    opts.push(Action::Pung);
                }
                if (k == 1 || quick_chow[o]) && is_suited(t) {
                    let suit_base = t / 9 * 9;
                    let lo = t.saturating_sub(2).max(suit_base);
                    for s in lo..=t {
                        if s % 9 > 6 {
                            continue;
                        }
                        let ok = (s..s + 3).filter(|&x| x != t).all(|x| c[x as usize] > 0);
                        if ok {
                            opts.push(Action::Chow(s));
                        }
                    }
                }
            }
            if !opts.is_empty() {
                opts.push(Action::Pass);
            }
            options[o] = opts;
        }
        let nobody = options.iter().all(|o| o.is_empty());
        if !nobody {
            h.step = Step::Claims {
                discarder: seat,
                tile: t,
                options,
                responses: Default::default(),
            };
        }
        self.push_log(format!("{} 打出 {}", name, zh(t)));
        self.push_event(seat, "discard", Some(t));
        if nobody {
            self.draw((seat + 1) % 4, false, false);
        } else {
            self.new_decision(self.settings.claim_timeout);
        }
    }

    // ───────────── 技能模式 ─────────────

    /// 這個座位的技能（技能模式關閉、或自訂角色沒有技能時是 None）
    pub fn skill_of(&self, seat: usize) -> Option<Skill> {
        if !self.settings.skills {
            return None;
        }
        self.seats.get(seat)?.as_ref().and_then(|i| Skill::of(&i.character))
    }

    fn can_use_skill(&self, seat: usize, h: &Hand, drawn: Option<Tile>, claimed: bool) -> bool {
        let Some(skill) = self.skill_of(seat) else { return false };
        if !skill.is_active() || h.players[seat].skill_used {
            return false;
        }
        match skill {
            Skill::Redraw => drawn.is_some() && !claimed && h.wall.len() > RESERVE + 1,
            Skill::Peek => h.wall.len() > RESERVE,
            Skill::SeeWaits | Skill::NoClaim | Skill::Rest | Skill::ReadMind => true,
            _ => false,
        }
    }

    fn announce_skill(&mut self, seat: usize, skill: Skill) {
        let name = self.name(seat);
        self.push_log(format!("{} 發動技能「{}」", name, skill.name()));
        self.push_event(seat, "skill", None);
    }

    fn use_skill(&mut self, seat: usize) {
        let Some(skill) = self.skill_of(seat) else { return };
        self.announce_skill(seat, skill);
        let h = self.hand.as_mut().unwrap();
        let p = &mut h.players[seat];
        p.skill_used = true;
        match skill {
            Skill::Redraw => {
                // 摸到的牌放回牌尾，從前面重摸
                let Step::Turn { drawn: Some(d), .. } = h.step else { return };
                Self::remove_tiles(&mut p.concealed, d, 1);
                p.draws = p.draws.saturating_sub(1);
                h.wall.push_back(d);
                self.draw(seat, false, false);
            }
            Skill::SeeWaits => p.see_waits_left = 3,
            Skill::NoClaim => p.no_claim_next = true,
            Skill::Peek => {
                p.peek = h.wall.iter().take(3).copied().collect();
                p.peek_at = h.front_drawn;
            }
            Skill::Rest => p.no_ron_next = true,
            Skill::ReadMind => {
                // 分數最高的另一家；同分時取離自己最近的下家方向
                let scores = self.scores;
                let target = (1..4)
                    .map(|k| (seat + k) % 4)
                    .max_by_key(|&o| (scores[o], std::cmp::Reverse((o + 4 - seat) % 4)));
                let p = &mut self.hand.as_mut().unwrap().players[seat];
                p.read_mind = target;
                p.read_mind_left = 3;
            }
            _ => {}
        }
    }

    /// 阿明的逆轉之魂：開局時分數最低（而且不是大家同分）就把配牌換好一點
    fn apply_comeback(&mut self) {
        for s in 0..4 {
            if self.skill_of(s) != Some(Skill::Comeback) {
                continue;
            }
            let lowest = (0..4).all(|o| o == s || self.scores[o] > self.scores[s]);
            if !lowest {
                continue;
            }
            let h = self.hand.as_mut().unwrap();
            let mut swaps = 0;
            for _ in 0..3 {
                let p = &h.players[s];
                let c = counts(&p.concealed);
                let before = shanten(&c, 5 - p.melds.len() as i32);
                let out = bot::best_discard(&p.concealed, 5, &c);
                let mut c2 = c;
                c2[out as usize] -= 1;
                // 只從會被摸到的那段牌牆裡挑（不動留牌）
                let reach = h.wall.len().saturating_sub(RESERVE);
                let pick = (0..reach).find(|&i| {
                    let t = h.wall[i];
                    if is_flower(t) {
                        return false;
                    }
                    let mut c3 = c2;
                    c3[t as usize] += 1;
                    shanten(&c3, 5) < before
                });
                let Some(i) = pick else { break };
                let p = &mut h.players[s];
                Self::remove_tiles(&mut p.concealed, out, 1);
                p.concealed.push(h.wall[i]);
                h.wall[i] = out;
                swaps += 1;
            }
            if swaps > 0 {
                self.hand.as_mut().unwrap().players[s].skill_used = true;
                self.announce_skill(s, Skill::Comeback);
            }
        }
    }

    /// 搶牌的優先權：胡（離打牌的人越近越優先）> 碰／槓 > 吃；過是 0。
    fn claim_prio(origin: usize, s: usize, a: &Action) -> i32 {
        match a {
            Action::Ron => 100 - ((s + 4 - origin) % 4) as i32,
            Action::Kong | Action::Pung => 50,
            Action::Chow(_) => 10,
            _ => 0,
        }
    }

    fn try_resolve(&mut self) {
        let h = self.hand.as_ref().unwrap();
        let (is_rob, origin, tile, options, responses) = match &h.step {
            Step::Claims { discarder, tile, options, responses } => {
                (false, *discarder, *tile, options.clone(), responses.clone())
            }
            Step::RobKong { seat, tile, options, responses } => {
                (true, *seat, *tile, options.clone(), responses.clone())
            }
            _ => return,
        };
        let prio = |s: usize, a: &Action| Self::claim_prio(origin, s, a);
        let mut best: Option<(usize, Action, i32)> = None;
        let mut undecided_max = -1;
        for s in 0..4 {
            if options[s].is_empty() {
                continue;
            }
            match &responses[s] {
                Some(a) => {
                    let p = prio(s, a);
                    if p > 0 && best.as_ref().map_or(true, |b| p > b.2) {
                        best = Some((s, a.clone(), p));
                    }
                }
                None => {
                    let m = options[s].iter().map(|a| prio(s, a)).max().unwrap_or(0);
                    undecided_max = undecided_max.max(m);
                }
            }
        }
        let best_p = best.as_ref().map_or(0, |b| b.2);
        if undecided_max > best_p {
            return; // 還有更高優先權的人沒決定
        }
        // 過水
        {
            let h = self.hand.as_mut().unwrap();
            for s in 0..4 {
                if options[s].contains(&Action::Ron) && responses[s] == Some(Action::Pass) {
                    h.players[s].skip_ron = true;
                }
            }
        }
        match best {
            None => {
                if is_rob {
                    self.complete_added_kong(origin, tile);
                } else {
                    self.draw((origin + 1) % 4, false, false);
                }
            }
            Some((s, Action::Ron, _)) => {
                let h = self.hand.as_mut().unwrap();
                if is_rob {
                    Self::remove_tiles(&mut h.players[origin].concealed, tile, 1);
                } else {
                    h.players[origin].discards.pop();
                }
                h.players[s].concealed.push(tile);
                let last_tile = h.wall.len() <= RESERVE;
                self.finish_win(
                    s,
                    Some(origin),
                    Some(tile),
                    WinFlags {
                        rob_kong: is_rob,
                        last_tile: last_tile && !is_rob,
                        ..Default::default()
                    },
                );
            }
            Some((s, a @ (Action::Pung | Action::Kong), _)) => {
                let is_kong = a == Action::Kong;
                let name = self.name(s);
                let h = self.hand.as_mut().unwrap();
                h.players[origin].discards.pop();
                h.any_claim = true;
                let p = &mut h.players[s];
                Self::remove_tiles(&mut p.concealed, tile, if is_kong { 3 } else { 2 });
                p.melds.push(Meld {
                    kind: if is_kong { MeldKind::Kong } else { MeldKind::Pung },
                    tiles: vec![tile; if is_kong { 4 } else { 3 }],
                    from: Some(origin),
                });
                self.push_log(format!("{} {} {}", name, if is_kong { "槓" } else { "碰" }, zh(tile)));
                self.push_event(s, if is_kong { "kong" } else { "pung" }, Some(tile));
                if is_kong {
                    self.draw(s, true, true);
                } else {
                    let h = self.hand.as_mut().unwrap();
                    h.step = Step::Turn { seat: s, drawn: None, after_kong: false, claimed: true };
                    self.new_decision(self.settings.turn_timeout);
                }
            }
            Some((s, Action::Chow(start), _)) => {
                if origin != (s + 3) % 4 {
                    // 小春的快攻：吃了不是上家的牌
                    self.announce_skill(s, Skill::QuickChow);
                    self.hand.as_mut().unwrap().players[s].skill_used = true;
                }
                let name = self.name(s);
                let h = self.hand.as_mut().unwrap();
                h.players[origin].discards.pop();
                h.any_claim = true;
                let p = &mut h.players[s];
                for x in start..start + 3 {
                    if x != tile {
                        Self::remove_tiles(&mut p.concealed, x, 1);
                    }
                }
                p.melds.push(Meld {
                    kind: MeldKind::Chow,
                    tiles: vec![start, start + 1, start + 2],
                    from: Some(origin),
                });
                h.step = Step::Turn { seat: s, drawn: None, after_kong: false, claimed: true };
                self.push_log(format!("{} 吃 {}", name, zh(tile)));
                self.push_event(s, "chow", Some(tile));
                self.new_decision(self.settings.turn_timeout);
            }
            Some(_) => {}
        }
    }

    fn do_tsumo(&mut self, seat: usize) {
        let h = self.hand.as_ref().unwrap();
        let Step::Turn { drawn, after_kong, .. } = h.step else { return };
        let p = &h.players[seat];
        let no_melds = p.melds.is_empty();
        let flags = WinFlags {
            self_draw: true,
            after_kong,
            last_tile: h.wall.len() <= RESERVE,
            heaven: seat == self.dealer && p.draws == 0 && !h.any_claim,
            earth: seat != self.dealer && p.draws == 1 && !h.any_claim && no_melds,
            ..Default::default()
        };
        self.finish_win(seat, None, drawn, flags);
    }

    fn do_concealed_kong(&mut self, seat: usize, t: Tile) {
        let name = self.name(seat);
        let h = self.hand.as_mut().unwrap();
        h.any_claim = true;
        let p = &mut h.players[seat];
        Self::remove_tiles(&mut p.concealed, t, 4);
        p.melds.push(Meld { kind: MeldKind::ConcealedKong, tiles: vec![t; 4], from: None });
        self.push_log(format!("{} 暗槓", name));
        self.push_event(seat, "kong", None);
        self.draw(seat, true, true);
    }

    fn do_added_kong(&mut self, seat: usize, t: Tile) {
        let h = self.hand.as_mut().unwrap();
        let mut options: [Vec<Action>; 4] = Default::default();
        for o in 0..4 {
            if o == seat {
                continue;
            }
            let q = &h.players[o];
            let mut c = counts(&q.concealed);
            c[t as usize] += 1;
            if !q.skip_ron && is_win(&c) {
                options[o] = vec![Action::Ron, Action::Pass];
            }
        }
        if options.iter().all(|o| o.is_empty()) {
            self.complete_added_kong(seat, t);
        } else {
            h.step = Step::RobKong { seat, tile: t, options, responses: Default::default() };
            let name = self.name(seat);
            self.push_log(format!("{} 要加槓 {}（可搶槓）", name, zh(t)));
            self.new_decision(self.settings.claim_timeout);
        }
    }

    fn complete_added_kong(&mut self, seat: usize, t: Tile) {
        let name = self.name(seat);
        let h = self.hand.as_mut().unwrap();
        h.any_claim = true;
        let p = &mut h.players[seat];
        Self::remove_tiles(&mut p.concealed, t, 1);
        if let Some(m) = p.melds.iter_mut().find(|m| m.kind == MeldKind::Pung && m.base() == t) {
            m.kind = MeldKind::Kong;
            m.tiles.push(t);
        }
        self.push_log(format!("{} 加槓 {}", name, zh(t)));
        self.push_event(seat, "kong", Some(t));
        self.draw(seat, true, true);
    }

    fn finish_win(&mut self, w: usize, payer: Option<usize>, win_tile: Option<Tile>, f: WinFlags) {
        let h = self.hand.as_ref().unwrap();
        let p = &h.players[w];
        let ctx = WinCtx {
            concealed: &p.concealed,
            melds: &p.melds,
            flowers: &p.flowers,
            win_tile,
            self_draw: f.self_draw,
            seat_wind: self.seat_wind(w),
            round_wind: self.round_wind,
            after_kong: f.after_kong,
            rob_kong: f.rob_kong,
            last_tile: f.last_tile,
            heaven: f.heaven,
            earth: f.earth,
            flower_win: f.flower_win,
        };
        let mut items = score(&ctx);
        // 子衡的「伏筆回收」：胡牌多 1 台
        let foreshadow = self.skill_of(w) == Some(Skill::Foreshadow);
        if foreshadow {
            items.push(ScoreItem { name: "伏筆回收".into(), tai: 1 });
        }
        let hand_tai: u32 = items.iter().map(|i| i.tai).sum();
        let mut dealer_items = vec![ScoreItem { name: "莊家".into(), tai: 1 }];
        if self.streak > 0 {
            dealer_items.push(ScoreItem {
                name: format!("連{}拉{}", self.streak, self.streak),
                tai: 2 * self.streak,
            });
        }
        let dealer_tai: u32 = dealer_items.iter().map(|i| i.tai).sum();
        let payers: Vec<usize> = match payer {
            Some(x) => vec![x],
            None => (0..4).filter(|&s| s != w).collect(),
        };
        let mut deltas = [0i64; 4];
        let mut payments = Vec::new();
        // 阿翼的銀之盾：放槍不用付錢（一場一次）
        let shielded = match (payer, f.self_draw) {
            (Some(l), false) if self.skill_of(l) == Some(Skill::Shield) && !self.shield_used[l] => Some(l),
            _ => None,
        };
        // 承岳的「穩固」：別人自摸時不用付錢（一場一次）
        let sturdy: Vec<usize> = if f.self_draw {
            payers.iter().copied().filter(|&s| self.skill_of(s) == Some(Skill::Sturdy) && !self.shield_used[s]).collect()
        } else {
            vec![]
        };
        for pr in payers {
            let tai = hand_tai + if w == self.dealer || pr == self.dealer { dealer_tai } else { 0 };
            let amount = if shielded == Some(pr) || sturdy.contains(&pr) { 0 } else { self.settings.base + tai as i64 * self.settings.per_tai };
            deltas[pr] -= amount;
            deltas[w] += amount;
            payments.push(Payment { from: pr, to: w, tai, amount });
        }
        for s in 0..4 {
            self.scores[s] += deltas[s];
        }
        let wname = self.name(w);
        // 顯示含莊家台的台數（自摸時取最高的那家）
        let shown_tai = payments.iter().map(|p| p.tai).max().unwrap_or(hand_tai);
        let mut summary = match (payer, f.self_draw) {
            (_, true) => format!("{} 自摸！{} 台", wname, shown_tai),
            (Some(l), false) => format!("{} 胡 {} 放的槍！{} 台", wname, self.name(l), shown_tai),
            (None, false) => format!("{} 胡牌！{} 台", wname, shown_tai),
        };
        if let Some(l) = shielded {
            self.shield_used[l] = true;
            self.hand.as_mut().unwrap().players[l].skill_used = true;
            summary.push_str(&format!("（{} 的銀之盾擋下了付款）", self.name(l)));
            self.announce_skill(l, Skill::Shield);
        }
        for &s in &sturdy {
            self.shield_used[s] = true;
            self.hand.as_mut().unwrap().players[s].skill_used = true;
            summary.push_str(&format!("（{} 的穩固：這次不用付）", self.name(s)));
            self.announce_skill(s, Skill::Sturdy);
        }
        if foreshadow {
            self.hand.as_mut().unwrap().players[w].skill_used = true;
            self.announce_skill(w, Skill::Foreshadow);
        }
        self.push_log(summary.clone());
        self.push_event(w, if f.self_draw { "tsumo" } else { "ron" }, win_tile);
        if let Some(l) = payer {
            self.push_event(l, "deal_in", win_tile);
        }
        self.result = Some(HandResult {
            draw: false,
            winner: Some(w),
            loser: payer,
            self_draw: f.self_draw,
            win_tile: win_tile.map(code),
            items,
            hand_tai,
            dealer_items,
            payments,
            deltas,
            dealer_stays: w == self.dealer,
            summary,
        });
        self.phase = Phase::HandOver;
        self.new_decision(self.settings.next_hand_delay);
    }

    fn next_hand(&mut self) {
        let stays = self.result.as_ref().map_or(true, |r| r.dealer_stays);
        if stays {
            self.streak += 1;
        } else {
            self.streak = 0;
            self.dealer = (self.dealer + 1) % 4;
            if self.dealer == self.start_dealer {
                self.round_wind += 1;
            }
        }
        if self.round_wind >= self.settings.rounds {
            self.phase = Phase::GameOver;
            self.deadline = None;
            self.new_decision(0);
            let mut order: Vec<usize> = (0..4).collect();
            order.sort_by_key(|&s| -self.scores[s]);
            let rank: Vec<String> = order
                .iter()
                .map(|&s| format!("{} {:+}", self.name(s), self.scores[s]))
                .collect();
            self.push_log(format!("遊戲結束！{}", rank.join("、")));
            return;
        }
        self.start_hand();
    }

    /// 每 ~200ms 呼叫一次：電腦出牌、逾時處理。回傳狀態是否有變。
    pub fn tick(&mut self) -> bool {
        let mut changed = false;
        for _ in 0..16 {
            if self.phase != Phase::Playing && self.phase != Phase::HandOver {
                break;
            }
            let now = Instant::now();
            let mut acted = false;
            if now.duration_since(self.decision_started) >= Duration::from_millis(self.settings.bot_delay_ms) {
                for s in 0..4 {
                    if !self.auto_plays(s) {
                        continue;
                    }
                    // 託管中的真人：一局結束要自己按「下一局」，才看得到誰胡牌（逾時仍照房間設定處理）
                    if self.phase == Phase::HandOver && !self.is_bot(s) {
                        continue;
                    }
                    let legal = self.legal_actions(s);
                    if legal.is_empty() {
                        continue;
                    }
                    let a = bot::choose(self, s, &legal);
                    if self.act(s, a).is_err() {
                        let _ = self.act(s, legal[legal.len() - 1].clone());
                    }
                    acted = true;
                    break;
                }
            }
            if !acted && self.deadline.map_or(false, |d| now >= d) {
                for s in 0..4 {
                    let legal = self.legal_actions(s);
                    if legal.is_empty() {
                        continue;
                    }
                    let a = self.timeout_action(s, &legal);
                    let name = self.name(s);
                    if self.phase == Phase::Playing {
                        self.push_log(format!("{} 逾時，自動處理", name));
                    }
                    let _ = self.act(s, a);
                    acted = true;
                    break;
                }
            }
            if !acted {
                break;
            }
            changed = true;
        }
        changed
    }

    fn timeout_action(&self, seat: usize, legal: &[Action]) -> Action {
        for pref in [Action::Tsumo, Action::Ron, Action::Next, Action::Pass] {
            if legal.contains(&pref) {
                return pref;
            }
        }
        bot::choose(self, seat, legal)
    }

    // ───────────── 輸出 ─────────────

    fn claim_tile(&self) -> Option<Tile> {
        match &self.hand.as_ref()?.step {
            Step::Claims { tile, .. } | Step::RobKong { tile, .. } => Some(*tile),
            _ => None,
        }
    }

    pub fn action_label(&self, a: &Action) -> String {
        let ct = self.claim_tile().map(zh).unwrap_or_default();
        match a {
            Action::Discard(t) => format!("打 {}", zh(*t)),
            Action::Tsumo => "自摸".into(),
            Action::ConcealedKong(t) => format!("暗槓 {}", zh(*t)),
            Action::AddedKong(t) => format!("加槓 {}", zh(*t)),
            Action::Ron => format!("胡 {}", ct),
            Action::Pung => format!("碰 {}", ct),
            Action::Kong => format!("槓 {}", ct),
            Action::Chow(s) => format!("吃 {}{}{}", zh(*s), zh(s + 1), zh(s + 2)),
            Action::Pass => "過".into(),
            Action::Next => "下一局".into(),
            Action::Skill => {
                let seat = self.hand.as_ref().and_then(|h| match h.step {
                    Step::Turn { seat, .. } => Some(seat),
                    _ => None,
                });
                match seat.and_then(|s| self.skill_of(s)) {
                    Some(k) => format!("技能：{}", k.name()),
                    None => "技能".into(),
                }
            }
        }
    }

    pub fn action_json(&self, id: usize, a: &Action) -> Value {
        let label = self.action_label(a);
        let ct = self.claim_tile().map(code);
        match a {
            Action::Discard(t) => json!({"id": id, "type": "discard", "tile": code(*t), "label": label}),
            Action::Tsumo => json!({"id": id, "type": "tsumo", "label": label}),
            Action::ConcealedKong(t) => json!({"id": id, "type": "concealed_kong", "tile": code(*t), "label": label}),
            Action::AddedKong(t) => json!({"id": id, "type": "added_kong", "tile": code(*t), "label": label}),
            Action::Ron => json!({"id": id, "type": "ron", "tile": ct, "label": label}),
            Action::Pung => json!({"id": id, "type": "pung", "tile": ct, "label": label}),
            Action::Kong => json!({"id": id, "type": "kong", "tile": ct, "label": label}),
            Action::Chow(s) => json!({"id": id, "type": "chow", "start": code(*s),
                "tiles": list_codes(&[*s, s + 1, s + 2]), "label": label}),
            Action::Pass => json!({"id": id, "type": "pass", "label": label}),
            Action::Next => json!({"id": id, "type": "next", "label": label}),
            Action::Skill => json!({"id": id, "type": "skill", "label": label}),
        }
    }

    /// 把外部送來的 JSON 轉成動作。接受 `{"action_id":3}` 或 `{"type":"discard","tile":"5m"}` 等。
    pub fn parse_action(&self, seat: usize, v: &Value) -> Result<Action, String> {
        let legal = self.legal_actions(seat);
        if legal.is_empty() {
            return Err("現在沒有輪到你做決定（沒有可執行的動作）".into());
        }
        if let Some(d) = v.get("decision_id").and_then(|x| x.as_u64()) {
            if d != self.decision_id {
                return Err(format!("decision_id {} 已過期，目前是 {}，請重新取得狀態", d, self.decision_id));
            }
        }
        let id = v.get("action_id").and_then(|x| x.as_u64().or_else(|| x.as_str()?.parse().ok()));
        if let Some(id) = id {
            return legal.get(id as usize).cloned().ok_or_else(|| format!("action_id {} 不存在", id));
        }
        let obj = v.get("action").filter(|a| a.is_object()).unwrap_or(v);
        let ty = obj
            .get("type")
            .and_then(|x| x.as_str())
            .or_else(|| v.get("action").and_then(|x| x.as_str()))
            .ok_or("需要 action_id 或 type")?
            .trim()
            .to_lowercase();
        let tile = obj.get("tile").and_then(|x| x.as_str()).and_then(parse);
        let a = match ty.as_str() {
            "discard" | "打" | "打牌" => Action::Discard(tile.ok_or("discard 需要 tile")?),
            "tsumo" | "zimo" | "self_draw" | "自摸" => Action::Tsumo,
            "ron" | "hu" | "win" | "胡" | "胡牌" => {
                if legal.contains(&Action::Tsumo) {
                    Action::Tsumo
                } else {
                    Action::Ron
                }
            }
            "pung" | "pong" | "peng" | "碰" => Action::Pung,
            "kong" | "gang" | "kan" | "槓" | "concealed_kong" | "added_kong" | "暗槓" | "加槓" => {
                let cands = [
                    tile.map(Action::ConcealedKong),
                    tile.map(Action::AddedKong),
                    Some(Action::Kong),
                ];
                cands
                    .into_iter()
                    .flatten()
                    .find(|a| legal.contains(a))
                    .or_else(|| {
                        let ks: Vec<&Action> = legal
                            .iter()
                            .filter(|a| matches!(a, Action::ConcealedKong(_) | Action::AddedKong(_)))
                            .collect();
                        (ks.len() == 1).then(|| ks[0].clone())
                    })
                    .ok_or("現在不能槓")?
            }
            "chow" | "chi" | "吃" => {
                let chows: Vec<Tile> = legal
                    .iter()
                    .filter_map(|a| if let Action::Chow(s) = a { Some(*s) } else { None })
                    .collect();
                let start = obj.get("start").and_then(|x| x.as_str()).and_then(parse);
                let tiles: Vec<Tile> = obj
                    .get("tiles")
                    .and_then(|x| x.as_array())
                    .map(|a| a.iter().filter_map(|t| t.as_str().and_then(parse)).collect())
                    .unwrap_or_default();
                let pick = if let Some(s) = start {
                    Some(s)
                } else if !tiles.is_empty() {
                    chows.iter().copied().find(|&s| tiles.iter().all(|&t| t >= s && t <= s + 2))
                } else if chows.len() == 1 {
                    Some(chows[0])
                } else {
                    None
                };
                Action::Chow(pick.ok_or("吃有多種組合，請指定 start（順子最小那張）或 action_id")?)
            }
            "pass" | "skip" | "過" | "不要" => Action::Pass,
            "next" | "ready" | "下一局" => Action::Next,
            "skill" | "技能" => Action::Skill,
            other => return Err(format!("未知的動作類型：{}", other)),
        };
        if legal.contains(&a) {
            Ok(a)
        } else {
            let opts: Vec<String> = legal.iter().map(|a| self.action_label(a)).collect();
            Err(format!("不合法的動作「{}」。可執行：{}", self.action_label(&a), opts.join("、")))
        }
    }

    fn step_info(&self) -> (Option<usize>, &'static str) {
        match self.hand.as_ref().map(|h| &h.step) {
            Some(Step::Turn { seat, .. }) => (Some(*seat), "turn"),
            Some(Step::Claims { discarder, .. }) => (Some(*discarder), "claims"),
            Some(Step::RobKong { seat, .. }) => (Some(*seat), "rob_kong"),
            None => (None, "none"),
        }
    }

    pub fn waiting_on(&self) -> Vec<usize> {
        (0..4).filter(|&s| self.seats[s].is_some() && self.has_actions(s)).collect()
    }

    pub fn view(&self, viewer: Option<usize>) -> Value {
        let reveal_all = matches!(self.phase, Phase::HandOver | Phase::GameOver);
        let started = self.phase != Phase::Lobby;
        let my_drawn = match (&self.hand, viewer) {
            (Some(h), Some(v)) if self.phase == Phase::Playing => match h.step {
                Step::Turn { seat, drawn, .. } if seat == v => drawn,
                _ => None,
            },
            _ => None,
        };
        let seats: Vec<Value> = (0..4)
            .map(|s| {
                let info = self.seats[s].as_ref();
                let mut v = json!({
                    "seat": s,
                    "name": info.map(|i| i.name.clone()),
                    "kind": info.map(|i| i.kind),
                    "connected": self.is_connected(s),
                    "score": self.scores[s],
                    "wind": if started { Some(WIND_ZH[self.seat_wind(s) as usize]) } else { None },
                    "is_dealer": started && s == self.dealer,
                    "auto": self.auto[s],
                    "character": info.map(|i| i.character.clone()),
                });
                if let Some(k) = self.skill_of(s) {
                    let used = if k.once_per_game() {
                        self.shield_used[s]
                    } else {
                        self.hand.as_ref().map_or(false, |h| h.players[s].skill_used)
                    };
                    v["skill"] = json!({ "id": k.id(), "name": k.name(), "desc": k.desc(), "active": k.is_active(), "used": used });
                }
                if let Some(h) = &self.hand {
                    let p = &h.players[s];
                    let show = reveal_all || viewer == Some(s);
                    let melds: Vec<Value> = p
                        .melds
                        .iter()
                        .map(|m| {
                            let hidden = m.kind == MeldKind::ConcealedKong && !show;
                            json!({
                                "kind": m.kind,
                                "tiles": if hidden { vec!["?".to_string(); 4] } else { list_codes(&m.tiles) },
                                "from": m.from,
                            })
                        })
                        .collect();
                    v["hand_count"] = json!(p.concealed.len());
                    v["melds"] = json!(melds);
                    v["flowers"] = json!(list_codes(&p.flowers));
                    v["discards"] = json!(list_codes(&p.discards));
                    if show {
                        let mut hand = p.concealed.clone();
                        hand.sort();
                        let drawn = if viewer == Some(s) { my_drawn } else { None };
                        if let Some(d) = drawn {
                            Self::remove_tiles(&mut hand, d, 1);
                        }
                        v["hand"] = json!(list_codes(&hand));
                        v["drawn"] = json!(drawn.map(code));
                    }
                }
                v
            })
            .collect();
        let actions: Vec<Value> = match viewer {
            Some(v) => self
                .legal_actions(v)
                .iter()
                .enumerate()
                .map(|(i, a)| self.action_json(i, a))
                .collect(),
            None => vec![],
        };
        let (turn, step) = self.step_info();
        let hand = self.hand.as_ref();
        let deadline_ms = self.deadline.map(|d| d.saturating_duration_since(Instant::now()).as_millis() as u64);
        let mut waits_v = Value::Null;
        let mut waits_left_v = Value::Null;
        let mut hint_v = Value::Null;
        if let (Some(h), Some(v)) = (hand, viewer) {
            let p = &h.players[v];
            if p.concealed.len() % 3 == 1 && self.phase == Phase::Playing {
                // 聽的牌，以及各還剩幾張沒出現（新手提示用）
                let wl = hints::waits_left(h, v);
                waits_v = json!(wl.iter().map(|&(t, _)| code(t)).collect::<Vec<_>>());
                waits_left_v = json!(wl.iter().map(|&(_, n)| n).collect::<Vec<_>>());
            }
            if self.phase == Phase::Playing {
                hint_v = hints::hint(self, v);
            }
        }
        let skill_info = match (hand, viewer) {
            (Some(h), Some(v)) if self.phase == Phase::Playing => self.skill_info(h, v),
            _ => Value::Null,
        };
        json!({
            "phase": self.phase,
            "you": viewer,
            "skill_info": skill_info,
            "seats": seats,
            "round_wind": WIND_ZH[(self.round_wind.min(3)) as usize],
            "dealer": self.dealer,
            "host_seat": self.host_seat,
            "streak": self.streak,
            "hand_no": self.hand_no,
            "wall_count": hand.map(|h| h.drawable()),
            "turn": turn,
            "step": step,
            "claim_tile": self.claim_tile().map(code),
            "last_discard": hand.and_then(|h| h.last_discard).map(|(s, t)| json!({"seat": s, "tile": code(t)})),
            "deadline_ms": deadline_ms,
            "decision_id": self.decision_id,
            "actions": actions,
            "waiting_on": self.waiting_on(),
            "my_waits": waits_v,
            "my_waits_left": waits_left_v,
            "hint": hint_v,
            "result": self.result,
            "log": self.log.iter().rev().take(30).rev().collect::<Vec<_>>(),
            "chat": self.chat,
            "events": self.events,
            "settings": self.settings,
        })
    }

    /// 只有自己看得到的技能效果：月華看穿的聽牌、蘭姐偷看的牌
    fn skill_info(&self, h: &Hand, v: usize) -> Value {
        let p = &h.players[v];
        let mut o = json!({});
        if p.see_waits_left > 0 {
            // 輪到對方、手上多一張時看不出聽什麼，就不顯示
            let seen: Vec<Value> = (0..4)
                .filter(|&s| s != v)
                .map(|s| {
                    let q = &h.players[s];
                    let w = (q.concealed.len() % 3 == 1).then(|| list_codes(&waits(&counts(&q.concealed))));
                    json!({ "seat": s, "waits": w })
                })
                .collect();
            o["see_waits"] = json!({ "left": p.see_waits_left, "seats": seen });
        }
        if let (Some(t), true) = (p.read_mind, p.read_mind_left > 0) {
            let mut tiles = h.players[t].concealed.clone();
            tiles.sort_unstable();
            o["read_mind"] = json!({ "seat": t, "left": p.read_mind_left, "tiles": list_codes(&tiles) });
        }
        let taken = h.front_drawn.saturating_sub(p.peek_at);
        if !p.peek.is_empty() && taken < p.peek.len() {
            o["peek"] = json!(list_codes(&p.peek[taken..]));
        }
        o
    }

    /// 給 AI 看的文字版狀態。
    pub fn text_view(&self, viewer: usize) -> String {
        use std::fmt::Write;
        let mut o = String::new();
        let ls = |ts: &[Tile]| -> String {
            if ts.is_empty() {
                "（無）".into()
            } else {
                ts.iter().map(|&t| label(t)).collect::<Vec<_>>().join(" ")
            }
        };
        let rel = |s: usize| -> &'static str {
            match (s + 4 - viewer) % 4 {
                0 => "你",
                1 => "下家",
                2 => "對家",
                _ => "上家",
            }
        };
        let _ = writeln!(o, "=== 虛擬台麻館・台灣十六張麻將 ===");
        match self.phase {
            Phase::Lobby => {
                let _ = writeln!(o, "階段：大廳，等待房主開始遊戲。");
                for s in 0..4 {
                    let _ = writeln!(o, "  座位{} {}", s, self.seats[s].as_ref().map(|i| i.name.as_str()).unwrap_or("（空）"));
                }
                let _ = writeln!(o, "請繼續呼叫 wait_for_turn 等待遊戲開始。");
                return o;
            }
            Phase::GameOver => {
                let _ = writeln!(o, "階段：遊戲結束。最終分數：");
                for s in 0..4 {
                    let _ = writeln!(o, "  座位{} {}（{}）: {:+}", s, self.name(s), rel(s), self.scores[s]);
                }
                if let Some(r) = &self.result {
                    let _ = writeln!(o, "最後一局：{}", r.summary);
                }
                return o;
            }
            _ => {}
        }
        let Some(h) = &self.hand else { return o };
        let _ = writeln!(
            o,
            "第{}局｜{}風圈｜莊家：{}{}｜牌牆可摸 {} 張｜decision_id={}",
            self.hand_no,
            WIND_ZH[self.round_wind.min(3) as usize],
            self.name(self.dealer),
            if self.streak > 0 { format!("（連{}）", self.streak) } else { String::new() },
            h.drawable(),
            self.decision_id
        );
        let _ = writeln!(o, "出牌順序：座位 0→1→2→3→0。只能吃「上家」（座位號比你小 1）打出的牌；碰、槓、胡不限");
        for s in 0..4 {
            let p = &h.players[s];
            let melds: Vec<String> = p
                .melds
                .iter()
                .map(|m| {
                    let k = match m.kind {
                        MeldKind::Chow => "吃",
                        MeldKind::Pung => "碰",
                        MeldKind::Kong => "明槓",
                        MeldKind::ConcealedKong => "暗槓",
                    };
                    if m.kind == MeldKind::ConcealedKong && s != viewer && self.phase == Phase::Playing {
                        format!("[{}]", k)
                    } else {
                        format!("[{} {}]", k, m.tiles.iter().map(|&t| code(t)).collect::<Vec<_>>().join(""))
                    }
                })
                .collect();
            let _ = writeln!(
                o,
                "\n座位{} {}（{}，{}家{}）分數 {:+}｜手牌 {} 張",
                s,
                self.name(s),
                rel(s),
                WIND_ZH[self.seat_wind(s) as usize],
                if s == self.dealer { "，莊" } else { "" },
                self.scores[s],
                p.concealed.len()
            );
            let _ = writeln!(o, "  副露：{}｜花：{}", if melds.is_empty() { "（無）".into() } else { melds.join(" ") }, ls(&p.flowers));
            let _ = writeln!(o, "  牌河：{}", ls(&p.discards));
        }
        let me = &h.players[viewer];
        let mut hand = me.concealed.clone();
        hand.sort();
        let drawn = match h.step {
            Step::Turn { seat, drawn, .. } if seat == viewer && self.phase == Phase::Playing => drawn,
            _ => None,
        };
        if let Some(d) = drawn {
            Self::remove_tiles(&mut hand, d, 1);
        }
        let _ = writeln!(o, "\n★ 你的手牌（{} 張）：{}", me.concealed.len(), ls(&hand));
        if let Some(k) = self.skill_of(viewer) {
            let used = if k.once_per_game() { self.shield_used[viewer] } else { me.skill_used };
            let _ = writeln!(o, "★ 你的技能「{}」：{}{}", k.name(), k.desc(), if used { "（已用過）" } else { "" });
            let info = self.skill_info(h, viewer);
            if let Some(sw) = info.get("see_waits") {
                for x in sw["seats"].as_array().into_iter().flatten() {
                    let s = x["seat"].as_u64().unwrap_or(0) as usize;
                    let w: Vec<String> = x["waits"].as_array().into_iter().flatten().filter_map(|t| t.as_str().map(String::from)).collect();
                    let txt = if x["waits"].is_null() { "（看不出來）".to_string() } else if w.is_empty() { "沒聽牌".to_string() } else { w.join(" ") };
                    let _ = writeln!(o, "  看穿 {}（{}）聽：{}", self.name(s), rel(s), txt);
                }
            }
            if let Some(rm) = info.get("read_mind") {
                let s = rm["seat"].as_u64().unwrap_or(0) as usize;
                let ts: Vec<&str> = rm["tiles"].as_array().into_iter().flatten().filter_map(|t| t.as_str()).collect();
                let _ = writeln!(o, "  讀心 {}（{}）的手牌：{}", self.name(s), rel(s), ts.join(" "));
            }
            if let Some(pk) = info.get("peek").and_then(|x| x.as_array()) {
                let ts: Vec<&str> = pk.iter().filter_map(|t| t.as_str()).collect();
                let _ = writeln!(o, "  牌牆接下來：{}", ts.join(" "));
            }
        }
        if let Some(d) = drawn {
            let _ = writeln!(o, "★ 剛摸到：{}", label(d));
        }
        if me.concealed.len() % 3 == 1 && self.phase == Phase::Playing {
            let w = waits(&counts(&me.concealed));
            if !w.is_empty() {
                let _ = writeln!(o, "★ 你目前聽牌：{}", ls(&w));
            }
        }
        if self.phase == Phase::HandOver {
            if let Some(r) = &self.result {
                let _ = writeln!(o, "\n【本局結束】{}", r.summary);
                for i in r.items.iter().chain(r.dealer_items.iter().filter(|_| !r.draw)) {
                    let _ = writeln!(o, "  {} {}台", i.name, i.tai);
                }
                for s in 0..4 {
                    let _ = writeln!(o, "  {}：{:+}", self.name(s), r.deltas[s]);
                }
            }
        } else {
            let (turn, step) = self.step_info();
            match (step, turn) {
                ("turn", Some(s)) if s == viewer => {
                    let _ = writeln!(o, "\n狀況：輪到你。請打出一張牌（或自摸 / 槓）。");
                }
                ("turn", Some(s)) => {
                    let _ = writeln!(o, "\n狀況：輪到 {}（{}）。", self.name(s), rel(s));
                }
                ("claims", Some(s)) | ("rob_kong", Some(s)) => {
                    let t = self.claim_tile().map(label).unwrap_or_default();
                    let what = if step == "claims" { "打出" } else { "要加槓" };
                    let _ = writeln!(o, "\n狀況：{}（{}）{} {}，等待其他人決定吃/碰/槓/胡。", self.name(s), rel(s), what, t);
                }
                _ => {}
            }
        }
        let legal = self.legal_actions(viewer);
        if legal.is_empty() {
            let _ = writeln!(o, "\n你現在沒有可執行的動作，請呼叫 wait_for_turn 等待。");
        } else {
            let _ = writeln!(o, "\n可執行動作（用 action_id 選擇，decision_id={}）：", self.decision_id);
            for (i, a) in legal.iter().enumerate() {
                let extra = match a {
                    Action::Discard(t) | Action::ConcealedKong(t) | Action::AddedKong(t) => format!(" [{}]", code(*t)),
                    _ => String::new(),
                };
                let _ = writeln!(o, "  {}: {}{}", i, self.action_label(a), extra);
            }
            if let Some(d) = self.deadline {
                let left = d.saturating_duration_since(Instant::now()).as_secs();
                let _ = writeln!(o, "（剩 {} 秒，逾時會自動處理）", left);
            }
        }
        if !self.chat.is_empty() {
            let _ = writeln!(o, "\n最近聊天：");
            for c in self.chat.iter().rev().take(5).rev() {
                let _ = writeln!(o, "  {}：{}", c.name, c.text);
            }
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_regular_characters_keep_their_identity_in_a_game() {
        let mut room = Room::new(Settings::default());
        for (id, name) in [("ziheng", "子衡"), ("chengyue", "承岳"), ("zhibai", "知白"), ("youchen", "佑辰")] {
            assert_eq!(character_name(id), Some(name));
            let (seat, _) = room.join(name, SeatKind::Human, Some(id)).unwrap();
            assert_eq!(room.seats[seat].as_ref().unwrap().character, id);
        }
        room.start_game().unwrap();
        let view = room.view(Some(0));
        for (seat, id) in ["ziheng", "chengyue", "zhibai", "youchen"].iter().enumerate() {
            assert_eq!(view["seats"][seat]["character"], *id);
        }
        assert!(!CHARACTERS.iter().any(|c| c.0 == "ayi"), "Ayi must stay out of random bot selection");
    }

    /// 四個電腦自己打完整場，確認不會卡住或 panic。
    #[test]
    fn bots_play_full_game() {
        let mut room = Room::new(Settings {
            bot_delay_ms: 0,
            rounds: 1,
            ..Default::default()
        });
        room.start_game().unwrap();
        let mut steps = 0;
        while room.phase != Phase::GameOver {
            assert!(room.tick() || room.phase == Phase::GameOver, "stuck at {:?}", room.phase);
            steps += 1;
            assert!(steps < 200_000);
        }
        assert_eq!(room.scores.iter().sum::<i64>(), 0);
        println!("hands: {} log tail: {:?}", room.hand_no, room.log.iter().rev().take(5).collect::<Vec<_>>());
    }

    /// 託管中的真人座位要被電腦打完整場，而且座位仍是真人的。
    #[test]
    fn auto_play_seat_finishes_game() {
        let mut room = Room::new(Settings { bot_delay_ms: 0, turn_timeout: 0, claim_timeout: 0, next_hand_delay: 0, ..Default::default() });
        let (seat, token) = room.join("悠太", SeatKind::Human, None).unwrap();
        room.set_auto(seat, true);
        room.start_game().unwrap();
        let mut steps = 0;
        while room.phase != Phase::GameOver {
            if room.phase == Phase::HandOver && room.legal_actions(seat).contains(&Action::Next) {
                // 託管時一局結束要自己按下一局
                room.tick();
                assert_eq!(room.phase, Phase::HandOver, "託管不該自動跳過結果");
                room.act(seat, Action::Next).unwrap();
                continue;
            }
            assert!(room.tick(), "stuck at {:?}", room.phase);
            steps += 1;
            assert!(steps < 200_000);
        }
        assert_eq!(room.seat_of_token(&token), Some(seat));
        assert_eq!(room.seats[seat].as_ref().unwrap().kind, SeatKind::Human);
        room.set_auto(seat, false);
        assert!(!room.auto[seat]);
    }

    /// 牌局設定成：座位 0 打出三萬（tile 2），下家（座位 1）可以吃、對家（座位 2）可以碰。
    fn claim_room() -> Room {
        let mut room = Room::new(Settings { bot_delay_ms: 0, turn_timeout: 0, claim_timeout: 0, next_hand_delay: 0, ..Default::default() });
        room.start_game().unwrap();
        let h = room.hand.as_mut().unwrap();
        h.players[0].concealed = vec![2, 30, 31, 32];
        h.players[1].concealed = vec![3, 4, 20];
        h.players[2].concealed = vec![2, 2, 25];
        h.players[3].concealed = vec![27, 28, 29];
        for p in h.players.iter_mut() {
            p.melds.clear();
        }
        h.step = Step::Turn { seat: 0, drawn: None, after_kong: false, claimed: false };
        room.do_discard(0, 2);
        room
    }

    /// 同時能碰和吃時，先問碰的人；他選「過」之後才輪到吃的人。
    #[test]
    fn pung_is_asked_before_chow() {
        let mut room = claim_room();
        assert!(room.legal_actions(1).is_empty(), "吃的人要等碰的人先決定");
        assert!(room.legal_actions(2).contains(&Action::Pung));
        assert_eq!(room.waiting_on(), vec![2]);
        let before = room.decision_id;
        room.act(2, Action::Pass).unwrap();
        assert!(room.legal_actions(1).contains(&Action::Chow(2)));
        assert!(room.decision_id > before, "輪到下一位時要重新計時");
        room.act(1, Action::Chow(2)).unwrap();
        let h = room.hand.as_ref().unwrap();
        assert!(matches!(h.step, Step::Turn { seat: 1, .. }));
        assert_eq!(h.players[1].melds[0].kind, MeldKind::Chow);
    }

    /// 碰的人碰下去，吃的人根本不會被問到。
    #[test]
    fn pung_wins_without_asking_chow() {
        let mut room = claim_room();
        room.act(2, Action::Pung).unwrap();
        let h = room.hand.as_ref().unwrap();
        assert!(matches!(h.step, Step::Turn { seat: 2, .. }));
        assert_eq!(h.players[2].melds[0].kind, MeldKind::Pung);
        assert!(h.players[1].melds.is_empty());
    }

    /// 技能模式的房間：四家依序是指定的角色，輪到座位 0 打牌（沒有摸牌）。
    fn skill_room(chars: [&str; 4]) -> Room {
        let mut room = Room::new(Settings { bot_delay_ms: 0, turn_timeout: 0, claim_timeout: 0, next_hand_delay: 0, skills: true, ..Default::default() });
        room.start_game().unwrap();
        for (s, c) in chars.iter().enumerate() {
            room.set_character(s, c).unwrap();
        }
        let h = room.hand.as_mut().unwrap();
        for p in h.players.iter_mut() {
            *p = PlayerState::default();
        }
        h.players[0].concealed = vec![2, 30, 31, 32];
        h.players[1].concealed = vec![9, 10, 20];
        h.players[2].concealed = vec![3, 4, 25];
        h.players[3].concealed = vec![27, 28, 29];
        h.step = Step::Turn { seat: 0, drawn: None, after_kong: false, claimed: false };
        room
    }

    fn bots_finish(settings: Settings, chars: [&str; 4]) {
        let mut room = Room::new(settings);
        room.start_game().unwrap();
        for (s, c) in chars.iter().enumerate() {
            room.set_character(s, c).unwrap();
        }
        let mut steps = 0;
        while room.phase != Phase::GameOver {
            assert!(room.tick() || room.phase == Phase::GameOver, "stuck at {:?}", room.phase);
            steps += 1;
            assert!(steps < 200_000);
        }
    }

    #[test]
    fn skills_off_by_default() {
        let mut room = skill_room(["xiaocui", "yuehua", "tangtang", "lanjie"]);
        room.settings.skills = false;
        assert!(!room.legal_actions(0).contains(&Action::Skill));
        assert!(room.view(Some(0))["seats"][0]["skill"].is_null());
    }

    #[test]
    fn bots_play_with_skills() {
        let s = Settings { bot_delay_ms: 0, rounds: 1, skills: true, ..Default::default() };
        bots_finish(s.clone(), ["xiaocui", "yuehua", "tangtang", "lanjie"]);
        bots_finish(s, ["aming", "xiaochun", "ayi", "yuehua"]);
    }

    /// 小翠：摸到的牌放回牌尾，重摸一張
    #[test]
    fn redraw_puts_tile_back() {
        let mut room = skill_room(["xiaocui", "yuehua", "tangtang", "lanjie"]);
        let h = room.hand.as_mut().unwrap();
        h.players[0].concealed.push(33);
        h.step = Step::Turn { seat: 0, drawn: Some(33), after_kong: false, claimed: false };
        // 重摸的那張固定不是花（摸到花會從牌尾補，測試就不穩定）
        h.wall.push_front(5);
        assert!(room.legal_actions(0).contains(&Action::Skill));
        room.act(0, Action::Skill).unwrap();
        let h = room.hand.as_ref().unwrap();
        assert_eq!(h.wall.back(), Some(&33));
        assert!(!h.players[0].concealed.contains(&33) || matches!(h.step, Step::Turn { drawn: Some(33), .. }));
        assert_eq!(h.players[0].concealed.len(), 5);
        assert!(h.players[0].skill_used);
        assert!(!room.legal_actions(0).contains(&Action::Skill), "每局一次");
    }

    /// 糖糖：發動後打出的牌別人不能碰
    #[test]
    fn no_claim_blocks_pung() {
        let mut room = skill_room(["tangtang", "yuehua", "xiaocui", "lanjie"]);
        room.hand.as_mut().unwrap().players[2].concealed = vec![2, 2, 25];
        room.act(0, Action::Skill).unwrap();
        room.act(0, Action::Discard(2)).unwrap();
        let h = room.hand.as_ref().unwrap();
        assert!(matches!(h.step, Step::Turn { seat: 1, .. }), "沒人能碰，直接換下家摸牌");
        // 下一張就恢復正常
        assert!(!h.players[0].no_claim_next);
    }

    /// 小春：可以吃對家打的牌，吃了技能就用掉
    #[test]
    fn quick_chow_from_anyone() {
        let mut room = skill_room(["lanjie", "yuehua", "xiaochun", "xiaocui"]);
        room.act(0, Action::Discard(2)).unwrap();
        assert!(room.legal_actions(2).contains(&Action::Chow(2)));
        room.act(2, Action::Chow(2)).unwrap();
        let h = room.hand.as_ref().unwrap();
        assert!(matches!(h.step, Step::Turn { seat: 2, .. }));
        assert!(h.players[2].skill_used);
        // 用過之後就只能吃上家
        let mut room = skill_room(["lanjie", "yuehua", "xiaocui", "xiaocui"]);
        room.act(0, Action::Discard(2)).unwrap();
        assert!(!room.legal_actions(2).iter().any(|a| matches!(a, Action::Chow(_))));
    }

    /// 蘭姐：看到牌牆接下來的 3 張，被摸走的會消失
    #[test]
    fn peek_shows_next_tiles() {
        let mut room = skill_room(["lanjie", "yuehua", "xiaocui", "tangtang"]);
        let next: Vec<String> = room.hand.as_ref().unwrap().wall.iter().take(3).map(|&t| code(t)).collect();
        room.act(0, Action::Skill).unwrap();
        assert_eq!(room.view(Some(0))["skill_info"]["peek"], json!(next));
        assert!(room.view(Some(1))["skill_info"]["peek"].is_null(), "只有自己看得到");
        room.act(0, Action::Discard(30)).unwrap();
        // 下家摸走第一張
        let left = room.view(Some(0))["skill_info"]["peek"].as_array().map(|a| a.len());
        assert!(left == Some(2) || room.hand.as_ref().unwrap().players[1].flowers.len() > 0);
    }

    /// 月華：看得到別人聽什麼
    #[test]
    fn see_waits_shows_opponents() {
        let mut room = skill_room(["yuehua", "xiaocui", "tangtang", "lanjie"]);
        // 下家聽三萬、六萬
        room.hand.as_mut().unwrap().players[1].concealed = vec![3, 4, 27, 27];
        room.act(0, Action::Skill).unwrap();
        let v = room.view(Some(0));
        let seats = v["skill_info"]["see_waits"]["seats"].as_array().unwrap().clone();
        let s1 = seats.iter().find(|x| x["seat"] == 1).unwrap();
        assert_eq!(s1["waits"], json!(["3m", "6m"]));
        assert!(room.text_view(0).contains("看穿"));
    }

    /// 阿翼：放槍不用付錢，一場一次
    #[test]
    fn shield_cancels_payment() {
        let mut room = skill_room(["ayi", "xiaocui", "tangtang", "lanjie"]);
        // 下家胡三萬
        room.hand.as_mut().unwrap().players[1].concealed = vec![3, 4, 27, 27];
        room.act(0, Action::Discard(2)).unwrap();
        room.act(1, Action::Ron).unwrap();
        let r = room.result.as_ref().unwrap();
        assert_eq!(r.loser, Some(0));
        assert_eq!(r.deltas, [0; 4]);
        assert!(room.shield_used[0]);
        assert!(r.summary.contains("銀之盾"));
    }

    /// 阿翼：別人自摸時銀之盾不會發動，也不會用掉；放槍用掉後，下一局仍然是用過的
    #[test]
    fn shield_only_for_deal_in_and_once_per_game() {
        let mut room = skill_room(["xiaocui", "ayi", "tangtang", "lanjie"]);
        // 座位 0 自摸：一般型 5 組 + 1 對
        let h = room.hand.as_mut().unwrap();
        h.players[0].concealed = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 27, 27];
        h.step = Step::Turn { seat: 0, drawn: Some(27), after_kong: false, claimed: false };
        h.players[0].draws = 3;
        room.act(0, Action::Tsumo).unwrap();
        let r = room.result.as_ref().unwrap();
        assert!(r.self_draw);
        assert!(r.deltas[1] < 0, "自摸時阿翼照樣要付");
        assert!(!room.shield_used[1], "自摸不會用掉銀之盾");
        // 下一局阿翼放槍：盾發動、之後一直是用過的
        room.next_hand();
        let h = room.hand.as_mut().unwrap();
        for p in h.players.iter_mut() {
            *p = PlayerState::default();
        }
        h.players[1].concealed = vec![2, 30];
        h.players[2].concealed = vec![3, 4, 27, 27];
        h.players[0].concealed = vec![28, 28, 29, 29];
        h.players[3].concealed = vec![31, 31, 32, 32];
        h.step = Step::Turn { seat: 1, drawn: None, after_kong: false, claimed: false };
        room.act(1, Action::Discard(2)).unwrap();
        room.act(2, Action::Ron).unwrap();
        assert!(room.shield_used[1]);
        assert_eq!(room.result.as_ref().unwrap().deltas, [0; 4]);
        room.next_hand();
        assert!(room.shield_used[1], "一場一次：下一局還是用過的");
        assert_eq!(room.view(Some(1))["seats"][1]["skill"]["used"], true);
        // 新的一場重新開始才會恢復
        room.phase = Phase::GameOver;
        room.start_game().unwrap();
        assert!(!room.shield_used[1]);
    }

    /// 知白：休止符那一張別人不能胡，吃碰還是可以；下一張恢復
    #[test]
    fn rest_blocks_ron_only() {
        let mut room = skill_room(["zhibai", "xiaocui", "tangtang", "lanjie"]);
        let h = room.hand.as_mut().unwrap();
        h.players[1].concealed = vec![3, 4, 27, 27]; // 下家聽二萬五萬
        h.players[2].concealed = vec![2, 2, 25]; // 對家可以碰二萬
        room.act(0, Action::Skill).unwrap();
        room.act(0, Action::Discard(2)).unwrap();
        let h = room.hand.as_ref().unwrap();
        let Step::Claims { options, .. } = &h.step else { panic!("對家還能碰，要等搶牌") };
        assert!(!options[1].contains(&Action::Ron), "休止符：不能胡");
        assert!(options[2].contains(&Action::Pung), "碰不受影響");
        assert!(!h.players[0].no_ron_next);
        assert!(h.players[0].skill_used);
    }

    /// 子衡：胡牌多 1 台
    #[test]
    fn foreshadow_adds_one_tai() {
        let mut room = skill_room(["xiaocui", "ziheng", "tangtang", "lanjie"]);
        room.hand.as_mut().unwrap().players[1].concealed = vec![3, 4, 27, 27];
        room.act(0, Action::Discard(2)).unwrap();
        room.act(1, Action::Ron).unwrap();
        let r = room.result.as_ref().unwrap();
        assert!(r.items.iter().any(|i| i.name == "伏筆回收" && i.tai == 1));
        // 沒有技能模式就不加
        let mut room = skill_room(["xiaocui", "ziheng", "tangtang", "lanjie"]);
        room.settings.skills = false;
        room.hand.as_mut().unwrap().players[1].concealed = vec![3, 4, 27, 27];
        room.act(0, Action::Discard(2)).unwrap();
        room.act(1, Action::Ron).unwrap();
        assert!(!room.result.as_ref().unwrap().items.iter().any(|i| i.name == "伏筆回收"));
    }

    /// 承岳：別人自摸時不用付，一場一次；放槍照付
    #[test]
    fn sturdy_skips_tsumo_payment_once_per_game() {
        let tsumo = |room: &mut Room| {
            let h = room.hand.as_mut().unwrap();
            for p in h.players.iter_mut() {
                *p = PlayerState::default();
            }
            h.players[0].concealed = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 27, 27];
            h.players[0].draws = 3;
            h.step = Step::Turn { seat: 0, drawn: Some(27), after_kong: false, claimed: false };
            room.act(0, Action::Tsumo).unwrap();
        };
        let mut room = skill_room(["xiaocui", "chengyue", "tangtang", "lanjie"]);
        tsumo(&mut room);
        let r = room.result.as_ref().unwrap();
        assert_eq!(r.deltas[1], 0, "穩固：不用付");
        assert!(r.deltas[2] < 0 && r.deltas[3] < 0);
        assert!(r.summary.contains("穩固"));
        assert!(room.shield_used[1]);
        room.next_hand();
        assert_eq!(room.view(Some(1))["seats"][1]["skill"]["used"], true, "一場一次");
        tsumo(&mut room);
        assert!(room.result.as_ref().unwrap().deltas[1] < 0, "第二次要付");
    }

    /// 佑辰：讀心術看到分數最高那家的手牌，自己打三張後消失，別人看不到
    #[test]
    fn read_mind_shows_leader_hand() {
        let mut room = skill_room(["youchen", "xiaocui", "tangtang", "lanjie"]);
        room.scores = [0, 10, 50, 50];
        room.hand.as_mut().unwrap().players[0].concealed = vec![2, 5, 6, 7, 30, 31, 32];
        room.act(0, Action::Skill).unwrap();
        let v = room.view(Some(0));
        assert_eq!(v["skill_info"]["read_mind"]["seat"], json!(2), "同分取離自己近的那家");
        assert_eq!(v["skill_info"]["read_mind"]["tiles"].as_array().unwrap().len(), 3);
        assert!(room.view(Some(1))["skill_info"]["read_mind"].is_null(), "只有自己看得到");
        assert!(room.text_view(0).contains("讀心"));
        for t in [5, 6, 7] {
            let h = room.hand.as_mut().unwrap();
            h.step = Step::Turn { seat: 0, drawn: None, after_kong: false, claimed: false };
            room.act(0, Action::Discard(t)).unwrap();
        }
        assert!(room.view(Some(0))["skill_info"]["read_mind"].is_null(), "打完三張就看不到了");
    }

    /// 新角色的技能也能讓電腦打完整場
    #[test]
    fn bots_play_with_new_skills() {
        let s = Settings { bot_delay_ms: 0, rounds: 1, skills: true, ..Default::default() };
        bots_finish(s, ["ziheng", "chengyue", "zhibai", "youchen"]);
    }

    /// 阿明：分數最低時開局換牌（洗牌是隨機的，試幾次一定會發動）
    #[test]
    fn comeback_improves_deal() {
        let mut fired = false;
        for _ in 0..20 {
            let mut room = Room::new(Settings { bot_delay_ms: 0, skills: true, ..Default::default() });
            room.start_game().unwrap();
            room.set_character(0, "aming").unwrap();
            room.scores = [-500, 100, 200, 200];
            room.start_hand();
            if room.phase == Phase::Playing && room.hand.as_ref().unwrap().players[0].skill_used {
                fired = true;
                break;
            }
        }
        assert!(fired);
    }

    #[test]
    fn text_view_works() {
        let mut room = Room::new(Settings { bot_delay_ms: 0, ..Default::default() });
        let (seat, _) = room.join("Claude", SeatKind::Agent, Some("yuehua")).unwrap();
        room.start_game().unwrap();
        assert_eq!(room.seats[seat].as_ref().unwrap().character, "yuehua");
        // 電腦的角色不重複
        let mut cs: Vec<String> = room.seats.iter().map(|s| s.as_ref().unwrap().character.clone()).collect();
        cs.sort();
        cs.dedup();
        assert_eq!(cs.len(), 4);
        let t = room.text_view(seat);
        assert!(t.contains("你的手牌"));
        let v = room.view(Some(seat));
        assert!(v["seats"][seat]["hand"].is_array());
    }
}
