use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_http_server_typecheck_and_codegen() {
    let source = r#"
        let mux = http.newServeMux();
        mux.use(fn(req: Any, res: Any, next: Any) => {
            next();
        });
        mux.get("/users/:id", fn(req: Any, res: Any) => {
            let id = http.pathValue(req, "id");
            let q = http.query(req, "search");
            if (id == "0") {
                http.error(res, "Not found", http.StatusNotFound);
                return ();
            }
            http.json(res, http.StatusOK, { id: id, q: q });
        });
        mux.post("/data", fn(req: Any, res: Any) => async {
            let body = await http.parseJson(req);
            match body {
                Ok(data) => {
                    http.json(res, http.StatusCreated, data);
                },
                Err(e) => {
                    http.error(res, e, http.StatusBadRequest);
                }
            }
        });
    "#;

    let res = compile(source, &[]).expect("Compilation of HTTP server should succeed");

    assert!(res.js_code.contains("class __AuraServeMux"));
    assert!(res.js_code.contains("http.newServeMux()"));
    assert!(res.js_code.contains("http.StatusOK"));
    assert!(res.js_code.contains("http.StatusCreated"));
    assert!(res.js_code.contains("http.StatusNotFound"));
    assert!(res.js_code.contains("http.pathValue"));
    assert!(res.js_code.contains("http.parseJson"));
    assert!(res.dts_code.contains("export interface ServeMux"));
    assert!(res.dts_code.contains("export declare const http:"));
}

#[test]
fn test_e2e_http_server_and_serve_mux_execution() {
    let aura_source = r#"
        export async fn main(): Task<Unit, String> {
            let mux = http.newServeMux();

            // Middleware
            mux.use(fn(req: Any, res: Any, next: Any) => {
                res.setHeader("X-Custom-Header", "Aura-Middleware-OK");
                next();
            });

            // Route with path param
            mux.get("/items/:itemId", fn(req: Any, res: Any) => {
                let id = http.pathValue(req, "itemId");
                let mode = http.query(req, "mode");
                http.json(res, http.StatusOK, {
                    item: id,
                    mode: mode
                });
            });

            // POST route with body
            mux.post("/items", fn(req: Any, res: Any) => async {
                let parsed = await http.parseJson(req);
                match parsed {
                    Ok(data) => {
                        http.json(res, http.StatusCreated, {
                            success: true,
                            received: data
                        });
                    },
                    Err(err) => {
                        http.error(res, err, http.StatusBadRequest);
                    }
                }
            });

            // HTML endpoint
            mux.get("/welcome", fn(req: Any, res: Any) => {
                http.html(res, http.StatusOK, "<h1>Hello Aura</h1>");
            });

            // Start server in background
            spawn(async {
                await mux.listenAndServe(":9191");
            });

            // Wait for server to start listening
            await sleep(100);

            // 1. Test GET /items/42?mode=fast
            let getRes = await fetch("http://127.0.0.1:9191/items/42?mode=fast");
            assert(getRes.status == 200, "GET /items/42 status should be 200");
            assert(getRes.headers.get("x-custom-header") == "Aura-Middleware-OK", "Custom header missing");
            let getData = await getRes.json();
            assert(getData.item == "42", "Item ID should match");
            assert(getData.mode == "fast", "Query param should match");

            // 2. Test POST /items with JSON
            let postRes = await fetch("http://127.0.0.1:9191/items", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ name: "Widget", price: 99 })
            });
            assert(postRes.status == 201, "POST /items status should be 201");
            let postData = await postRes.json();
            assert(postData.success == true, "POST success flag should be true");
            assert(postData.received.name == "Widget", "Received body mismatch");

            // 3. Test 404 Not Found
            let notFoundRes = await fetch("http://127.0.0.1:9191/nonexistent");
            assert(notFoundRes.status == 404, "Unknown route should 404");

            // 4. Test 405 Method Not Allowed (POST to GET /welcome)
            let methodRes = await fetch("http://127.0.0.1:9191/welcome", {
                method: "POST"
            });
            assert(methodRes.status == 405, "Method mismatch should 405");

            // Close server
            await mux.close();
            println("ALL_HTTP_TESTS_PASSED");
        }
    "#;

    let res = compile(aura_source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nawait main();\n");

    let tmp_file = "test_http_exec.tmp.mjs";
    fs::write(tmp_file, &js_runner).expect("Failed to write tmp runner");

    let output = Command::new("node")
        .arg(tmp_file)
        .output()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("ALL_HTTP_TESTS_PASSED"),
        "Test failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}
