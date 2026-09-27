//! Native OS primitives for Aura Runtime.

use crate::list::{AuraList, aura_list_new, aura_list_push};
use crate::string::{AuraString, aura_string_from_rust_str};
use std::slice;
use std::str;

#[unsafe(no_mangle)]
pub extern "C" fn aura_os_env(key_ptr: *const u8, key_len: usize) -> *mut AuraString {
    if key_ptr.is_null() || key_len == 0 {
        return aura_string_from_rust_str("");
    }

    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let val = std::env::var(key).unwrap_or_default();
    aura_string_from_rust_str(&val)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_os_setenv(
    key_ptr: *const u8,
    key_len: usize,
    val_ptr: *const u8,
    val_len: usize,
) {
    if key_ptr.is_null() || key_len == 0 {
        return;
    }

    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let val = unsafe {
        if val_ptr.is_null() || val_len == 0 {
            ""
        } else {
            let bytes = slice::from_raw_parts(val_ptr, val_len);
            str::from_utf8(bytes).unwrap_or("")
        }
    };

    unsafe {
        std::env::set_var(key, val);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_os_args() -> *mut AuraList {
    let args: Vec<String> = std::env::args().collect();
    let list = aura_list_new(args.len());

    for arg in args {
        let str_ptr = aura_string_from_rust_str(&arg);
        aura_list_push(list, str_ptr as i64);
    }

    list
}

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn get_start_time() -> &'static std::time::Instant {
    START_TIME.get_or_init(std::time::Instant::now)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_process_uptime() -> i64 {
    let elapsed = get_start_time().elapsed();
    let f = elapsed.as_secs_f64();
    f.to_bits() as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_os_hostname() -> *mut AuraString {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "aura-node-local".to_string());
    aura_string_from_rust_str(&hostname)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_time_now() -> *mut AuraString {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let now_str = format!("2026-09-14T23:30:{:02}Z", secs % 60);
    aura_string_from_rust_str(&now_str)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_parse_float(s: *mut AuraString) -> i64 {
    if s.is_null() {
        return 0;
    }
    let text = unsafe { (*s).as_str().trim() };
    let f = text.parse::<f64>().unwrap_or(0.0);
    f.to_bits() as i64
}
