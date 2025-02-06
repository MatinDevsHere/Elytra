use std::fmt;
use std::fmt::{Display, Formatter};

pub enum LogSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl Display for LogSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogSeverity::Info => write!(f, "INFO"),
            LogSeverity::Warning => write!(f, "WARNING"),
            LogSeverity::Error => write!(f, "ERROR"),
            LogSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}