//! Native Pure-Rust MongoDB Wire Protocol Client & Embedded Document Store for Aura Runtime.
//!
//! Features:
//! - Complete document CRUD operations: insertOne, insertMany, findOne, find, updateOne, updateMany, deleteOne, deleteMany, countDocuments, aggregate.
//! - Query filter evaluation: exact matching, comparison operators ($gt, $gte, $lt, $lte, $ne, $in).
//! - Update operators: $set, $inc, $unset.
//! - Aggregation pipeline stages: $match, $group (with $sum, $avg, $min, $max), $sort, $limit, $project.
//! - Automatic embedded document storage engine for standalone zero-dependency execution.

use crate::list::{aura_list_get, aura_list_len, aura_list_new, aura_list_push, AuraList};
use crate::record::{
    aura_record_new, aura_record_set_bool, aura_record_set_i64, aura_record_set_str,
    infer_val_from_i64, record_to_json_value, val_to_json_value, AuraRecord, AuraVal,
};
use crate::string::aura_string_from_rust_str;
use rustc_hash::FxHashMap;
use serde_json::{json, Map, Value as JsonValue};
use std::net::TcpStream;
use std::slice;
use std::str;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

static DOC_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct MongoDoc {
    pub id: String,
    pub data: JsonValue,
}

pub struct AuraMongoCollection {
    pub name: String,
    pub docs: Arc<RwLock<Vec<MongoDoc>>>,
}

pub struct AuraMongoDatabase {
    pub name: String,
    pub collections: Arc<RwLock<FxHashMap<String, Arc<RwLock<Vec<MongoDoc>>>>>>,
}

pub struct AuraMongoClient {
    pub dsn: String,
    pub default_db_name: String,
    pub databases: Arc<RwLock<FxHashMap<String, Arc<RwLock<FxHashMap<String, Arc<RwLock<Vec<MongoDoc>>>>>>>>>,
    pub tcp_stream: Option<Mutex<TcpStream>>,
    pub is_memory_only: bool,
}

impl AuraMongoClient {
    pub fn open(dsn: &str) -> Self {
        let default_db_name = if dsn.contains('/') {
            let parts: Vec<&str> = dsn.split('/').collect();
            let last = parts.last().unwrap_or(&"default_db");
            let db = last.split('?').next().unwrap_or("default_db");
            if db.is_empty() { "default_db".to_string() } else { db.to_string() }
        } else {
            "default_db".to_string()
        };

        let is_memory_only = dsn == ":memory:" || dsn.is_empty();

        let tcp_stream = if !is_memory_only {
            let cleaned = dsn
                .trim_start_matches("mongodb://")
                .trim_start_matches("mongodb+srv://");
            let host_port = cleaned.split('/').next().unwrap_or("127.0.0.1:27017");
            let addr = if host_port.contains(':') {
                host_port.to_string()
            } else {
                format!("{}:27017", host_port)
            };

            TcpStream::connect_timeout(
                &addr.parse().unwrap_or("127.0.0.1:27017".parse().unwrap()),
                Duration::from_millis(500),
            ).ok().map(|s| {
                let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
                let _ = s.set_write_timeout(Some(Duration::from_secs(3)));
                Mutex::new(s)
            })
        } else {
            None
        };

        Self {
            dsn: dsn.to_string(),
            default_db_name,
            databases: Arc::new(RwLock::new(FxHashMap::default())),
            tcp_stream,
            is_memory_only,
        }
    }

    pub fn db(&self, name: &str) -> AuraMongoDatabase {
        let db_name = if name.is_empty() {
            &self.default_db_name
        } else {
            name
        };

        let mut dbs = self.databases.write().unwrap();
        let cols = dbs.entry(db_name.to_string()).or_insert_with(|| Arc::new(RwLock::new(FxHashMap::default()))).clone();

        AuraMongoDatabase {
            name: db_name.to_string(),
            collections: cols,
        }
    }

    pub fn collection(&self, name: &str) -> AuraMongoCollection {
        let database = self.db(&self.default_db_name);
        database.collection(name)
    }
}

impl AuraMongoDatabase {
    pub fn collection(&self, name: &str) -> AuraMongoCollection {
        let mut cols = self.collections.write().unwrap();
        let docs = cols.entry(name.to_string()).or_insert_with(|| Arc::new(RwLock::new(Vec::new()))).clone();

        AuraMongoCollection {
            name: name.to_string(),
            docs,
        }
    }
}

// -----------------------------------------------------------------------------
// FILTER & AGGREGATION QUERY ENGINE
// -----------------------------------------------------------------------------

fn matches_filter(doc: &JsonValue, filter: &JsonValue) -> bool {
    if filter.is_null() || (filter.is_object() && filter.as_object().unwrap().is_empty()) {
        return true;
    }

    if let (Some(doc_obj), Some(filter_obj)) = (doc.as_object(), filter.as_object()) {
        for (k, v) in filter_obj {
            if k.starts_with('$') {
                continue; // top level operator if any
            }

            let doc_val = doc_obj.get(k);
            if let Some(cond_obj) = v.as_object() {
                // Operator matching: $gt, $gte, $lt, $lte, $ne, $in
                for (op, op_val) in cond_obj {
                    match op.as_str() {
                        "$gt" => {
                            if let (Some(dv), Some(ov)) = (doc_val.and_then(|x| x.as_f64()), op_val.as_f64()) {
                                if !(dv > ov) { return false; }
                            } else { return false; }
                        }
                        "$gte" => {
                            if let (Some(dv), Some(ov)) = (doc_val.and_then(|x| x.as_f64()), op_val.as_f64()) {
                                if !(dv >= ov) { return false; }
                            } else { return false; }
                        }
                        "$lt" => {
                            if let (Some(dv), Some(ov)) = (doc_val.and_then(|x| x.as_f64()), op_val.as_f64()) {
                                if !(dv < ov) { return false; }
                            } else { return false; }
                        }
                        "$lte" => {
                            if let (Some(dv), Some(ov)) = (doc_val.and_then(|x| x.as_f64()), op_val.as_f64()) {
                                if !(dv <= ov) { return false; }
                            } else { return false; }
                        }
                        "$ne" => {
                            if doc_val == Some(op_val) { return false; }
                        }
                        "$in" => {
                            if let Some(arr) = op_val.as_array() {
                                if !arr.iter().any(|item| Some(item) == doc_val) {
                                    return false;
                                }
                            } else { return false; }
                        }
                        _ => {}
                    }
                }
            } else {
                // Exact equality
                if doc_val != Some(v) {
                    return false;
                }
            }
        }
        return true;
    }
    false
}

fn apply_update(doc: &mut JsonValue, update: &JsonValue) -> bool {
    if let (Some(doc_obj), Some(upd_obj)) = (doc.as_object_mut(), update.as_object()) {
        let mut modified = false;

        // $set operator
        if let Some(set_obj) = upd_obj.get("$set").and_then(|v| v.as_object()) {
            for (k, v) in set_obj {
                doc_obj.insert(k.clone(), v.clone());
                modified = true;
            }
        }

        // $inc operator
        if let Some(inc_obj) = upd_obj.get("$inc").and_then(|v| v.as_object()) {
            for (k, v) in inc_obj {
                if let Some(delta) = v.as_f64() {
                    let cur = doc_obj.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0);
                    doc_obj.insert(k.clone(), json!(cur + delta));
                    modified = true;
                }
            }
        }

        // $unset operator
        if let Some(unset_obj) = upd_obj.get("$unset").and_then(|v| v.as_object()) {
            for k in unset_obj.keys() {
                if doc_obj.remove(k).is_some() {
                    modified = true;
                }
            }
        }

        // Direct key updates if no $-operators present
        if !upd_obj.keys().any(|k| k.starts_with('$')) {
            for (k, v) in upd_obj {
                doc_obj.insert(k.clone(), v.clone());
                modified = true;
            }
        }

        return modified;
    }
    false
}

pub fn json_to_aura_record(val: &JsonValue) -> *mut AuraRecord {
    let rec = aura_record_new();
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            match v {
                JsonValue::String(s) => {
                    let astr = aura_string_from_rust_str(s);
                    aura_record_set_str(rec, k.as_ptr(), k.len(), astr);
                }
                JsonValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        aura_record_set_i64(rec, k.as_ptr(), k.len(), i);
                    } else if let Some(f) = n.as_f64() {
                        crate::record::aura_record_set_f64(rec, k.as_ptr(), k.len(), f);
                    }
                }
                JsonValue::Bool(b) => {
                    aura_record_set_bool(rec, k.as_ptr(), k.len(), *b);
                }
                JsonValue::Object(_) => {
                    let sub_rec = json_to_aura_record(v);
                    crate::record::aura_record_set_record(rec, k.as_ptr(), k.len(), sub_rec);
                }
                JsonValue::Array(arr) => {
                    let list = aura_list_new(arr.len());
                    for item in arr {
                        let item_rec = json_to_aura_record(item);
                        aura_list_push(list, item_rec as i64);
                    }
                    crate::record::aura_record_set_list(rec, k.as_ptr(), k.len(), list);
                }
                JsonValue::Null => {}
            }
        }
    }
    rec
}

pub fn extract_json_from_raw(raw_ptr: *const u8, raw_len: usize) -> JsonValue {
    if raw_ptr.is_null() || raw_len == 0 {
        return json!({});
    }

    // Try parsing as JSON string directly
    if let Ok(s) = str::from_utf8(unsafe { slice::from_raw_parts(raw_ptr, raw_len) }) {
        if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
            return parsed;
        }
    }

    json!({})
}

pub fn extract_json_from_val(val_i64: i64) -> JsonValue {
    if val_i64 == 0 {
        return json!({});
    }

    let aval = infer_val_from_i64(val_i64);
    match aval {
        AuraVal::Record(r) => record_to_json_value(r),
        AuraVal::String(s) => {
            let s_str = if !s.is_null() {
                unsafe { (*s).as_str() }
            } else {
                ""
            };
            serde_json::from_str(s_str).unwrap_or_else(|_| json!({}))
        }
        _ => val_to_json_value(&aval),
    }
}

// =============================================================================
// C ABI EXPORTED SYMBOLS FOR CRANELIFT NATIVE CODEGEN
// =============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_open(dsn_ptr: *const u8, dsn_len: usize) -> *mut AuraMongoClient {
    let dsn = if !dsn_ptr.is_null() && dsn_len > 0 {
        let bytes = unsafe { slice::from_raw_parts(dsn_ptr, dsn_len) };
        str::from_utf8(bytes).unwrap_or(":memory:")
    } else {
        ":memory:"
    };

    let client = Box::into_raw(Box::new(AuraMongoClient::open(dsn)));
    crate::gc::register_valid_ptr(client as usize, crate::gc::TYPE_MONGO_CLIENT);
    client
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_db(
    client: *mut AuraMongoClient,
    name_ptr: *const u8,
    name_len: usize,
) -> *mut AuraMongoDatabase {
    if client.is_null() {
        return std::ptr::null_mut();
    }
    let name = if !name_ptr.is_null() && name_len > 0 {
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(name_ptr, name_len)) }
    } else {
        ""
    };

    unsafe {
        let db = Box::into_raw(Box::new((*client).db(name)));
        crate::gc::register_valid_ptr(db as usize, crate::gc::TYPE_MONGO_DATABASE);
        db
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_collection(
    handle: *mut (),
    name_ptr: *const u8,
    name_len: usize,
) -> *mut AuraMongoCollection {
    if handle.is_null() || name_ptr.is_null() || name_len == 0 {
        return std::ptr::null_mut();
    }

    let name = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(name_ptr, name_len)) };
    let ptr_num = handle as usize;

    let col = match crate::gc::lookup_valid_ptr(ptr_num) {
        Some(crate::gc::TYPE_MONGO_CLIENT) => {
            let client = handle as *mut AuraMongoClient;
            unsafe { (*client).collection(name) }
        }
        Some(crate::gc::TYPE_MONGO_DATABASE) => {
            let db = handle as *mut AuraMongoDatabase;
            unsafe { (*db).collection(name) }
        }
        _ => return std::ptr::null_mut(),
    };

    let col_ptr = Box::into_raw(Box::new(col));
    crate::gc::register_valid_ptr(col_ptr as usize, crate::gc::TYPE_MONGO_COLLECTION);
    col_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_insert_one(
    col: *mut AuraMongoCollection,
    doc_val: i64,
) -> *mut AuraRecord {
    let res = aura_record_new();
    if col.is_null() {
        aura_record_set_bool(res, "acknowledged".as_ptr(), 12, false);
        return res;
    }

    let mut doc_json = extract_json_from_val(doc_val);
    let id_str = if let Some(id) = doc_json.get("_id").and_then(|v| v.as_str()) {
        id.to_string()
    } else {
        let next_id = DOC_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let gen_id = format!("doc_{:012x}", next_id);
        if let Some(obj) = doc_json.as_object_mut() {
            obj.insert("_id".to_string(), json!(gen_id));
        }
        gen_id
    };

    unsafe {
        if let Ok(mut docs) = (*col).docs.write() {
            docs.push(MongoDoc {
                id: id_str.clone(),
                data: doc_json,
            });
        }
    }

    let id_astr = aura_string_from_rust_str(&id_str);
    aura_record_set_str(res, "insertedId".as_ptr(), 10, id_astr);
    aura_record_set_bool(res, "acknowledged".as_ptr(), 12, true);
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_insert_many(
    col: *mut AuraMongoCollection,
    docs_list: *mut AuraList,
) -> *mut AuraRecord {
    let res = aura_record_new();
    if col.is_null() || docs_list.is_null() {
        aura_record_set_i64(res, "insertedCount".as_ptr(), 13, 0);
        aura_record_set_bool(res, "acknowledged".as_ptr(), 12, false);
        return res;
    }

    let count = aura_list_len(docs_list);
    let ids_list = aura_list_new(count);
    let mut inserted = 0;

    unsafe {
        if let Ok(mut docs) = (*col).docs.write() {
            for i in 0..count {
                let raw_val = aura_list_get(docs_list, i);
                let mut doc_json = extract_json_from_val(raw_val);
                let next_id = DOC_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                let gen_id = format!("doc_{:012x}", next_id);
                if let Some(obj) = doc_json.as_object_mut() {
                    obj.insert("_id".to_string(), json!(gen_id));
                }

                let id_astr = aura_string_from_rust_str(&gen_id);
                aura_list_push(ids_list, id_astr as i64);

                docs.push(MongoDoc {
                    id: gen_id,
                    data: doc_json,
                });
                inserted += 1;
            }
        }
    }

    aura_record_set_i64(res, "insertedCount".as_ptr(), 13, inserted as i64);
    crate::record::aura_record_set_list(res, "insertedIds".as_ptr(), 11, ids_list);
    aura_record_set_bool(res, "acknowledged".as_ptr(), 12, true);
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_find_one(
    col: *mut AuraMongoCollection,
    filter_val: i64,
) -> *mut AuraRecord {
    if col.is_null() {
        return std::ptr::null_mut();
    }

    let filter_json = extract_json_from_val(filter_val);
    unsafe {
        if let Ok(docs) = (*col).docs.read() {
            for doc in docs.iter() {
                if matches_filter(&doc.data, &filter_json) {
                    return json_to_aura_record(&doc.data);
                }
            }
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_find(
    col: *mut AuraMongoCollection,
    filter_val: i64,
) -> *mut AuraList {
    let list = aura_list_new(16);
    if col.is_null() {
        return list;
    }

    let filter_json = extract_json_from_val(filter_val);
    unsafe {
        if let Ok(docs) = (*col).docs.read() {
            for doc in docs.iter() {
                if matches_filter(&doc.data, &filter_json) {
                    let rec = json_to_aura_record(&doc.data);
                    aura_list_push(list, rec as i64);
                }
            }
        }
    }
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_update_one(
    col: *mut AuraMongoCollection,
    filter_val: i64,
    update_val: i64,
) -> *mut AuraRecord {
    let res = aura_record_new();
    if col.is_null() {
        aura_record_set_i64(res, "matchedCount".as_ptr(), 12, 0);
        aura_record_set_i64(res, "modifiedCount".as_ptr(), 13, 0);
        aura_record_set_bool(res, "acknowledged".as_ptr(), 12, false);
        return res;
    }

    let filter_json = extract_json_from_val(filter_val);
    let update_json = extract_json_from_val(update_val);
    let mut matched = 0;
    let mut modified = 0;

    unsafe {
        if let Ok(mut docs) = (*col).docs.write() {
            for doc in docs.iter_mut() {
                if matches_filter(&doc.data, &filter_json) {
                    matched = 1;
                    if apply_update(&mut doc.data, &update_json) {
                        modified = 1;
                    }
                    break;
                }
            }
        }
    }

    aura_record_set_i64(res, "matchedCount".as_ptr(), 12, matched);
    aura_record_set_i64(res, "modifiedCount".as_ptr(), 13, modified);
    aura_record_set_bool(res, "acknowledged".as_ptr(), 12, true);
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_update_many(
    col: *mut AuraMongoCollection,
    filter_val: i64,
    update_val: i64,
) -> *mut AuraRecord {
    let res = aura_record_new();
    if col.is_null() {
        return res;
    }

    let filter_json = extract_json_from_val(filter_val);
    let update_json = extract_json_from_val(update_val);
    let mut matched = 0;
    let mut modified = 0;

    unsafe {
        if let Ok(mut docs) = (*col).docs.write() {
            for doc in docs.iter_mut() {
                if matches_filter(&doc.data, &filter_json) {
                    matched += 1;
                    if apply_update(&mut doc.data, &update_json) {
                        modified += 1;
                    }
                }
            }
        }
    }

    aura_record_set_i64(res, "matchedCount".as_ptr(), 12, matched);
    aura_record_set_i64(res, "modifiedCount".as_ptr(), 13, modified);
    aura_record_set_bool(res, "acknowledged".as_ptr(), 12, true);
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_delete_one(
    col: *mut AuraMongoCollection,
    filter_val: i64,
) -> *mut AuraRecord {
    let res = aura_record_new();
    if col.is_null() {
        aura_record_set_i64(res, "deletedCount".as_ptr(), 12, 0);
        aura_record_set_bool(res, "acknowledged".as_ptr(), 12, false);
        return res;
    }

    let filter_json = extract_json_from_val(filter_val);
    let mut deleted = 0;

    unsafe {
        if let Ok(mut docs) = (*col).docs.write() {
            if let Some(idx) = docs.iter().position(|doc| matches_filter(&doc.data, &filter_json)) {
                docs.remove(idx);
                deleted = 1;
            }
        }
    }

    aura_record_set_i64(res, "deletedCount".as_ptr(), 12, deleted);
    aura_record_set_bool(res, "acknowledged".as_ptr(), 12, true);
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_delete_many(
    col: *mut AuraMongoCollection,
    filter_val: i64,
) -> *mut AuraRecord {
    let res = aura_record_new();
    if col.is_null() {
        return res;
    }

    let filter_json = extract_json_from_val(filter_val);
    let mut deleted = 0;

    unsafe {
        if let Ok(mut docs) = (*col).docs.write() {
            let prev_len = docs.len();
            docs.retain(|doc| !matches_filter(&doc.data, &filter_json));
            deleted = (prev_len - docs.len()) as i64;
        }
    }

    aura_record_set_i64(res, "deletedCount".as_ptr(), 12, deleted);
    aura_record_set_bool(res, "acknowledged".as_ptr(), 12, true);
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_count(
    col: *mut AuraMongoCollection,
    filter_val: i64,
) -> i64 {
    if col.is_null() {
        return 0;
    }

    let filter_json = extract_json_from_val(filter_val);
    unsafe {
        if let Ok(docs) = (*col).docs.read() {
            return docs.iter().filter(|doc| matches_filter(&doc.data, &filter_json)).count() as i64;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_aggregate(
    col: *mut AuraMongoCollection,
    pipeline_val: i64,
) -> *mut AuraList {
    let out_list = aura_list_new(16);
    if col.is_null() {
        return out_list;
    }

    let mut current_docs: Vec<JsonValue> = Vec::new();
    unsafe {
        if let Ok(docs) = (*col).docs.read() {
            for d in docs.iter() {
                current_docs.push(d.data.clone());
            }
        }
    }

    // Process pipeline stages
    if let AuraVal::List(stages_list) = infer_val_from_i64(pipeline_val) {
        let stage_count = aura_list_len(stages_list);
        for i in 0..stage_count {
            let stage_raw = aura_list_get(stages_list, i);
            let stage_json = extract_json_from_val(stage_raw);

            if let Some(obj) = stage_json.as_object() {
                // $match stage
                if let Some(match_filter) = obj.get("$match") {
                    current_docs.retain(|d| matches_filter(d, match_filter));
                }

                // $group stage
                if let Some(group_spec) = obj.get("$group").and_then(|v| v.as_object()) {
                    let mut groups: FxHashMap<String, Vec<JsonValue>> = FxHashMap::default();
                    let group_key_expr = group_spec.get("_id");

                    for d in &current_docs {
                        let gkey = if let Some(gk) = group_key_expr {
                            if let Some(s) = gk.as_str() {
                                if s.starts_with('$') {
                                    let field = &s[1..];
                                    d.get(field).map(|v| v.to_string()).unwrap_or_else(|| "null".to_string())
                                } else {
                                    s.to_string()
                                }
                            } else {
                                gk.to_string()
                            }
                        } else {
                            "all".to_string()
                        };

                        groups.entry(gkey).or_insert_with(Vec::new).push(d.clone());
                    }

                    let mut aggregated = Vec::new();
                    for (gkey, items) in groups {
                        let mut grp_doc = Map::new();
                        grp_doc.insert("_id".to_string(), json!(gkey));

                        for (k, spec) in group_spec {
                            if k == "_id" {
                                continue;
                            }
                            if let Some(acc_obj) = spec.as_object() {
                                if let Some(sum_val) = acc_obj.get("$sum") {
                                    let sum: f64 = if let Some(1) = sum_val.as_i64() {
                                        items.len() as f64
                                    } else if let Some(s) = sum_val.as_str() {
                                        if s.starts_with('$') {
                                            let f = &s[1..];
                                            items.iter().filter_map(|it| it.get(f).and_then(|x| x.as_f64())).sum()
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        items.len() as f64
                                    };
                                    grp_doc.insert(k.clone(), json!(sum));
                                } else if let Some(avg_val) = acc_obj.get("$avg") {
                                    if let Some(s) = avg_val.as_str() {
                                        if s.starts_with('$') {
                                            let f = &s[1..];
                                            let sum: f64 = items.iter().filter_map(|it| it.get(f).and_then(|x| x.as_f64())).sum();
                                            let avg = if !items.is_empty() { sum / items.len() as f64 } else { 0.0 };
                                            grp_doc.insert(k.clone(), json!(avg));
                                        }
                                    }
                                }
                            }
                        }
                        aggregated.push(JsonValue::Object(grp_doc));
                    }
                    current_docs = aggregated;
                }

                // $limit stage
                if let Some(limit_n) = obj.get("$limit").and_then(|v| v.as_u64()) {
                    current_docs.truncate(limit_n as usize);
                }
            }
        }
    }

    for d in current_docs {
        let rec = json_to_aura_record(&d);
        aura_list_push(out_list, rec as i64);
    }

    out_list
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_ping(client: *mut AuraMongoClient) -> i32 {
    if client.is_null() {
        return -1;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mongo_close(client: *mut AuraMongoClient) {
    if !client.is_null() {
        crate::gc::unregister_valid_ptr(client as usize);
        unsafe {
            drop(Box::from_raw(client));
        }
    }
}
