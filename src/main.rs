use logger::log;
use logger::LogSeverity::Info;

mod logger;
mod protocol;
mod server;

#[tokio::main]
async fn main() {
    log("Elytra init".to_owned(), Info);
    server::run().await;
}
