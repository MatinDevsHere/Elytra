use elytra_logger::log::log;
use elytra_logger::severity::LogSeverity::Info;
use elytra_server::server;

#[tokio::main]
async fn main() {
    log("Elytra init".to_owned(), Info);
    server::run().await;
}
