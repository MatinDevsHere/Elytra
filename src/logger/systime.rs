use std::ffi::CStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current time in the format YYYY-MM-DD HH:MM:SS TZ
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

/// Returns the current time in the format YYYY-MM-DD HH:MM:SS TZ
#[cfg(target_family = "windows")]
pub fn now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = now.as_secs() as i64;

    let mut tm: windows_sys::Win32::System::Time::SYSTEMTIME = unsafe { std::mem::zeroed() };
    let mut tz: windows_sys::Win32::System::Time::TIME_ZONE_INFORMATION = unsafe { std::mem::zeroed() };

    unsafe {
        windows_sys::Win32::System::Time::GetLocalTime(&mut tm);
        windows_sys::Win32::System::Time::GetTimeZoneInformation(&mut tz);
    }

    let mut buf = [0i8; 100];
    let fmt = std::ffi::CString::new("%Y-%m-%d %H:%M:%S %Z").unwrap();

    unsafe {
        let mut time = windows_sys::Win32::System::Time::SYSTEMTIME {
            wYear: tm.wYear,
            wMonth: tm.wMonth,
            wDayOfWeek: tm.wDayOfWeek,
            wDay: tm.wDay,
            wHour: tm.wHour,
            wMinute: tm.wMinute,
            wSecond: tm.wSecond,
            wMilliseconds: tm.wMilliseconds,
        };

        let mut buf = [0u16; 100];
        let len = windows_sys::Win32::System::Time::GetDateFormatW(
            windows_sys::Win32::System::SystemServices::LOCALE_USER_DEFAULT,
            0,
            &time,
            windows_sys::core::w!("yyyy-MM-dd HH:mm:ss"),
            buf.as_mut_ptr(),
            100,
        );

        let time_str = String::from_utf16_lossy(&buf[..len as usize - 1]);
        format!("{} {}", time_str, get_timezone_name())
    }
}

#[cfg(target_family = "windows")]
fn get_timezone_name() -> String {
    let mut tz: windows_sys::Win32::System::Time::TIME_ZONE_INFORMATION = unsafe { std::mem::zeroed() };
    unsafe {
        windows_sys::Win32::System::Time::GetTimeZoneInformation(&mut tz);
        let tz_name = tz.StandardName;
        let mut name = Vec::new();
        for c in tz_name.iter() {
            if *c == 0 {
                break;
            }
            name.push(*c);
        }
        String::from_utf16_lossy(&name)
    }
}

/// Returns the current Unix timestamp in seconds
pub fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}
