#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // let _ = diavolo_server::run("127.0.0.1:3000").await;
}
