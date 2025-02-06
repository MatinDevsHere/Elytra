mod logger;
mod protocol;
mod server;

#[tokio::main]
async fn main() {
    logger::log("Elytra init".to_string(), logger::LogSeverity::Info);
    server::run().await;
}
