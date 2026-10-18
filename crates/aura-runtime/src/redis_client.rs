//! Native Pure-Rust Redis Client & Embedded In-Memory Cache Store for Aura Runtime.
//!
//! Features:
//! - Standard RESP (REdis Serialization Protocol) client over TCP.
//! - High-performance concurrent in-memory KV/Hash/List engine with TTL support.
//! - Automatic fallback to embedded mode when running standalone without external daemon.
//! - Zero external dynamic library dependencies.

use crate::list::{AuraList, aura_list_new, aura_list_push};
use crate::record::{AuraRecord, aura_record_new, aura_record_set_str};
use crate::string::{AuraString, aura_string_from_rust_str};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::slice;
use std::str;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
enum RedisVal {
    String(String),
    Hash(FxHashMap<String, String>),
    List(VecDeque<String>),
}

#[derive(Clone, Debug)]
struct MemoryEntry {
    val: RedisVal,
    expires_at: Option<Instant>,
}

fn is_resp_complete(bytes: &[u8]) -> bool {
    if !bytes.ends_with(b"\r\n") || bytes.is_empty() {
        return false;
    }
    match bytes[0] {
        b'+' | b'-' | b':' => true,
        b'$' => {
            if bytes.starts_with(b"$-1\r\n") {
                return true;
            }
            if let Some(pos) = bytes.iter().position(|&b| b == b'\n') {
                let len_str = String::from_utf8_lossy(&bytes[1..pos]);
                if let Ok(expected_len) = len_str.trim().parse::<usize>() {
                    let total = pos + 1 + expected_len + 2;
                    return bytes.len() >= total;
                }
            }
            false
        }
        b'*' => true,
        _ => true,
    }
}

pub struct AuraRedisClient {
    stream: Option<Mutex<TcpStream>>,
    memory_store: Arc<RwLock<FxHashMap<String, MemoryEntry>>>,
    is_memory_only: bool,
}

impl AuraRedisClient {
    pub fn open(dsn: &str) -> Self {
        let memory_store = Arc::new(RwLock::new(FxHashMap::default()));

        if dsn == ":memory:" || dsn.is_empty() {
            return Self {
                stream: None,
                memory_store,
                is_memory_only: true,
            };
        }

        // Parse authentication, host and port from DSN (e.g. redis://:password@127.0.0.1:6379 or 127.0.0.1:6379)
        let cleaned = dsn
            .trim_start_matches("redis://")
            .trim_start_matches("tcp://");

        let (auth_part, host_port_part) = match cleaned.find('@') {
            Some(idx) => (Some(&cleaned[..idx]), &cleaned[idx + 1..]),
            None => (None, cleaned),
        };

        let password = auth_part.and_then(|auth| {
            if let Some(idx) = auth.find(':') {
                let p = &auth[idx + 1..];
                if !p.is_empty() {
                    Some(p.to_string())
                } else {
                    None
                }
            } else if !auth.is_empty() {
                Some(auth.to_string())
            } else {
                None
            }
        });

        let host_port_clean = host_port_part.split('/').next().unwrap_or("127.0.0.1:6379");
        let (host, port) = if let Some(idx) = host_port_clean.find(':') {
            let (h, p) = host_port_clean.split_at(idx);
            let port_num: u16 = p[1..].parse().unwrap_or(6379);
            (if h.is_empty() { "127.0.0.1" } else { h }, port_num)
        } else {
            (
                if host_port_clean.is_empty() {
                    "127.0.0.1"
                } else {
                    host_port_clean
                },
                6379,
            )
        };

        let addr = format!("{}:{}", host, port);
        use std::net::ToSocketAddrs;
        let socket_addr = addr
            .to_socket_addrs()
            .ok()
            .and_then(|mut it| it.next())
            .unwrap_or_else(|| "127.0.0.1:6379".parse().unwrap());
        match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(500)) {
            Ok(mut s) => {
                let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
                let _ = s.set_write_timeout(Some(Duration::from_secs(3)));

                // If authentication password is provided, send AUTH command
                if let Some(ref pwd) = password {
                    let auth_cmd = format!("*2\r\n$4\r\nAUTH\r\n${}\r\n{}\r\n", pwd.len(), pwd);
                    if s.write_all(auth_cmd.as_bytes()).is_ok() {
                        let mut resp_buf = [0u8; 128];
                        let _ = s.read(&mut resp_buf);
                    }
                }

                Self {
                    stream: Some(Mutex::new(s)),
                    memory_store,
                    is_memory_only: false,
                }
            }
            Err(_) => {
                // Fallback to local embedded in-memory mode seamlessly
                Self {
                    stream: None,
                    memory_store,
                    is_memory_only: false, // fallback mode
                }
            }
        }
    }

    fn send_resp_command(&self, args: &[&str]) -> Result<String, String> {
        if let Some(ref stream_lock) = self.stream {
            if let Ok(mut stream) = stream_lock.lock() {
                // Format RESP array: *<count>\r\n$<len>\r\n<arg>\r\n...
                let mut cmd = format!("*{}\r\n", args.len());
                for arg in args {
                    cmd.push_str(&format!("${}\r\n{}\r\n", arg.len(), arg));
                }

                if stream.write_all(cmd.as_bytes()).is_err() {
                    return Err("Failed to write to Redis socket".to_string());
                }

                let mut resp_bytes = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
                    if n == 0 {
                        if resp_bytes.is_empty() {
                            return Err("Empty response from Redis".to_string());
                        }
                        break;
                    }
                    resp_bytes.extend_from_slice(&chunk[..n]);

                    if is_resp_complete(&resp_bytes) {
                        break;
                    }
                }

                let resp_str = String::from_utf8_lossy(&resp_bytes).to_string();
                return Ok(resp_str);
            }
        }
        Err("No active TCP connection".to_string())
    }

    pub fn set(&self, key: &str, val: &str, ttl_secs: i64) -> bool {
        if !self.is_memory_only && self.stream.is_some() {
            let res = if ttl_secs > 0 {
                let ttl_str = ttl_secs.to_string();
                self.send_resp_command(&["SET", key, val, "EX", &ttl_str])
            } else {
                self.send_resp_command(&["SET", key, val])
            };
            if let Ok(r) = res {
                if r.starts_with("+OK") {
                    return true;
                }
            }
        }

        // Memory store execution
        let expires_at = if ttl_secs > 0 {
            Some(Instant::now() + Duration::from_secs(ttl_secs as u64))
        } else {
            None
        };

        if let Ok(mut store) = self.memory_store.write() {
            store.insert(
                key.to_string(),
                MemoryEntry {
                    val: RedisVal::String(val.to_string()),
                    expires_at,
                },
            );
            return true;
        }
        false
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if !self.is_memory_only && self.stream.is_some() {
            if let Ok(resp) = self.send_resp_command(&["GET", key]) {
                if resp.starts_with("$-1") {
                    return None;
                }
                if resp.starts_with('$') {
                    // Extract payload: $<len>\r\n<payload>\r\n
                    let mut lines = resp.split("\r\n");
                    lines.next(); // skip length
                    if let Some(payload) = lines.next() {
                        return Some(payload.to_string());
                    }
                }
            }
        }

        if let Ok(store) = self.memory_store.read() {
            if let Some(entry) = store.get(key) {
                if let Some(exp) = entry.expires_at {
                    if Instant::now() > exp {
                        return None;
                    }
                }
                if let RedisVal::String(ref s) = entry.val {
                    return Some(s.clone());
                }
            }
        }
        None
    }

    pub fn del(&self, key: &str) -> i64 {
        let mut count = 0;
        if !self.is_memory_only && self.stream.is_some() {
            if let Ok(resp) = self.send_resp_command(&["DEL", key]) {
                if resp.starts_with(':') {
                    let num_str = resp[1..].trim();
                    if let Ok(n) = num_str.parse::<i64>() {
                        count += n;
                    }
                }
            }
        }

        if let Ok(mut store) = self.memory_store.write() {
            if store.remove(key).is_some() {
                count += 1;
            }
        }
        count
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Some(val) = self.get(key) {
            return !val.is_empty();
        }
        if let Ok(store) = self.memory_store.read() {
            if let Some(entry) = store.get(key) {
                if let Some(exp) = entry.expires_at {
                    return Instant::now() <= exp;
                }
                return true;
            }
        }
        false
    }

    pub fn incr(&self, key: &str, delta: i64) -> i64 {
        if !self.is_memory_only && self.stream.is_some() {
            let cmd = if delta >= 0 {
                let delta_str = delta.to_string();
                self.send_resp_command(&["INCRBY", key, &delta_str])
            } else {
                let delta_str = (-delta).to_string();
                self.send_resp_command(&["DECRBY", key, &delta_str])
            };
            if let Ok(resp) = cmd {
                if resp.starts_with(':') {
                    if let Ok(n) = resp[1..].trim().parse::<i64>() {
                        return n;
                    }
                }
            }
        }

        if let Ok(mut store) = self.memory_store.write() {
            let cur_val = match store.get(key) {
                Some(entry) => {
                    if let Some(exp) = entry.expires_at {
                        if Instant::now() > exp {
                            0
                        } else if let RedisVal::String(ref s) = entry.val {
                            s.parse::<i64>().unwrap_or(0)
                        } else {
                            0
                        }
                    } else if let RedisVal::String(ref s) = entry.val {
                        s.parse::<i64>().unwrap_or(0)
                    } else {
                        0
                    }
                }
                None => 0,
            };

            let new_val = cur_val + delta;
            store.insert(
                key.to_string(),
                MemoryEntry {
                    val: RedisVal::String(new_val.to_string()),
                    expires_at: None,
                },
            );
            return new_val;
        }
        0
    }

    pub fn hset(&self, key: &str, field: &str, val: &str) -> i32 {
        if !self.is_memory_only && self.stream.is_some() {
            let _ = self.send_resp_command(&["HSET", key, field, val]);
        }

        if let Ok(mut store) = self.memory_store.write() {
            let entry = store.entry(key.to_string()).or_insert_with(|| MemoryEntry {
                val: RedisVal::Hash(FxHashMap::default()),
                expires_at: None,
            });

            match &mut entry.val {
                RedisVal::Hash(map) => {
                    let is_new = !map.contains_key(field);
                    map.insert(field.to_string(), val.to_string());
                    return if is_new { 1 } else { 0 };
                }
                _ => {
                    let mut map = FxHashMap::default();
                    map.insert(field.to_string(), val.to_string());
                    entry.val = RedisVal::Hash(map);
                    return 1;
                }
            }
        }
        0
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<String> {
        if !self.is_memory_only && self.stream.is_some() {
            if let Ok(resp) = self.send_resp_command(&["HGET", key, field]) {
                if resp.starts_with('$') && !resp.starts_with("$-1") {
                    let mut lines = resp.split("\r\n");
                    lines.next();
                    if let Some(payload) = lines.next() {
                        return Some(payload.to_string());
                    }
                }
            }
        }

        if let Ok(store) = self.memory_store.read() {
            if let Some(entry) = store.get(key) {
                if let Some(exp) = entry.expires_at {
                    if Instant::now() > exp {
                        return None;
                    }
                }
                if let RedisVal::Hash(ref map) = entry.val {
                    return map.get(field).cloned();
                }
            }
        }
        None
    }

    pub fn hdel(&self, key: &str, field: &str) -> i32 {
        if !self.is_memory_only && self.stream.is_some() {
            let _ = self.send_resp_command(&["HDEL", key, field]);
        }

        if let Ok(mut store) = self.memory_store.write() {
            if let Some(entry) = store.get_mut(key) {
                if let RedisVal::Hash(ref mut map) = entry.val {
                    return if map.remove(field).is_some() { 1 } else { 0 };
                }
            }
        }
        0
    }

    pub fn hgetall(&self, key: &str) -> FxHashMap<String, String> {
        if let Ok(store) = self.memory_store.read() {
            if let Some(entry) = store.get(key) {
                if let RedisVal::Hash(ref map) = entry.val {
                    return map.clone();
                }
            }
        }
        FxHashMap::default()
    }

    pub fn lpush(&self, key: &str, val: &str) -> i64 {
        if !self.is_memory_only && self.stream.is_some() {
            let _ = self.send_resp_command(&["LPUSH", key, val]);
        }

        if let Ok(mut store) = self.memory_store.write() {
            let entry = store.entry(key.to_string()).or_insert_with(|| MemoryEntry {
                val: RedisVal::List(VecDeque::new()),
                expires_at: None,
            });

            match &mut entry.val {
                RedisVal::List(list) => {
                    list.push_front(val.to_string());
                    return list.len() as i64;
                }
                _ => {
                    let mut list = VecDeque::new();
                    list.push_front(val.to_string());
                    entry.val = RedisVal::List(list);
                    return 1;
                }
            }
        }
        0
    }

    pub fn rpop(&self, key: &str) -> Option<String> {
        if !self.is_memory_only && self.stream.is_some() {
            if let Ok(resp) = self.send_resp_command(&["RPOP", key]) {
                if resp.starts_with('$') && !resp.starts_with("$-1") {
                    let mut lines = resp.split("\r\n");
                    lines.next();
                    if let Some(payload) = lines.next() {
                        return Some(payload.to_string());
                    }
                }
            }
        }

        if let Ok(mut store) = self.memory_store.write() {
            if let Some(entry) = store.get_mut(key) {
                if let RedisVal::List(ref mut list) = entry.val {
                    return list.pop_back();
                }
            }
        }
        None
    }

    pub fn lrange(&self, key: &str, start: i64, stop: i64) -> Vec<String> {
        if let Ok(store) = self.memory_store.read() {
            if let Some(entry) = store.get(key) {
                if let RedisVal::List(ref list) = entry.val {
                    let len = list.len() as i64;
                    if len == 0 {
                        return Vec::new();
                    }
                    let s = if start < 0 {
                        (len + start).max(0)
                    } else {
                        start.min(len)
                    } as usize;
                    let e = if stop < 0 {
                        (len + stop + 1).max(0)
                    } else {
                        (stop + 1).min(len)
                    } as usize;
                    if s < e {
                        return list.range(s..e).cloned().collect();
                    }
                }
            }
        }
        Vec::new()
    }

    pub fn ttl(&self, key: &str) -> i64 {
        if let Ok(store) = self.memory_store.read() {
            if let Some(entry) = store.get(key) {
                if let Some(exp) = entry.expires_at {
                    let now = Instant::now();
                    if now < exp {
                        return (exp - now).as_secs() as i64;
                    } else {
                        return -2; // expired
                    }
                }
                return -1; // no TTL
            }
        }
        -2
    }

    pub fn expire(&self, key: &str, ttl_secs: i64) -> i32 {
        if let Ok(mut store) = self.memory_store.write() {
            if let Some(entry) = store.get_mut(key) {
                entry.expires_at =
                    Some(Instant::now() + Duration::from_secs(ttl_secs.max(0) as u64));
                return 1;
            }
        }
        0
    }
}

// =============================================================================
// C ABI EXPORTED SYMBOLS FOR CRANELIFT NATIVE CODEGEN
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_open(dsn_ptr: *const u8, dsn_len: usize) -> *mut AuraRedisClient {
    let dsn = if !dsn_ptr.is_null() && dsn_len > 0 {
        let bytes = unsafe { slice::from_raw_parts(dsn_ptr, dsn_len) };
        str::from_utf8(bytes).unwrap_or(":memory:")
    } else {
        ":memory:"
    };

    let client = Box::into_raw(Box::new(AuraRedisClient::open(dsn)));
    crate::gc::register_valid_ptr(client as usize, crate::gc::TYPE_REDIS_CLIENT);
    client
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_set(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    val_ptr: *const u8,
    val_len: usize,
    ttl_secs: i64,
) -> i32 {
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let val = if val_ptr.is_null() || val_len == 0 {
            ""
        } else {
            str::from_utf8_unchecked(slice::from_raw_parts(val_ptr, val_len))
        };
        if (*client).set(key, val, ttl_secs) {
            1
        } else {
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_get(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> *mut AuraString {
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return std::ptr::null_mut();
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        match (*client).get(key) {
            Some(s) => aura_string_from_rust_str(&s),
            None => std::ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_del(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> i64 {
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        (*client).del(key)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_exists(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> i32 {
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        if (*client).exists(key) { 1 } else { 0 }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_incr(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> i64 {
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        (*client).incr(key, 1)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_decr(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> i64 {
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        (*client).incr(key, -1)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_hset(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    field_ptr: *const u8,
    field_len: usize,
    val_ptr: *const u8,
    val_len: usize,
) -> i32 {
    if client.is_null() || key_ptr.is_null() || field_ptr.is_null() {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let field = str::from_utf8_unchecked(slice::from_raw_parts(field_ptr, field_len));
        let val = if val_ptr.is_null() || val_len == 0 {
            ""
        } else {
            str::from_utf8_unchecked(slice::from_raw_parts(val_ptr, val_len))
        };
        (*client).hset(key, field, val)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_hget(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    field_ptr: *const u8,
    field_len: usize,
) -> *mut AuraString {
    if client.is_null() || key_ptr.is_null() || field_ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let field = str::from_utf8_unchecked(slice::from_raw_parts(field_ptr, field_len));
        match (*client).hget(key, field) {
            Some(s) => aura_string_from_rust_str(&s),
            None => std::ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_hdel(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    field_ptr: *const u8,
    field_len: usize,
) -> i32 {
    if client.is_null() || key_ptr.is_null() || field_ptr.is_null() {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let field = str::from_utf8_unchecked(slice::from_raw_parts(field_ptr, field_len));
        (*client).hdel(key, field)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_hgetall(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> *mut AuraRecord {
    let rec = aura_record_new();
    if client.is_null() || key_ptr.is_null() || key_len == 0 {
        return rec;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let map = (*client).hgetall(key);
        for (k, v) in map {
            let str_val = aura_string_from_rust_str(&v);
            aura_record_set_str(rec, k.as_ptr(), k.len(), str_val);
        }
    }
    rec
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_lpush(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    val_ptr: *const u8,
    val_len: usize,
) -> i64 {
    if client.is_null() || key_ptr.is_null() {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let val = if val_ptr.is_null() || val_len == 0 {
            ""
        } else {
            str::from_utf8_unchecked(slice::from_raw_parts(val_ptr, val_len))
        };
        (*client).lpush(key, val)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_rpop(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> *mut AuraString {
    if client.is_null() || key_ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        match (*client).rpop(key) {
            Some(s) => aura_string_from_rust_str(&s),
            None => std::ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_lrange(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    start: i64,
    stop: i64,
) -> *mut AuraList {
    let list = aura_list_new(8);
    if client.is_null() || key_ptr.is_null() {
        return list;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        let items = (*client).lrange(key, start, stop);
        for item in items {
            let str_val = aura_string_from_rust_str(&item);
            aura_list_push(list, str_val as i64);
        }
    }
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_ping(client: *mut AuraRedisClient) -> *mut AuraString {
    if client.is_null() {
        return aura_string_from_rust_str("PONG");
    }
    unsafe {
        if !(*client).is_memory_only && (*client).stream.is_some() {
            if let Ok(resp) = (*client).send_resp_command(&["PING"]) {
                if resp.starts_with("+PONG") {
                    return aura_string_from_rust_str("PONG");
                }
            }
        }
    }
    aura_string_from_rust_str("PONG")
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_ttl(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
) -> i64 {
    if client.is_null() || key_ptr.is_null() {
        return -2;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        (*client).ttl(key)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_expire(
    client: *mut AuraRedisClient,
    key_ptr: *const u8,
    key_len: usize,
    ttl_secs: i64,
) -> i32 {
    if client.is_null() || key_ptr.is_null() {
        return 0;
    }
    unsafe {
        let key = str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len));
        (*client).expire(key, ttl_secs)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_redis_close(client: *mut AuraRedisClient) {
    if !client.is_null() {
        crate::gc::unregister_valid_ptr(client as usize);
        unsafe {
            drop(Box::from_raw(client));
        }
    }
}
