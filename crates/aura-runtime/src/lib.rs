//! Native Standalone Runtime for Aura Language.
//! Provides M:N Fiber Scheduler, Garbage Collector (GC), CSP Channels, and C ABI.

// The C ABI functions exported by this runtime receive raw pointers across FFI from Cranelift native binaries.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod c_api;
pub mod channel;
pub mod context;
pub mod db;
pub mod fiber;
pub mod gc;
pub mod http;
pub mod json;
pub mod list;
pub mod mongo_client;
pub mod mysql_client;
pub mod os;
pub mod postgres_client;
pub mod record;
pub mod redis_client;
pub mod scheduler;
pub mod string;
pub mod sync;
pub mod variant;

pub use c_api::*;
pub use channel::{AuraChannel, channel_metrics};
pub use db::{
    AuraDbConn, AuraDbPool, AuraDbRows, AuraDbTx, AuraExecResult, aura_db_begin, aura_db_close_any,
    aura_db_exec, aura_db_exec_free, aura_db_exec_last_insert_id, aura_db_exec_rows_affected,
    aura_db_open, aura_db_ping, aura_db_pool_close, aura_db_query, aura_db_rows_close,
    aura_db_rows_col_count, aura_db_rows_col_name, aura_db_rows_get_bool, aura_db_rows_get_f64,
    aura_db_rows_get_i64, aura_db_rows_get_str, aura_db_rows_get_val, aura_db_rows_next,
    aura_db_rows_to_list, aura_db_rows_to_record, aura_db_tx_close, aura_db_tx_commit,
    aura_db_tx_rollback,
};
pub use gc::{collect_garbage, gc_alloc, gc_safepoint};
pub use http::{AuraHttpRequest, AuraHttpResponse, AuraServeMux};
pub use json::AuraJsonDoc;
pub use list::AuraList;
pub use mongo_client::{
    AuraMongoClient, AuraMongoCollection, AuraMongoDatabase, aura_mongo_aggregate,
    aura_mongo_close, aura_mongo_collection, aura_mongo_count, aura_mongo_db,
    aura_mongo_delete_many, aura_mongo_delete_one, aura_mongo_find, aura_mongo_find_one,
    aura_mongo_insert_many, aura_mongo_insert_one, aura_mongo_open, aura_mongo_ping,
    aura_mongo_update_many, aura_mongo_update_one,
};
pub use record::{AuraRecord, AuraVal};
pub use redis_client::{
    AuraRedisClient, aura_redis_close, aura_redis_decr, aura_redis_del, aura_redis_exists,
    aura_redis_expire, aura_redis_get, aura_redis_hdel, aura_redis_hget, aura_redis_hgetall,
    aura_redis_hset, aura_redis_incr, aura_redis_lpush, aura_redis_lrange, aura_redis_open,
    aura_redis_ping, aura_redis_rpop, aura_redis_set, aura_redis_ttl,
};
pub use scheduler::{get_scheduler, init_scheduler, yield_now};
pub use string::AuraString;
pub use sync::AuraMutex;
pub use variant::AuraVariant;

#[unsafe(no_mangle)]
pub extern "C" fn aura_universal_get(target: *mut (), arg: i64) -> i64 {
    if target.is_null() {
        return 0;
    }
    let target_type = crate::gc::lookup_valid_ptr(target as usize);
    match target_type {
        Some(crate::gc::TYPE_REDIS_CLIENT) => {
            let key_opt =
                if let Some(crate::gc::TYPE_STRING) = crate::gc::lookup_valid_ptr(arg as usize) {
                    let astr = arg as *mut crate::string::AuraString;
                    unsafe { Some((*astr).as_str().to_string()) }
                } else {
                    None
                };
            if let Some(key_str) = key_opt {
                let client = target as *mut crate::redis_client::AuraRedisClient;
                unsafe {
                    match (*client).get(&key_str) {
                        Some(s) => crate::string::aura_string_from_rust_str(&s) as i64,
                        None => 0,
                    }
                }
            } else {
                0
            }
        }
        Some(crate::gc::TYPE_DB_ROWS) => {
            crate::db::aura_db_rows_get_val(target as *mut crate::db::AuraDbRows, arg as i32)
        }
        Some(crate::gc::TYPE_RECORD) => {
            if let Some(crate::gc::TYPE_STRING) = crate::gc::lookup_valid_ptr(arg as usize) {
                let astr = arg as *mut crate::string::AuraString;
                unsafe {
                    crate::record::aura_record_get(
                        target as *mut crate::record::AuraRecord,
                        (*astr).ptr,
                        (*astr).len,
                    )
                }
            } else {
                0
            }
        }
        Some(crate::gc::TYPE_LIST) => {
            crate::list::aura_list_get(target as *mut crate::list::AuraList, arg as usize)
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

    #[test]
    fn test_gc_alloc_and_collect() {
        let ptr1 = gc_alloc(64, 1);
        assert!(!ptr1.is_null());
        unsafe {
            std::ptr::write(ptr1 as *mut i64, 42);
            assert_eq!(*(ptr1 as *const i64), 42);
        }

        let ptr2 = gc_alloc(128, 2);
        assert!(!ptr2.is_null());

        collect_garbage();
    }

    static TEST_SCHEDULER_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_fiber_execution_and_yielding() {
        let _guard = TEST_SCHEDULER_MUTEX.lock().unwrap();
        init_scheduler();
        let sched = get_scheduler();

        static COUNTER: AtomicI64 = AtomicI64::new(0);

        extern "C" fn worker_fiber(_arg: *mut ()) {
            for _ in 0..5 {
                COUNTER.fetch_add(1, Ordering::SeqCst);
                yield_now();
            }
        }

        sched.spawn(worker_fiber, std::ptr::null_mut());
        sched.spawn(worker_fiber, std::ptr::null_mut());

        sched.run_workers(1);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_csp_channel_handoff_between_fibers() {
        let _guard = TEST_SCHEDULER_MUTEX.lock().unwrap();
        init_scheduler();
        let sched = get_scheduler();

        let chan = Arc::new(AuraChannel::new(0)); // Unbuffered rendezvous channel
        let chan_clone = Arc::clone(&chan);

        static RESULT: AtomicI64 = AtomicI64::new(0);

        let raw_chan1 = Arc::into_raw(chan) as *mut ();
        let raw_chan2 = Arc::into_raw(chan_clone) as *mut ();

        extern "C" fn sender(arg: *mut ()) {
            let chan = unsafe { &*(arg as *const AuraChannel) };
            chan.send(42 as *mut ());
        }

        extern "C" fn receiver(arg: *mut ()) {
            let chan = unsafe { &*(arg as *const AuraChannel) };
            let val = chan.recv();
            RESULT.store(val as i64, Ordering::SeqCst);
        }

        sched.spawn(sender, raw_chan1);
        sched.spawn(receiver, raw_chan2);

        sched.run_workers(1);

        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_mutex_lock_and_unlock() {
        let m = sync::aura_mutex_new();
        sync::aura_mutex_lock(m);
        sync::aura_mutex_unlock(m);
        sync::aura_mutex_free(m);
    }

    #[test]
    fn test_list_operations() {
        let list = list::aura_list_new(2);
        assert_eq!(list::aura_list_len(list), 0);

        list::aura_list_push(list, 100);
        list::aura_list_push(list, 200);
        list::aura_list_push(list, 300);

        assert_eq!(list::aura_list_len(list), 3);
        assert_eq!(list::aura_list_get(list, 0), 100);
        assert_eq!(list::aura_list_get(list, 1), 200);
        assert_eq!(list::aura_list_get(list, 2), 300);

        list::aura_list_set(list, 1, 999);
        assert_eq!(list::aura_list_get(list, 1), 999);

        list::aura_list_free(list);
    }

    #[test]
    fn test_compact_byte_list_operations() {
        let byte_list = list::aura_list_new_byte(4);
        assert_eq!(list::aura_list_len(byte_list), 0);
        assert_eq!(list::aura_list_elem_size(byte_list), 1);

        list::aura_list_push(byte_list, 42);
        list::aura_list_push(byte_list, 255);
        list::aura_list_push(byte_list, 1);

        assert_eq!(list::aura_list_len(byte_list), 3);
        assert_eq!(list::aura_list_get(byte_list, 0), 42);
        assert_eq!(list::aura_list_get(byte_list, 1), 255);
        assert_eq!(list::aura_list_get(byte_list, 2), 1);

        list::aura_list_set(byte_list, 1, 0);
        assert_eq!(list::aura_list_get(byte_list, 1), 0);

        let repeat_byte = list::aura_list_repeat_byte(1, 100);
        assert_eq!(list::aura_list_len(repeat_byte), 100);
        assert_eq!(list::aura_list_elem_size(repeat_byte), 1);
        for i in 0..100 {
            assert_eq!(list::aura_list_get(repeat_byte, i), 1);
        }
        list::aura_list_set(repeat_byte, 50, 0);
        assert_eq!(list::aura_list_get(repeat_byte, 50), 0);

        list::aura_list_free(byte_list);
        list::aura_list_free(repeat_byte);
    }

    #[test]
    fn test_json_parsing_and_stringify() {
        let json_text = r#"{"id":"ord-101","total":227.29,"active":true}"#;
        let doc = json::aura_json_parse(json_text.as_ptr(), json_text.len());

        let id_str = json::aura_json_get_str(doc, "id".as_ptr(), 2);
        assert_eq!(unsafe { (*id_str).as_str() }, "ord-101");

        let serialized = json::aura_json_stringify(doc);
        let s = unsafe { (*serialized).as_str() };
        assert!(s.contains("ord-101"));
    }

    #[test]
    fn test_http_serve_mux_registration() {
        let mux = http::aura_http_new_serve_mux();

        extern "C" fn dummy_handler(_req: *mut AuraHttpRequest, _res: *mut AuraHttpResponse) {}

        let path = "/api/orders/:id";
        http::aura_http_mux_handle(
            mux,
            "GET".as_ptr(),
            3,
            path.as_ptr(),
            path.len(),
            dummy_handler,
        );

        let mux_ref = unsafe { &*mux };
        assert_eq!(mux_ref.routes.len(), 1);
        assert_eq!(mux_ref.routes[0].method, "GET");
        assert_eq!(mux_ref.routes[0].pattern, "/api/orders/:id");
    }

    #[test]
    fn test_record_json_serialization() {
        let rec = record::aura_record_new();
        let key_id = "id";
        let key_total = "total";
        let id_val = string::aura_string_from_rust_str("ord-202");
        record::aura_record_set_str(rec, key_id.as_ptr(), key_id.len(), id_val);
        record::aura_record_set_f64(rec, key_total.as_ptr(), key_total.len(), 99.95);

        let json_str_ptr = record::aura_record_to_json(rec);
        let s = unsafe { (*json_str_ptr).as_str() };
        assert!(s.contains("\"id\":\"ord-202\""));
        assert!(s.contains("\"total\":99.95"));
    }

    #[test]
    fn test_variant_option_and_result() {
        let ok_res = variant::aura_result_ok(42);
        assert_eq!(variant::aura_variant_tag(ok_res), 1);
        assert_eq!(variant::aura_variant_val(ok_res), 42);

        let err_res = variant::aura_result_err(99);
        assert_eq!(variant::aura_variant_tag(err_res), 2);
        assert_eq!(variant::aura_variant_val(err_res), 99);
    }

    #[test]
    fn test_security_arbitrary_int_not_dereferenced() {
        let test_vals = [
            0x10008i64,
            0x20000i64,
            1_048_576i64,
            1_726_000_000i64,
            0x7fff_0000_0008i64,
        ];

        for &val in &test_vals {
            let aval = record::infer_val_from_i64(val);
            match aval {
                record::AuraVal::Int(i) => assert_eq!(i, val),
                other => panic!("Expected AuraVal::Int for {}, got {:?}", val, other),
            }
        }
    }

    #[test]
    fn test_security_http_error_json_escaping() {
        let res = Box::into_raw(Box::new(http::AuraHttpResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }));

        let malicious_msg = "test\", \"admin\": true, \"inject\": \"";
        http::aura_http_error(res, malicious_msg.as_ptr(), malicious_msg.len(), 400);

        let res_ref = unsafe { &*res };
        let body_str = std::str::from_utf8(&res_ref.body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(body_str).expect("Must be valid JSON");
        assert_eq!(parsed["error"], malicious_msg);
        assert!(parsed.get("admin").is_none());

        unsafe {
            drop(Box::from_raw(res));
        }
    }

    #[test]
    fn test_security_crlf_header_injection_prevention() {
        let res = Box::into_raw(Box::new(http::AuraHttpResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }));

        let k = "X-Injected\r\nSet-Cookie: evil=true";
        let v = "safe_val\r\nEvil-Header: true";
        http::aura_http_set_header(res, k.as_ptr(), k.len(), v.as_ptr(), v.len());

        let res_ref = unsafe { &*res };
        for (header_k, header_v) in &res_ref.headers {
            assert!(!header_k.contains('\r') && !header_k.contains('\n'));
            assert!(!header_v.contains('\r') && !header_v.contains('\n'));
        }

        unsafe {
            drop(Box::from_raw(res));
        }
    }

    #[test]
    fn test_security_channel_send_closed_no_panic() {
        let chan = channel::AuraChannel::new(1);
        chan.close();
        chan.send(123 as *mut ());
        assert!(chan.recv().is_null());
    }

    #[test]
    fn test_native_sqlite_db_lifecycle_and_crud() {
        let driver = "sqlite";
        let dsn = ":memory:";
        let pool = aura_db_open(driver.as_ptr(), driver.len(), dsn.as_ptr(), dsn.len(), 5);
        assert!(!pool.is_null());

        // Ping test
        assert_eq!(aura_db_ping(pool), 0);

        // CREATE TABLE
        let ddl = "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, balance REAL NOT NULL, active INTEGER DEFAULT 1);";
        let exec_res = aura_db_exec(
            pool as *mut (),
            ddl.as_ptr(),
            ddl.len(),
            std::ptr::null_mut(),
        );
        assert!(!exec_res.is_null());
        assert_eq!(aura_db_exec_rows_affected(exec_res), 0);
        aura_db_exec_free(exec_res);

        // INSERT Alice
        let insert_sql = "INSERT INTO users (name, balance, active) VALUES (?, ?, ?);";
        let params = list::aura_list_new(3);
        let alice_str = string::aura_string_from_rust_str("Alice");
        list::aura_list_push(params, alice_str as i64);
        list::aura_list_push(params, 1500 as i64); // balance
        list::aura_list_push(params, 1 as i64); // active = true

        let insert_res = aura_db_exec(
            pool as *mut (),
            insert_sql.as_ptr(),
            insert_sql.len(),
            params,
        );
        assert!(!insert_res.is_null());
        assert_eq!(aura_db_exec_rows_affected(insert_res), 1);
        assert_eq!(aura_db_exec_last_insert_id(insert_res), 1);
        aura_db_exec_free(insert_res);

        // INSERT Bob
        let params_bob = list::aura_list_new(3);
        let bob_str = string::aura_string_from_rust_str("Bob");
        list::aura_list_push(params_bob, bob_str as i64);
        list::aura_list_push(params_bob, 2500 as i64);
        list::aura_list_push(params_bob, 1 as i64);

        let insert_bob_res = aura_db_exec(
            pool as *mut (),
            insert_sql.as_ptr(),
            insert_sql.len(),
            params_bob,
        );
        assert_eq!(aura_db_exec_rows_affected(insert_bob_res), 1);
        assert_eq!(aura_db_exec_last_insert_id(insert_bob_res), 2);
        aura_db_exec_free(insert_bob_res);

        // SELECT Query
        let select_sql = "SELECT id, name, balance, active FROM users ORDER BY id ASC;";
        let rows = aura_db_query(
            pool as *mut (),
            select_sql.as_ptr(),
            select_sql.len(),
            std::ptr::null_mut(),
        );
        assert!(!rows.is_null());
        assert_eq!(aura_db_rows_col_count(rows), 4);

        // Row 1 (Alice)
        assert_eq!(aura_db_rows_next(rows), 1);
        let id1 = aura_db_rows_get_i64(rows, 0);
        assert_eq!(id1, 1);
        let name1 = aura_db_rows_get_str(rows, 1);
        assert_eq!(unsafe { (*name1).as_str() }, "Alice");

        // Convert Row 1 to AuraRecord
        let rec1 = aura_db_rows_to_record(rows);
        assert!(!rec1.is_null());
        let rec_name = record::aura_record_get(rec1, "name".as_ptr(), 4);
        assert_eq!(
            unsafe { (*(rec_name as *mut string::AuraString)).as_str() },
            "Alice"
        );

        // Row 2 (Bob)
        assert_eq!(aura_db_rows_next(rows), 1);
        let id2 = aura_db_rows_get_i64(rows, 0);
        assert_eq!(id2, 2);
        let name2 = aura_db_rows_get_str(rows, 1);
        assert_eq!(unsafe { (*name2).as_str() }, "Bob");

        // End of rows
        assert_eq!(aura_db_rows_next(rows), 0);
        aura_db_rows_close(rows);

        // Transaction Rollback test
        let tx = aura_db_begin(pool);
        assert!(!tx.is_null());

        let insert_temp = "INSERT INTO users (name, balance) VALUES ('TempUser', 100);";
        let tx_exec = aura_db_exec(
            tx as *mut (),
            insert_temp.as_ptr(),
            insert_temp.len(),
            std::ptr::null_mut(),
        );
        aura_db_exec_free(tx_exec);

        assert_eq!(aura_db_tx_rollback(tx), 0);

        aura_db_pool_close(pool);
    }

    #[test]
    fn test_native_redis_client_and_cache() {
        let client = aura_redis_open(std::ptr::null(), 0);
        assert!(!client.is_null());

        // SET & GET
        let k = "session:user:1";
        let v = "token_abc_123";
        assert_eq!(
            aura_redis_set(client, k.as_ptr(), k.len(), v.as_ptr(), v.len(), 3600),
            1
        );
        assert_eq!(aura_redis_exists(client, k.as_ptr(), k.len()), 1);

        let get_val = aura_redis_get(client, k.as_ptr(), k.len());
        assert!(!get_val.is_null());
        assert_eq!(unsafe { (*get_val).as_str() }, v);

        // INCR / DECR
        let cnt_k = "pageviews";
        assert_eq!(aura_redis_incr(client, cnt_k.as_ptr(), cnt_k.len()), 1);
        assert_eq!(aura_redis_incr(client, cnt_k.as_ptr(), cnt_k.len()), 2);
        assert_eq!(aura_redis_decr(client, cnt_k.as_ptr(), cnt_k.len()), 1);

        // HSET & HGET & HGETALL
        let h_k = "user:profile:1";
        let f1 = "theme";
        let v1 = "dark";
        assert_eq!(
            aura_redis_hset(
                client,
                h_k.as_ptr(),
                h_k.len(),
                f1.as_ptr(),
                f1.len(),
                v1.as_ptr(),
                v1.len()
            ),
            1
        );

        let h_got = aura_redis_hget(client, h_k.as_ptr(), h_k.len(), f1.as_ptr(), f1.len());
        assert!(!h_got.is_null());
        assert_eq!(unsafe { (*h_got).as_str() }, "dark");

        let h_all = aura_redis_hgetall(client, h_k.as_ptr(), h_k.len());
        assert!(!h_all.is_null());
        let theme_str = record::aura_record_get(h_all, f1.as_ptr(), f1.len());
        assert_eq!(
            unsafe { (*(theme_str as *mut string::AuraString)).as_str() },
            "dark"
        );

        // LIST: LPUSH, RPOP, LRANGE
        let list_k = "tasks_queue";
        let item1 = "task_alpha";
        let item2 = "task_beta";
        aura_redis_lpush(
            client,
            list_k.as_ptr(),
            list_k.len(),
            item1.as_ptr(),
            item1.len(),
        );
        aura_redis_lpush(
            client,
            list_k.as_ptr(),
            list_k.len(),
            item2.as_ptr(),
            item2.len(),
        );

        let l_range = aura_redis_lrange(client, list_k.as_ptr(), list_k.len(), 0, -1);
        assert_eq!(list::aura_list_len(l_range), 2);

        let r_pop = aura_redis_rpop(client, list_k.as_ptr(), list_k.len());
        assert!(!r_pop.is_null());
        assert_eq!(unsafe { (*r_pop).as_str() }, "task_alpha");

        // PING
        let ping = aura_redis_ping(client);
        assert_eq!(unsafe { (*ping).as_str() }, "PONG");

        // DEL
        assert_eq!(aura_redis_del(client, k.as_ptr(), k.len()), 1);
        assert_eq!(aura_redis_exists(client, k.as_ptr(), k.len()), 0);

        aura_redis_close(client);
    }

    #[test]
    fn test_native_mongo_document_crud_and_aggregate() {
        let client = aura_mongo_open(std::ptr::null(), 0);
        assert!(!client.is_null());

        let col_name = "users";
        let col = aura_mongo_collection(client as *mut (), col_name.as_ptr(), col_name.len());
        assert!(!col.is_null());

        // Insert Alice
        let alice_rec = record::aura_record_new();
        let alice_name = string::aura_string_from_rust_str("Alice");
        let alice_role = string::aura_string_from_rust_str("admin");
        record::aura_record_set_str(alice_rec, "name".as_ptr(), 4, alice_name);
        record::aura_record_set_str(alice_rec, "role".as_ptr(), 4, alice_role);
        record::aura_record_set_i64(alice_rec, "score".as_ptr(), 5, 95);

        let ins_res = aura_mongo_insert_one(col, alice_rec as i64);
        assert!(!ins_res.is_null());
        let ins_id = record::aura_record_get(ins_res, "insertedId".as_ptr(), 10);
        assert_ne!(ins_id, 0);

        // Insert Bob
        let bob_rec = record::aura_record_new();
        let bob_name = string::aura_string_from_rust_str("Bob");
        let bob_role = string::aura_string_from_rust_str("developer");
        record::aura_record_set_str(bob_rec, "name".as_ptr(), 4, bob_name);
        record::aura_record_set_str(bob_rec, "role".as_ptr(), 4, bob_role);
        record::aura_record_set_i64(bob_rec, "score".as_ptr(), 5, 88);
        aura_mongo_insert_one(col, bob_rec as i64);

        // Count
        let filter_all = record::aura_record_new();
        assert_eq!(aura_mongo_count(col, filter_all as i64), 2);

        // FindOne
        let filter_alice = record::aura_record_new();
        let query_name = string::aura_string_from_rust_str("Alice");
        record::aura_record_set_str(filter_alice, "name".as_ptr(), 4, query_name);

        let found_alice = aura_mongo_find_one(col, filter_alice as i64);
        assert!(!found_alice.is_null());
        let got_role = record::aura_record_get(found_alice, "role".as_ptr(), 4);
        assert_eq!(
            unsafe { (*(got_role as *mut string::AuraString)).as_str() },
            "admin"
        );

        // UpdateOne
        let update_doc = record::aura_record_new();
        let set_doc = record::aura_record_new();
        record::aura_record_set_i64(set_doc, "score".as_ptr(), 5, 100);
        record::aura_record_set_record(update_doc, "$set".as_ptr(), 4, set_doc);

        let upd_res = aura_mongo_update_one(col, filter_alice as i64, update_doc as i64);
        assert!(!upd_res.is_null());

        // Aggregate Group by Role
        let pipeline = list::aura_list_new(2);
        let group_stage = record::aura_record_new();
        let group_spec = record::aura_record_new();
        let id_field = string::aura_string_from_rust_str("$role");
        record::aura_record_set_str(group_spec, "_id".as_ptr(), 3, id_field);
        record::aura_record_set_i64(group_spec, "total".as_ptr(), 5, 1);
        record::aura_record_set_record(group_stage, "$group".as_ptr(), 6, group_spec);
        list::aura_list_push(pipeline, group_stage as i64);

        let agg_res = aura_mongo_aggregate(col, pipeline as i64);
        assert!(list::aura_list_len(agg_res) >= 1);

        // DeleteOne
        let del_res = aura_mongo_delete_one(col, filter_alice as i64);
        assert!(!del_res.is_null());
        assert_eq!(aura_mongo_count(col, filter_all as i64), 1);

        aura_mongo_close(client);
    }

    #[test]
    fn test_tlab_allocation_and_binary_search_gc() {
        use crate::gc;

        let before_collections = gc::get_collections_count();
        // Allocate across TLAB batches (more than 64 items to trigger flush)
        let mut ptrs = Vec::new();
        for _ in 0..120 {
            let p = gc::gc_alloc(64, gc::TYPE_RECORD);
            assert!(!p.is_null());
            ptrs.push(p);
        }

        // Flush remaining in thread-local buffer
        gc::flush_tlab();

        // Run garbage collection; since ptrs are not in stack scan range or global roots,
        // they should be safely swept with binary search
        gc::collect_garbage();
        assert_eq!(gc::get_collections_count(), before_collections + 1);
    }

    #[test]
    fn test_fiber_stack_canary_protection() {
        use crate::context::{DEFAULT_STACK_SIZE, FiberStack, STACK_CANARY_MAGIC};

        let stack = FiberStack::new(DEFAULT_STACK_SIZE);
        assert!(stack.verify_canary(), "Canary must be valid upon creation");

        // Intentionally tamper with the canary
        unsafe {
            *(stack.bottom() as *mut usize) = 0x1234_5678;
        }
        assert!(!stack.verify_canary(), "Tampered canary must be detected");

        // Restore canary for clean drop
        unsafe {
            *(stack.bottom() as *mut usize) = STACK_CANARY_MAGIC;
        }
        assert!(stack.verify_canary());
    }

    #[test]
    fn test_json_max_depth_protection() {
        use crate::json::{aura_json_free, aura_json_parse};

        // 1. Normal JSON (shallow depth)
        let normal_json = b"{\"a\": {\"b\": {\"c\": 123}}}";
        let doc1 = aura_json_parse(normal_json.as_ptr(), normal_json.len());
        assert!(!doc1.is_null());
        assert!(!unsafe { (*doc1).value.is_null() });
        aura_json_free(doc1);

        // 2. Overly deep JSON (70 nested arrays > 64)
        let mut deep_json = Vec::new();
        for _ in 0..70 {
            deep_json.push(b'[');
        }
        deep_json.push(b'1');
        for _ in 0..70 {
            deep_json.push(b']');
        }

        let doc2 = aura_json_parse(deep_json.as_ptr(), deep_json.len());
        assert!(!doc2.is_null());
        assert!(
            unsafe { (*doc2).value.is_null() },
            "Excessive depth must be rejected with Null to prevent Stack Overflow"
        );
        aura_json_free(doc2);
    }

    #[test]
    fn test_postgres_scram_sha256_cryptography() {
        use crate::postgres_client::{base64_decode, base64_encode, hmac_sha256, pbkdf2_hmac_sha256, sha256_hash};

        // 1. Test Base64 encode/decode
        let sample = b"hello aura postgres";
        let b64 = base64_encode(sample);
        let decoded = base64_decode(&b64).expect("Base64 decode must succeed");
        assert_eq!(decoded, sample);

        // 2. Test SHA-256 standard NIST vector for "abc"
        // ba7816bf 8f01cfea 414140de 5dae2223 b00361a3 96177a9c b410ff61 f20015ad
        let hash = sha256_hash(b"abc");
        let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");

        // 3. Test HMAC-SHA256
        let hmac = hmac_sha256(b"key", b"The quick brown fox jumps over the lazy dog");
        let hmac_hex: String = hmac.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hmac_hex, "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8");

        // 4. Test RFC 7677 SCRAM-SHA-256 vector
        // password="pencil", salt="W22ZaJ0SNY7soNVgvFoSnQ==", iterations=4096
        let rfc_salt = base64_decode("W22ZaJ0SNY7soNVgvFoSnQ==").unwrap();
        let salted_pw = pbkdf2_hmac_sha256(b"pencil", &rfc_salt, 4096);
        let client_key = hmac_sha256(&salted_pw, b"Client Key");
        let stored_key = sha256_hash(&client_key);
        let stored_key_hex: String = stored_key.iter().map(|b| format!("{:02x}", b)).collect();
        // RFC 7677 Section 3: StoredKey matches exact SHA-256 PBKDF2 derivative
        assert_eq!(stored_key_hex, "a07156c3493aec5e8b30542f283009d2c210b21ada2ca3f061dfd440162cea2c");
    }
}
