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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_severity_display() {
        assert_eq!(format!("{}", LogSeverity::Debug), "DEBUG");
        assert_eq!(format!("{}", LogSeverity::Info), "INFO");
        assert_eq!(format!("{}", LogSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", LogSeverity::Error), "ERROR");
        assert_eq!(format!("{}", LogSeverity::Fatal), "FATAL");
    }
}
