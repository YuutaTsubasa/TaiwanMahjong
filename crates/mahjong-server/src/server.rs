//! 遊戲伺服器：
//! - `/ws`          人類玩家（Svelte 介面）用的 WebSocket
//! - `/api/*`       AI / 程式用的 HTTP API（長輪詢）
//! - `/mcp`         MCP（Streamable HTTP），給 Claude Code / ChatGPT 連線
//! - `/llms.txt`    給 AI 看的連線說明
//! - `/openapi.json` 給 ChatGPT GPT Actions 用
//! - 其他路徑       前端靜態檔（讓沒裝 App 的人用瀏覽器加入）
//!
//! 兩種模式：
//! - 單一房間（桌面版開房、區網）：上面的路徑直接對應唯一的房間
//! - 公開伺服器（`--public`，自己架主機用）：`POST /api/rooms` 開房拿到房號，
//!   每個房間的路徑都在 `/r/{房號}/` 底下，例如 `/r/K7QX2/ws`、`/r/K7QX2/mcp`

use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequestParts, Path, Query, Request, State,
    },
    middleware::{self, Next},
    http::{header, request::Parts, HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use mahjong_core::protocol::{client_disconnected, client_view, handle_client_msg, ClientCtx};
use mahjong_core::room::{Phase, Room, SeatKind, Settings, CHARACTERS};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tower_http::cors::CorsLayer;

pub type AssetFn = Arc<dyn Fn(&str) -> Option<(Vec<u8>, String)> + Send + Sync>;

/// 一個房間（一張牌桌）
pub struct AppState {
    pub room: Mutex<Room>,
    notify: watch::Sender<u64>,
    shutdown: watch::Receiver<bool>,
    pub host_key: String,
    /// 公開伺服器上的房號；單一房間模式是 None
    pub code: Option<String>,
    mcp_sessions: Mutex<HashMap<String, String>>,
    /// 目前開著的 WebSocket 數量，和最後一次有人動作的時間：用來回收沒人的房間
    conns: AtomicUsize,
    last_active: Mutex<Instant>,
}

impl AppState {
    fn new(settings: Settings, code: Option<String>, shutdown: watch::Receiver<bool>) -> Self {
        let mut room = Room::new(Settings::default());
        room.update_settings(settings);
        let (notify, _) = watch::channel(0u64);
        AppState {
            room: Mutex::new(room),
            notify,
            shutdown,
            host_key: gen_key(20),
            code,
            mcp_sessions: Mutex::new(HashMap::new()),
            conns: AtomicUsize::new(0),
            last_active: Mutex::new(Instant::now()),
        }
    }

    fn mark_active(&self) {
        *self.last_active.lock().unwrap() = Instant::now();
    }

    /// 這個房間在網址上的前綴（"/r/K7QX2"；單一房間是空字串）
    fn prefix(&self) -> String {
        self.code.as_ref().map(|c| format!("/r/{}", c)).unwrap_or_default()
    }

    pub fn bump(&self) {
        self.notify.send_modify(|v| *v += 1);
    }

    fn version(&self) -> u64 {
        *self.notify.borrow()
    }

    fn view(&self, room: &Room, seat: Option<usize>) -> Value {
        let mut v = room.view(seat);
        v["version"] = json!(self.version());
        v
    }
}

pub struct ServerHandle {
    pub state: Arc<AppState>,
    shutdown: watch::Sender<bool>,
    pub port: u16,
    pub host_key: String,
}

impl ServerHandle {
    pub fn stop(&self) {
        let _ = self.shutdown.send(true);
    }
}

fn gen_key(n: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(n).map(char::from).collect()
}

/// 公開伺服器的限制
const MAX_ROOMS: usize = 300;
/// 同一個 IP 在 CREATE_WINDOW 內最多開幾間房
const MAX_CREATES_PER_IP: usize = 6;
const CREATE_WINDOW: Duration = Duration::from_secs(10 * 60);
/// 沒有任何連線、也沒人動作超過這麼久的房間會被回收
const ROOM_IDLE: Duration = Duration::from_secs(15 * 60);
/// 房號用的字元（去掉容易看錯的 0/O、1/I/L）
const CODE_CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
const CODE_LEN: usize = 5;

// 防止被惡意連線灌爆（公開伺服器才限制；只有一個房間的區網伺服器不受影響）
/// 每個房間最多幾個 WebSocket（4 個座位＋觀戰）
const MAX_CONNS_PER_ROOM: usize = 12;
/// 同一個 IP 最多同時幾個 WebSocket
const MAX_CONNS_PER_IP: usize = 10;
/// 整台伺服器最多幾個 WebSocket
const MAX_TOTAL_CONNS: usize = 2000;
/// 每個 WebSocket 每秒最多幾則訊息（可以短暫衝到 WS_BURST），超過就斷線
const WS_RATE: f64 = 8.0;
const WS_BURST: f64 = 20.0;
/// 每個 IP 每秒最多幾個 HTTP 請求（可以短暫衝到 HTTP_BURST），超過回 429
const HTTP_RATE: f64 = 10.0;
const HTTP_BURST: f64 = 40.0;

/// 令牌桶：每秒補 rate 個，最多存 burst 個，每次動作拿一個
struct Bucket {
    tokens: f64,
    last: Instant,
}

impl Bucket {
    fn new(burst: f64) -> Self {
        Bucket { tokens: burst, last: Instant::now() }
    }

    fn take(&mut self, rate: f64, burst: f64) -> bool {
        let now = Instant::now();
        self.tokens = (self.tokens + now.duration_since(self.last).as_secs_f64() * rate).min(burst);
        self.last = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// 整台伺服器：單一房間模式只有 `default`；公開模式的房間放在 `rooms`
pub struct Hub {
    default: Option<Arc<AppState>>,
    rooms: Mutex<HashMap<String, Arc<AppState>>>,
    public: bool,
    assets: Option<AssetFn>,
    shutdown: watch::Receiver<bool>,
    creates: Mutex<HashMap<String, Vec<Instant>>>,
    ip_conns: Mutex<HashMap<String, usize>>,
    total_conns: AtomicUsize,
    http_buckets: Mutex<HashMap<String, Bucket>>,
}

impl Hub {
    fn get_room(&self, code: &str) -> Option<Arc<AppState>> {
        self.rooms.lock().unwrap().get(&code.to_ascii_uppercase()).cloned()
    }

    fn all_rooms(&self) -> Vec<Arc<AppState>> {
        let mut v: Vec<_> = self.rooms.lock().unwrap().values().cloned().collect();
        v.extend(self.default.clone());
        v
    }

    fn create_room(&self, ip: &str, settings: Settings) -> Result<Arc<AppState>, String> {
        {
            let mut creates = self.creates.lock().unwrap();
            let list = creates.entry(ip.to_string()).or_default();
            list.retain(|t| t.elapsed() < CREATE_WINDOW);
            if list.len() >= MAX_CREATES_PER_IP {
                return Err("開房太頻繁了，請過幾分鐘再試".into());
            }
            list.push(Instant::now());
        }
        let mut rooms = self.rooms.lock().unwrap();
        if rooms.len() >= MAX_ROOMS {
            return Err("伺服器的房間滿了，請稍後再試".into());
        }
        let mut rng = rand::thread_rng();
        let code = loop {
            let c: String = (0..CODE_LEN).map(|_| CODE_CHARS[rng.gen_range(0..CODE_CHARS.len())] as char).collect();
            // 至少要有一個英文字母，才不會跟「只打通訊埠號碼」的位址搞混
            if !rooms.contains_key(&c) && c.bytes().any(|b| b.is_ascii_alphabetic()) {
                break c;
            }
        };
        let st = Arc::new(AppState::new(settings, Some(code.clone()), self.shutdown.clone()));
        rooms.insert(code, st.clone());
        Ok(st)
    }

    /// 回收沒人的房間
    fn sweep(&self) {
        self.rooms.lock().unwrap().retain(|_, st| {
            st.conns.load(Ordering::Relaxed) > 0 || st.last_active.lock().unwrap().elapsed() < ROOM_IDLE
        });
        self.creates.lock().unwrap().retain(|_, v| v.iter().any(|t| t.elapsed() < CREATE_WINDOW));
        self.http_buckets.lock().unwrap().retain(|_, b| b.last.elapsed() < Duration::from_secs(60));
    }

    /// 公開伺服器：檢查連線數上限，通過就占一個名額（ConnGuard 消失時歸還）
    fn admit(self: &Arc<Self>, st: &AppState, ip: &str) -> Result<Option<ConnGuard>, &'static str> {
        if !self.public {
            return Ok(None);
        }
        if st.conns.load(Ordering::Relaxed) >= MAX_CONNS_PER_ROOM {
            return Err("這個房間的連線已滿");
        }
        if self.total_conns.load(Ordering::Relaxed) >= MAX_TOTAL_CONNS {
            return Err("伺服器忙碌中，請稍後再試");
        }
        let mut ips = self.ip_conns.lock().unwrap();
        let n = ips.entry(ip.to_string()).or_insert(0);
        if *n >= MAX_CONNS_PER_IP {
            return Err("同時開太多連線了");
        }
        *n += 1;
        self.total_conns.fetch_add(1, Ordering::Relaxed);
        Ok(Some(ConnGuard { hub: self.clone(), ip: ip.to_string() }))
    }
}

struct ConnGuard {
    hub: Arc<Hub>,
    ip: String,
}

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.hub.total_conns.fetch_sub(1, Ordering::Relaxed);
        let mut ips = self.hub.ip_conns.lock().unwrap();
        if let Some(n) = ips.get_mut(&self.ip) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                ips.remove(&self.ip);
            }
        }
    }
}

/// 公開伺服器：每個 IP 的 HTTP 請求頻率限制
async fn rate_limit(State(hub): State<Arc<Hub>>, req: Request, next: Next) -> Response {
    if hub.public {
        let ip = client_ip(req.headers());
        let ok = hub
            .http_buckets
            .lock()
            .unwrap()
            .entry(ip)
            .or_insert_with(|| Bucket::new(HTTP_BURST))
            .take(HTTP_RATE, HTTP_BURST);
        if !ok {
            return (StatusCode::TOO_MANY_REQUESTS, Json(json!({"ok": false, "error": "請求太頻繁，請放慢一點"})))
                .into_response();
        }
    }
    next.run(req).await
}

/// 單一房間模式（桌面版開房、區網伺服器）
pub async fn start(port: u16, settings: Settings, assets: Option<AssetFn>) -> Result<ServerHandle, String> {
    let (sd_tx, sd_rx) = watch::channel(false);
    let state = Arc::new(AppState::new(settings, None, sd_rx.clone()));
    let host_key = state.host_key.clone();
    let hub = Hub {
        default: Some(state.clone()),
        rooms: Mutex::new(HashMap::new()),
        public: false,
        assets,
        shutdown: sd_rx,
        creates: Mutex::new(HashMap::new()),
        ip_conns: Mutex::new(HashMap::new()),
        total_conns: AtomicUsize::new(0),
        http_buckets: Mutex::new(HashMap::new()),
    };
    let port = serve(port, hub, sd_tx.subscribe()).await?;
    Ok(ServerHandle { state, shutdown: sd_tx, port, host_key })
}

/// 公開伺服器：任何人都能開房（自己架在主機上用）。執行到收到 Ctrl+C 為止。
pub async fn run_public(port: u16, assets: Option<AssetFn>) -> Result<(), String> {
    let (sd_tx, sd_rx) = watch::channel(false);
    let hub = Hub {
        default: None,
        rooms: Mutex::new(HashMap::new()),
        public: true,
        assets,
        shutdown: sd_rx,
        creates: Mutex::new(HashMap::new()),
        ip_conns: Mutex::new(HashMap::new()),
        total_conns: AtomicUsize::new(0),
        http_buckets: Mutex::new(HashMap::new()),
    };
    let port = serve(port, hub, sd_tx.subscribe()).await?;
    println!("虛擬台麻館公開伺服器已啟動，埠 {}", port);
    let _ = tokio::signal::ctrl_c().await;
    let _ = sd_tx.send(true);
    Ok(())
}

async fn serve(port: u16, hub: Hub, sd_rx: watch::Receiver<bool>) -> Result<u16, String> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| format!("無法監聽埠 {}：{}", port, e))?;
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    let hub = Arc::new(hub);

    // 計時器：電腦出牌、逾時；公開伺服器順便回收沒人的房間
    let h = hub.clone();
    let mut sd1 = sd_rx.clone();
    tokio::spawn(async move {
        let mut iv = tokio::time::interval(Duration::from_millis(150));
        let mut n: u32 = 0;
        loop {
            tokio::select! {
                _ = iv.tick() => {
                    for st in h.all_rooms() {
                        let changed = st.room.lock().unwrap().tick();
                        if changed { st.bump(); }
                    }
                    n = n.wrapping_add(1);
                    if h.public && n % 200 == 0 { h.sweep(); }
                }
                _ = sd1.changed() => break,
            }
        }
    });

    let app = router(hub);
    let mut sd2 = sd_rx;
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = sd2.changed().await;
            })
            .await;
    });
    Ok(port)
}

/// 從網址找出要操作的房間：`/r/{code}/…` 用房號找，其他路徑用單一房間
struct RoomRef(Arc<AppState>);

impl FromRequestParts<Arc<Hub>> for RoomRef {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, hub: &Arc<Hub>) -> Result<Self, Self::Rejection> {
        let code = Path::<HashMap<String, String>>::from_request_parts(parts, hub)
            .await
            .ok()
            .and_then(|p| p.0.get("code").cloned());
        let st = match code {
            Some(c) => hub.get_room(&c).ok_or("找不到這個房號（房間可能已經關閉了）"),
            None => hub.default.clone().ok_or("這是公開伺服器，請在網址加上房號，例如 /r/房號/llms.txt"),
        };
        st.map(RoomRef)
            .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"ok": false, "error": e}))).into_response())
    }
}

fn room_routes() -> Router<Arc<Hub>> {
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/info", get(api_info))
        .route("/api/join", post(api_join))
        .route("/api/state", get(api_state))
        .route("/api/wait", get(api_wait))
        .route("/api/action", post(api_action))
        .route("/api/chat", post(api_chat))
        .route("/llms.txt", get(llms_txt))
        .route("/openapi.json", get(openapi))
        .route("/mcp", post(mcp_post).get(mcp_get).delete(mcp_delete))
}

pub fn router(hub: Arc<Hub>) -> Router {
    Router::new()
        .merge(room_routes())
        .nest("/r/{code}", room_routes())
        .route("/api/rooms", post(api_create_room))
        .route("/api/health", get(|| async { "ok" }))
        .fallback(static_handler)
        .layer(middleware::from_fn_with_state(hub.clone(), rate_limit))
        .layer(CorsLayer::permissive())
        .with_state(hub)
}

// ───────────── 開房（公開伺服器） ─────────────

fn client_ip(headers: &HeaderMap) -> String {
    // 主機的代理伺服器會帶上真正的來源 IP（Railway：X-Real-IP；Fly.io：Fly-Client-IP）
    headers
        .get("x-real-ip")
        .or_else(|| headers.get("fly-client-ip"))
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "local".into())
}

async fn api_create_room(State(hub): State<Arc<Hub>>, headers: HeaderMap, body: Bytes) -> Response {
    if !hub.public {
        return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "error": "這台伺服器只有一個房間，不能另外開房"})))
            .into_response();
    }
    let v = match parse_body(&body) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let settings = v
        .get("settings")
        .cloned()
        .and_then(|s| serde_json::from_value::<Settings>(s).ok())
        .unwrap_or_default();
    match hub.create_room(&client_ip(&headers), settings) {
        Ok(st) => Json(json!({"ok": true, "code": st.code, "host_key": st.host_key})).into_response(),
        Err(e) => (StatusCode::TOO_MANY_REQUESTS, Json(json!({"ok": false, "error": e}))).into_response(),
    }
}

// ───────────── WebSocket（人類） ─────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(hub): State<Arc<Hub>>,
    headers: HeaderMap,
    RoomRef(st): RoomRef,
) -> Response {
    let guard = match hub.admit(&st, &client_ip(&headers)) {
        Ok(g) => g,
        Err(e) => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"ok": false, "error": e}))).into_response(),
    };
    let limited = hub.public;
    ws.max_message_size(64 * 1024).on_upgrade(move |sock| async move {
        ws_session(sock, st, limited).await;
        drop(guard);
    })
}

async fn ws_session(socket: WebSocket, st: Arc<AppState>, limited: bool) {
    let mut bucket = Bucket::new(WS_BURST);
    let (mut out, mut inp) = socket.split();
    let mut ctx = ClientCtx::default();
    let mut notify = st.notify.subscribe();
    let mut shutdown = st.shutdown.clone();
    let mut ping = tokio::time::interval(Duration::from_secs(20));
    st.conns.fetch_add(1, Ordering::Relaxed);
    st.mark_active();
    loop {
        let mut msgs: Vec<Value> = Vec::new();
        let mut send_state = false;
        tokio::select! {
            m = inp.next() => {
                match m {
                    Some(Ok(Message::Text(t))) => {
                        // 狂送訊息的連線直接斷掉（每則訊息都會回傳整張牌桌，太多會把流量灌爆）
                        if limited && !bucket.take(WS_RATE, WS_BURST) { break; }
                        st.mark_active();
                        let v: Value = serde_json::from_str(t.as_str()).unwrap_or(Value::Null);
                        let (replies, changed) = handle_client_msg(&mut st.room.lock().unwrap(), &st.host_key, &mut ctx, &v);
                        msgs = replies;
                        if changed { st.bump(); }
                        send_state = true;
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
            r = notify.changed() => {
                if r.is_err() { break; }
                notify.borrow_and_update();
                send_state = true;
            }
            _ = shutdown.changed() => break,
            _ = ping.tick() => {
                if out.send(Message::Ping(Vec::new().into())).await.is_err() { break; }
            }
        }
        if send_state {
            let view = {
                let room = st.room.lock().unwrap();
                let mut v = client_view(&room, &ctx);
                v["version"] = json!(st.version());
                v
            };
            msgs.push(json!({"type": "state", "state": view}));
        }
        let mut failed = false;
        for m in msgs {
            if out.send(Message::Text(m.to_string().into())).await.is_err() {
                failed = true;
                break;
            }
        }
        if failed {
            break;
        }
    }
    if ctx.seat.is_some() {
        client_disconnected(&mut st.room.lock().unwrap(), &ctx);
        st.bump();
    }
    st.conns.fetch_sub(1, Ordering::Relaxed);
    st.mark_active();
}

// ───────────── HTTP API（AI / 程式） ─────────────

fn bad(msg: impl Into<String>) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": msg.into()}))).into_response()
}

async fn api_info(RoomRef(st): RoomRef) -> Json<Value> {
    let room = st.room.lock().unwrap();
    let seats: Vec<Value> = (0..4)
        .map(|s| {
            json!({
                "seat": s,
                "name": room.seats[s].as_ref().map(|i| i.name.clone()),
                "kind": room.seats[s].as_ref().map(|i| i.kind),
            })
        })
        .collect();
    Json(json!({
        "game": "虛擬台麻館",
        "rules": "台灣十六張麻將",
        "phase": room.phase,
        "seats": seats,
        "docs": format!("{}/llms.txt", st.prefix()),
        "mcp": format!("{}/mcp", st.prefix()),
        "openapi": format!("{}/openapi.json", st.prefix()),
    }))
}

/// 解析 JSON 請求本文。失敗時說明原因——最常見的是 Windows 終端機把中文用 Big5 送出。
fn parse_body(body: &Bytes) -> Result<Value, Response> {
    if body.iter().all(|b| b.is_ascii_whitespace()) {
        return Ok(json!({}));
    }
    if std::str::from_utf8(body).is_err() {
        return Err(bad(
            r#"請求本文不是 UTF-8。Windows 的 Git Bash / cmd 送中文時常被轉成 Big5；請把中文寫成 \uXXXX 跳脫（例如 "\u5b89\u5b89" = 安安），或把 JSON 存成 UTF-8 檔案再用 curl --data-binary @body.json 送出。"#,
        ));
    }
    serde_json::from_slice(body).map_err(|e| bad(format!("JSON 格式錯誤：{}", e)))
}

#[derive(Deserialize, Default)]
struct JoinReq {
    name: Option<String>,
    character: Option<String>,
    token: Option<String>,
}

fn do_join(st: &AppState, name: &str, token: Option<&str>, character: Option<&str>) -> Result<(usize, String), String> {
    st.mark_active();
    let mut room = st.room.lock().unwrap();
    if let Some(tok) = token {
        if let Some(s) = room.seat_of_token(tok) {
            room.touch(s);
            return Ok((s, tok.to_string()));
        }
    }
    let (s, tok) = room.join(name, SeatKind::Agent, character)?;
    room.touch(s);
    drop(room);
    st.bump();
    Ok((s, tok))
}

async fn api_join(RoomRef(st): RoomRef, body: Bytes) -> Response {
    let req: JoinReq = match parse_body(&body) {
        Ok(v) => serde_json::from_value(v).unwrap_or_default(),
        Err(r) => return r,
    };
    match do_join(&st, req.name.as_deref().unwrap_or("AI"), req.token.as_deref(), req.character.as_deref()) {
        Ok((seat, token)) => Json(json!({
            "ok": true,
            "seat": seat,
            "character": st.room.lock().unwrap().seats[seat].as_ref().map(|i| i.character.clone()),
            "token": token,
            "next": format!("GET {}/api/wait?token={}&format=text", st.prefix(), token),
        }))
        .into_response(),
        Err(e) => bad(e),
    }
}

#[derive(Deserialize)]
struct StateQ {
    token: Option<String>,
    format: Option<String>,
    timeout: Option<u64>,
}

fn state_response(st: &AppState, seat: usize, format: Option<&str>, extra: Value) -> Response {
    let room = st.room.lock().unwrap();
    let text = room.text_view(seat);
    if format == Some("text") {
        let mut t = String::new();
        if let Some(m) = extra.get("message").and_then(|x| x.as_str()) {
            t.push_str(m);
            t.push_str("\n\n");
        }
        t.push_str(&text);
        return ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], t).into_response();
    }
    let mut v = json!({"ok": true, "text": text, "state": st.view(&room, Some(seat))});
    if let Value::Object(m) = extra {
        for (k, x) in m {
            v[k] = x;
        }
    }
    Json(v).into_response()
}

fn seat_from_token(st: &AppState, token: Option<&str>) -> Result<usize, String> {
    let mut room = st.room.lock().unwrap();
    let s = room
        .seat_of_token(token.unwrap_or(""))
        .ok_or("token 無效。請先呼叫 join 取得 token（如果你被換成電腦代打，需要重新加入）")?;
    room.touch(s);
    st.mark_active();
    Ok(s)
}

async fn api_state(RoomRef(st): RoomRef, Query(q): Query<StateQ>) -> Response {
    match seat_from_token(&st, q.token.as_deref()) {
        Ok(s) => state_response(&st, s, q.format.as_deref(), json!({})),
        Err(e) => bad(e),
    }
}

/// 等到該玩家有動作可做（或遊戲結束、逾時）才回傳。
async fn wait_for_turn(st: &AppState, token: &str, timeout: Duration) -> Result<(usize, bool), String> {
    let deadline = tokio::time::Instant::now() + timeout;
    let mut rx = st.notify.subscribe();
    loop {
        {
            let seat = seat_from_token(st, Some(token))?;
            let room = st.room.lock().unwrap();
            if room.has_actions(seat) || room.phase == Phase::GameOver {
                return Ok((seat, true));
            }
        }
        match tokio::time::timeout_at(deadline, rx.changed()).await {
            Ok(Ok(())) => {
                rx.borrow_and_update();
            }
            _ => {
                let seat = seat_from_token(st, Some(token))?;
                return Ok((seat, false));
            }
        }
    }
}

async fn api_wait(RoomRef(st): RoomRef, Query(q): Query<StateQ>) -> Response {
    let timeout = Duration::from_secs(q.timeout.unwrap_or(30).clamp(1, 120));
    match wait_for_turn(&st, q.token.as_deref().unwrap_or(""), timeout).await {
        Ok((seat, ready)) => {
            let msg = if ready { "輪到你了。" } else { "還沒輪到你（等待逾時），請再呼叫一次 /api/wait。" };
            state_response(&st, seat, q.format.as_deref(), json!({"your_turn": ready, "message": msg}))
        }
        Err(e) => bad(e),
    }
}

fn do_action(st: &AppState, seat: usize, v: &Value) -> Result<String, String> {
    let mut room = st.room.lock().unwrap();
    let a = room.parse_action(seat, v)?;
    let label = room.action_label(&a);
    room.act(seat, a)?;
    drop(room);
    st.bump();
    Ok(label)
}

async fn api_action(RoomRef(st): RoomRef, Query(q): Query<StateQ>, body: Bytes) -> Response {
    let v = match parse_body(&body) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let token = v.get("token").and_then(|x| x.as_str()).or(q.token.as_deref()).map(String::from);
    let seat = match seat_from_token(&st, token.as_deref()) {
        Ok(s) => s,
        Err(e) => return bad(e),
    };
    match do_action(&st, seat, &v) {
        Ok(label) => state_response(&st, seat, q.format.as_deref(), json!({"message": format!("已執行：{}", label)})),
        Err(e) => bad(e),
    }
}

async fn api_chat(RoomRef(st): RoomRef, body: Bytes) -> Response {
    let v = match parse_body(&body) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let token = v.get("token").and_then(|x| x.as_str());
    let seat = match seat_from_token(&st, token) {
        Ok(s) => s,
        Err(e) => return bad(e),
    };
    let text = v.get("text").and_then(|x| x.as_str()).unwrap_or("");
    st.room.lock().unwrap().push_chat(Some(seat), text);
    st.bump();
    Json(json!({"ok": true})).into_response()
}

// ───────────── 說明文件 ─────────────

pub const RULES: &str = r#"# 台灣十六張麻將 — 規則摘要（虛擬台麻館）

## 牌的代碼
- 萬子 1m..9m、筒子 1p..9p、條子 1s..9s
- 字牌 E 東、S 南、W 西、N 北、C 中、F 發、P 白
- 花牌 f1..f8（春夏秋冬梅蘭竹菊），摸到會自動補花
- 動作也接受中文牌名，例如 "五萬"、"東"、"紅中"

## 基本
- 每人 16 張，莊家 17 張；胡牌 = 5 組（順子或刻子）+ 1 對，共 17 張
- 出牌順序 座位 0→1→2→3→0。只能「吃」上家（座位編號比你小 1）打出的牌；碰、槓、胡可以對任何人
- 優先順序：胡 > 碰/槓 > 吃。多人同時胡時，離放槍者最近的下家優先（截胡）
- 過水：放過別人打的可胡牌之後，到你下次摸牌前都不能胡別人打的牌
- 牌牆保留 16 張，摸完即流局（莊家連莊）
- 嚦咕嚦咕（七對＋一刻，門清）也可以胡

## 台數（每台 = 設定的台錢；底 + 台數×台錢）
莊家 1、連N拉N 2N、門清 1、自摸 1、門清自摸 3、三元牌刻 1、圈風刻 1、門風刻 1、
正花 1、花槓 2、平胡 2、全求人 2、半求人 1、獨聽 1、海底撈月 1、河底撈魚 1、槓上開花 1、搶槓 1、
三暗刻 2、四暗刻 5、五暗刻 8、碰碰胡 4、混一色 4、清一色 8、字一色 16、
小三元 4、大三元 8、小四喜 8、大四喜 16、嚦咕嚦咕 8、八仙過海 8、七搶一 8、地胡 16、天胡 24
放槍：放槍者一人付；自摸：三家都付。牽涉莊家的付款加計莊家與連莊台。

## 逾時
吃碰胡、出牌都有時間限制（房主設定）。逾時時：能胡就自動胡，否則自動「過」或由電腦幫你打一張。
"#;

fn base_url(headers: &HeaderMap) -> String {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:7777");
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or(if host.contains("trycloudflare.com") || host.contains("ngrok") { "https" } else { "http" });
    format!("{}://{}", proto, host)
}

async fn llms_txt(RoomRef(st): RoomRef, headers: HeaderMap) -> Response {
    let base = format!("{}{}", base_url(&headers), st.prefix());
    let body = format!(
        r#"# 虛擬台麻館 — AI 玩家連線說明

這是「虛擬台麻館」：一間線上的台灣十六張麻將館，伺服器在 {base}
人類、Claude Code、ChatGPT 或任何程式都可以坐上同一張牌桌。

## 方法一：MCP（推薦）
MCP 端點：{base}/mcp （Streamable HTTP）

Claude Code：
    claude mcp add --transport http mahjong {base}/mcp
加好之後要「開一個新的 Claude Code 工作階段」，MCP 工具只會在工作階段啟動時載入；
然後告訴 Claude：「用 mahjong 工具加入麻將遊戲並打到結束」。
（如果當前工作階段沒有 mahjong 工具，直接改用下面的 HTTP API 也能玩。）

ChatGPT（需要 https 公開網址，可用 cloudflared / ngrok 建立通道）：
設定 → 連接器 → 進階 → 開發者模式 → 新增 MCP 伺服器，網址填 https://你的網址/mcp

工具：join_game → 反覆 (wait_for_turn → take_action) 直到遊戲結束。

## 方法二：HTTP API（curl 就能玩）
1. 加入：
    curl -s -X POST {base}/api/join -H "Content-Type: application/json" -d '{{"name":"Claude","character":"yuehua"}}'
   會拿到 token 與座位。character 可省略；可選：{chars}
2. 等待輪到你（長輪詢，最多 timeout 秒）：
    curl -s "{base}/api/wait?token=TOKEN&timeout=60&format=text"
3. 執行動作（action_id 來自上一步列出的可執行動作）：
    curl -s -X POST "{base}/api/action?format=text" -H "Content-Type: application/json" -d '{{"token":"TOKEN","action_id":0}}'
   也可以用 {{"token":"TOKEN","type":"discard","tile":"5m"}}、{{"type":"pung"}}、{{"type":"chow","start":"3m"}}、{{"type":"pass"}}、{{"type":"tsumo"}}、{{"type":"ron"}}、{{"type":"next"}}
4. 聊天：
    curl -s -X POST {base}/api/chat -H "Content-Type: application/json" -d '{{"token":"TOKEN","text":"\u5b89\u5b89"}}'
   （Windows 的 Git Bash 直接打中文可能被轉成 Big5 而被拒絕；用 \uXXXX 跳脫或 --data-binary @UTF-8檔案 最保險）
5. 不斷重複 2→3，直到狀態顯示「遊戲結束」。

拿掉 format=text 會回傳完整 JSON（state.actions、state.seats…）。
OpenAPI 規格：{base}/openapi.json

## 小提示
- 每次決定都有時間限制，請別想太久；逾時會自動處理
- 一局結束時要送 next（下一局）
- 技能模式開啟時，輪到你可能多一個「技能」動作（type=skill）；文字版狀態會寫出你的技能與效果
- 被房主換成電腦代打後，token 會失效，需要重新 join

{rules}"#,
        base = base,
        rules = RULES,
        chars = CHARACTERS.iter().map(|c| format!("{}={}", c.0, c.1)).collect::<Vec<_>>().join("、")
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

async fn openapi(RoomRef(st): RoomRef, headers: HeaderMap) -> Json<Value> {
    let base = format!("{}{}", base_url(&headers), st.prefix());
    let token_param = json!({"name": "token", "in": "query", "required": true, "schema": {"type": "string"}});
    let text_resp = json!({"200": {"description": "文字版牌桌狀態", "content": {"text/plain": {"schema": {"type": "string"}}}}});
    Json(json!({
        "openapi": "3.1.0",
        "info": {"title": "虛擬台麻館", "version": "0.1.0",
                 "description": "加入麻將桌並遊玩。流程：joinGame → 反覆 waitForTurn → takeAction，直到遊戲結束。"},
        "servers": [{"url": base}],
        "paths": {
            "/api/join": {"post": {
                "operationId": "joinGame",
                "summary": "加入房間，取得 token",
                "requestBody": {"required": true, "content": {"application/json": {"schema": {
                    "type": "object", "properties": {"name": {"type": "string"}, "token": {"type": "string", "description": "重連用"}}}}}},
                "responses": {"200": {"description": "OK", "content": {"application/json": {"schema": {"type": "object",
                    "properties": {"ok": {"type": "boolean"}, "seat": {"type": "integer"}, "token": {"type": "string"}}}}}}}
            }},
            "/api/wait": {"get": {
                "operationId": "waitForTurn",
                "summary": "等待直到輪到你做決定，回傳牌桌與可執行動作",
                "parameters": [token_param,
                    {"name": "timeout", "in": "query", "schema": {"type": "integer", "default": 30}},
                    {"name": "format", "in": "query", "schema": {"type": "string", "enum": ["text"]}, "required": true}],
                "responses": text_resp
            }},
            "/api/state": {"get": {
                "operationId": "getState",
                "summary": "取得目前牌桌狀態",
                "parameters": [token_param, {"name": "format", "in": "query", "schema": {"type": "string", "enum": ["text"]}, "required": true}],
                "responses": text_resp
            }},
            "/api/action": {"post": {
                "operationId": "takeAction",
                "summary": "執行動作（建議用 action_id）",
                "parameters": [{"name": "format", "in": "query", "schema": {"type": "string", "enum": ["text"]}, "required": true}],
                "requestBody": {"required": true, "content": {"application/json": {"schema": {
                    "type": "object", "required": ["token"],
                    "properties": {
                        "token": {"type": "string"},
                        "action_id": {"type": "integer", "description": "可執行動作清單中的編號"},
                        "type": {"type": "string", "enum": ["discard", "tsumo", "ron", "pung", "kong", "chow", "pass", "next", "skill"]},
                        "tile": {"type": "string", "description": "例如 5m、E、C"},
                        "start": {"type": "string", "description": "吃牌時順子最小那張"},
                        "decision_id": {"type": "integer"}
                    }}}}},
                "responses": text_resp
            }},
            "/api/chat": {"post": {
                "operationId": "sendChat",
                "summary": "在牌桌聊天",
                "requestBody": {"required": true, "content": {"application/json": {"schema": {
                    "type": "object", "required": ["token", "text"],
                    "properties": {"token": {"type": "string"}, "text": {"type": "string"}}}}}},
                "responses": {"200": {"description": "OK"}}
            }}
        }
    }))
}

// ───────────── MCP（Streamable HTTP） ─────────────
//
// 每個工具都標上 annotations（MCP 規範的四個提示），讓 Claude、ChatGPT 等主機知道：
// - readOnlyHint：只讀不改（get_state、wait_for_turn、get_rules），主機可以不必每次都請使用者確認
// - destructiveHint：都是 false——出手、聊天只影響這一局，不會刪除或覆寫任何資料
// - idempotentHint：只讀的才是 true；同樣的出手、聊天呼叫兩次結果不同
// - openWorldHint：都是 false——只跟這台遊戲伺服器互動，不碰外部網路

fn mcp_tools() -> Value {
    let token = json!({"type": "string", "description": "join_game 回傳的 token（同一個 MCP 連線中可省略）"});
    let char_ids: Vec<&str> = CHARACTERS.iter().map(|c| c.0).collect();
    let char_desc = CHARACTERS
        .iter()
        .map(|c| format!("{}={}（{}）", c.0, c.1, c.2))
        .collect::<Vec<_>>()
        .join("、");
    json!([
        {
            "name": "join_game",
            "title": "加入牌桌",
            "annotations": {"readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false},
            "description": "加入虛擬台麻館的台灣十六張麻將牌桌。回傳 token 與座位。加入後請反覆呼叫 wait_for_turn → take_action 直到遊戲結束。若之前加入過，可帶 token 重連。",
            "inputSchema": {"type": "object", "properties": {
                "name": {"type": "string", "description": "你在牌桌上的名字，例如 Claude 或 ChatGPT"},
                "character": {"type": "string", "enum": char_ids, "description": format!("想扮演的二次元角色（可省略，會自動分配）：{}", char_desc)},
                "token": token
            }}
        },
        {
            "name": "wait_for_turn",
            "title": "等待輪到自己",
            "annotations": {"readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false},
            "description": "等待直到輪到你做決定（打牌、吃碰槓胡、下一局）或遊戲結束。回傳文字版牌桌、你的手牌與編號的可執行動作。若回傳「還沒輪到你」就再呼叫一次。",
            "inputSchema": {"type": "object", "properties": {
                "token": token,
                "timeout_seconds": {"type": "integer", "description": "最多等待秒數，預設 45", "default": 45}
            }}
        },
        {
            "name": "get_state",
            "title": "查看牌桌",
            "annotations": {"readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false},
            "description": "立即取得目前牌桌狀態（不等待）。",
            "inputSchema": {"type": "object", "properties": {"token": token}}
        },
        {
            "name": "take_action",
            "title": "出手（打牌、吃碰槓胡）",
            "annotations": {"readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false},
            "description": "執行一個動作。最簡單是傳 action_id（wait_for_turn 列出的編號）。也可以傳 type + tile，例如 type=discard tile=5m；type=chow start=3m；type=pung / kong / ron / tsumo / pass / next / skill（技能模式發動角色技能）。",
            "inputSchema": {"type": "object", "properties": {
                "token": token,
                "action_id": {"type": "integer", "description": "可執行動作的編號"},
                "type": {"type": "string", "enum": ["discard", "tsumo", "ron", "pung", "kong", "chow", "pass", "next", "skill"]},
                "tile": {"type": "string", "description": "牌代碼，例如 5m、9p、3s、E、C"},
                "start": {"type": "string", "description": "吃牌時順子最小那張"},
                "decision_id": {"type": "integer", "description": "可選；避免對過期的局面出手"}
            }}
        },
        {
            "name": "send_chat",
            "title": "發聊天訊息",
            "annotations": {"readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false},
            "description": "在牌桌上發聊天訊息給其他玩家。",
            "inputSchema": {"type": "object", "properties": {"token": token, "text": {"type": "string"}}, "required": ["text"]}
        },
        {
            "name": "get_rules",
            "title": "查看規則",
            "annotations": {"readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false},
            "description": "取得規則、牌代碼與台數表。",
            "inputSchema": {"type": "object", "properties": {}}
        }
    ])
}

fn tool_text(text: String, is_error: bool) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": is_error})
}

async fn call_tool(st: &AppState, session: Option<&str>, name: &str, args: &Value) -> Value {
    let token = args
        .get("token")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| session.and_then(|s| st.mcp_sessions.lock().unwrap().get(s).cloned()));
    match name {
        "get_rules" => tool_text(RULES.to_string(), false),
        "join_game" => {
            let name = args.get("name").and_then(|x| x.as_str()).unwrap_or("AI");
            let character = args.get("character").and_then(|x| x.as_str());
            match do_join(st, name, token.as_deref(), character) {
                Ok((seat, tok)) => {
                    let ch = st.room.lock().unwrap().seats[seat].as_ref().map(|i| i.character.clone()).unwrap_or_default();
                    let ch_desc = CHARACTERS
                        .iter()
                        .find(|c| c.0 == ch)
                        .map(|c| format!("{}（{}）", c.1, c.2))
                        .unwrap_or(ch);
                    if let Some(s) = session {
                        st.mcp_sessions.lock().unwrap().insert(s.to_string(), tok.clone());
                    }
                    tool_text(
                        format!(
                            "已加入！你的座位：{}\n你的角色：{}——聊天時可以用這個角色的口吻說話\ntoken：{}\n（之後呼叫工具時帶上這個 token）\n\n接下來：呼叫 wait_for_turn 等待輪到你，再用 take_action 傳 action_id 出手，重複直到遊戲結束。\n房主在 App 按「開始」後遊戲才會開始。\n\n{}",
                            seat, ch_desc, tok, RULES
                        ),
                        false,
                    )
                }
                Err(e) => tool_text(e, true),
            }
        }
        "wait_for_turn" | "get_state" => {
            let Some(tok) = token else {
                return tool_text("缺少 token，請先呼叫 join_game".into(), true);
            };
            if name == "get_state" {
                return match seat_from_token(st, Some(&tok)) {
                    Ok(s) => tool_text(st.room.lock().unwrap().text_view(s), false),
                    Err(e) => tool_text(e, true),
                };
            }
            let secs = args.get("timeout_seconds").and_then(|x| x.as_u64()).unwrap_or(45).clamp(1, 110);
            match wait_for_turn(st, &tok, Duration::from_secs(secs)).await {
                Ok((s, ready)) => {
                    let room = st.room.lock().unwrap();
                    let head = if room.phase == Phase::GameOver {
                        "遊戲已結束。"
                    } else if ready {
                        "輪到你了！"
                    } else {
                        "還沒輪到你，請再呼叫 wait_for_turn。"
                    };
                    tool_text(format!("{}\n\n{}", head, room.text_view(s)), false)
                }
                Err(e) => tool_text(e, true),
            }
        }
        "take_action" => {
            let Some(tok) = token else {
                return tool_text("缺少 token，請先呼叫 join_game".into(), true);
            };
            let seat = match seat_from_token(st, Some(&tok)) {
                Ok(s) => s,
                Err(e) => return tool_text(e, true),
            };
            match do_action(st, seat, args) {
                Ok(label) => {
                    let room = st.room.lock().unwrap();
                    let more = if room.has_actions(seat) {
                        "你還有動作要做（見下方）。"
                    } else {
                        "接下來請呼叫 wait_for_turn。"
                    };
                    tool_text(format!("已執行：{}。{}\n\n{}", label, more, room.text_view(seat)), false)
                }
                Err(e) => {
                    let room = st.room.lock().unwrap();
                    tool_text(format!("失敗：{}\n\n{}", e, room.text_view(seat)), true)
                }
            }
        }
        "send_chat" => {
            let Some(tok) = token else {
                return tool_text("缺少 token，請先呼叫 join_game".into(), true);
            };
            match seat_from_token(st, Some(&tok)) {
                Ok(s) => {
                    let text = args.get("text").and_then(|x| x.as_str()).unwrap_or("");
                    st.room.lock().unwrap().push_chat(Some(s), text);
                    st.bump();
                    tool_text("已送出".into(), false)
                }
                Err(e) => tool_text(e, true),
            }
        }
        _ => tool_text(format!("未知工具 {}", name), true),
    }
}

/// 回傳 (回應, 新的 session id)；通知（沒有 id）回傳 None。
async fn handle_rpc(st: &AppState, req: &Value, session: Option<&str>) -> Option<(Value, Option<String>)> {
    let id = req.get("id").cloned()?;
    let method = req.get("method").and_then(|x| x.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(json!({}));
    let mut new_session = None;
    let result: Result<Value, (i64, String)> = match method {
        "initialize" => {
            let sid = gen_key(24);
            new_session = Some(sid);
            let pv = params.get("protocolVersion").and_then(|x| x.as_str()).unwrap_or("2025-06-18");
            Ok(json!({
                "protocolVersion": pv,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "taiwan-mahjong", "title": "虛擬台麻館", "version": "0.1.0"},
                "instructions": "這是虛擬台麻館（台灣十六張麻將）的伺服器。先 join_game 加入，然後反覆 wait_for_turn → take_action(action_id) 直到遊戲結束。每一手都有時間限制，請快速決定。"
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": mcp_tools()})),
        "tools/call" => {
            let name = params.get("name").and_then(|x| x.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            Ok(call_tool(st, session, name, &args).await)
        }
        "resources/list" => Ok(json!({"resources": []})),
        "prompts/list" => Ok(json!({"prompts": []})),
        _ => Err((-32601, format!("Method not found: {}", method))),
    };
    let resp = match result {
        Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
        Err((code, message)) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}),
    };
    Some((resp, new_session))
}

async fn mcp_post(RoomRef(st): RoomRef, headers: HeaderMap, body: String) -> Response {
    let req: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => {
            return Json(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "Parse error"}}))
                .into_response()
        }
    };
    let session = headers.get("mcp-session-id").and_then(|h| h.to_str().ok()).map(String::from);
    if let Value::Array(items) = &req {
        let mut out = Vec::new();
        let mut sid = None;
        for r in items {
            if let Some((resp, s)) = handle_rpc(&st, r, session.as_deref()).await {
                out.push(resp);
                sid = sid.or(s);
            }
        }
        if out.is_empty() {
            return StatusCode::ACCEPTED.into_response();
        }
        let mut r = Json(Value::Array(out)).into_response();
        if let Some(s) = sid.and_then(|s| s.parse().ok()) {
            r.headers_mut().insert("mcp-session-id", s);
        }
        return r;
    }
    match handle_rpc(&st, &req, session.as_deref()).await {
        None => StatusCode::ACCEPTED.into_response(),
        Some((resp, sid)) => {
            let mut r = Json(resp).into_response();
            if let Some(s) = sid.and_then(|s| s.parse().ok()) {
                r.headers_mut().insert("mcp-session-id", s);
            }
            r
        }
    }
}

async fn mcp_get() -> Response {
    (StatusCode::METHOD_NOT_ALLOWED, "This MCP server does not offer an SSE stream; use POST.").into_response()
}

async fn mcp_delete(RoomRef(st): RoomRef, headers: HeaderMap) -> Response {
    if let Some(s) = headers.get("mcp-session-id").and_then(|h| h.to_str().ok()) {
        st.mcp_sessions.lock().unwrap().remove(s);
    }
    StatusCode::OK.into_response()
}

// ───────────── 靜態檔 ─────────────

async fn static_handler(State(hub): State<Arc<Hub>>, uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    // index.html 不能快取，否則改版後瀏覽器會繼續載入舊的程式；
    // assets/ 底下的檔名帶雜湊值，可以放心長期快取
    let cache = |p: &str| {
        if p.starts_with("assets/") {
            "public, max-age=31536000, immutable"
        } else if p.ends_with(".html") {
            "no-cache"
        } else {
            "public, max-age=3600"
        }
    };
    if let Some(f) = &hub.assets {
        if let Some((bytes, mime)) = f(path) {
            return ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache(path).to_string())], bytes).into_response();
        }
        if !path.contains('.') {
            if let Some((bytes, mime)) = f("index.html") {
                return ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "no-cache".to_string())], bytes)
                    .into_response();
            }
        }
    }
    let base = base_url(&headers);
    (
        StatusCode::NOT_FOUND,
        Html(format!(
            "<meta charset=utf-8><title>虛擬台麻館伺服器</title><body style='font-family:sans-serif;max-width:640px;margin:40px auto'>\
             <h1>🀄 虛擬台麻館伺服器</h1><p>伺服器運作中，但沒有內建網頁介面（開發模式）。</p>\
             <ul><li>AI 連線說明：<a href='/llms.txt'>{base}/llms.txt</a></li>\
             <li>MCP：<code>{base}/mcp</code></li><li>OpenAPI：<a href='/openapi.json'>{base}/openapi.json</a></li></ul>"
        )),
    )
        .into_response()
}

/// 取得本機區網 IP（讓房主把位址告訴朋友）
pub fn local_ips() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(a) = sock.local_addr() {
                v.push(a.ip().to_string());
            }
        }
    }
    v.push("127.0.0.1".into());
    v
}

#[cfg(test)]
mod mcp_tests {
    use super::*;
    use mahjong_core::room::{Action, Step};
    use mahjong_core::tiles::code;

    fn state() -> AppState {
        let settings = Settings { bot_delay_ms: 0, turn_timeout: 0, claim_timeout: 0, next_hand_delay: 0, ..Default::default() };
        AppState::new(settings, None, watch::channel(false).1)
    }

    async fn call(st: &AppState, session: &str, name: &str, args: Value) -> Value {
        let req = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": name, "arguments": args}});
        let (resp, _) = handle_rpc(st, &req, Some(session)).await.unwrap();
        resp["result"].clone()
    }

    fn text(result: &Value) -> String {
        result["content"][0]["text"].as_str().unwrap_or("").to_string()
    }

    #[tokio::test]
    async fn every_tool_declares_all_four_annotation_hints() {
        let st = state();
        let (resp, _) = handle_rpc(&st, &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}), None).await.unwrap();
        let tools = resp["result"]["tools"].as_array().unwrap().clone();
        assert_eq!(tools.len(), 6);
        for tool in &tools {
            let name = tool["name"].as_str().unwrap();
            assert!(tool["title"].is_string(), "{name} 缺 title");
            for hint in ["readOnlyHint", "destructiveHint", "idempotentHint", "openWorldHint"] {
                assert!(tool["annotations"][hint].is_boolean(), "{name} 缺 {hint}");
            }
        }
        let read_only: Vec<&str> = tools
            .iter()
            .filter(|t| t["annotations"]["readOnlyHint"] == json!(true))
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(read_only, ["wait_for_turn", "get_state", "get_rules"]);
    }

    /// 一個 AI 玩家從加入到出手：join_game → get_rules → get_state → wait_for_turn → take_action → send_chat
    #[tokio::test]
    async fn ai_player_can_use_every_tool() {
        let st = state();
        let (init, session) = handle_rpc(&st, &json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {}}), None).await.unwrap();
        assert_eq!(init["result"]["serverInfo"]["name"], "taiwan-mahjong");
        let session = session.unwrap();

        let joined = call(&st, &session, "join_game", json!({"name": "Claude", "character": "yuehua"})).await;
        assert_eq!(joined["isError"], false, "{}", text(&joined));
        assert!(text(&joined).contains("已加入"));

        let rules = call(&st, &session, "get_rules", json!({})).await;
        assert_eq!(rules["isError"], false);
        assert!(!text(&rules).is_empty());

        let state_text = call(&st, &session, "get_state", json!({})).await;
        assert_eq!(state_text["isError"], false, "{}", text(&state_text));

        // 其他三家補電腦、開局，然後讓電腦打到輪到 AI 玩家打牌
        let seat = {
            let mut room = st.room.lock().unwrap();
            for _ in 0..3 {
                room.add_bot(None).unwrap();
            }
            room.start_game().unwrap();
            let seat = (0..4).find(|&s| room.seats[s].as_ref().map_or(false, |i| i.name == "Claude")).unwrap();
            for _ in 0..10_000 {
                let my_discard = matches!(room.hand.as_ref().map(|h| &h.step), Some(Step::Turn { seat: s, .. }) if *s == seat)
                    && room.phase == Phase::Playing;
                if my_discard {
                    break;
                }
                if room.has_actions(seat) {
                    // 輪到別人打的牌可以吃碰：先過
                    let _ = room.act(seat, Action::Pass);
                }
                room.tick();
            }
            seat
        };

        let waited = call(&st, &session, "wait_for_turn", json!({"timeout_seconds": 2})).await;
        assert_eq!(waited["isError"], false, "{}", text(&waited));

        let tile = {
            let room = st.room.lock().unwrap();
            code(room.hand.as_ref().unwrap().players[seat].concealed[0])
        };
        let acted = call(&st, &session, "take_action", json!({"type": "discard", "tile": tile})).await;
        assert_eq!(acted["isError"], false, "{}", text(&acted));
        assert!(text(&acted).contains("已執行"));

        let chatted = call(&st, &session, "send_chat", json!({"text": "這把我要胡！"})).await;
        assert_eq!(chatted["isError"], false, "{}", text(&chatted));
        assert!(st.room.lock().unwrap().chat.iter().any(|c| c.text == "這把我要胡！"));
    }
}
