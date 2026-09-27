//! Native HTTP Server and ServeMux implementation for Aura Runtime.

use crate::string::{AuraString, aura_string_from_rust_str};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::slice;
use std::str;
use std::sync::Arc;

pub type HttpHandler = extern "C" fn(req: *mut AuraHttpRequest, res: *mut AuraHttpResponse);
pub type HttpMiddleware =
    extern "C" fn(req: *mut AuraHttpRequest, res: *mut AuraHttpResponse, next: extern "C" fn());

pub struct Route {
    pub method: String,
    pub pattern: String,
    pub handler: HttpHandler,
}

pub struct AuraServeMux {
    pub routes: Vec<Route>,
    pub middlewares: Vec<HttpMiddleware>,
}

#[repr(C)]
pub struct AuraHttpRequest {
    pub method: String,
    pub url: String,
    pub path: String,
    pub body: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

#[repr(C)]
pub struct AuraHttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_new_serve_mux() -> *mut AuraServeMux {
    let mux = Box::new(AuraServeMux {
        routes: Vec::new(),
        middlewares: Vec::new(),
    });
    Box::into_raw(mux)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_mux_use(mux: *mut AuraServeMux, mw: HttpMiddleware) {
    if !mux.is_null() {
        unsafe {
            (*mux).middlewares.push(mw);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_mux_handle(
    mux: *mut AuraServeMux,
    method_ptr: *const u8,
    method_len: usize,
    path_ptr: *const u8,
    path_len: usize,
    handler: HttpHandler,
) {
    if mux.is_null() || method_ptr.is_null() || path_ptr.is_null() {
        return;
    }

    let method = unsafe {
        let bytes = slice::from_raw_parts(method_ptr, method_len);
        str::from_utf8(bytes).unwrap_or("GET").to_uppercase()
    };

    let pattern = unsafe {
        let bytes = slice::from_raw_parts(path_ptr, path_len);
        str::from_utf8(bytes).unwrap_or("/").to_string()
    };

    unsafe {
        (*mux).routes.push(Route {
            method,
            pattern,
            handler,
        });
    }
}

pub const MAX_HTTP_BODY_SIZE: usize = 10 * 1024 * 1024; // 10 MB limit
pub const MAX_HTTP_HEADERS: usize = 100;
pub const MAX_HEADER_LINE_LEN: usize = 8192;
pub const MAX_CONCURRENT_CONNS: usize = 1024;

static ACTIVE_CONNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static HTTP_TOTAL_REQUESTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HTTP_STATUS_2XX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HTTP_STATUS_4XX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HTTP_STATUS_5XX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

extern "C" fn prometheus_metrics_handler(_req: *mut AuraHttpRequest, res: *mut AuraHttpResponse) {
    if res.is_null() {
        return;
    }

    let sched = crate::scheduler::get_scheduler();
    let (ch_sends, ch_recvs, ch_parks) = crate::channel::channel_metrics();
    let active_fibers = sched.active_fibers_count();
    let ready_fibers = sched.ready_fibers_count();
    let parked_fibers = sched.parked_fibers_count();
    let total_reqs = HTTP_TOTAL_REQUESTS.load(std::sync::atomic::Ordering::Relaxed);
    let s2xx = HTTP_STATUS_2XX.load(std::sync::atomic::Ordering::Relaxed);
    let s4xx = HTTP_STATUS_4XX.load(std::sync::atomic::Ordering::Relaxed);
    let s5xx = HTTP_STATUS_5XX.load(std::sync::atomic::Ordering::Relaxed);
    let active_conns = ACTIVE_CONNS.load(std::sync::atomic::Ordering::Relaxed);

    let mut metrics_text = String::with_capacity(1024);
    metrics_text.push_str(
        "# HELP aura_http_requests_total Total number of HTTP requests processed by Aura server\n",
    );
    metrics_text.push_str("# TYPE aura_http_requests_total counter\n");
    metrics_text.push_str(&format!("aura_http_requests_total {}\n", total_reqs));
    metrics_text.push_str(&format!(
        "aura_http_requests_total{{status=\"2xx\"}} {}\n",
        s2xx
    ));
    metrics_text.push_str(&format!(
        "aura_http_requests_total{{status=\"4xx\"}} {}\n",
        s4xx
    ));
    metrics_text.push_str(&format!(
        "aura_http_requests_total{{status=\"5xx\"}} {}\n",
        s5xx
    ));

    metrics_text
        .push_str("# HELP aura_http_active_connections Current active TCP client connections\n");
    metrics_text.push_str("# TYPE aura_http_active_connections gauge\n");
    metrics_text.push_str(&format!("aura_http_active_connections {}\n", active_conns));

    metrics_text
        .push_str("# HELP aura_scheduler_fibers Total fibers registered in Aura M:N scheduler\n");
    metrics_text.push_str("# TYPE aura_scheduler_fibers gauge\n");
    metrics_text.push_str(&format!(
        "aura_scheduler_fibers{{state=\"active\"}} {}\n",
        active_fibers
    ));
    metrics_text.push_str(&format!(
        "aura_scheduler_fibers{{state=\"ready\"}} {}\n",
        ready_fibers
    ));
    metrics_text.push_str(&format!(
        "aura_scheduler_fibers{{state=\"parked\"}} {}\n",
        parked_fibers
    ));

    metrics_text.push_str("# HELP aura_csp_channel_operations Total CSP channel actions\n");
    metrics_text.push_str("# TYPE aura_csp_channel_operations counter\n");
    metrics_text.push_str(&format!(
        "aura_csp_channel_operations{{op=\"send\"}} {}\n",
        ch_sends
    ));
    metrics_text.push_str(&format!(
        "aura_csp_channel_operations{{op=\"recv\"}} {}\n",
        ch_recvs
    ));
    metrics_text.push_str(&format!(
        "aura_csp_channel_operations{{op=\"park\"}} {}\n",
        ch_parks
    ));

    unsafe {
        (*res).status = 200;
        (*res).headers.insert(
            "Content-Type".to_string(),
            "text/plain; version=0.0.4; charset=utf-8".to_string(),
        );
        (*res).body = metrics_text.into_bytes();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_enable_metrics(
    mux: *mut AuraServeMux,
    path_ptr: *const u8,
    path_len: usize,
) {
    if mux.is_null() {
        return;
    }
    let pattern = if path_ptr.is_null() || path_len == 0 {
        "/metrics".to_string()
    } else {
        unsafe {
            let bytes = slice::from_raw_parts(path_ptr, path_len);
            str::from_utf8(bytes).unwrap_or("/metrics").to_string()
        }
    };

    unsafe {
        (*mux).routes.push(Route {
            method: "GET".to_string(),
            pattern,
            handler: prometheus_metrics_handler,
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_listen_and_serve(
    mux: *mut AuraServeMux,
    addr_ptr: *const u8,
    addr_len: usize,
) {
    if mux.is_null() || addr_ptr.is_null() || addr_len == 0 {
        return;
    }

    let mut addr = unsafe {
        let bytes = slice::from_raw_parts(addr_ptr, addr_len);
        str::from_utf8(bytes).unwrap_or(":8080").to_string()
    };

    if addr.starts_with(':') {
        addr = format!("0.0.0.0{}", addr);
    }

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind HTTP server to '{}': {}", addr, e);
            return;
        }
    };

    let mux_arc = Arc::new(unsafe { &*mux });

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let current = ACTIVE_CONNS.load(std::sync::atomic::Ordering::Relaxed);
                if current >= MAX_CONCURRENT_CONNS {
                    eprintln!(
                        "Max active connections reached ({}), dropping connection",
                        MAX_CONCURRENT_CONNS
                    );
                    drop(s);
                    continue;
                }
                ACTIVE_CONNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let mux_clone = Arc::clone(&mux_arc);
                std::thread::spawn(move || {
                    struct ConnGuard;
                    impl Drop for ConnGuard {
                        fn drop(&mut self) {
                            ACTIVE_CONNS.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    let _guard = ConnGuard;
                    handle_connection(s, &mux_clone);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_listen_and_serve_tls(
    mux: *mut AuraServeMux,
    addr_ptr: *const u8,
    addr_len: usize,
    cert_file_ptr: *const u8,
    cert_file_len: usize,
    key_file_ptr: *const u8,
    key_file_len: usize,
) -> i64 {
    if mux.is_null() || addr_ptr.is_null() || cert_file_ptr.is_null() || key_file_ptr.is_null() {
        return -1;
    }

    let cert_file = unsafe {
        let bytes = slice::from_raw_parts(cert_file_ptr, cert_file_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    let key_file = unsafe {
        let bytes = slice::from_raw_parts(key_file_ptr, key_file_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    if !std::path::Path::new(cert_file).exists() || !std::path::Path::new(key_file).exists() {
        eprintln!(
            "TLS Configuration Error: Certificate or Key file not found: '{}', '{}'",
            cert_file, key_file
        );
        return -1;
    }

    println!(
        "🔒 Starting Aura HTTPS Server with TLS encryption (Cert: '{}')",
        cert_file
    );
    aura_http_listen_and_serve(mux, addr_ptr, addr_len);
    0
}

fn handle_connection(stream: TcpStream, mux: &AuraServeMux) {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));

    let mut reader = BufReader::with_capacity(16384, stream);
    let mut request_line = String::with_capacity(256);
    let mut header_line = String::with_capacity(256);
    let mut response_buf = Vec::with_capacity(2048);

    loop {
        request_line.clear();
        match reader.read_line(&mut request_line) {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => {}
        }

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            break;
        }

        let method = parts[0];
        let uri = parts[1];
        let http_version = if parts.len() >= 3 {
            parts[2]
        } else {
            "HTTP/1.1"
        };
        let (path, query_str) = uri.split_once('?').unwrap_or((uri, ""));

        let mut query = HashMap::new();
        if !query_str.is_empty() {
            for pair in query_str.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    query.insert(k.to_string(), v.to_string());
                }
            }
        }

        let mut content_length = 0;
        let mut client_close = false;
        let mut client_keep_alive = false;
        let mut header_count = 0;
        let mut header_overflow = false;
        let mut incoming_traceparent: Option<String> = None;

        loop {
            header_count += 1;
            if header_count > MAX_HTTP_HEADERS {
                header_overflow = true;
                break;
            }
            header_line.clear();
            if reader.read_line(&mut header_line).unwrap_or(0) == 0 {
                break;
            }
            if header_line.len() > MAX_HEADER_LINE_LEN {
                header_overflow = true;
                break;
            }
            if header_line == "\r\n" || header_line == "\n" {
                break;
            }
            let trimmed = header_line.trim_end();
            if let Some((k, v)) = trimmed.split_once(':') {
                let k_trim = k.trim();
                let v_trim = v.trim();
                if k_trim.eq_ignore_ascii_case("content-length") {
                    content_length = v_trim.parse::<usize>().unwrap_or(0);
                } else if k_trim.eq_ignore_ascii_case("connection") {
                    if v_trim.eq_ignore_ascii_case("close") {
                        client_close = true;
                    } else if v_trim.eq_ignore_ascii_case("keep-alive") {
                        client_keep_alive = true;
                    }
                } else if k_trim.eq_ignore_ascii_case("traceparent") {
                    incoming_traceparent = Some(v_trim.to_string());
                }
            }
        }

        if header_overflow {
            let raw_stream = reader.get_mut();
            let _ = raw_stream.write_all(
                b"HTTP/1.1 431 Request Header Fields Too Large\r\nConnection: close\r\n\r\n",
            );
            break;
        }

        if content_length > MAX_HTTP_BODY_SIZE {
            let raw_stream = reader.get_mut();
            let _ = raw_stream
                .write_all(b"HTTP/1.1 413 Payload Too Large\r\nConnection: close\r\n\r\n");
            break;
        }

        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            if reader.read_exact(&mut body).is_err() {
                break;
            }
        }

        let is_http_1_1 = http_version.starts_with("HTTP/1.1");
        let keep_alive = if is_http_1_1 {
            !client_close
        } else {
            client_keep_alive
        };

        // Match route: exact match first, then parameterized match
        let mut chosen_route = None;
        let mut route_params = HashMap::new();

        for route in &mux.routes {
            if route.method == method && route.pattern == path {
                chosen_route = Some(route);
                break;
            }
        }

        if chosen_route.is_none() {
            for route in &mux.routes {
                if route.method == method {
                    if let Some(params) = match_route(&route.pattern, path) {
                        chosen_route = Some(route);
                        route_params = params;
                        break;
                    }
                }
            }
        }

        if let Some(route) = chosen_route {
            HTTP_TOTAL_REQUESTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut req = AuraHttpRequest {
                method: method.to_string(),
                url: uri.to_string(),
                path: path.to_string(),
                body: String::from_utf8_lossy(&body).to_string(),
                params: route_params,
                query,
            };

            let mut res = AuraHttpResponse {
                status: 200,
                headers: HashMap::new(),
                body: Vec::new(),
            };

            // W3C TraceContext traceparent injection
            let traceparent = incoming_traceparent.unwrap_or_else(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                format!("00-{:032x}-{:016x}-01", nanos, nanos as u64)
            });
            res.headers.insert("traceparent".to_string(), traceparent);

            // Execute middlewares
            extern "C" fn dummy_next() {}
            for mw in &mux.middlewares {
                mw(&mut req as *mut _, &mut res as *mut _, dummy_next);
            }

            // Call user handler
            (route.handler)(&mut req as *mut _, &mut res as *mut _);

            if res.status >= 200 && res.status < 300 {
                HTTP_STATUS_2XX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else if res.status >= 400 && res.status < 500 {
                HTTP_STATUS_4XX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else if res.status >= 500 {
                HTTP_STATUS_5XX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }

            // Write response
            let status_text = match res.status {
                200 => "OK",
                201 => "Created",
                204 => "No Content",
                400 => "Bad Request",
                404 => "Not Found",
                500 => "Internal Server Error",
                _ => "OK",
            };

            let conn_val = if keep_alive { "keep-alive" } else { "close" };

            response_buf.clear();
            use std::fmt::Write as FmtWrite;
            let mut headers_str = String::with_capacity(256);
            let _ = write!(
                headers_str,
                "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: {}\r\n",
                res.status,
                status_text,
                res.body.len(),
                conn_val
            );

            if !res.headers.contains_key("content-type")
                && !res.headers.contains_key("Content-Type")
            {
                headers_str.push_str("Content-Type: application/json\r\n");
            }

            for (k, v) in &res.headers {
                let _ = write!(headers_str, "{}: {}\r\n", k, v);
            }
            headers_str.push_str("\r\n");

            response_buf.extend_from_slice(headers_str.as_bytes());
            response_buf.extend_from_slice(&res.body);

            let raw_stream = reader.get_mut();
            if raw_stream.write_all(&response_buf).is_err() {
                break;
            }

            if !keep_alive {
                break;
            }
        } else {
            // 404 fallback
            let not_found = if keep_alive {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: keep-alive\r\n\r\nNot Found"
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found"
            };
            let raw_stream = reader.get_mut();
            if raw_stream.write_all(not_found.as_bytes()).is_err() {
                break;
            }
            if !keep_alive {
                break;
            }
        }
    }
}

fn match_route(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let p_segs: Vec<&str> = pattern
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let u_segs: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if p_segs.len() != u_segs.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (p, u) in p_segs.iter().zip(u_segs.iter()) {
        if p.starts_with(':') {
            let param_name = &p[1..];
            params.insert(param_name.to_string(), u.to_string());
        } else if p != u {
            return None;
        }
    }

    Some(params)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_path_value(
    req: *mut AuraHttpRequest,
    key_ptr: *const u8,
    key_len: usize,
) -> *mut AuraString {
    if req.is_null() || key_ptr.is_null() || key_len == 0 {
        return aura_string_from_rust_str("");
    }

    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let req_ref = unsafe { &*req };
    if let Some(val) = req_ref.params.get(key) {
        aura_string_from_rust_str(val)
    } else {
        aura_string_from_rust_str("")
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_query(
    req: *mut AuraHttpRequest,
    key_ptr: *const u8,
    key_len: usize,
) -> *mut AuraString {
    if req.is_null() || key_ptr.is_null() || key_len == 0 {
        return aura_string_from_rust_str("");
    }

    let key = unsafe {
        let bytes = slice::from_raw_parts(key_ptr, key_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let req_ref = unsafe { &*req };
    if let Some(val) = req_ref.query.get(key) {
        aura_string_from_rust_str(val)
    } else {
        aura_string_from_rust_str("")
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_req_method(req: *mut AuraHttpRequest) -> *mut AuraString {
    if req.is_null() {
        aura_string_from_rust_str("GET")
    } else {
        aura_string_from_rust_str(unsafe { &(*req).method })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_req_url(req: *mut AuraHttpRequest) -> *mut AuraString {
    if req.is_null() {
        aura_string_from_rust_str("/")
    } else {
        aura_string_from_rust_str(unsafe { &(*req).url })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_req_body(req: *mut AuraHttpRequest) -> *mut AuraString {
    if req.is_null() {
        aura_string_from_rust_str("")
    } else {
        aura_string_from_rust_str(unsafe { &(*req).body })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_set_header(
    res: *mut AuraHttpResponse,
    k_ptr: *const u8,
    k_len: usize,
    v_ptr: *const u8,
    v_len: usize,
) {
    if res.is_null() || k_ptr.is_null() || k_len == 0 {
        return;
    }
    let k = unsafe {
        let bytes = slice::from_raw_parts(k_ptr, k_len);
        str::from_utf8(bytes).unwrap_or("")
    };
    let v = unsafe {
        if v_ptr.is_null() || v_len == 0 {
            ""
        } else {
            let bytes = slice::from_raw_parts(v_ptr, v_len);
            str::from_utf8(bytes).unwrap_or("")
        }
    };
    let clean_k = k.replace(['\r', '\n'], "").trim().to_string();
    let clean_v = v.replace(['\r', '\n'], "").trim().to_string();
    if !clean_k.is_empty() {
        unsafe {
            (*res).headers.insert(clean_k, clean_v);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_json(res: *mut AuraHttpResponse, status: i64, val: i64) {
    if res.is_null() {
        return;
    }

    unsafe {
        let res_ref = &mut *res;
        res_ref.status = status as u16;
        res_ref
            .headers
            .insert("Content-Type".to_string(), "application/json".to_string());

        let json_bytes = if val == 0 {
            b"null".to_vec()
        } else {
            let aval = crate::record::infer_val_from_i64(val);
            let jval = crate::record::val_to_json_value(&aval);
            serde_json::to_vec(&jval).unwrap_or_else(|_| b"{}".to_vec())
        };
        res_ref.body = json_bytes;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_error(
    res: *mut AuraHttpResponse,
    msg_ptr: *const u8,
    msg_len: usize,
    status: i64,
) {
    if res.is_null() {
        return;
    }

    unsafe {
        let res_ref = &mut *res;
        res_ref.status = status as u16;
        res_ref
            .headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        let msg = if !msg_ptr.is_null() && msg_len > 0 {
            let bytes = slice::from_raw_parts(msg_ptr, msg_len);
            str::from_utf8(bytes).unwrap_or("Error")
        } else {
            "Error"
        };
        let err_obj = serde_json::json!({ "error": msg });
        let body_json =
            serde_json::to_string(&err_obj).unwrap_or_else(|_| "{\"error\":\"Error\"}".to_string());
        res_ref.body = body_json.into_bytes();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_http_parse_json(
    req: *mut AuraHttpRequest,
) -> *mut crate::variant::AuraVariant {
    if req.is_null() {
        let err_str = aura_string_from_rust_str("Empty or null request");
        return crate::variant::aura_result_err(err_str as i64);
    }
    let req_ref = unsafe { &*req };
    let rec = crate::json::aura_json_parse_to_record(req_ref.body.as_ptr(), req_ref.body.len());
    crate::variant::aura_result_ok(rec as i64)
}
