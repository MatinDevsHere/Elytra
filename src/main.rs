mod logger;

use logger::{log, LogSeverity::*};

#[tokio::main]
async fn main() {
    log("Elytra init".to_string(), Info);
}