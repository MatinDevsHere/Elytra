use crate::logger::time::now;
use crate::logger::severity::LogSeverity;

pub fn log(msg: String, log_severity: LogSeverity) {
    println!("[{}] {} {}", log_severity, now(), msg);
}
