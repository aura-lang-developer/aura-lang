//! Native dynamic Record / Struct implementation for Aura Runtime.

use crate::list::{AuraList, aura_list_get, aura_list_len};
use crate::string::{AuraString, aura_string_from_rust_str};
use serde_json::{Map, Number, Value as JsonValue};
use std::collections::HashMap;
use std::slice;
use std::str;

#[derive(Debug, Clone)]
pub enum AuraVal {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(*mut AuraString),
    Record(*mut AuraRecord),
    List(*mut AuraList),
}

pub const AURA_RECORD_MAGIC: u64 = 0xA052_4543_4F52_0002;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AuraRecord {
    pub magic: u64,
    pub fields: HashMap<String, AuraVal>,
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_new() -> *mut AuraRecord {
    let rec = Box::new(AuraRecord {
        magic: AURA_RECORD_MAGIC,
        fields: HashMap::new(),
    });
    let raw = Box::into_raw(rec);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_RECORD);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set_i64(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: i64,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        (*rec).fields.insert(key.to_string(), AuraVal::Int(val));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set_f64(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: f64,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        (*rec).fields.insert(key.to_string(), AuraVal::Float(val));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set_bool(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: bool,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        (*rec).fields.insert(key.to_string(), AuraVal::Bool(val));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set_str(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: *mut AuraString,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        (*rec).fields.insert(key.to_string(), AuraVal::String(val));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set_record(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: *mut AuraRecord,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        (*rec).fields.insert(key.to_string(), AuraVal::Record(val));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set_list(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: *mut AuraList,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        (*rec).fields.insert(key.to_string(), AuraVal::List(val));
    }
}

/// Generic untyped setter: stores val directly as i64 / ptr
#[unsafe(no_mangle)]
pub extern "C" fn aura_record_set(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
    val: i64,
) {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        // Introspect value: if it's a known pointer or scalar
        let auraval = infer_val_from_i64(val);
        (*rec).fields.insert(key.to_string(), auraval);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_get(rec: *mut AuraRecord, key_ptr: *const u8, key_len: usize) -> i64 {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        let magic = *(rec as *const u64);
        if magic == crate::list::AURA_LIST_MAGIC {
            if key == "length" {
                return (*(rec as *mut AuraList)).len as i64;
            }
            return 0;
        }
        if magic == crate::string::AURA_STRING_MAGIC {
            if key == "length" {
                return (*(rec as *mut AuraString)).len as i64;
            }
            return 0;
        }

        match (*rec).fields.get(key) {
            Some(AuraVal::Null) => 0,
            Some(AuraVal::Bool(b)) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Some(AuraVal::Int(i)) => *i,
            Some(AuraVal::Float(f)) => f.to_bits() as i64,
            Some(AuraVal::String(s)) => *s as i64,
            Some(AuraVal::Record(r)) => *r as i64,
            Some(AuraVal::List(l)) => *l as i64,
            None => 0,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_get_f64(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
) -> f64 {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0.0;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe {
        match (*rec).fields.get(key) {
            Some(AuraVal::Float(f)) => *f,
            Some(AuraVal::Int(i)) => *i as f64,
            Some(AuraVal::Bool(b)) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_has(
    rec: *mut AuraRecord,
    key_ptr: *const u8,
    key_len: usize,
) -> bool {
    if rec.is_null() || key_ptr.is_null() || key_len == 0 {
        return false;
    }
    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    unsafe { (*rec).fields.contains_key(key) }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_to_json(rec: *mut AuraRecord) -> *mut AuraString {
    if rec.is_null() {
        return aura_string_from_rust_str("null");
    }
    let json_val = record_to_json_value(rec);
    let s = serde_json::to_string(&json_val).unwrap_or_else(|_| "{}".to_string());
    aura_string_from_rust_str(&s)
}

pub fn record_to_json_value(rec: *mut AuraRecord) -> JsonValue {
    if rec.is_null() {
        return JsonValue::Null;
    }
    let mut map = Map::new();
    unsafe {
        for (k, v) in &(*rec).fields {
            map.insert(k.clone(), val_to_json_value(v));
        }
    }
    JsonValue::Object(map)
}

pub fn list_to_json_value(list: *mut AuraList) -> JsonValue {
    if list.is_null() {
        return JsonValue::Array(Vec::new());
    }
    let len = aura_list_len(list);
    let mut vec = Vec::with_capacity(len);
    for i in 0..len {
        let raw = aura_list_get(list, i);
        let aval = infer_val_from_i64(raw);
        vec.push(val_to_json_value(&aval));
    }
    JsonValue::Array(vec)
}

pub fn val_to_json_value(val: &AuraVal) -> JsonValue {
    match val {
        AuraVal::Null => JsonValue::Null,
        AuraVal::Bool(b) => JsonValue::Bool(*b),
        AuraVal::Int(i) => JsonValue::Number(Number::from(*i)),
        AuraVal::Float(f) => {
            if let Some(n) = Number::from_f64(*f) {
                JsonValue::Number(n)
            } else {
                JsonValue::Null
            }
        }
        AuraVal::String(s) => {
            if s.is_null() {
                JsonValue::String(String::new())
            } else {
                let str_slice = unsafe { (*(*s)).as_str() };
                JsonValue::String(str_slice.to_string())
            }
        }
        AuraVal::Record(r) => record_to_json_value(*r),
        AuraVal::List(l) => list_to_json_value(*l),
    }
}

pub fn infer_val_from_i64(val: i64) -> AuraVal {
    if val == 0 {
        return AuraVal::Null;
    }

    if let Some(type_id) = crate::gc::lookup_valid_ptr(val as usize) {
        match type_id {
            crate::gc::TYPE_STRING => return AuraVal::String(val as *mut AuraString),
            crate::gc::TYPE_RECORD => return AuraVal::Record(val as *mut AuraRecord),
            crate::gc::TYPE_LIST => return AuraVal::List(val as *mut AuraList),
            _ => {}
        }
    }

    AuraVal::Int(val)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_record_free(rec: *mut AuraRecord) {
    if !rec.is_null() {
        crate::gc::unregister_valid_ptr(rec as usize);
        unsafe {
            drop(Box::from_raw(rec));
        }
    }
}
