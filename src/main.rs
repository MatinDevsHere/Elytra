extern crate libc;
use crate::LogSeverity::*;
use std::ffi::CStr;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    println!("wow");
    log("wow".to_string(), Info);
}

fn log(msg: String, log_severity: LogSeverity) {
    println!("[{}] {} {}", log_severity, now(), msg);
}

#[cfg(target_os = "linux")]
fn now() -> String {
    // Obtain the current time as a duration since the UNIX epoch.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = now.as_secs() as libc::time_t;

    // Prepare a zeroed `tm` structure.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };

    // Convert the timestamp to local time.
    // Note: `localtime_r` is the thread-safe version of `localtime`.
    unsafe {
        libc::localtime_r(&secs, &mut tm);
    }

    // Create a buffer to hold the formatted date/time string.
    // We use %Y-%m-%d %H:%M:%S %Z for: Year-Month-Day Hour:Minute:Second TimeZone
    let mut buf = [0i8; 100];
    let fmt = std::ffi::CString::new("%Y-%m-%d %H:%M:%S %Z").unwrap();

    // Format the local time into our buffer.
    unsafe {
        libc::strftime(buf.as_mut_ptr(), buf.len(), fmt.as_ptr(), &tm);
        let c_str = CStr::from_ptr(buf.as_ptr());

        c_str.to_string_lossy().to_string()
    }
}

enum LogSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl Display for LogSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Info => write!(f, "INFO"),
            Warning => write!(f, "WARNING"),
            Error => write!(f, "ERROR"),
            Fatal => write!(f, "FATAL"),
        }
    }
}
