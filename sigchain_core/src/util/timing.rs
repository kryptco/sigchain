pub use std::time;

#[cfg(not(target_os = "android"))]
#[macro_export]
macro_rules! time_fn {
    ($name:expr) => {
        use std::time;
        let start = time::Instant::now();
        defer!({
            let delta = start.elapsed();
            info!("{} took {}ms", $name, (delta.as_secs() * 1_000) + (delta.subsec_nanos() / 1_000_000) as u64);
        });
    };
}

#[cfg(not(target_os = "android"))]
#[macro_export]
macro_rules! time_expr {
    ($name:expr, $($rest:tt)+) => {
        let start = time::Instant::now();
        $($rest)*
        let delta = start.elapsed();
        info!("{} took {}ms", $name, (delta.as_secs() * 1_000) + (delta.subsec_nanos() / 1_000_000) as u64);
    };
}

#[cfg(target_os = "android")]
#[macro_use]
pub mod android_timing {
    //
    //       android/log.h
    //
    pub const ANDROID_LOG_DEBUG: i32 = 3;
    pub const ANDROID_LOG_DEFAULT: i32 = 1;
    pub const ANDROID_LOG_ERROR: i32 = 6;
    pub const ANDROID_LOG_FATAL: i32 = 7;
    pub const ANDROID_LOG_INFO: i32 = 4;
    pub const ANDROID_LOG_SILENT: i32 = 8;
    pub const ANDROID_LOG_UNKNOWN: i32 = 0;
    pub const ANDROID_LOG_VERBOSE: i32 = 2;
    pub const ANDROID_LOG_WARN: i32 = 5;

    use std::os::raw::c_char;
    use std::os::raw::c_int;

    extern { pub fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int; }

    pub type android_LogPriority = i32;

    pub fn debug_log(msg: &str) {
        use std::ffi::CString;
        if let (Ok(tag), Ok(msg)) = (CString::new("sigchain"), CString::new(msg)) {
            unsafe {
                __android_log_write(
                    ANDROID_LOG_DEBUG,
                    tag.as_ptr(),
                    msg.as_ptr(),
                );
            }
        }
    }

    #[macro_export]
    macro_rules! time_fn {
        ($name:expr) => {
            use std::time;
            use android_timing::debug_log;
            let start = time::Instant::now();
            defer!({
                let delta = start.elapsed();
                let msg = format!("{} took {}ms", $name, (delta.as_secs() * 1_000) + (delta.subsec_nanos() / 1_000_000) as u64);
                let _ = debug_log(&msg);
            });
        };
    }

    #[macro_export]
    macro_rules! time_expr {
    ($name:expr, $($rest:tt)+) => {
        let start = time::Instant::now();
        $($rest)*
        let delta = start.elapsed();
        let msg = format!("{} took {}ms", $name, (delta.as_secs() * 1_000) + (delta.subsec_nanos() / 1_000_000) as u64);
        let _ = debug_log(&msg);
    };
}
}
#[cfg(target_os = "android")]
pub use self::android_timing::*;
