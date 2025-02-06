mod logger;

use logger::{log, LogSeverity::*};

#[tokio::main]
async fn main() {
    log("wow".to_string(), Info);
}