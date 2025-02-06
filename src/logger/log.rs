use crate::logger::severity::LogSeverity;
use crate::logger::time::now;

/// Logs a message to the console
pub fn log(msg: String, log_severity: LogSeverity) {
    println!("[{}] {} {}", log_severity, now(), msg);
}
