//! 虛擬台麻館的牌局引擎（台灣十六張麻將）。
//!
//! 不依賴網路或作業系統：伺服器、桌面 App、瀏覽器（WebAssembly 單機版）共用同一套規則。

pub mod bot;
pub mod room;
pub mod rules;
pub mod scoring;
pub mod tiles;
