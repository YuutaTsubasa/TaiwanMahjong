//! 不開視窗、只跑伺服器。程式在 crates/mahjong-server，這裡只是讓桌面版安裝檔也附上這個執行檔。

#[tokio::main]
async fn main() {
    mahjong_server::cli::run().await;
}
