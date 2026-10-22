//! 不開視窗、只跑伺服器（適合放在主機上，或測試用）。
//!
//! 用法：mahjong-server [--port 7777] [--dist ../dist] [--bots 3] [--start] [--public]
//!   --bots N  先放 N 個電腦進房間
//!   --start   坐滿後自動開始（空位補電腦）
//!   --public  公開伺服器：任何人都能用 POST /api/rooms 開房，房間在 /r/{房號}/ 底下
//!             （放在 Fly.io 用；通訊埠也可以用環境變數 PORT 指定）
//! 房主金鑰會印在終端機上，網頁介面輸入金鑰後就能當房主。

use crate::server;
use mahjong_core::room::Settings;
use std::path::PathBuf;
use std::sync::Arc;

fn mime_of(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "css" => "text/css",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

pub async fn run() {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    let port: u16 = get("--port")
        .or_else(|| std::env::var("PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(7777);
    let bots: usize = get("--bots").and_then(|p| p.parse().ok()).unwrap_or(0);
    let dist = get("--dist").map(PathBuf::from).or_else(|| {
        let p = PathBuf::from("dist");
        if p.join("index.html").exists() {
            Some(p)
        } else {
            let p = PathBuf::from("../dist");
            p.join("index.html").exists().then_some(p)
        }
    });
    let mut settings = Settings::default();
    if let Some(s) = get("--settings") {
        settings = serde_json::from_str(&s).expect("--settings 需要 JSON");
    }
    let assets: Option<server::AssetFn> = dist.clone().map(|d| {
        let f: server::AssetFn = Arc::new(move |path: &str| {
            if path.contains("..") {
                return None;
            }
            let p = d.join(path);
            std::fs::read(&p).ok().map(|b| (b, mime_of(path).to_string()))
        });
        f
    });
    if args.iter().any(|a| a == "--public") {
        if let Err(e) = server::run_public(port, assets).await {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        return;
    }
    let h = server::start(port, settings, assets).await.unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    {
        let mut room = h.state.room.lock().unwrap();
        for _ in 0..bots.min(4) {
            let _ = room.add_bot(None);
        }
        if args.iter().any(|a| a == "--start") {
            room.start_game().unwrap();
        }
    }
    println!("虛擬台麻館伺服器已啟動（台灣十六張麻將）");
    println!("  網頁：http://localhost:{}/ {}", h.port, if dist.is_some() { "" } else { "（找不到 dist，網頁介面停用）" });
    for ip in server::local_ips() {
        println!("  區網：http://{}:{}/", ip, h.port);
    }
    println!("  MCP： http://localhost:{}/mcp", h.port);
    println!("  AI 說明：http://localhost:{}/llms.txt", h.port);
    println!("  房主金鑰：{}", h.host_key);
    tokio::signal::ctrl_c().await.ok();
    h.stop();
}
