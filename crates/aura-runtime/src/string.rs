//! Native String implementation for Aura Runtime.

use std::alloc::{Layout, alloc};
use std::ptr;
use std::slice;
use std::str;

pub const AURA_STRING_MAGIC: u64 = 0xA054_5249_4E47_0001;

#[repr(C)]
#[derive(Debug)]
pub struct AuraString {
    pub magic: u64,
    pub ptr: *mut u8,
    pub len: usize,
}

impl AuraString {
    pub fn as_str(&self) -> &str {
        if self.ptr.is_null() || self.len == 0 {
            return "";
        }
        unsafe {
            let bytes = slice::from_raw_parts(self.ptr, self.len);
            str::from_utf8(bytes).unwrap_or("")
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_new(bytes: *const u8, len: usize) -> *mut AuraString {
    let s_box = Box::new(AuraString {
        magic: AURA_STRING_MAGIC,
        ptr: if len > 0 && !bytes.is_null() {
            unsafe {
                let layout = Layout::from_size_align(len + 1, 1).unwrap();
                let buf = alloc(layout);
                ptr::copy_nonoverlapping(bytes, buf, len);
                *buf.add(len) = 0; // null-terminate for C safety
                buf
            }
        } else {
            ptr::null_mut()
        },
        len,
    });
    let raw = Box::into_raw(s_box);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_STRING);
    raw
}

pub fn aura_string_from_rust_str(s: &str) -> *mut AuraString {
    aura_string_new(s.as_ptr(), s.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_len(s: *mut AuraString) -> usize {
    if s.is_null() { 0 } else { unsafe { (*s).len } }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_concat(s1: *mut AuraString, s2: *mut AuraString) -> *mut AuraString {
    let str1 = if s1.is_null() {
        ""
    } else {
        unsafe { (*s1).as_str() }
    };
    let str2 = if s2.is_null() {
        ""
    } else {
        unsafe { (*s2).as_str() }
    };

    let combined = format!("{}{}", str1, str2);
    aura_string_from_rust_str(&combined)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_eq(s1: *mut AuraString, s2: *mut AuraString) -> bool {
    let str1 = if s1.is_null() {
        ""
    } else {
        unsafe { (*s1).as_str() }
    };
    let str2 = if s2.is_null() {
        ""
    } else {
        unsafe { (*s2).as_str() }
    };
    str1 == str2
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_to_lower(s: *mut AuraString) -> *mut AuraString {
    let text = if s.is_null() {
        ""
    } else {
        unsafe { (*s).as_str() }
    };
    let lowered = text.to_lowercase();
    aura_string_from_rust_str(&lowered)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_contains(s: *mut AuraString, sub: *mut AuraString) -> bool {
    let str1 = if s.is_null() {
        ""
    } else {
        unsafe { (*s).as_str() }
    };
    let str2 = if sub.is_null() {
        ""
    } else {
        unsafe { (*sub).as_str() }
    };
    str1.contains(str2)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_cstr(s: *mut AuraString) -> *const u8 {
    if s.is_null() || unsafe { (*s).ptr.is_null() } {
        b"\0".as_ptr()
    } else {
        unsafe { (*s).ptr }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_string_free(s: *mut AuraString) {
    if !s.is_null() {
        crate::gc::unregister_valid_ptr(s as usize);
        unsafe {
            let str_box = Box::from_raw(s);
            if !str_box.ptr.is_null() && str_box.len > 0 {
                let layout = Layout::from_size_align(str_box.len + 1, 1).unwrap();
                std::alloc::dealloc(str_box.ptr, layout);
            }
        }
    }
}
