use aura_lang::{compile, compile_to_go};
use std::fs;
use std::process::Command;

#[test]
fn test_native_http_package_typecheck_and_imports() {
    let source = r#"
        import { Server, Request, Response, ResponseWriter, HttpRequest, HttpResponse, StatusOK, StatusCreated } from "net/http";

        type User = {
            id: String,
            name: String,
        }

        let server = Server.new(":8080");

        // Handler with ResponseWriter and Request
        server.get("/users/:id", fn(req: Request, res: ResponseWriter): Unit => {
            let id = req.pathValue("id");
            let role = req.query("role");
            let auth = req.header("Authorization");
            res.setHeader("X-Custom", "Aura-Test");
            res.json({ id: id, role: role, auth: auth }, StatusOK);
        });

        // Handler returning a Response object
        server.get("/health", fn(req: Request): Response => {
            Response.json({ status: "ok", code: StatusOK })
        });

        // Static helper constructors
        let customReq = Request.get("http://localhost:8080/test");
        let customRes = Response.text("Hello World", StatusCreated);
    "#;

    let res = compile(source, &[]).expect("Compilation of native HTTP package should succeed");

    assert!(res.js_code.contains("__AuraHttpRequest"));
    assert!(res.js_code.contains("__AuraHttpResponse"));
    assert!(res.js_code.contains("__AuraResponseWriter"));
    assert!(
        res.js_code
            .contains("export const Request = __AuraHttpRequest;")
    );
    assert!(
        res.js_code
            .contains("export const Response = __AuraHttpResponse;")
    );
    assert!(
        res.js_code
            .contains("export const ResponseWriter = __AuraResponseWriter;")
    );
    assert!(
        res.js_code
            .contains("export const Server = __AuraServeMux;")
    );
    assert!(res.dts_code.contains("export interface HttpRequest"));
    assert!(res.dts_code.contains("export interface ResponseWriter"));
    assert!(res.dts_code.contains("export declare const Response:"));
}

#[test]
fn test_native_http_struct_receiver_handler() {
    let source = r#"
        import { Request, ResponseWriter, Server } from "net/http";

        type ApiHandler = {
            version: String,
        }

        fn (h: ApiHandler) Handle(req: Request, res: ResponseWriter): Unit => {
            let item = req.param("item");
            res.json({ version: h.version, item: item });
        }

        let srv = Server.new(":3000");
        let api = ApiHandler { version: "v1.0" };
        srv.get("/api/:item", fn(req: Request, res: ResponseWriter) => {
            api.Handle(req, res);
        });
    "#;

    let res =
        compile(source, &[]).expect("Compilation of struct receiver HTTP handler should succeed");
    assert!(res.js_code.contains("api.Handle(req, res)"));
}

#[test]
fn test_native_http_golang_codegen() {
    let source = r#"
        import { Server, Request, Response, ResponseWriter } from "net/http";

        type Greeting = {
            msg: String,
        }

        let srv = Server.new(":8080");

        srv.get("/greet/:name", fn(req: Request, res: ResponseWriter) => {
            let name = req.pathValue("name");
            res.json(Greeting { msg: `Hello ${name}` });
        });

        srv.get("/ping", fn(req: Request): Response => {
            Response.json({ pong: true })
        });
    "#;

    let go_code =
        compile_to_go(source).expect("Golang transpilation should succeed for native HTTP package");

    assert!(go_code.contains("type ResponseWriter = auraResponse"));
    assert!(go_code.contains("var Response = auraResponseStatic{}"));
    assert!(go_code.contains("var Request = auraRequestStatic{}"));
    assert!(go_code.contains("var Server = auraServerStatic{}"));
    assert!(go_code.contains("func (auraResponseStatic) Json"));
    assert!(go_code.contains("func (r *auraResponse) Json"));
    assert!(go_code.contains("func (r *auraRequest) PathValue"));
    assert!(go_code.contains("go_http \"net/http\""));
}

#[test]
fn test_native_http_e2e_node_execution() {
    let source = r#"
        import { Server, Request, Response, ResponseWriter, StatusOK, StatusCreated } from "net/http";

        export async fn main(): Task<Unit, String> {
            let srv = Server.new(":9393");

            srv.get("/hello/:name", fn(req: Request, res: ResponseWriter) => {
                let name = req.pathValue("name");
                let lang = req.query("lang");
                res.setHeader("X-Aura", "NativeHTTP");
                res.json({ message: `Hello ${name}`, lang: lang }, StatusOK);
            });

            srv.get("/status", fn(req: Request): Response => {
                Response.json({ status: "alive" }, StatusCreated)
            });

            srv.post("/echo", fn(req: Request, res: ResponseWriter) => async {
                let parseRes = await req.parseJson();
                match parseRes {
                    Ok(body) => {
                        res.json({ received: body });
                    },
                    Err(err) => {
                        res.error(err, 400);
                    }
                }
            });

            spawn(async {
                await srv.listenAndServe(":9393");
            });

            await sleep(120);

            // 1. Test GET /hello/AuraDev?lang=es
            let res1 = await fetch("http://127.0.0.1:9393/hello/AuraDev?lang=es");
            assert(res1.status == 200, "status should be 200");
            let body1 = await res1.json();
            assert(body1.message == "Hello AuraDev", "message should match");
            assert(body1.lang == "es", "lang should match");

            // 2. Test GET /status
            let res2 = await fetch("http://127.0.0.1:9393/status");
            assert(res2.status == 201, "status should be 201");
            let body2 = await res2.json();
            assert(body2.status == "alive", "status body should match");

            // 3. Test POST /echo
            let res3 = await fetch("http://127.0.0.1:9393/echo", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ item: "book", qty: 2 })
            });
            assert(res3.status == 200, "status should be 200");
            let body3 = await res3.json();
            assert(body3.received.item == "book", "echo item should match");

            await srv.close();
            ()
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of native HTTP e2e test should succeed");

    let dir = std::env::temp_dir().join("aura_native_http_test");
    let _ = fs::create_dir_all(&dir);
    let js_path = dir.join("http_test.mjs");
    fs::write(&js_path, &res.js_code).expect("Should write JS test file");

    let output = Command::new("node")
        .arg(&js_path)
        .output()
        .expect("Should execute Node.js");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "Node test execution failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
        stdout,
        stderr
    );
}
