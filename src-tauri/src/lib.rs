pub mod game;
pub mod server;

use game::room::Settings;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

#[derive(Default)]
struct HostState(Mutex<Option<server::ServerHandle>>);

#[derive(Serialize)]
struct HostInfo {
    port: u16,
    host_key: String,
    ips: Vec<String>,
}

#[tauri::command]
async fn start_host(
    app: AppHandle,
    state: State<'_, HostState>,
    port: u16,
    settings: Option<Settings>,
) -> Result<HostInfo, String> {
    let mut guard = state.0.lock().await;
    if let Some(h) = guard.as_ref() {
        return Ok(HostInfo { port: h.port, host_key: h.host_key.clone(), ips: server::local_ips() });
    }
    let app2 = app.clone();
    let assets: server::AssetFn = Arc::new(move |path: &str| {
        app2.asset_resolver()
            .get(path.to_string())
            .map(|a| (a.bytes().to_vec(), a.mime_type().to_string()))
    });
    let h = server::start(port, settings.unwrap_or_default(), Some(assets)).await?;
    let info = HostInfo { port: h.port, host_key: h.host_key.clone(), ips: server::local_ips() };
    *guard = Some(h);
    Ok(info)
}

#[tauri::command]
async fn stop_host(state: State<'_, HostState>) -> Result<(), String> {
    if let Some(h) = state.0.lock().await.take() {
        h.stop();
    }
    Ok(())
}

#[tauri::command]
async fn host_status(state: State<'_, HostState>) -> Result<Option<HostInfo>, String> {
    Ok(state
        .0
        .lock()
        .await
        .as_ref()
        .map(|h| HostInfo { port: h.port, host_key: h.host_key.clone(), ips: server::local_ips() }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(HostState::default())
        .invoke_handler(tauri::generate_handler![start_host, stop_host, host_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
