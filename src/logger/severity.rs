use std::fmt;
use std::fmt::{Display, Formatter};

/// Log severity
pub enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

/// Display impl for LogSeverity
impl Display for LogSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogSeverity::Debug => write!(f, "DEBUG"),
            LogSeverity::Info => write!(f, "INFO"),
            LogSeverity::Warning => write!(f, "WARNING"),
            LogSeverity::Error => write!(f, "ERROR"),
            LogSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}
