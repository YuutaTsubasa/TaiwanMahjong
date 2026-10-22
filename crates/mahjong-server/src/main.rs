#[tokio::main]
async fn main() {
    mahjong_server::cli::run().await;
}
