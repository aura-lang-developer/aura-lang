//! Native JSON serialization and parsing for Aura Runtime.

use crate::string::{AuraString, aura_string_from_rust_str};
use serde_json::Value;
use std::slice;
use std::str;

#[repr(C)]
#[derive(Debug)]
pub struct AuraJsonDoc {
    pub value: Value,
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_json_parse(ptr: *const u8, len: usize) -> *mut AuraJsonDoc {
    if ptr.is_null() || len == 0 {
        return Box::into_raw(Box::new(AuraJsonDoc { value: Value::Null }));
    }

    let slice_bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let parsed: Value = match serde_json::from_slice(slice_bytes) {
        Ok(v) => v,
        Err(_) => Value::Null,
    };

    Box::into_raw(Box::new(AuraJsonDoc { value: parsed }))
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_json_get_str(
    doc: *mut AuraJsonDoc,
    key_ptr: *const u8,
    key_len: usize,
) -> *mut AuraString {
    if doc.is_null() || key_ptr.is_null() || key_len == 0 {
        return aura_string_from_rust_str("");
    }

    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let doc_ref = unsafe { &*doc };
    if let Value::Object(map) = &doc_ref.value {
        if let Some(val) = map.get(key) {
            match val {
                Value::String(s) => return aura_string_from_rust_str(s),
                _ => return aura_string_from_rust_str(&val.to_string()),
            }
        }
    }

    aura_string_from_rust_str("")
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_json_get_i64(
    doc: *mut AuraJsonDoc,
    key_ptr: *const u8,
    key_len: usize,
) -> i64 {
    if doc.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }

    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let doc_ref = unsafe { &*doc };
    if let Value::Object(map) = &doc_ref.value {
        if let Some(val) = map.get(key) {
            if let Some(n) = val.as_i64() {
                return n;
            }
        }
    }

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_json_stringify(doc: *mut AuraJsonDoc) -> *mut AuraString {
    if doc.is_null() {
        return aura_string_from_rust_str("null");
    }

    let doc_ref = unsafe { &*doc };
    let json_text = serde_json::to_string(&doc_ref.value).unwrap_or_else(|_| "null".to_string());
    aura_string_from_rust_str(&json_text)
}

use crate::list::{aura_list_new, aura_list_push};
use crate::record::{AuraRecord, AuraVal};

pub fn json_value_to_aura_val(val: &Value) -> AuraVal {
    match val {
        Value::Null => AuraVal::Null,
        Value::Bool(b) => AuraVal::Bool(*b),
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                AuraVal::Int(i)
            } else if let Some(f) = num.as_f64() {
                AuraVal::Float(f)
            } else {
                AuraVal::Int(0)
            }
        }
        Value::String(s) => {
            let str_ptr = aura_string_from_rust_str(s);
            AuraVal::String(str_ptr)
        }
        Value::Array(arr) => {
            let list = aura_list_new(arr.len());
            for item in arr {
                let aval = json_value_to_aura_val(item);
                let raw = match aval {
                    AuraVal::Null => 0,
                    AuraVal::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    AuraVal::Int(i) => i,
                    AuraVal::Float(f) => f.to_bits() as i64,
                    AuraVal::String(s) => s as i64,
                    AuraVal::Record(r) => r as i64,
                    AuraVal::List(l) => l as i64,
                };
                aura_list_push(list, raw);
            }
            AuraVal::List(list)
        }
        Value::Object(map) => {
            let rec_ptr = crate::record::aura_record_new();
            for (k, v) in map {
                let aval = json_value_to_aura_val(v);
                unsafe {
                    (*rec_ptr).fields.insert(k.clone(), aval);
                }
            }
            AuraVal::Record(rec_ptr)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_json_parse_to_record(ptr: *const u8, len: usize) -> *mut AuraRecord {
    if ptr.is_null() || len == 0 {
        return crate::record::aura_record_new();
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let parsed: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(_) => return crate::record::aura_record_new(),
    };

    match json_value_to_aura_val(&parsed) {
        AuraVal::Record(r) => r,
        _ => crate::record::aura_record_new(),
    }
}
