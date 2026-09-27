//! Native Standalone Runtime for Aura Language.
//! Provides M:N Fiber Scheduler, Garbage Collector (GC), CSP Channels, and C ABI.

pub mod c_api;
pub mod channel;
pub mod context;
pub mod fiber;
pub mod gc;
pub mod http;
pub mod json;
pub mod list;
pub mod os;
pub mod record;
pub mod scheduler;
pub mod string;
pub mod sync;
pub mod variant;

pub use c_api::*;
pub use channel::{AuraChannel, channel_metrics};
pub use gc::{collect_garbage, gc_alloc, gc_safepoint};
pub use http::{AuraHttpRequest, AuraHttpResponse, AuraServeMux};
pub use json::AuraJsonDoc;
pub use list::AuraList;
pub use record::{AuraRecord, AuraVal};
pub use scheduler::{get_scheduler, init_scheduler, yield_now};
pub use string::AuraString;
pub use sync::AuraMutex;
pub use variant::AuraVariant;

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

    #[test]
    fn test_fiber_execution_and_yielding() {
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
}
