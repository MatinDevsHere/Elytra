use elytra::logger::{log, LogSeverity::Info};
use elytra::server;

#[tokio::main]
async fn main() {
    log("Elytra init".to_string(), Info);
    server::run().await;
}
