use std::ffi::CStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_family = "unix")]
pub fn now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = now.as_secs() as libc::time_t;

    let mut tm: libc::tm = unsafe { std::mem::zeroed() };

    unsafe {
        libc::localtime_r(&secs, &mut tm);
    }

    let mut buf = [0i8; 100];
    let fmt = std::ffi::CString::new("%Y-%m-%d %H:%M:%S %Z").unwrap();

    unsafe {
        libc::strftime(buf.as_mut_ptr(), buf.len(), fmt.as_ptr(), &tm);
        let c_str = CStr::from_ptr(buf.as_ptr());

        c_str.to_string_lossy().to_string()
    }
}