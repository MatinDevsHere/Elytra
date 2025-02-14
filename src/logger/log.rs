use crate::logger::severity::LogSeverity;
use crate::logger::systime::now;

/// Logs a message to the console
pub fn log(msg: String, log_severity: LogSeverity) {
    println!("[{}] {} {}", log_severity, now(), msg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::Once;

    // Use a mutex to capture output in tests
    static TEST_MUTEX: Mutex<()> = Mutex::new(());
    static INIT: Once = Once::new();

    #[test]
    fn test_log_output_format() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let msg = "Test message";
        log(msg.to_string(), LogSeverity::Info);
        // Since we can't easily capture stdout, we'll just verify the code runs
        // and check the format of the components
        assert_eq!(format!("{}", LogSeverity::Info), "INFO");
        assert!(!now().is_empty());
    }

    #[test]
    fn test_log_different_severities() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let severities = vec![
            LogSeverity::Debug,
            LogSeverity::Info,
            LogSeverity::Warning,
            LogSeverity::Error,
            LogSeverity::Fatal,
        ];

        for severity in severities {
            let severity_str = format!("{}", severity);
            log(format!("{} test", severity_str), severity);
            // Verify severity string format
            assert!(!severity_str.is_empty());
        }
    }

    #[test]
    fn test_log_empty_message() {
        let _lock = TEST_MUTEX.lock().unwrap();
        log("".to_string(), LogSeverity::Info);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_log_special_characters() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let test_messages = vec![
            "Test with spaces",
            "Test with symbols !@#$%^&*()",
            "Test with Unicode 你好",
            "Test with emoji 🦀",
            "Test with newline\n",
        ];

        for msg in test_messages {
            log(msg.to_string(), LogSeverity::Info);
            // Test passes if no panic occurs
        }
    }
}
