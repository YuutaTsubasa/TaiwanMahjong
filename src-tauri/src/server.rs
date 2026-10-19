//! 遊戲伺服器：
//! - `/ws`          人類玩家（Svelte 介面）用的 WebSocket
//! - `/api/*`       AI / 程式用的 HTTP API（長輪詢）
//! - `/mcp`         MCP（Streamable HTTP），給 Claude Code / ChatGPT 連線
//! - `/llms.txt`    給 AI 看的連線說明
//! - `/openapi.json` 給 ChatGPT GPT Actions 用
//! - 其他路徑       前端靜態檔（讓沒裝 App 的人用瀏覽器加入）

use crate::game::room::{Phase, Room, SeatKind, Settings};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::watch;
use tower_http::cors::CorsLayer;

pub type AssetFn = Arc<dyn Fn(&str) -> Option<(Vec<u8>, String)> + Send + Sync>;

pub struct AppState {
    pub room: Mutex<Room>,
    notify: watch::Sender<u64>,
    shutdown: watch::Receiver<bool>,
    pub host_key: String,
    assets: Option<AssetFn>,
    mcp_sessions: Mutex<HashMap<String, String>>,
}

impl AppState {
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

pub async fn start(port: u16, settings: Settings, assets: Option<AssetFn>) -> Result<ServerHandle, String> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| format!("無法監聽埠 {}：{}", port, e))?;
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    let (sd_tx, sd_rx) = watch::channel(false);
    let (notify, _) = watch::channel(0u64);
    let mut room = Room::new(Settings::default());
    room.update_settings(settings);
    let host_key = gen_key(20);
    let state = Arc::new(AppState {
        room: Mutex::new(room),
        notify,
        shutdown: sd_rx.clone(),
        host_key: host_key.clone(),
        assets,
        mcp_sessions: Mutex::new(HashMap::new()),
    });

    // 計時器：電腦出牌、逾時
    let st = state.clone();
    let mut sd1 = sd_rx.clone();
    tokio::spawn(async move {
        let mut iv = tokio::time::interval(Duration::from_millis(150));
        loop {
            tokio::select! {
                _ = iv.tick() => {
                    let changed = st.room.lock().unwrap().tick();
                    if changed { st.bump(); }
                }
                _ = sd1.changed() => break,
            }
        }
    });

    let app = router(state.clone());
    let mut sd2 = sd_rx.clone();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = sd2.changed().await;
            })
            .await;
    });

    Ok(ServerHandle { state, shutdown: sd_tx, port, host_key })
}

pub fn router(state: Arc<AppState>) -> Router {
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
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ───────────── WebSocket（人類） ─────────────

async fn ws_handler(ws: WebSocketUpgrade, State(st): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |sock| ws_session(sock, st))
}

struct WsCtx {
    seat: Option<usize>,
    is_host: bool,
}

async fn ws_session(socket: WebSocket, st: Arc<AppState>) {
    let (mut out, mut inp) = socket.split();
    let mut ctx = WsCtx { seat: None, is_host: false };
    let mut notify = st.notify.subscribe();
    let mut shutdown = st.shutdown.clone();
    let mut ping = tokio::time::interval(Duration::from_secs(20));
    loop {
        let mut msgs: Vec<Value> = Vec::new();
        let mut send_state = false;
        tokio::select! {
            m = inp.next() => {
                match m {
                    Some(Ok(Message::Text(t))) => {
                        let v: Value = serde_json::from_str(t.as_str()).unwrap_or(Value::Null);
                        let (replies, changed) = handle_ws_msg(&st, &mut ctx, &v);
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
                let mut v = st.view(&room, ctx.seat);
                v["is_host"] = json!(ctx.is_host);
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
    if let Some(s) = ctx.seat {
        st.room.lock().unwrap().disconnect(s);
        st.bump();
    }
}

fn err(msg: impl Into<String>) -> Value {
    json!({"type": "error", "message": msg.into()})
}

fn handle_ws_msg(st: &AppState, ctx: &mut WsCtx, v: &Value) -> (Vec<Value>, bool) {
    let ty = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
    let mut room = st.room.lock().unwrap();
    match ty {
        "hello" => {
            if let Some(k) = v.get("host_key").and_then(|x| x.as_str()) {
                ctx.is_host = !k.is_empty() && k == st.host_key;
            }
            if let Some(s) = ctx.seat.take() {
                room.disconnect(s);
            }
            let token = v.get("token").and_then(|x| x.as_str()).unwrap_or("");
            if let Some(s) = room.seat_of_token(token) {
                ctx.seat = Some(s);
                room.connect(s);
                return (vec![json!({"type": "welcome", "seat": s, "token": token, "is_host": ctx.is_host})], true);
            }
            if v.get("spectate").and_then(|x| x.as_bool()).unwrap_or(false) {
                return (vec![json!({"type": "welcome", "seat": null, "token": null, "is_host": ctx.is_host})], false);
            }
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
            match room.join(name, SeatKind::Human) {
                Ok((s, tok)) => {
                    ctx.seat = Some(s);
                    room.connect(s);
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

// ───────────── HTTP API（AI / 程式） ─────────────

fn bad(msg: impl Into<String>) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({"ok": false, "error": msg.into()}))).into_response()
}

async fn api_info(State(st): State<Arc<AppState>>) -> Json<Value> {
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
        "game": "台灣十六張麻將",
        "phase": room.phase,
        "seats": seats,
        "docs": "/llms.txt",
        "mcp": "/mcp",
        "openapi": "/openapi.json",
    }))
}

#[derive(Deserialize, Default)]
struct JoinReq {
    name: Option<String>,
    token: Option<String>,
}

fn do_join(st: &AppState, name: &str, token: Option<&str>) -> Result<(usize, String), String> {
    let mut room = st.room.lock().unwrap();
    if let Some(tok) = token {
        if let Some(s) = room.seat_of_token(tok) {
            room.touch(s);
            return Ok((s, tok.to_string()));
        }
    }
    let (s, tok) = room.join(name, SeatKind::Agent)?;
    room.touch(s);
    drop(room);
    st.bump();
    Ok((s, tok))
}

async fn api_join(State(st): State<Arc<AppState>>, body: Option<Json<JoinReq>>) -> Response {
    let req = body.map(|b| b.0).unwrap_or_default();
    match do_join(&st, req.name.as_deref().unwrap_or("AI"), req.token.as_deref()) {
        Ok((seat, token)) => Json(json!({
            "ok": true,
            "seat": seat,
            "token": token,
            "next": format!("GET /api/wait?token={}&format=text", token),
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
    Ok(s)
}

async fn api_state(State(st): State<Arc<AppState>>, Query(q): Query<StateQ>) -> Response {
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

async fn api_wait(State(st): State<Arc<AppState>>, Query(q): Query<StateQ>) -> Response {
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

async fn api_action(State(st): State<Arc<AppState>>, Query(q): Query<StateQ>, Json(v): Json<Value>) -> Response {
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

async fn api_chat(State(st): State<Arc<AppState>>, Json(v): Json<Value>) -> Response {
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

pub const RULES: &str = r#"# 台灣十六張麻將 — 規則摘要

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

async fn llms_txt(headers: HeaderMap) -> Response {
    let base = base_url(&headers);
    let body = format!(
        r#"# 台灣十六張麻將 — AI 玩家連線說明

這是一個可以連線遊玩的台灣十六張麻將伺服器：{base}
人類、Claude Code、ChatGPT 或任何程式都可以坐上同一張牌桌。

## 方法一：MCP（推薦）
MCP 端點：{base}/mcp （Streamable HTTP）

Claude Code：
    claude mcp add --transport http mahjong {base}/mcp
然後告訴 Claude：「用 mahjong 工具加入麻將遊戲並打到結束」。

ChatGPT（需要 https 公開網址，可用 cloudflared / ngrok 建立通道）：
設定 → 連接器 → 進階 → 開發者模式 → 新增 MCP 伺服器，網址填 https://你的網址/mcp

工具：join_game → 反覆 (wait_for_turn → take_action) 直到遊戲結束。

## 方法二：HTTP API（curl 就能玩）
1. 加入：
    curl -s -X POST {base}/api/join -H "Content-Type: application/json" -d '{{"name":"Claude"}}'
   會拿到 token 與座位。
2. 等待輪到你（長輪詢，最多 timeout 秒）：
    curl -s "{base}/api/wait?token=TOKEN&timeout=60&format=text"
3. 執行動作（action_id 來自上一步列出的可執行動作）：
    curl -s -X POST "{base}/api/action?format=text" -H "Content-Type: application/json" -d '{{"token":"TOKEN","action_id":0}}'
   也可以用 {{"token":"TOKEN","type":"discard","tile":"5m"}}、{{"type":"pung"}}、{{"type":"chow","start":"3m"}}、{{"type":"pass"}}、{{"type":"tsumo"}}、{{"type":"ron"}}、{{"type":"next"}}
4. 聊天：
    curl -s -X POST {base}/api/chat -H "Content-Type: application/json" -d '{{"token":"TOKEN","text":"安安"}}'
5. 不斷重複 2→3，直到狀態顯示「遊戲結束」。

拿掉 format=text 會回傳完整 JSON（state.actions、state.seats…）。
OpenAPI 規格：{base}/openapi.json

## 小提示
- 每次決定都有時間限制，請別想太久；逾時會自動處理
- 一局結束時要送 next（下一局）
- 被房主換成電腦代打後，token 會失效，需要重新 join

{rules}"#,
        base = base,
        rules = RULES
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

async fn openapi(headers: HeaderMap) -> Json<Value> {
    let base = base_url(&headers);
    let token_param = json!({"name": "token", "in": "query", "required": true, "schema": {"type": "string"}});
    let text_resp = json!({"200": {"description": "文字版牌桌狀態", "content": {"text/plain": {"schema": {"type": "string"}}}}});
    Json(json!({
        "openapi": "3.1.0",
        "info": {"title": "台灣十六張麻將", "version": "0.1.0",
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
                        "type": {"type": "string", "enum": ["discard", "tsumo", "ron", "pung", "kong", "chow", "pass", "next"]},
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

fn mcp_tools() -> Value {
    let token = json!({"type": "string", "description": "join_game 回傳的 token（同一個 MCP 連線中可省略）"});
    json!([
        {
            "name": "join_game",
            "description": "加入台灣十六張麻將牌桌。回傳 token 與座位。加入後請反覆呼叫 wait_for_turn → take_action 直到遊戲結束。若之前加入過，可帶 token 重連。",
            "inputSchema": {"type": "object", "properties": {
                "name": {"type": "string", "description": "你在牌桌上的名字，例如 Claude 或 ChatGPT"},
                "token": token
            }}
        },
        {
            "name": "wait_for_turn",
            "description": "等待直到輪到你做決定（打牌、吃碰槓胡、下一局）或遊戲結束。回傳文字版牌桌、你的手牌與編號的可執行動作。若回傳「還沒輪到你」就再呼叫一次。",
            "inputSchema": {"type": "object", "properties": {
                "token": token,
                "timeout_seconds": {"type": "integer", "description": "最多等待秒數，預設 45", "default": 45}
            }}
        },
        {
            "name": "get_state",
            "description": "立即取得目前牌桌狀態（不等待）。",
            "inputSchema": {"type": "object", "properties": {"token": token}}
        },
        {
            "name": "take_action",
            "description": "執行一個動作。最簡單是傳 action_id（wait_for_turn 列出的編號）。也可以傳 type + tile，例如 type=discard tile=5m；type=chow start=3m；type=pung / kong / ron / tsumo / pass / next。",
            "inputSchema": {"type": "object", "properties": {
                "token": token,
                "action_id": {"type": "integer", "description": "可執行動作的編號"},
                "type": {"type": "string", "enum": ["discard", "tsumo", "ron", "pung", "kong", "chow", "pass", "next"]},
                "tile": {"type": "string", "description": "牌代碼，例如 5m、9p、3s、E、C"},
                "start": {"type": "string", "description": "吃牌時順子最小那張"},
                "decision_id": {"type": "integer", "description": "可選；避免對過期的局面出手"}
            }}
        },
        {
            "name": "send_chat",
            "description": "在牌桌上發聊天訊息給其他玩家。",
            "inputSchema": {"type": "object", "properties": {"token": token, "text": {"type": "string"}}, "required": ["text"]}
        },
        {
            "name": "get_rules",
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
            match do_join(st, name, token.as_deref()) {
                Ok((seat, tok)) => {
                    if let Some(s) = session {
                        st.mcp_sessions.lock().unwrap().insert(s.to_string(), tok.clone());
                    }
                    tool_text(
                        format!(
                            "已加入！你的座位：{}\ntoken：{}\n（之後呼叫工具時帶上這個 token）\n\n接下來：呼叫 wait_for_turn 等待輪到你，再用 take_action 傳 action_id 出手，重複直到遊戲結束。\n房主在 App 按「開始」後遊戲才會開始。\n\n{}",
                            seat, tok, RULES
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
                "serverInfo": {"name": "taiwan-mahjong", "title": "台灣十六張麻將", "version": "0.1.0"},
                "instructions": "這是台灣十六張麻將伺服器。先 join_game 加入，然後反覆 wait_for_turn → take_action(action_id) 直到遊戲結束。每一手都有時間限制，請快速決定。"
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

async fn mcp_post(State(st): State<Arc<AppState>>, headers: HeaderMap, body: String) -> Response {
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

async fn mcp_delete(State(st): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Some(s) = headers.get("mcp-session-id").and_then(|h| h.to_str().ok()) {
        st.mcp_sessions.lock().unwrap().remove(s);
    }
    StatusCode::OK.into_response()
}

// ───────────── 靜態檔 ─────────────

async fn static_handler(State(st): State<Arc<AppState>>, uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    if let Some(f) = &st.assets {
        if let Some((bytes, mime)) = f(path) {
            return ([(header::CONTENT_TYPE, mime)], bytes).into_response();
        }
        if !path.contains('.') {
            if let Some((bytes, mime)) = f("index.html") {
                return ([(header::CONTENT_TYPE, mime)], bytes).into_response();
            }
        }
    }
    let base = base_url(&headers);
    (
        StatusCode::NOT_FOUND,
        Html(format!(
            "<meta charset=utf-8><title>台灣麻將伺服器</title><body style='font-family:sans-serif;max-width:640px;margin:40px auto'>\
             <h1>🀄 台灣十六張麻將伺服器</h1><p>伺服器運作中，但沒有內建網頁介面（開發模式）。</p>\
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
