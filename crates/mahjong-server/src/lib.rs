//! 虛擬台麻館的連線伺服器。桌面版（src-tauri）開房時用 `server::start`，
//! 線上公開伺服器（Railway）用 `mahjong-server --public`。
pub mod cli;
pub mod server;
