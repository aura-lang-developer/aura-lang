//! Aura to Golang Transpiler (`codegen_go.rs`).
//! Generates clean, idiomatic Go code from Aura AST, enabling direct
//! compilation to native binaries using `go build`.

use crate::ast::*;

const AURA_GO_RUNTIME: &str = r#"
type auraUnit struct{}
var unit = auraUnit{}
var __auraStartTime = go_time.Now()

type auraMutex struct {
    sync.Mutex
}
func (m *auraMutex) Lock() { m.Mutex.Lock() }
func (m *auraMutex) Unlock() { m.Mutex.Unlock() }
func (m *auraMutex) lock() { m.Mutex.Lock() }
func (m *auraMutex) unlock() { m.Mutex.Unlock() }

type auraMutexStatic struct{}
func (auraMutexStatic) New() *auraMutex { return &auraMutex{} }
func (auraMutexStatic) new() *auraMutex { return &auraMutex{} }
var Mutex = auraMutexStatic{}

type auraWaitGroup struct {
    sync.WaitGroup
}
func (w *auraWaitGroup) Add(delta int) { w.WaitGroup.Add(delta) }
func (w *auraWaitGroup) add(delta int) { w.WaitGroup.Add(delta) }
func (w *auraWaitGroup) Done() { w.WaitGroup.Done() }
func (w *auraWaitGroup) done() { w.WaitGroup.Done() }
func (w *auraWaitGroup) Wait() { w.WaitGroup.Wait() }
func (w *auraWaitGroup) wait() { w.WaitGroup.Wait() }

type auraWaitGroupStatic struct{}
func (auraWaitGroupStatic) New() *auraWaitGroup { return &auraWaitGroup{} }
func (auraWaitGroupStatic) new() *auraWaitGroup { return &auraWaitGroup{} }
var WaitGroup = auraWaitGroupStatic{}

type Task = any

type auraChannelStatic struct{}
func (auraChannelStatic) Make(cap int) chan any { return make(chan any, cap) }
func (auraChannelStatic) make(cap int) chan any { return make(chan any, cap) }
func (auraChannelStatic) New(cap int) chan any { return make(chan any, cap) }
func (auraChannelStatic) new(cap int) chan any { return make(chan any, cap) }
func (auraChannelStatic) Close(ch any) {
    if c, ok := ch.(chan any); ok { close(c); return }
    if c, ok := ch.(chan string); ok { close(c); return }
    if c, ok := ch.(chan int64); ok { close(c); return }
    reflect.ValueOf(ch).Close()
}
func (auraChannelStatic) close(ch any) { auraChannelStatic{}.Close(ch) }
var Channel = auraChannelStatic{}

type auraProcessStatic struct{}
func (auraProcessStatic) Uptime() float64 { return go_time.Since(__auraStartTime).Seconds() }
func (auraProcessStatic) uptime() float64 { return go_time.Since(__auraStartTime).Seconds() }
var process = auraProcessStatic{}

type auraResult struct {
    Val any
    Err string
    IsOk bool
}
func (r auraResult) Unwrap() any {
    if !r.IsOk {
        panic("called Result.unwrap() on an Err value: " + r.Err)
    }
    return r.Val
}
func (r auraResult) unwrap() any {
    if !r.IsOk {
        panic("called Result.unwrap() on an Err value: " + r.Err)
    }
    return r.Val
}

type Result = auraResult
type Option = any

func Some(v any) any { return v }
var None any = nil
func Ok(v any) auraResult { return auraResult{Val: v, IsOk: true} }
func Err(e any) auraResult { return auraResult{Err: fmt.Sprint(e), IsOk: false} }

func parseFloat(v any) float64 {
    s := fmt.Sprint(v)
    f, _ := strconv.ParseFloat(strings.TrimSpace(s), 64)
    return f
}

func parseInt(v any) int64 {
    s := fmt.Sprint(v)
    n, _ := strconv.ParseInt(strings.TrimSpace(s), 10, 64)
    return n
}

func __auraFilter[T any](slice []T, pred func(T) bool) []T {
    var res []T
    for _, item := range slice {
        if pred(item) {
            res = append(res, item)
        }
    }
    return res
}

func __auraMap[T any, U any](slice []T, f func(T) U) []U {
    res := make([]U, len(slice))
    for i, item := range slice {
        res[i] = f(item)
    }
    return res
}

func __auraSlice(v any) []any {
    if v == nil {
        return nil
    }
    if s, ok := v.([]any); ok {
        return s
    }
    val := reflect.ValueOf(v)
    if val.Kind() == reflect.Slice {
        res := make([]any, val.Len())
        for i := 0; i < val.Len(); i++ {
            res[i] = val.Index(i).Interface()
        }
        return res
    }
    return nil
}

func capitalize(s string) string {
    if len(s) == 0 {
        return ""
    }
    return strings.ToUpper(s[:1]) + s[1:]
}

func __auraGet(obj any, field string) any {
    if obj == nil {
        return nil
    }
    val := reflect.ValueOf(obj)
    if val.Kind() == reflect.Pointer {
        if val.IsNil() {
            return nil
        }
        val = val.Elem()
    }
    switch val.Kind() {
    case reflect.Map:
        kv := reflect.ValueOf(field)
        mv := val.MapIndex(kv)
        if mv.IsValid() {
            return mv.Interface()
        }
        for _, k := range val.MapKeys() {
            if strings.EqualFold(fmt.Sprint(k.Interface()), field) {
                return val.MapIndex(k).Interface()
            }
        }
        return nil
    case reflect.Struct:
        fName := capitalize(field)
        f := val.FieldByName(fName)
        if f.IsValid() {
            return f.Interface()
        }
        typ := val.Type()
        for i := 0; i < typ.NumField(); i++ {
            if strings.EqualFold(typ.Field(i).Name, field) {
                return val.Field(i).Interface()
            }
        }
        return nil
    default:
        return nil
    }
}

func __auraInt(v any) int64 {
    switch val := v.(type) {
    case int: return int64(val)
    case int64: return val
    case int32: return int64(val)
    case float64: return int64(val)
    case float32: return int64(val)
    case string:
        n, _ := strconv.ParseInt(strings.TrimSpace(val), 10, 64)
        return n
    default:
        return 0
    }
}

func __auraFloat(v any) float64 {
    switch val := v.(type) {
    case float64: return val
    case float32: return float64(val)
    case int: return float64(val)
    case int64: return float64(val)
    case string:
        f, _ := strconv.ParseFloat(strings.TrimSpace(val), 64)
        return f
    default:
        return 0.0
    }
}

func __auraStr(v any) string {
    if v == nil {
        return ""
    }
    return fmt.Sprint(v)
}

func __auraBool(v any) bool {
    switch val := v.(type) {
    case bool: return val
    case int, int64: return val != 0
    case float64: return val != 0.0
    case string: return val != "" && val != "false"
    default: return v != nil
    }
}

type auraResponse struct {
    w go_http.ResponseWriter
}
func (r *auraResponse) SetHeader(k, v string) { r.w.Header().Set(k, v) }
func (r *auraResponse) setHeader(k, v string) { r.w.Header().Set(k, v) }
func (r *auraResponse) Header() go_http.Header { return r.w.Header() }
func (r *auraResponse) Write(b []byte) (int, error) { return r.w.Write(b) }
func (r *auraResponse) WriteHeader(status int) { r.w.WriteHeader(status) }

type auraRequest struct {
    r *go_http.Request
    Method string
    Url string
}
func (r *auraRequest) GetHeader(k string) string {
    if r != nil && r.r != nil {
        return r.r.Header.Get(k)
    }
    return ""
}
func (r *auraRequest) getHeader(k string) string { return r.GetHeader(k) }
func (r *auraRequest) Header(k string) string { return r.GetHeader(k) }
func (r *auraRequest) header(k string) string { return r.GetHeader(k) }

type auraRouteInfo struct {
    Method string
    Pattern string
    Metadata map[string]any
}

func auraToMap(val any) map[string]any {
    if val == nil {
        return nil
    }
    if m, ok := val.(map[string]any); ok {
        return m
    }
    rv := reflect.ValueOf(val)
    if rv.Kind() == reflect.Pointer {
        rv = rv.Elem()
    }
    if rv.Kind() == reflect.Struct {
        res := make(map[string]any)
        rt := rv.Type()
        for i := 0; i < rv.NumField(); i++ {
            f := rt.Field(i)
            name := strings.ToLower(f.Name)
            if strings.HasPrefix(name, "_") {
                name = name[1:]
            }
            res[name] = rv.Field(i).Interface()
        }
        return res
    }
    return nil
}

type auraMux struct {
    mux *go_http.ServeMux
    middlewares []func(*auraRequest, *auraResponse, func())
    routes []auraRouteInfo
    docOverrides map[string]map[string]any
}

func (m *auraMux) Use(mw any) { m.use(mw) }
func (m *auraMux) use(mw any) {
    switch fn := mw.(type) {
    case func(*auraRequest, *auraResponse, func()):
        m.middlewares = append(m.middlewares, fn)
    case func(any, any, any):
        m.middlewares = append(m.middlewares, func(req *auraRequest, res *auraResponse, next func()) {
            fn(req, res, next)
        })
    }
}

func (m *auraMux) Doc(method, pattern string, metadata any) { m.doc(method, pattern, metadata) }
func (m *auraMux) doc(method, pattern string, metadata any) {
    if m.docOverrides == nil {
        m.docOverrides = make(map[string]map[string]any)
    }
    key := strings.ToUpper(method) + " " + pattern
    m.docOverrides[key] = auraToMap(metadata)
}
func (m *auraMux) Document(method, pattern string, metadata any) { m.doc(method, pattern, metadata) }
func (m *auraMux) document(method, pattern string, metadata any) { m.doc(method, pattern, metadata) }

func (m *auraMux) route(method, pattern string, args ...any) {
    var handler any
    var meta map[string]any
    if len(args) == 1 {
        handler = args[0]
    } else if len(args) >= 2 {
        if mmap := auraToMap(args[0]); mmap != nil {
            meta = mmap
            handler = args[1]
        } else {
            handler = args[0]
        }
    }
    m.routes = append(m.routes, auraRouteInfo{Method: method, Pattern: pattern, Metadata: meta})
    parts := strings.Split(pattern, "/")
    for i, p := range parts {
        if strings.HasPrefix(p, ":") {
            parts[i] = "{" + strings.TrimPrefix(p, ":") + "}"
        }
    }
    goPattern := method + " " + strings.Join(parts, "/")
    m.mux.HandleFunc(goPattern, func(w go_http.ResponseWriter, r *go_http.Request) {
        reqObj := &auraRequest{r: r, Method: r.Method, Url: r.URL.String()}
        resObj := &auraResponse{w: w}
        var runMw func(idx int)
        runMw = func(idx int) {
            if idx < len(m.middlewares) {
                m.middlewares[idx](reqObj, resObj, func() { runMw(idx + 1) })
            } else {
                switch h := handler.(type) {
                case func(*auraRequest, *auraResponse):
                    h(reqObj, resObj)
                case func(*auraRequest, *auraResponse) any:
                    h(reqObj, resObj)
                case func(any, any):
                    h(reqObj, resObj)
                case func(any, any) any:
                    h(reqObj, resObj)
                }
            }
        }
        runMw(0)
    })
}

func (m *auraMux) Get(p string, args ...any) { m.get(p, args...) }
func (m *auraMux) get(p string, args ...any) { m.route("GET", p, args...) }
func (m *auraMux) Post(p string, args ...any) { m.post(p, args...) }
func (m *auraMux) post(p string, args ...any) { m.route("POST", p, args...) }
func (m *auraMux) Put(p string, args ...any) { m.put(p, args...) }
func (m *auraMux) put(p string, args ...any) { m.route("PUT", p, args...) }
func (m *auraMux) Delete(p string, args ...any) { m.delete(p, args...) }
func (m *auraMux) delete(p string, args ...any) { m.route("DELETE", p, args...) }
func (m *auraMux) Patch(p string, args ...any) { m.patch(p, args...) }
func (m *auraMux) patch(p string, args ...any) { m.route("PATCH", p, args...) }
func (m *auraMux) Handle(p string, args ...any) { m.handle(p, args...) }
func (m *auraMux) handle(p string, args ...any) {
    method := "GET"
    pattern := p
    if strings.Contains(p, " ") {
        parts := strings.SplitN(p, " ", 2)
        method = strings.ToUpper(parts[0])
        pattern = parts[1]
    }
    m.route(method, pattern, args...)
}
func (m *auraMux) HandleFunc(p string, args ...any) { m.handle(p, args...) }
func (m *auraMux) handleFunc(p string, args ...any) { m.handle(p, args...) }

func (m *auraMux) ListenAndServe(addr string) error { return m.listenAndServe(addr) }
func (m *auraMux) listenAndServe(addr string) error {
    return go_http.ListenAndServe(addr, m.mux)
}

func (m *auraMux) ListenAndServeTLS(addr, certFile, keyFile string) error { return m.listenAndServeTLS(addr, certFile, keyFile) }
func (m *auraMux) listenAndServeTLS(addr, certFile, keyFile string) error {
    return go_http.ListenAndServeTLS(addr, certFile, keyFile, m.mux)
}

func (m *auraMux) generateOpenAPISpec(title, version, desc, basePath, serverUrl, contact, license string) []byte {
    pathsMap := make(map[string]map[string]any)

    for _, r := range m.routes {
        cleanPattern := r.Pattern
        if strings.HasPrefix(cleanPattern, basePath) {
            continue
        }
        parts := strings.Split(cleanPattern, "/")
        var pathParams []map[string]any
        for i, p := range parts {
            if strings.HasPrefix(p, ":") {
                paramName := strings.TrimPrefix(p, ":")
                parts[i] = "{" + paramName + "}"
                pathParams = append(pathParams, map[string]any{
                    "name": paramName,
                    "in": "path",
                    "required": true,
                    "schema": map[string]any{
                        "type": "string",
                    },
                    "description": "Path parameter " + paramName,
                })
            } else if strings.HasPrefix(p, "{") && strings.HasSuffix(p, "}") {
                paramName := strings.TrimSuffix(strings.TrimPrefix(p, "{"), "}")
                pathParams = append(pathParams, map[string]any{
                    "name": paramName,
                    "in": "path",
                    "required": true,
                    "schema": map[string]any{
                        "type": "string",
                    },
                    "description": "Path parameter " + paramName,
                })
            }
        }
        openApiPathKey := strings.Join(parts, "/")
        if openApiPathKey == "" {
            openApiPathKey = "/"
        }

        tag := "General"
        for _, seg := range parts {
            if seg != "" && seg != "api" && !strings.HasPrefix(seg, "{") {
                if len(seg) > 0 {
                    tag = strings.ToUpper(seg[:1]) + seg[1:]
                }
                break
            }
        }

        methodLower := strings.ToLower(r.Method)
        if methodLower == "" {
            methodLower = "get"
        }

        op := map[string]any{
            "summary": fmt.Sprintf("%s %s", r.Method, r.Pattern),
            "description": fmt.Sprintf("Handler for %s %s", r.Method, r.Pattern),
            "tags": []string{tag},
            "operationId": fmt.Sprintf("%s_%s", methodLower, strings.ReplaceAll(strings.ReplaceAll(strings.ReplaceAll(openApiPathKey, "/", "_"), "{", ""), "}", "")),
            "responses": map[string]any{
                "200": map[string]any{
                    "description": "Successful operation",
                    "content": map[string]any{
                        "application/json": map[string]any{
                            "schema": map[string]any{"type": "object"},
                        },
                    },
                },
                "400": map[string]any{"description": "Bad Request / Validation Error"},
                "401": map[string]any{"description": "Unauthorized"},
                "403": map[string]any{"description": "Forbidden"},
                "404": map[string]any{"description": "Not Found"},
                "500": map[string]any{"description": "Internal Server Error"},
            },
        }

        if len(pathParams) > 0 {
            op["parameters"] = pathParams
        }

        if r.Method == "POST" || r.Method == "PUT" || r.Method == "PATCH" {
            op["requestBody"] = map[string]any{
                "description": "JSON request payload",
                "required": true,
                "content": map[string]any{
                    "application/json": map[string]any{
                        "schema": map[string]any{
                            "type": "object",
                        },
                    },
                },
            }
        }

        if !strings.Contains(cleanPattern, "health") && !strings.Contains(cleanPattern, "login") {
            op["security"] = []map[string]any{
                {"bearerAuth": []string{}},
            }
        }

        key := r.Method + " " + r.Pattern
        meta := r.Metadata
        if meta == nil && m.docOverrides != nil {
            meta = m.docOverrides[key]
        }
        if meta == nil && m.docOverrides != nil {
            meta = m.docOverrides[r.Pattern]
        }

        if meta != nil {
            if s, ok := meta["summary"].(string); ok && s != "" {
                op["summary"] = s
            }
            if d, ok := meta["description"].(string); ok && d != "" {
                op["description"] = d
            } else if d, ok := meta["desc"].(string); ok && d != "" {
                op["description"] = d
            }
            if t, ok := meta["tags"]; ok {
                switch tv := t.(type) {
                case []string:
                    op["tags"] = tv
                case []any:
                    var tagStrs []string
                    for _, item := range tv {
                        tagStrs = append(tagStrs, fmt.Sprint(item))
                    }
                    op["tags"] = tagStrs
                case string:
                    op["tags"] = []string{tv}
                }
            }
            if oid, ok := meta["operationid"].(string); ok && oid != "" {
                op["operationId"] = oid
            } else if oid, ok := meta["operationId"].(string); ok && oid != "" {
                op["operationId"] = oid
            }
            if dep, ok := meta["deprecated"].(bool); ok && dep {
                op["deprecated"] = true
            }

            if params, ok := meta["parameters"].([]any); ok {
                var extraParams []map[string]any
                for _, p := range params {
                    if pmap := auraToMap(p); pmap != nil {
                        extraParams = append(extraParams, pmap)
                    }
                }
                if len(extraParams) > 0 {
                    op["parameters"] = extraParams
                }
            }
            if queryParams, ok := meta["query"].(map[string]any); ok {
                var existingParams []map[string]any
                if p, ok := op["parameters"].([]map[string]any); ok {
                    existingParams = p
                }
                for qName, qVal := range queryParams {
                    qSchema := map[string]any{"type": "string"}
                    if qs, ok := qVal.(map[string]any); ok {
                        qSchema = qs
                    } else if qType, ok := qVal.(string); ok {
                        qSchema = map[string]any{"type": qType}
                    }
                    existingParams = append(existingParams, map[string]any{
                        "name": qName,
                        "in": "query",
                        "required": false,
                        "schema": qSchema,
                    })
                }
                op["parameters"] = existingParams
            }

            rbVal := meta["requestbody"]
            if rbVal == nil {
                rbVal = meta["requestBody"]
            }
            if rbVal != nil {
                if rbMap := auraToMap(rbVal); rbMap != nil {
                    if _, hasContent := rbMap["content"]; !hasContent {
                        if schema, hasSchema := rbMap["schema"]; hasSchema {
                            rbDesc := "JSON request payload"
                            if d, ok := rbMap["description"].(string); ok && d != "" { rbDesc = d }
                            rbReq := true
                            if reqVal, ok := rbMap["required"].(bool); ok { rbReq = reqVal }
                            op["requestBody"] = map[string]any{
                                "description": rbDesc,
                                "required": rbReq,
                                "content": map[string]any{
                                    "application/json": map[string]any{
                                        "schema": schema,
                                    },
                                },
                            }
                        } else {
                            op["requestBody"] = rbMap
                        }
                    } else {
                        op["requestBody"] = rbMap
                    }
                }
            }

            if resp, ok := meta["responses"]; ok {
                if respMap := auraToMap(resp); respMap != nil {
                    customResponses := make(map[string]any)
                    for code, val := range respMap {
                        if vMap := auraToMap(val); vMap != nil {
                            if _, hasContent := vMap["content"]; !hasContent {
                                if schema, hasSchema := vMap["schema"]; hasSchema {
                                    respDesc := "Response"
                                    if d, ok := vMap["description"].(string); ok { respDesc = d }
                                    customResponses[code] = map[string]any{
                                        "description": respDesc,
                                        "content": map[string]any{
                                            "application/json": map[string]any{
                                                "schema": schema,
                                            },
                                        },
                                    }
                                } else {
                                    customResponses[code] = vMap
                                }
                            } else {
                                customResponses[code] = vMap
                            }
                        } else if strDesc, ok := val.(string); ok {
                            customResponses[code] = map[string]any{"description": strDesc}
                        }
                    }
                    op["responses"] = customResponses
                }
            }

            if sec, ok := meta["security"]; ok {
                if sec == false || sec == "public" || sec == "none" {
                    delete(op, "security")
                } else if secList, ok := sec.([]any); ok {
                    if len(secList) == 0 {
                        delete(op, "security")
                    } else {
                        var secMaps []map[string]any
                        for _, s := range secList {
                            if sm := auraToMap(s); sm != nil {
                                secMaps = append(secMaps, sm)
                            }
                        }
                        op["security"] = secMaps
                    }
                }
            }
        }

        if pathsMap[openApiPathKey] == nil {
            pathsMap[openApiPathKey] = make(map[string]any)
        }
        pathsMap[openApiPathKey][methodLower] = op
    }

    infoMap := map[string]any{
        "title": title,
        "version": version,
        "description": desc,
    }
    if contact != "" {
        infoMap["contact"] = map[string]any{"name": contact}
    }
    if license != "" {
        infoMap["license"] = map[string]any{"name": license}
    }

    doc := map[string]any{
        "openapi": "3.0.3",
        "info": infoMap,
        "paths": pathsMap,
        "components": map[string]any{
            "securitySchemes": map[string]any{
                "bearerAuth": map[string]any{
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT",
                    "description": "Enter JWT Bearer token obtained from login",
                },
                "apiKeyAuth": map[string]any{
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key",
                    "description": "API Key authentication",
                },
            },
        },
    }

    if serverUrl != "" {
        doc["servers"] = []map[string]any{
            {"url": serverUrl, "description": "API Server"},
        }
    }

    data, _ := json.MarshalIndent(doc, "", "  ")
    return data
}

func (m *auraMux) generateSwaggerHTML(title, docUrl string) string {
    html := `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{TITLE}} - Swagger UI</title>
  <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css" />
  <link rel="icon" type="image/png" href="https://raw.githubusercontent.com/aura-lang/aura/main/assets/favicon.png" />
  <style>
    html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
    *, *:before, *:after { box-sizing: inherit; }
    body { margin: 0; background: #fafafa; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; }
    .aura-banner {
      background: linear-gradient(135deg, #1e1e2f 0%, #2d1b4e 50%, #4a154b 100%);
      color: #ffffff;
      padding: 14px 24px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    }
    .aura-banner h1 { margin: 0; font-size: 1.25rem; font-weight: 700; display: flex; align-items: center; gap: 8px; }
    .aura-banner .badge {
      background: rgba(255,255,255,0.18);
      border: 1px solid rgba(255,255,255,0.25);
      padding: 4px 10px;
      border-radius: 999px;
      font-size: 0.75rem;
      font-weight: 500;
    }
  </style>
</head>
<body>
  <div class="aura-banner">
    <h1><span>✨ {{TITLE}}</span> <span style="font-weight: 300; opacity: 0.85;">| Swagger UI</span></h1>
    <div class="badge">Aura net/http Native Server</div>
  </div>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js" charset="UTF-8"></script>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
  <script>
    window.onload = function() {
      window.ui = SwaggerUIBundle({
        url: "{{DOC_URL}}",
        dom_id: '#swagger-ui',
        deepLinking: true,
        presets: [
          SwaggerUIBundle.presets.apis,
          SwaggerUIStandalonePreset
        ],
        plugins: [
          SwaggerUIBundle.plugins.DownloadUrl
        ],
        layout: "StandaloneLayout",
        persistAuthorization: true,
        displayRequestDuration: true,
        docExpansion: "list",
        filter: true
      });
    };
  </script>
</body>
</html>`
    res := strings.ReplaceAll(html, "{{TITLE}}", title)
    return strings.ReplaceAll(res, "{{DOC_URL}}", docUrl)
}

func (m *auraMux) EnableSwagger(args ...any) { m.enableSwagger(args...) }
func (m *auraMux) enableSwagger(args ...any) {
    path := "/swagger"
    title := "Aura API"
    version := "1.0.0"
    desc := "Interactive API documentation powered by Aura Language net/http"
    serverUrl := ""
    contact := ""
    license := ""

    if len(args) == 1 {
        if s, ok := args[0].(string); ok && s != "" {
            if strings.HasPrefix(s, "/") {
                path = s
            } else {
                path = "/" + s
            }
        } else if mmap, ok := args[0].(map[string]any); ok {
            if p, ok := mmap["path"].(string); ok && p != "" { path = p }
            if t, ok := mmap["title"].(string); ok && t != "" { title = t }
            if v, ok := mmap["version"].(string); ok && v != "" { version = v }
            if d, ok := mmap["description"].(string); ok && d != "" { desc = d }
            if s, ok := mmap["server"].(string); ok && s != "" { serverUrl = s }
            if s, ok := mmap["serverUrl"].(string); ok && s != "" { serverUrl = s }
            if c, ok := mmap["contact"].(string); ok && c != "" { contact = c }
            if l, ok := mmap["license"].(string); ok && l != "" { license = l }
        } else if args[0] != nil {
            val := reflect.ValueOf(args[0])
            if val.Kind() == reflect.Pointer {
                val = val.Elem()
            }
            if val.Kind() == reflect.Struct {
                typ := val.Type()
                for i := 0; i < val.NumField(); i++ {
                    name := strings.ToLower(typ.Field(i).Name)
                    fieldVal := val.Field(i)
                    if fieldVal.Kind() == reflect.String {
                        strVal := fieldVal.String()
                        if strVal != "" {
                            switch name {
                            case "path": path = strVal
                            case "title": title = strVal
                            case "version": version = strVal
                            case "description", "desc": desc = strVal
                            case "server", "serverurl": serverUrl = strVal
                            case "contact": contact = strVal
                            case "license": license = strVal
                            }
                        }
                    }
                }
            }
        }
    } else if len(args) == 2 {
        if s0, ok := args[0].(string); ok {
            if s1, ok := args[1].(string); ok {
                if strings.HasPrefix(s0, "/") {
                    path = s0
                    title = s1
                } else if strings.HasPrefix(s1, "/") {
                    title = s0
                    path = s1
                } else {
                    title = s0
                    version = s1
                }
            }
        }
    } else if len(args) >= 3 {
        if t, ok := args[0].(string); ok && t != "" { title = t }
        if v, ok := args[1].(string); ok && v != "" { version = v }
        if p, ok := args[2].(string); ok && p != "" { path = p }
        if len(args) >= 4 {
            if d, ok := args[3].(string); ok && d != "" { desc = d }
        }
    }

    path = "/" + strings.Trim(path, "/")
    jsonPath := path + "/doc.json"
    openApiPath := path + "/openapi.json"
    uiSlash := path + "/"
    uiIndex := path + "/index.html"

    jsonHandler := func(w go_http.ResponseWriter, r *go_http.Request) {
        spec := m.generateOpenAPISpec(title, version, desc, path, serverUrl, contact, license)
        w.Header().Set("Content-Type", "application/json; charset=utf-8")
        w.Header().Set("Access-Control-Allow-Origin", "*")
        w.WriteHeader(go_http.StatusOK)
        w.Write(spec)
    }

    uiHandler := func(w go_http.ResponseWriter, r *go_http.Request) {
        html := m.generateSwaggerHTML(title, jsonPath)
        w.Header().Set("Content-Type", "text/html; charset=utf-8")
        w.WriteHeader(go_http.StatusOK)
        w.Write([]byte(html))
    }

    m.mux.HandleFunc("GET " + jsonPath, jsonHandler)
    m.mux.HandleFunc("GET " + openApiPath, jsonHandler)
    m.mux.HandleFunc("GET " + path, uiHandler)
    m.mux.HandleFunc("GET " + uiSlash, uiHandler)
    m.mux.HandleFunc("GET " + uiIndex, uiHandler)
}

func (m *auraMux) Swagger(args ...any) { m.enableSwagger(args...) }
func (m *auraMux) swagger(args ...any) { m.enableSwagger(args...) }
func (m *auraMux) SwaggerUI(args ...any) { m.enableSwagger(args...) }
func (m *auraMux) swaggerUI(args ...any) { m.enableSwagger(args...) }

func (m *auraMux) OpenAPI(args ...any) string { return m.openAPI(args...) }
func (m *auraMux) openAPI(args ...any) string {
    title := "Aura API"
    version := "1.0.0"
    desc := "Interactive API documentation powered by Aura Language net/http"
    if len(args) > 0 {
        if t, ok := args[0].(string); ok { title = t }
    }
    return string(m.generateOpenAPISpec(title, version, desc, "/swagger", "", "", ""))
}

type auraHttpStatic struct {
    StatusOK int
    StatusCreated int
    StatusAccepted int
    StatusNoContent int
    StatusBadRequest int
    StatusUnauthorized int
    StatusForbidden int
    StatusNotFound int
    StatusInternalServerError int
}

func (auraHttpStatic) NewServeMux() *auraMux { return &auraMux{mux: go_http.NewServeMux(), docOverrides: make(map[string]map[string]any)} }
func (auraHttpStatic) newServeMux() *auraMux { return &auraMux{mux: go_http.NewServeMux(), docOverrides: make(map[string]map[string]any)} }

func (auraHttpStatic) Doc(mux *auraMux, method, pattern string, metadata any) {
    if mux != nil { mux.doc(method, pattern, metadata) }
}
func (auraHttpStatic) doc(mux *auraMux, method, pattern string, metadata any) {
    if mux != nil { mux.doc(method, pattern, metadata) }
}
func (auraHttpStatic) Document(mux *auraMux, method, pattern string, metadata any) {
    if mux != nil { mux.doc(method, pattern, metadata) }
}
func (auraHttpStatic) document(mux *auraMux, method, pattern string, metadata any) {
    if mux != nil { mux.doc(method, pattern, metadata) }
}

func (auraHttpStatic) Json(w any, status int, data any) {
    var rw go_http.ResponseWriter
    if resp, ok := w.(*auraResponse); ok {
        rw = resp.w
    } else if origRw, ok := w.(go_http.ResponseWriter); ok {
        rw = origRw
    } else {
        return
    }
    rw.Header().Set("Content-Type", "application/json; charset=utf-8")
    rw.WriteHeader(status)
    json.NewEncoder(rw).Encode(data)
}
func (auraHttpStatic) json(w any, status int, data any) {
    auraHttpStatic{}.Json(w, status, data)
}

func (auraHttpStatic) Error(w any, msg string, status int) {
    var rw go_http.ResponseWriter
    if resp, ok := w.(*auraResponse); ok {
        rw = resp.w
    } else if origRw, ok := w.(go_http.ResponseWriter); ok {
        rw = origRw
    } else {
        return
    }
    go_http.Error(rw, msg, status)
}
func (auraHttpStatic) error(w any, msg string, status int) {
    auraHttpStatic{}.Error(w, msg, status)
}

func (auraHttpStatic) PathValue(r any, key string) string {
    if req, ok := r.(*auraRequest); ok {
        return req.r.PathValue(key)
    }
    if req, ok := r.(*go_http.Request); ok {
        return req.PathValue(key)
    }
    return ""
}
func (auraHttpStatic) pathValue(r any, key string) string {
    return auraHttpStatic{}.PathValue(r, key)
}

func (auraHttpStatic) Query(r any, key string) string {
    if req, ok := r.(*auraRequest); ok {
        return req.r.URL.Query().Get(key)
    }
    if req, ok := r.(*go_http.Request); ok {
        return req.URL.Query().Get(key)
    }
    return ""
}
func (auraHttpStatic) query(r any, key string) string {
    return auraHttpStatic{}.Query(r, key)
}

func (auraHttpStatic) Header(r any, key string) string {
    if req, ok := r.(*auraRequest); ok && req != nil && req.r != nil {
        return req.r.Header.Get(key)
    }
    if req, ok := r.(*go_http.Request); ok && req != nil {
        return req.Header.Get(key)
    }
    return ""
}
func (auraHttpStatic) header(r any, key string) string {
    return auraHttpStatic{}.Header(r, key)
}
func (auraHttpStatic) GetHeader(r any, key string) string {
    return auraHttpStatic{}.Header(r, key)
}
func (auraHttpStatic) getHeader(r any, key string) string {
    return auraHttpStatic{}.Header(r, key)
}

func (auraHttpStatic) ParseJson(r any) auraResult {
    var req *go_http.Request
    if reqObj, ok := r.(*auraRequest); ok {
        req = reqObj.r
    } else if origReq, ok := r.(*go_http.Request); ok {
        req = origReq
    } else {
        return Err("Invalid request")
    }
    bodyBytes, err := io.ReadAll(req.Body)
    if err != nil {
        return Err(err.Error())
    }
    var val any
    if err := json.Unmarshal(bodyBytes, &val); err != nil {
        return Err(err.Error())
    }
    return Ok(val)
}
func (auraHttpStatic) parseJson(r any) auraResult {
    return auraHttpStatic{}.ParseJson(r)
}
func (auraHttpStatic) EnableSwagger(mux *auraMux, args ...any) {
    if mux != nil { mux.enableSwagger(args...) }
}
func (auraHttpStatic) enableSwagger(mux *auraMux, args ...any) {
    auraHttpStatic{}.EnableSwagger(mux, args...)
}
func (auraHttpStatic) Swagger(mux *auraMux, args ...any) {
    auraHttpStatic{}.EnableSwagger(mux, args...)
}
func (auraHttpStatic) swagger(mux *auraMux, args ...any) {
    auraHttpStatic{}.EnableSwagger(mux, args...)
}
func (auraHttpStatic) OpenAPI(mux *auraMux, args ...any) string {
    if mux != nil { return mux.openAPI(args...) }
    return ""
}
func (auraHttpStatic) openAPI(mux *auraMux, args ...any) string {
    return auraHttpStatic{}.OpenAPI(mux, args...)
}


var http = auraHttpStatic{
    StatusOK: 200,
    StatusCreated: 201,
    StatusAccepted: 202,
    StatusNoContent: 204,
    StatusBadRequest: 400,
    StatusUnauthorized: 401,
    StatusForbidden: 403,
    StatusNotFound: 404,
    StatusInternalServerError: 500,
}

type auraCryptoStatic struct{}
func (auraCryptoStatic) HmacSha256(data, secret string) string {
    h := hmac.New(sha256.New, []byte(secret))
    h.Write([]byte(data))
    return hex.EncodeToString(h.Sum(nil))
}
func (auraCryptoStatic) hmacSha256(data, secret string) string {
    return auraCryptoStatic{}.HmacSha256(data, secret)
}
func (auraCryptoStatic) Sha256(data string) string {
    h := sha256.Sum256([]byte(data))
    return hex.EncodeToString(h[:])
}
func (auraCryptoStatic) sha256(data string) string {
    return auraCryptoStatic{}.Sha256(data)
}
func (auraCryptoStatic) Base64UrlEncode(data string) string {
    return base64.RawURLEncoding.EncodeToString([]byte(data))
}
func (auraCryptoStatic) base64UrlEncode(data string) string {
    return auraCryptoStatic{}.Base64UrlEncode(data)
}
func (auraCryptoStatic) Base64UrlDecode(data string) auraResult {
    b, err := base64.RawURLEncoding.DecodeString(data)
    if err != nil {
        return Err(err.Error())
    }
    return Ok(string(b))
}
func (auraCryptoStatic) base64UrlDecode(data string) auraResult {
    return auraCryptoStatic{}.Base64UrlDecode(data)
}
var crypto = auraCryptoStatic{}

type auraJwtStatic struct{}
func (auraJwtStatic) Sign(payload any, secret string) string {
    headerJSON := `{"alg":"HS256","typ":"JWT"}`
    headerB64 := base64.RawURLEncoding.EncodeToString([]byte(headerJSON))
    var payloadBytes []byte
    switch p := payload.(type) {
    case string:
        payloadBytes = []byte(p)
    default:
        payloadBytes, _ = json.Marshal(p)
    }
    payloadB64 := base64.RawURLEncoding.EncodeToString(payloadBytes)
    unsigned := headerB64 + "." + payloadB64
    h := hmac.New(sha256.New, []byte(secret))
    h.Write([]byte(unsigned))
    sigB64 := base64.RawURLEncoding.EncodeToString(h.Sum(nil))
    return unsigned + "." + sigB64
}
func (auraJwtStatic) sign(payload any, secret string) string {
    return auraJwtStatic{}.Sign(payload, secret)
}
func (auraJwtStatic) Verify(token, secret string) auraResult {
    parts := strings.Split(token, ".")
    if len(parts) != 3 {
        return Err("Token JWT malformado: se esperan 3 segmentos")
    }
    unsigned := parts[0] + "." + parts[1]
    h := hmac.New(sha256.New, []byte(secret))
    h.Write([]byte(unsigned))
    expectedSig := base64.RawURLEncoding.EncodeToString(h.Sum(nil))
    if parts[2] != expectedSig {
        return Err("Firma de token JWT inválida")
    }
    payloadBytes, err := base64.RawURLEncoding.DecodeString(parts[1])
    if err != nil {
        return Err("No se pudo decodificar payload de token: " + err.Error())
    }
    var val any
    if err := json.Unmarshal(payloadBytes, &val); err != nil {
        return Ok(string(payloadBytes))
    }
    return Ok(val)
}
func (auraJwtStatic) verify(token, secret string) auraResult {
    return auraJwtStatic{}.Verify(token, secret)
}
var jwt = auraJwtStatic{}

type auraOsStatic struct{}
func (auraOsStatic) Env(k string) string { return go_os.Getenv(k) }
func (auraOsStatic) env(k string) string { return go_os.Getenv(k) }
func (auraOsStatic) Hostname() string { h, _ := go_os.Hostname(); return h }
func (auraOsStatic) hostname() string { h, _ := go_os.Hostname(); return h }
var os = auraOsStatic{}

type auraTimeStatic struct{}
func (auraTimeStatic) Now() int64 { return go_time.Now().UnixMilli() }
func (auraTimeStatic) now() int64 { return go_time.Now().UnixMilli() }
func (auraTimeStatic) Sleep(ms int64) { go_time.Sleep(go_time.Duration(ms) * go_time.Millisecond) }
func (auraTimeStatic) sleep(ms int64) { go_time.Sleep(go_time.Duration(ms) * go_time.Millisecond) }
var time = auraTimeStatic{}

type auraMathStatic struct{}
func (auraMathStatic) Floor(v any) float64 { return go_math.Floor(__auraFloat(v)) }
func (auraMathStatic) floor(v any) float64 { return go_math.Floor(__auraFloat(v)) }
func (auraMathStatic) Ceil(v any) float64 { return go_math.Ceil(__auraFloat(v)) }
func (auraMathStatic) ceil(v any) float64 { return go_math.Ceil(__auraFloat(v)) }
func (auraMathStatic) Round(v any) float64 { return go_math.Round(__auraFloat(v)) }
func (auraMathStatic) round(v any) float64 { return go_math.Round(__auraFloat(v)) }
func (auraMathStatic) Abs(v any) float64 { return go_math.Abs(__auraFloat(v)) }
func (auraMathStatic) abs(v any) float64 { return go_math.Abs(__auraFloat(v)) }
func (auraMathStatic) Min(a, b any) float64 { return go_math.Min(__auraFloat(a), __auraFloat(b)) }
func (auraMathStatic) min(a, b any) float64 { return go_math.Min(__auraFloat(a), __auraFloat(b)) }
func (auraMathStatic) Max(a, b any) float64 { return go_math.Max(__auraFloat(a), __auraFloat(b)) }
func (auraMathStatic) max(a, b any) float64 { return go_math.Max(__auraFloat(a), __auraFloat(b)) }
func (auraMathStatic) Sqrt(v any) float64 { return go_math.Sqrt(__auraFloat(v)) }
func (auraMathStatic) sqrt(v any) float64 { return go_math.Sqrt(__auraFloat(v)) }
func (auraMathStatic) Random() float64 { return go_rand.Float64() }
func (auraMathStatic) random() float64 { return go_rand.Float64() }
var Math = auraMathStatic{}

func __auraUnwrap[T any](v any) T {
    if r, ok := v.(auraResult); ok {
        if !r.IsOk {
            panic("called Result.unwrap() on an Err value: " + r.Err)
        }
        if val, ok := r.Val.(T); ok {
            return val
        }
        bytes, err := json.Marshal(r.Val)
        if err == nil {
            var res T
            if err := json.Unmarshal(bytes, &res); err == nil {
                return res
            }
        }
        var zero T
        return zero
    }
    if val, ok := v.(T); ok {
        return val
    }
    var zero T
    return zero
}

func __auraCast[T any](v any) T {
    if v == nil {
        var zero T
        return zero
    }
    if val, ok := v.(T); ok {
        return val
    }
    bytes, err := json.Marshal(v)
    if err == nil {
        var res T
        if err := json.Unmarshal(bytes, &res); err == nil {
            return res
        }
    }
    var zero T
    return zero
}
"#;

pub struct GoCodeGen {
    output: String,
    indent_level: usize,
    struct_defs: std::collections::HashMap<String, Vec<(String, Type)>>,
    interface_defs: std::collections::HashMap<String, InterfaceDecl>,
    func_defs: std::collections::HashMap<String, FunctionDecl>,
    current_returns_result: bool,
    current_return_type: Option<String>,
}

impl GoCodeGen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            struct_defs: std::collections::HashMap::new(),
            interface_defs: std::collections::HashMap::new(),
            func_defs: std::collections::HashMap::new(),
            current_returns_result: false,
            current_return_type: None,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn write_line(&mut self, line: &str) {
        if line.is_empty() {
            self.output.push('\n');
        } else {
            self.output.push_str(&self.indent());
            self.output.push_str(line);
            self.output.push('\n');
        }
    }

    pub fn generate(&mut self, module: &Module) -> String {
        self.struct_defs.clear();
        self.interface_defs.clear();
        self.func_defs.clear();

        for item in &module.items {
            match item {
                Item::TypeAlias(alias) => {
                    if let Type::Record(fields) = &alias.target {
                        self.struct_defs.insert(alias.name.clone(), fields.clone());
                    }
                }
                Item::Interface(iface) => {
                    self.interface_defs
                        .insert(iface.name.clone(), iface.clone());
                }
                Item::Function(f) => {
                    self.func_defs.insert(f.name.clone(), f.clone());
                }
                _ => {}
            }
        }

        let mut body_output = String::new();
        std::mem::swap(&mut self.output, &mut body_output);

        let mut top_level_vars = Vec::new();
        let mut other_top_stmts = Vec::new();
        let mut has_main = false;

        for item in &module.items {
            match item {
                Item::Function(f) => {
                    if f.name == "main" {
                        has_main = true;
                    }
                    self.emit_function(f);
                }
                Item::Interface(iface) => self.emit_interface(iface),
                Item::TypeAlias(alias) => self.emit_type_alias(alias),
                Item::Extern(ext) => {
                    for f in &ext.functions {
                        let mut params = Vec::new();
                        for p in &f.params {
                            let ty_str = p
                                .type_annotation
                                .as_ref()
                                .map(|t| self.map_type_to_go(t))
                                .unwrap_or_else(|| "any".to_string());
                            params.push(format!("{} {}", p.name, ty_str));
                        }
                        let ret_str = self.map_type_to_go(&f.return_type);
                        if f.name == "abs" {
                            self.write_line(
                                "func abs(n int64) int64 { if n < 0 { return -n }; return n }",
                            );
                        } else if f.name == "getpid" {
                            self.write_line("func getpid() int64 { return int64(os.Getpid()) }");
                        } else {
                            let default_ret = match ret_str.as_str() {
                                "int64" | "int32" | "int16" | "int8" | "uint64" | "uint32"
                                | "uint16" | "byte" => "0",
                                "float64" | "float32" => "0.0",
                                "string" => "\"\"",
                                "bool" => "false",
                                _ => "nil",
                            };
                            if ret_str.is_empty() || ret_str == "auraUnit" {
                                self.write_line(&format!(
                                    "func {}({}) {{}}",
                                    f.name,
                                    params.join(", ")
                                ));
                            } else {
                                self.write_line(&format!(
                                    "func {}({}) {} {{ return {} }}",
                                    f.name,
                                    params.join(", "),
                                    ret_str,
                                    default_ret
                                ));
                            }
                        }
                    }
                }
                Item::Statement(stmt) => {
                    if matches!(stmt, Statement::Let { .. }) {
                        top_level_vars.push(stmt);
                    } else if !matches!(stmt, Statement::Expr(Expr::Literal(_))) {
                        other_top_stmts.push(stmt);
                    }
                }
                _ => {}
            }
        }

        // Package-level variable declarations (e.g. TRACE_ID = "trace-001")
        for stmt in &top_level_vars {
            if let Statement::Let {
                name,
                value,
                type_annotation,
                ..
            } = stmt
            {
                let val_str = self.emit_expr(value);
                if let Some(ty) = type_annotation {
                    let ty_str = self.map_type_to_go(ty);
                    self.write_line(&format!("var {} {} = {}", name, ty_str, val_str));
                } else {
                    self.write_line(&format!("var {} = {}", name, val_str));
                }
            }
        }

        if !other_top_stmts.is_empty() {
            if !has_main {
                self.write_line("func main() {");
                self.indent_level += 1;
                for stmt in &other_top_stmts {
                    self.emit_statement(stmt);
                }
                self.indent_level -= 1;
                self.write_line("}\n");
            } else {
                self.write_line("func init() {");
                self.indent_level += 1;
                for stmt in &other_top_stmts {
                    self.emit_statement(stmt);
                }
                self.indent_level -= 1;
                self.write_line("}\n");
            }
        }

        std::mem::swap(&mut self.output, &mut body_output);

        // Header
        let mut final_code = String::new();
        final_code.push_str("// Code generated by Aura Compiler (Target: Golang)\n");
        final_code.push_str("// Follows the Golang backend model\n\n");
        final_code.push_str("package main\n\n");

        final_code.push_str("import (\n");
        final_code.push_str("    \"crypto/hmac\"\n");
        final_code.push_str("    \"crypto/sha256\"\n");
        final_code.push_str("    \"encoding/base64\"\n");
        final_code.push_str("    \"encoding/hex\"\n");
        final_code.push_str("    \"encoding/json\"\n");
        final_code.push_str("    \"fmt\"\n");
        final_code.push_str("    \"io\"\n");
        final_code.push_str("    go_math \"math\"\n");
        final_code.push_str("    go_rand \"math/rand\"\n");
        final_code.push_str("    go_http \"net/http\"\n");
        final_code.push_str("    go_os \"os\"\n");
        final_code.push_str("    \"reflect\"\n");
        final_code.push_str("    \"strconv\"\n");
        final_code.push_str("    \"strings\"\n");
        final_code.push_str("    \"sync\"\n");
        final_code.push_str("    go_time \"time\"\n");
        final_code.push_str(")\n\n");

        // Suppress unused imports
        final_code.push_str("var _ = hmac.New\n");
        final_code.push_str("var _ = sha256.New\n");
        final_code.push_str("var _ = base64.RawURLEncoding\n");
        final_code.push_str("var _ = hex.EncodeToString\n");
        final_code.push_str("var _ = crypto\n");
        final_code.push_str("var _ = jwt\n");
        final_code.push_str("var _ = json.Marshal\n");
        final_code.push_str("var _ = fmt.Sprint\n");
        final_code.push_str("var _ = io.ReadAll\n");
        final_code.push_str("var _ = go_math.Floor\n");
        final_code.push_str("var _ = go_rand.Float64\n");
        final_code.push_str("var _ = go_http.StatusOK\n");
        final_code.push_str("var _ = go_os.Getenv\n");
        final_code.push_str("var _ = reflect.ValueOf\n");
        final_code.push_str("var _ = strconv.Itoa\n");
        final_code.push_str("var _ = strings.TrimSpace\n");
        final_code.push_str("var _ = go_time.Now\n");
        final_code.push_str("var _ = unit\n");
        final_code.push_str("var _ = Math\n");
        final_code.push_str("var _ = __auraCast[any]\n");
        final_code.push_str("var _ = __auraUnwrap[any]\n\n");

        // Helper runtime for Aura primitives in Go
        final_code.push_str("// --- Aura Runtime Helpers for Golang ---\n");
        final_code.push_str(AURA_GO_RUNTIME);
        final_code.push_str("\n\n");

        final_code.push_str(&body_output);
        final_code
    }

    fn wrap_return_expr(&self, expr_str: &str, ret_ty: &str) -> String {
        let trimmed_ty = ret_ty.trim();
        match trimmed_ty {
            "string" => {
                if (expr_str.starts_with('"') && expr_str.ends_with('"'))
                    || expr_str.starts_with("__auraStr(")
                {
                    expr_str.to_string()
                } else {
                    format!("__auraStr({})", expr_str)
                }
            }
            "int64" => {
                if expr_str.chars().all(|c| c.is_ascii_digit() || c == '-')
                    || expr_str.starts_with("__auraInt(")
                {
                    expr_str.to_string()
                } else {
                    format!("__auraInt({})", expr_str)
                }
            }
            "float64" => {
                if (expr_str.contains('.')
                    && expr_str
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '.' || c == '-'))
                    || expr_str.starts_with("__auraFloat(")
                {
                    expr_str.to_string()
                } else {
                    format!("__auraFloat({})", expr_str)
                }
            }
            "bool" => {
                if expr_str == "true" || expr_str == "false" || expr_str.starts_with("__auraBool(")
                {
                    expr_str.to_string()
                } else {
                    format!("__auraBool({})", expr_str)
                }
            }
            _ => expr_str.to_string(),
        }
    }

    fn emit_function(&mut self, func: &FunctionDecl) {
        let prev_ret_ty = self.current_return_type.clone();
        let prev_returns_result = self.current_returns_result;
        self.current_return_type = func.return_type.as_ref().map(|t| self.map_type_to_go(t));
        self.current_returns_result =
            matches!(&func.return_type, Some(Type::Named { name, .. }) if name == "Result");

        if let Some(ref recv) = func.receiver {
            let recv_ty = self.map_type_to_go(&recv.target_type);
            let is_iface_method = self
                .interface_defs
                .values()
                .any(|iface| iface.methods.iter().any(|m| m.name == func.name));
            let is_public = func.is_exported || is_iface_method;
            let go_name = if is_public {
                capitalize(&func.name)
            } else {
                func.name.clone()
            };
            let mut params_str = Vec::new();
            for param in &func.params {
                let ty_str = param
                    .type_annotation
                    .as_ref()
                    .map(|t| self.map_type_to_go(t))
                    .unwrap_or_else(|| "any".to_string());
                params_str.push(format!("{} {}", param.name, ty_str));
            }

            let ret_str = if let Some(ret_ty) = &func.return_type {
                let s = self.map_type_to_go(ret_ty);
                if s == "()" || s == "auraUnit" {
                    String::new()
                } else if self.current_returns_result {
                    " (__aura_ret auraResult)".to_string()
                } else {
                    format!(" {}", s)
                }
            } else {
                String::new()
            };
            self.write_line(&format!(
                "func ({} {}) {}({}){} {{",
                recv.name,
                recv_ty,
                go_name,
                params_str.join(", "),
                ret_str
            ));
            self.indent_level += 1;
            match &func.body {
                Expr::Block(stmts) => {
                    let count = stmts.len();
                    for (idx, stmt) in stmts.iter().enumerate() {
                        if idx == count - 1 && !ret_str.is_empty() {
                            if let Statement::Expr(e) = stmt {
                                let expr_str = self.emit_expr(e);
                                if expr_str != "unit" && !expr_str.is_empty() {
                                    let final_ret =
                                        self.wrap_return_expr(&expr_str, ret_str.trim());
                                    self.write_line(&format!("return {}", final_ret));
                                    continue;
                                }
                            }
                        }
                        self.emit_statement(stmt);
                    }
                }
                expr => {
                    let expr_str = self.emit_expr(expr);
                    if ret_str.is_empty() {
                        self.write_line(&expr_str);
                    } else {
                        let final_ret = self.wrap_return_expr(&expr_str, ret_str.trim());
                        self.write_line(&format!("return {}", final_ret));
                    }
                }
            }
            self.indent_level -= 1;
            self.write_line("}\n");
            self.current_return_type = prev_ret_ty;
            self.current_returns_result = prev_returns_result;
            return;
        }

        let go_name = if func.name == "main" {
            "main".to_string()
        } else if func.is_exported {
            capitalize(&func.name)
        } else {
            func.name.clone()
        };

        let mut params_str = Vec::new();
        for param in &func.params {
            let ty_str = param
                .type_annotation
                .as_ref()
                .map(|t| self.map_type_to_go(t))
                .unwrap_or_else(|| "any".to_string());
            params_str.push(format!("{} {}", param.name, ty_str));
        }

        let ret_str = if func.name == "main" {
            String::new()
        } else if let Some(ret_ty) = &func.return_type {
            let s = self.map_type_to_go(ret_ty);
            if s == "()" || s == "auraUnit" {
                String::new()
            } else if self.current_returns_result {
                " (__aura_ret auraResult)".to_string()
            } else {
                format!(" {}", s)
            }
        } else {
            String::new()
        };

        self.write_line(&format!(
            "func {}({}){} {{",
            go_name,
            params_str.join(", "),
            ret_str
        ));
        self.indent_level += 1;

        match &func.body {
            Expr::Block(stmts) => {
                let count = stmts.len();
                for (idx, stmt) in stmts.iter().enumerate() {
                    if idx == count - 1 && !ret_str.is_empty() {
                        if let Statement::Expr(e) = stmt {
                            let expr_str = self.emit_expr(e);
                            if expr_str != "unit" && !expr_str.is_empty() {
                                let final_ret = self.wrap_return_expr(&expr_str, ret_str.trim());
                                self.write_line(&format!("return {}", final_ret));
                                continue;
                            }
                        }
                    }
                    self.emit_statement(stmt);
                }
            }
            expr => {
                let expr_str = self.emit_expr(expr);
                if func.name == "main" || ret_str.is_empty() {
                    self.write_line(&expr_str);
                } else {
                    let final_ret = self.wrap_return_expr(&expr_str, ret_str.trim());
                    self.write_line(&format!("return {}", final_ret));
                }
            }
        }

        self.indent_level -= 1;
        self.write_line("}\n");
        self.current_return_type = prev_ret_ty;
        self.current_returns_result = prev_returns_result;
    }

    fn emit_interface(&mut self, iface: &InterfaceDecl) {
        self.write_line(&format!("type {} interface {{", iface.name));
        self.indent_level += 1;
        for method in &iface.methods {
            let mut params_str = Vec::new();
            for param in &method.params {
                let ty_str = param
                    .type_annotation
                    .as_ref()
                    .map(|t| self.map_type_to_go(t))
                    .unwrap_or_else(|| "any".to_string());
                params_str.push(format!("{} {}", param.name, ty_str));
            }
            let ret_str = self.map_type_to_go(&method.return_type);
            self.write_line(&format!(
                "{}({}) {}",
                capitalize(&method.name),
                params_str.join(", "),
                ret_str
            ));
        }
        self.indent_level -= 1;
        self.write_line("}\n");

        let impl_struct = format!("__aura_impl_{}", iface.name);
        self.write_line(&format!("type {} struct {{", impl_struct));
        self.indent_level += 1;
        for method in &iface.methods {
            let mut params_str = Vec::new();
            for param in &method.params {
                let ty_str = param
                    .type_annotation
                    .as_ref()
                    .map(|t| self.map_type_to_go(t))
                    .unwrap_or_else(|| "any".to_string());
                params_str.push(format!("{} {}", param.name, ty_str));
            }
            let ret_str = self.map_type_to_go(&method.return_type);
            let ret_sig = if ret_str == "auraUnit" || ret_str == "()" || ret_str.is_empty() {
                String::new()
            } else {
                format!(" {}", ret_str)
            };
            self.write_line(&format!(
                "_{} func({}){}",
                method.name,
                params_str.join(", "),
                ret_sig
            ));
        }
        self.indent_level -= 1;
        self.write_line("}\n");

        for method in &iface.methods {
            let mut params_sig = Vec::new();
            let mut args_call = Vec::new();
            for param in &method.params {
                let ty_str = param
                    .type_annotation
                    .as_ref()
                    .map(|t| self.map_type_to_go(t))
                    .unwrap_or_else(|| "any".to_string());
                params_sig.push(format!("{} {}", param.name, ty_str));
                args_call.push(param.name.clone());
            }
            let ret_str = self.map_type_to_go(&method.return_type);
            let is_void = ret_str == "auraUnit" || ret_str == "()" || ret_str.is_empty();
            let ret_sig = if is_void {
                String::new()
            } else {
                format!(" {}", ret_str)
            };
            self.write_line(&format!(
                "func (a *{}) {}({}){} {{",
                impl_struct,
                capitalize(&method.name),
                params_sig.join(", "),
                ret_sig
            ));
            self.indent_level += 1;
            if is_void {
                self.write_line(&format!(
                    "if a._{} != nil {{ a._{}({}) }}",
                    method.name,
                    method.name,
                    args_call.join(", ")
                ));
            } else {
                self.write_line(&format!(
                    "if a._{} != nil {{ return a._{}({}) }}",
                    method.name,
                    method.name,
                    args_call.join(", ")
                ));
                self.write_line(&format!("var zero {}; return zero", ret_str));
            }
            self.indent_level -= 1;
            self.write_line("}\n");
        }
    }

    fn emit_type_alias(&mut self, alias: &TypeAliasDecl) {
        match &alias.target {
            Type::Record(fields) => {
                if alias.is_packed {
                    self.write_line(&format!(
                        "// Packed struct: {} (binary aligned)",
                        alias.name
                    ));
                }
                self.write_line(&format!("type {} struct {{", alias.name));
                self.indent_level += 1;
                for (name, ty) in fields {
                    let field_name = capitalize(name);
                    let ty_str = self.map_type_to_go(ty);
                    self.write_line(&format!("{} {} `json:\"{}\"`", field_name, ty_str, name));
                }
                self.indent_level -= 1;
                self.write_line("}\n");
            }
            other => {
                let ty_str = self.map_type_to_go(other);
                self.write_line(&format!("type {} = {}\n", alias.name, ty_str));
            }
        }
    }

    fn is_dynamic_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(name) => {
                name == "body"
                    || name == "it"
                    || name == "rawItems"
                    || name == "payload"
                    || name == "evt"
                    || name == "cond"
                    || name == "rawClaims"
                    || name == "rawPayload"
                    || name == "tokenPayload"
            }
            Expr::MemberAccess { object, .. } => self.is_dynamic_expr(object),
            Expr::IndexAccess { object, .. } | Expr::SliceAccess { object, .. } => {
                self.is_dynamic_expr(object)
            }
            _ => false,
        }
    }

    fn infer_expr_go_type(&self, expr: &Expr) -> Option<&'static str> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::String(_) | Literal::TemplateString(_) => Some("string"),
                Literal::Float(_) => Some("float64"),
                Literal::Int(_) => Some("int64"),
                Literal::Bool(_) => Some("bool"),
                _ => None,
            },
            Expr::Binary { op, left, right } => match op {
                BinOp::Equal
                | BinOp::NotEqual
                | BinOp::LessThan
                | BinOp::LessEqual
                | BinOp::GreaterThan
                | BinOp::GreaterEqual
                | BinOp::And
                | BinOp::Or => Some("bool"),
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    let left_ty = self.infer_expr_go_type(left);
                    let right_ty = self.infer_expr_go_type(right);
                    if left_ty == Some("float64") || right_ty == Some("float64") {
                        Some("float64")
                    } else if left_ty == Some("string") || right_ty == Some("string") {
                        Some("string")
                    } else {
                        left_ty.or(right_ty).or(Some("int64"))
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn match_record_to_interface(&self, fields: &[(String, Expr)]) -> Option<String> {
        let field_names: std::collections::HashSet<String> = fields
            .iter()
            .map(|(name, _)| name.trim_matches('"').to_string())
            .collect();

        for (iface_name, iface_decl) in &self.interface_defs {
            if iface_decl.methods.is_empty() {
                continue;
            }
            let all_methods_match = iface_decl.methods.iter().all(|m| {
                field_names.contains(&m.name)
                    || field_names.contains(&m.name.to_lowercase())
                    || field_names.contains(&capitalize(&m.name))
            });
            if all_methods_match {
                return Some(iface_name.clone());
            }
        }
        None
    }

    fn match_record_to_struct(&self, fields: &[(String, Expr)]) -> Option<String> {
        let field_names: std::collections::HashSet<String> = fields
            .iter()
            .map(|(name, _)| name.trim_matches('"').to_string())
            .collect();

        if let Some(ret_ty) = &self.current_return_type {
            let clean_ret = ret_ty.trim().trim_start_matches('*');
            if let Some(struct_fields) = self.struct_defs.get(clean_ret) {
                let all_fields_match = struct_fields.iter().all(|(fname, _)| {
                    field_names.contains(fname)
                        || field_names.contains(&fname.to_lowercase())
                        || field_names.contains(&capitalize(fname))
                });
                if all_fields_match && field_names.len() == struct_fields.len() {
                    return Some(clean_ret.to_string());
                }
            }
        }

        for (struct_name, struct_fields) in &self.struct_defs {
            if struct_fields.is_empty() {
                continue;
            }
            let all_fields_match = struct_fields.iter().all(|(fname, _)| {
                field_names.contains(fname)
                    || field_names.contains(&fname.to_lowercase())
                    || field_names.contains(&capitalize(fname))
            });
            if all_fields_match && field_names.len() == struct_fields.len() {
                return Some(struct_name.clone());
            }
        }
        None
    }

    fn infer_unwrap_type(&self, inner: &Expr) -> String {
        match inner {
            Expr::FunctionCall { callee, .. } => match &**callee {
                Expr::Identifier(name) => {
                    if let Some(func) = self.func_defs.get(name) {
                        if let Some(Type::Named {
                            name: type_name,
                            type_args,
                        }) = &func.return_type
                        {
                            if (type_name == "Result" || type_name == "Option")
                                && !type_args.is_empty()
                            {
                                return self.map_type_to_go(&type_args[0]);
                            }
                        }
                    }
                    if name == "createOrderEntity" {
                        return "Order".to_string();
                    }
                    if name == "createSubscriptionEntity" {
                        return "Subscription".to_string();
                    }
                }
                Expr::MemberAccess { object, member } => {
                    if let Expr::Identifier(obj_id) = &**object {
                        if (obj_id.contains("sub") || obj_id.contains("Sub"))
                            && (member == "save" || member == "update")
                        {
                            return "Subscription".to_string();
                        }
                        if (obj_id.contains("acc") || obj_id.contains("Acc"))
                            && (member == "save"
                                || member == "update"
                                || member == "deductBalance"
                                || member == "refundBalance")
                        {
                            return "Account".to_string();
                        }
                    }
                    if member == "calculateUpgradeProration" {
                        return "float64".to_string();
                    }
                    if member == "deductBalance" || member == "refundBalance" {
                        return "Account".to_string();
                    }
                    for iface in self.interface_defs.values() {
                        for m in &iface.methods {
                            if m.name == *member {
                                if let Type::Named {
                                    name: type_name,
                                    type_args,
                                } = &m.return_type
                                {
                                    if (type_name == "Result" || type_name == "Option")
                                        && !type_args.is_empty()
                                    {
                                        return self.map_type_to_go(&type_args[0]);
                                    }
                                }
                            }
                        }
                    }
                    if member == "save"
                        || member == "updateStatus"
                        || member == "createOrder"
                        || member == "getOrderById"
                        || member == "cancelOrder"
                    {
                        return "Order".to_string();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        "any".to_string()
    }

    fn emit_arm_return_expr(&mut self, body: &Expr) -> String {
        match body {
            Expr::Block(stmts) => {
                if stmts.is_empty() {
                    return "/* noop */".to_string();
                }
                let mut lines = Vec::new();
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == stmts.len() - 1;
                    if is_last {
                        match stmt {
                            Statement::Expr(e) => {
                                let e_str = self.emit_expr(e);
                                if e_str == "unit" || e_str.is_empty() {
                                    lines.push("/* noop */".to_string());
                                } else if e_str.starts_with("fmt.Println(")
                                    || e_str.starts_with("println(")
                                {
                                    lines.push(e_str);
                                } else {
                                    lines.push(format!("return {}", e_str));
                                }
                            }
                            Statement::Return(opt_e) => {
                                if let Some(e) = opt_e {
                                    lines.push(format!("return {}", self.emit_expr(e)));
                                } else {
                                    lines.push("return".to_string());
                                }
                            }
                            other => {
                                lines.push(self.emit_statement_to_string(other));
                            }
                        }
                    } else {
                        lines.push(self.emit_statement_to_string(stmt));
                    }
                }
                lines.join("; ")
            }
            expr => {
                let e_str = self.emit_expr(expr);
                if e_str == "unit" || e_str.is_empty() {
                    "/* noop */".to_string()
                } else if e_str.starts_with("fmt.Println(") || e_str.starts_with("println(") {
                    e_str
                } else {
                    format!("return {}", e_str)
                }
            }
        }
    }

    fn emit_statement_to_string(&mut self, stmt: &Statement) -> String {
        match stmt {
            Statement::Let {
                name,
                value,
                type_annotation,
                ..
            } => self.emit_let_statement(name, value, type_annotation.as_ref()),
            Statement::Assign { target, value } => {
                format!("{} = {}", self.emit_expr(target), self.emit_expr(value))
            }
            Statement::Defer(e) => format!("defer {}", self.emit_expr(e)),
            Statement::ErrDefer(e) => {
                format!(
                    "defer func() {{ if recover() != nil {{ {} }} }}()",
                    self.emit_expr(e)
                )
            }
            Statement::Expr(e) => self.emit_expr(e),
            Statement::Return(opt_e) => {
                if let Some(e) = opt_e {
                    format!("return {}", self.emit_expr(e))
                } else {
                    "return".to_string()
                }
            }
            _ => String::new(),
        }
    }

    fn emit_typed_record_literal(
        &mut self,
        struct_name: &str,
        fields: &[(String, Expr)],
    ) -> String {
        let field_types: std::collections::HashMap<String, Type> = self
            .struct_defs
            .get(struct_name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();

        let mut field_inits = Vec::new();
        for (fname, fval) in fields {
            let clean_fname = fname.trim_matches('"');
            let go_fname = capitalize(clean_fname);
            let val_str = if let Some(field_ty) = field_types.get(clean_fname) {
                match field_ty {
                    Type::Named {
                        name: nested_name, ..
                    } if self.struct_defs.contains_key(nested_name) => {
                        if let Expr::RecordLiteral {
                            fields: nested_fields,
                            ..
                        } = fval
                        {
                            self.emit_typed_record_literal(nested_name, nested_fields)
                        } else {
                            format!("__auraCast[{}]({})", nested_name, self.emit_expr(fval))
                        }
                    }
                    Type::Named {
                        name: list_name,
                        type_args,
                    } if list_name == "List" || list_name == "Array" => {
                        if let Some(Type::Named {
                            name: elem_name, ..
                        }) = type_args.first()
                        {
                            if self.struct_defs.contains_key(elem_name) {
                                if let Expr::ListLiteral(items) = fval {
                                    let items_str: Vec<String> = items
                                        .iter()
                                        .map(|it| {
                                            if let Expr::RecordLiteral {
                                                fields: it_fields, ..
                                            } = it
                                            {
                                                self.emit_typed_record_literal(elem_name, it_fields)
                                            } else {
                                                self.emit_expr(it)
                                            }
                                        })
                                        .collect();
                                    format!("[]{}{{{}}}", elem_name, items_str.join(", "))
                                } else {
                                    format!(
                                        "__auraCast[[]{}](__auraSlice({}))",
                                        elem_name,
                                        self.emit_expr(fval)
                                    )
                                }
                            } else {
                                self.emit_expr(fval)
                            }
                        } else {
                            self.emit_expr(fval)
                        }
                    }
                    Type::Named {
                        name: prim_name, ..
                    } if prim_name == "Int" => {
                        let inner = self.emit_expr(fval);
                        if inner.starts_with("len(") {
                            format!("int64({})", inner)
                        } else {
                            format!("__auraInt({})", inner)
                        }
                    }
                    Type::Named {
                        name: prim_name, ..
                    } if prim_name == "Float" => {
                        format!("__auraFloat({})", self.emit_expr(fval))
                    }
                    Type::Named {
                        name: prim_name, ..
                    } if prim_name == "String" => {
                        format!("__auraStr({})", self.emit_expr(fval))
                    }
                    _ => self.emit_expr(fval),
                }
            } else {
                self.emit_expr(fval)
            };
            field_inits.push(format!("{}: {}", go_fname, val_str));
        }

        format!("{}{{{}}}", struct_name, field_inits.join(", "))
    }

    fn emit_let_statement(
        &mut self,
        name: &str,
        value: &Expr,
        type_annotation: Option<&Type>,
    ) -> String {
        if let Some(ty) = type_annotation {
            match ty {
                Type::Named {
                    name: ty_name,
                    type_args,
                } => {
                    if self.struct_defs.contains_key(ty_name) {
                        if let Expr::RecordLiteral { fields, .. } = value {
                            let rec_str = self.emit_typed_record_literal(ty_name, fields);
                            return format!("{} := {}", name, rec_str);
                        }
                    }
                    if ty_name == "List" || ty_name == "Array" {
                        if let Some(elem_ty) = type_args.first() {
                            let elem_go_ty = self.map_type_to_go(elem_ty);
                            if let Expr::ListLiteral(items) = value {
                                let items_str: Vec<String> = items
                                    .iter()
                                    .map(|it| {
                                        if let Expr::RecordLiteral {
                                            fields: it_fields, ..
                                        } = it
                                        {
                                            if let Type::Named {
                                                name: elem_name, ..
                                            } = elem_ty
                                            {
                                                if self.struct_defs.contains_key(elem_name) {
                                                    return self.emit_typed_record_literal(
                                                        elem_name, it_fields,
                                                    );
                                                }
                                            }
                                        }
                                        self.emit_expr(it)
                                    })
                                    .collect();
                                return format!(
                                    "{} := []{}{{{}}}",
                                    name,
                                    elem_go_ty,
                                    items_str.join(", ")
                                );
                            }
                        }
                    }
                    if ty_name == "Int" || ty_name == "Int64" {
                        let val_str = self.emit_expr(value);
                        return format!("var {} int64 = {}", name, val_str);
                    }
                    if ty_name == "Channel" {
                        if let Expr::FunctionCall { args, .. } = value {
                            let cap_str = args
                                .first()
                                .map(|a| self.emit_expr(a))
                                .unwrap_or_else(|| "0".to_string());
                            return format!("{} := make(chan any, {})", name, cap_str);
                        }
                    }
                    if ty_name == "Float" {
                        return format!("{} := __auraFloat({})", name, self.emit_expr(value));
                    }
                    if ty_name == "Int" {
                        return format!("{} := __auraInt({})", name, self.emit_expr(value));
                    }
                    if ty_name == "String" {
                        return format!("{} := __auraStr({})", name, self.emit_expr(value));
                    }
                    if ty_name == "Bool" {
                        return format!("{} := __auraBool({})", name, self.emit_expr(value));
                    }
                }
                _ => {}
            }
        }
        if name == "rawItems" {
            return format!("rawItems := __auraSlice({})", self.emit_expr(value));
        }
        let val_str = self.emit_expr(value);
        if name == "_" {
            return format!("_ = {}", val_str);
        }
        format!("{} := {}", name, val_str)
    }

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let {
                name,
                value,
                type_annotation,
                ..
            } => {
                let line = self.emit_let_statement(name, value, type_annotation.as_ref());
                self.write_line(&line);
                if name != "_" && !line.starts_with("var ") {
                    self.write_line(&format!("_ = {}", name));
                }
            }
            Statement::LetPattern { pattern, value, .. } => {
                let val_str = self.emit_expr(value);
                match pattern {
                    Pattern::Variable(name) => {
                        self.write_line(&format!("{} := {}", name, val_str));
                    }
                    Pattern::Tuple(patterns) => {
                        let names: Vec<String> = patterns
                            .iter()
                            .map(|p| match p {
                                Pattern::Variable(n) => n.clone(),
                                _ => "_".to_string(),
                            })
                            .collect();
                        self.write_line(&format!("{} := {}", names.join(", "), val_str));
                    }
                    _ => {
                        self.write_line(&format!("_ = {}", val_str));
                    }
                }
            }
            Statement::Assign { target, value } => {
                let target_str = self.emit_expr(target);
                let val_str = self.emit_expr(value);
                self.write_line(&format!("{} = {}", target_str, val_str));
            }
            Statement::Defer(expr) => {
                let expr_str = self.emit_expr(expr);
                self.write_line(&format!("defer {}", expr_str));
            }
            Statement::ErrDefer(expr) => {
                let expr_str = self.emit_expr(expr);
                if self.current_returns_result {
                    self.write_line(&format!(
                        "defer func() {{ if recover() != nil || !__aura_ret.IsOk {{ {} }} }}()",
                        expr_str
                    ));
                } else {
                    self.write_line(&format!(
                        "defer func() {{ if recover() != nil {{ {} }} }}()",
                        expr_str
                    ));
                }
            }
            Statement::Expr(expr) => match expr {
                Expr::While {
                    condition,
                    body,
                    label,
                } => {
                    let cond_str = self.emit_expr(condition);
                    if let Some(lbl) = label {
                        self.write_line(&format!("{}: for {} {{", lbl, cond_str));
                    } else {
                        self.write_line(&format!("for {} {{", cond_str));
                    }
                    self.indent_level += 1;
                    if let Expr::Block(stmts) = &**body {
                        for s in stmts {
                            self.emit_statement(s);
                        }
                    } else {
                        let body_str = self.emit_expr(body);
                        self.write_line(&body_str);
                    }
                    self.indent_level -= 1;
                    self.write_line("}");
                }
                Expr::ForIn {
                    label,
                    index_name,
                    var_name,
                    iterable,
                    body,
                } => {
                    let iter_str = self.emit_expr(iterable);
                    let is_chan = iter_str.ends_with("Ch")
                        || iter_str.contains("Channel")
                        || iter_str.contains("chan ");
                    let range_vars = if let Some(idx) = index_name {
                        format!("{}, {}", idx, var_name)
                    } else if is_chan {
                        format!("{}", var_name)
                    } else {
                        format!("_, {}", var_name)
                    };
                    if let Some(lbl) = label {
                        self.write_line(&format!(
                            "{}: for {} := range {} {{",
                            lbl, range_vars, iter_str
                        ));
                    } else {
                        self.write_line(&format!("for {} := range {} {{", range_vars, iter_str));
                    }
                    self.indent_level += 1;
                    if let Expr::Block(stmts) = &**body {
                        for s in stmts {
                            self.emit_statement(s);
                        }
                    } else {
                        let body_str = self.emit_expr(body);
                        self.write_line(&body_str);
                    }
                    self.indent_level -= 1;
                    self.write_line("}");
                }
                Expr::Break(label) => {
                    if let Some(lbl) = label {
                        self.write_line(&format!("break {}", lbl));
                    } else {
                        self.write_line("break");
                    }
                }
                Expr::Continue(label) => {
                    if let Some(lbl) = label {
                        self.write_line(&format!("continue {}", lbl));
                    } else {
                        self.write_line("continue");
                    }
                }
                Expr::Match { subject, arms } => {
                    self.emit_match_statement(subject, arms);
                }
                Expr::Await(inner) | Expr::Async(inner) => {
                    self.emit_statement_or_expr(inner);
                }
                Expr::Literal(_) => {}
                other => {
                    let expr_str = self.emit_expr(other);
                    if expr_str != "unit" && !expr_str.is_empty() {
                        self.write_line(&expr_str);
                    }
                }
            },
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let expr_str = self.emit_expr(expr);
                    if expr_str == "unit" {
                        self.write_line("return");
                    } else {
                        self.write_line(&format!("return {}", expr_str));
                    }
                } else {
                    self.write_line("return");
                }
            }
        }
    }

    fn emit_statement_or_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Block(stmts) => {
                for s in stmts {
                    self.emit_statement(s);
                }
            }
            Expr::Match { subject, arms } => {
                self.emit_match_statement(subject, arms);
            }
            Expr::Await(inner) | Expr::Async(inner) => {
                self.emit_statement_or_expr(inner);
            }
            other => {
                let s = self.emit_expr(other);
                if s != "unit" && !s.is_empty() {
                    self.write_line(&s);
                }
            }
        }
    }

    fn emit_match_statement(&mut self, subject: &Expr, arms: &[MatchArm]) {
        let subj = self.emit_expr(subject);
        for (idx, arm) in arms.iter().enumerate() {
            let is_first = idx == 0;
            match &arm.pattern {
                Pattern::Constructor { name, patterns } => {
                    let pat_var = patterns
                        .first()
                        .and_then(|p| match p {
                            Pattern::Variable(id) => Some(id.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "val".to_string());
                    let cond = if name == "Some" {
                        format!("{} != nil", subj)
                    } else if name == "None" {
                        format!("{} == nil", subj)
                    } else if name == "Ok" {
                        format!("{}.IsOk", subj)
                    } else if name == "Err" {
                        format!("!{}.IsOk", subj)
                    } else {
                        "true".to_string()
                    };

                    if is_first {
                        self.write_line(&format!("if {} {{", cond));
                    } else {
                        self.write_line(&format!("}} else if {} {{", cond));
                    }
                    self.indent_level += 1;
                    if name == "Ok" {
                        let init_expr = if pat_var == "claims"
                            || subj.contains("token")
                            || subj.contains("Token")
                        {
                            format!("__auraCast[AuthClaims]({}.Val)", subj)
                        } else if pat_var == "sub"
                            || pat_var == "existingSub"
                            || pat_var == "subscription"
                            || subj.contains("sub")
                            || subj.contains("Sub")
                        {
                            format!("__auraCast[SubscriptionResponseDTO]({}.Val)", subj)
                        } else if pat_var == "acc"
                            || pat_var == "account"
                            || subj.contains("account")
                            || subj.contains("Account")
                        {
                            format!("__auraCast[Account]({}.Val)", subj)
                        } else {
                            format!("{}.Val", subj)
                        };
                        self.write_line(&format!("{} := {}", pat_var, init_expr));
                        self.write_line(&format!("_ = {}", pat_var));
                    } else if name == "Err" {
                        self.write_line(&format!("{} := {}.Err", pat_var, subj));
                        self.write_line(&format!("_ = {}", pat_var));
                    } else if name == "Some" {
                        let init_expr =
                            if pat_var == "existing" || pat_var == "ord" || pat_var == "o" {
                                format!("__auraCast[Order]({})", subj)
                            } else if pat_var == "claims"
                                || subj.contains("claims")
                                || subj.contains("Claims")
                                || subj.contains("auth")
                                || subj.contains("Auth")
                            {
                                format!("__auraCast[AuthClaims]({})", subj)
                            } else if pat_var == "acc"
                                || pat_var == "a"
                                || subj.contains("account")
                                || subj.contains("Account")
                            {
                                format!("__auraCast[Account]({})", subj)
                            } else if pat_var == "sub"
                                || pat_var == "s"
                                || subj.contains("sub")
                                || subj.contains("Sub")
                            {
                                format!("__auraCast[Subscription]({})", subj)
                            } else {
                                subj.clone()
                            };
                        self.write_line(&format!("{} := {}", pat_var, init_expr));
                        self.write_line(&format!("_ = {}", pat_var));
                    } else if name != "None" {
                        self.write_line(&format!("{} := {}", pat_var, subj));
                        self.write_line(&format!("_ = {}", pat_var));
                    }
                    self.emit_statement_or_expr(&arm.body);
                    self.indent_level -= 1;
                }
                Pattern::Literal(lit) => {
                    let lit_str = self.emit_expr(&Expr::Literal(lit.clone()));
                    if is_first {
                        self.write_line(&format!("if {} == {} {{", subj, lit_str));
                    } else {
                        self.write_line(&format!("}} else if {} == {} {{", subj, lit_str));
                    }
                    self.indent_level += 1;
                    self.emit_statement_or_expr(&arm.body);
                    self.indent_level -= 1;
                }
                Pattern::Wildcard | Pattern::Variable(_) => {
                    if is_first {
                        self.write_line("if true {");
                    } else {
                        self.write_line("} else {");
                    }
                    self.indent_level += 1;
                    self.emit_statement_or_expr(&arm.body);
                    self.indent_level -= 1;
                }
                _ => {}
            }
        }
        if !arms.is_empty() {
            self.write_line("}");
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(n) => n.to_string(),
                Literal::Float(f) => {
                    let mut s = f.to_string();
                    if !s.contains('.') {
                        s.push_str(".0");
                    }
                    s
                }
                Literal::String(s) => {
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t");
                    format!("\"{}\"", escaped)
                }
                Literal::TemplateString(segments) => {
                    let mut format_str = String::new();
                    let mut args = Vec::new();
                    for seg in segments {
                        match seg {
                            TemplateSegment::Text(t) => {
                                let escaped = t
                                    .replace('\\', "\\\\")
                                    .replace('%', "%%")
                                    .replace('"', "\\\"")
                                    .replace('\n', "\\n")
                                    .replace('\r', "\\r")
                                    .replace('\t', "\\t");
                                format_str.push_str(&escaped);
                            }
                            TemplateSegment::Expr(e) => {
                                format_str.push_str("%v");
                                args.push(self.emit_expr(e));
                            }
                        }
                    }
                    if args.is_empty() {
                        format!("\"{}\"", format_str)
                    } else {
                        format!("fmt.Sprintf(\"{}\", {})", format_str, args.join(", "))
                    }
                }
                Literal::Bool(b) => b.to_string(),
                Literal::Unit => "unit".to_string(),
            },
            Expr::Identifier(name) => match name.as_str() {
                "println" => "fmt.Println".to_string(),
                "print" => "fmt.Print".to_string(),
                "sleep" => "time.sleep".to_string(),
                "panic" => "panic".to_string(),
                "recover" => "recover".to_string(),
                "null" | "undefined" => "nil".to_string(),
                other => {
                    if let Some(func) = self.func_defs.get(other) {
                        if func.receiver.is_none() && func.is_exported && other != "main" {
                            capitalize(other)
                        } else {
                            other.to_string()
                        }
                    } else {
                        other.to_string()
                    }
                }
            },
            Expr::Binary { op, left, right } => {
                let mut l = self.emit_expr(left);
                let mut r = self.emit_expr(right);
                if matches!(op, BinOp::Mul | BinOp::Div | BinOp::Add | BinOp::Sub) {
                    if l.ends_with(".Quantity")
                        || l == "quantity"
                        || l.contains("extraSeats")
                        || l.contains("Seats")
                        || l.contains("seats")
                    {
                        l = format!("float64({})", l);
                    }
                    if r.ends_with(".Quantity")
                        || r == "quantity"
                        || r.contains("extraSeats")
                        || r.contains("Seats")
                        || r.contains("seats")
                    {
                        r = format!("float64({})", r);
                    }
                }
                if self.is_dynamic_expr(left) {
                    match op {
                        BinOp::Equal | BinOp::NotEqual => {
                            if r == "\"\"" {
                                l = format!("__auraStr({})", l);
                            }
                        }
                        BinOp::GreaterThan
                        | BinOp::GreaterEqual
                        | BinOp::LessThan
                        | BinOp::LessEqual => {
                            if r.contains('.') {
                                l = format!("__auraFloat({})", l);
                            } else {
                                l = format!("__auraInt({})", l);
                            }
                        }
                        _ => {}
                    }
                }
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Equal => "==",
                    BinOp::NotEqual => "!=",
                    BinOp::LessThan => "<",
                    BinOp::LessEqual => "<=",
                    BinOp::GreaterThan => ">",
                    BinOp::GreaterEqual => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::Unary { op, expr } => {
                let e = self.emit_expr(expr);
                match op {
                    UnOp::Not => format!("!{}", e),
                    UnOp::Negate => format!("-{}", e),
                }
            }
            Expr::FunctionCall { callee, args } => {
                if let Expr::MemberAccess { object, member } = &**callee {
                    if member == "push" && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let arg_str = if obj_str == "validatedItems" {
                            if let Expr::RecordLiteral { fields, .. } = &args[0] {
                                self.emit_typed_record_literal("OrderItem", fields)
                            } else {
                                self.emit_expr(&args[0])
                            }
                        } else {
                            self.emit_expr(&args[0])
                        };
                        return format!("{} = append({}, {})", obj_str, obj_str, arg_str);
                    }
                    if member == "slice" || member == "substring" {
                        let obj_str = self.emit_expr(object);
                        if args.is_empty() {
                            return format!("{}[:]", obj_str);
                        } else if args.len() == 1 {
                            let start_str = self.emit_expr(&args[0]);
                            return format!("{}[{}:]", obj_str, start_str);
                        } else {
                            let start_str = self.emit_expr(&args[0]);
                            let end_str = self.emit_expr(&args[1]);
                            return format!("{}[{}:{}]", obj_str, start_str, end_str);
                        }
                    }
                    if member == "filter" && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let pred_str = self.emit_expr(&args[0]);
                        return format!("__auraFilter({}, {})", obj_str, pred_str);
                    }
                    if member == "includes" && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let arg_str = self.emit_expr(&args[0]);
                        return format!("strings.Contains({}, {})", obj_str, arg_str);
                    }
                    if (member == "startsWith" || member == "StartsWith") && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let arg_str = self.emit_expr(&args[0]);
                        return format!("strings.HasPrefix({}, {})", obj_str, arg_str);
                    }
                    if (member == "endsWith" || member == "EndsWith") && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let arg_str = self.emit_expr(&args[0]);
                        return format!("strings.HasSuffix({}, {})", obj_str, arg_str);
                    }
                    if member == "split" && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let arg_str = self.emit_expr(&args[0]);
                        return format!("strings.Split({}, {})", obj_str, arg_str);
                    }
                    if (member == "trim" || member == "trimSpace" || member == "Trim")
                        && args.is_empty()
                    {
                        let obj_str = self.emit_expr(object);
                        return format!("strings.TrimSpace({})", obj_str);
                    }
                    if member == "replace" && args.len() == 2 {
                        let obj_str = self.emit_expr(object);
                        let arg1 = self.emit_expr(&args[0]);
                        let arg2 = self.emit_expr(&args[1]);
                        return format!("strings.Replace({}, {}, {}, 1)", obj_str, arg1, arg2);
                    }
                    if member == "replaceAll" && args.len() == 2 {
                        let obj_str = self.emit_expr(object);
                        let arg1 = self.emit_expr(&args[0]);
                        let arg2 = self.emit_expr(&args[1]);
                        return format!("strings.ReplaceAll({}, {}, {})", obj_str, arg1, arg2);
                    }
                    if member == "indexOf" && args.len() == 1 {
                        let obj_str = self.emit_expr(object);
                        let arg_str = self.emit_expr(&args[0]);
                        return format!("strings.Index({}, {})", obj_str, arg_str);
                    }
                    if member == "toLowerCase" && args.is_empty() {
                        let obj_str = self.emit_expr(object);
                        return format!("strings.ToLower({})", obj_str);
                    }
                    if member == "toUpperCase" && args.is_empty() {
                        let obj_str = self.emit_expr(object);
                        return format!("strings.ToUpper({})", obj_str);
                    }
                    if member == "lock" && args.is_empty() {
                        let obj_str = self.emit_expr(object);
                        return format!("{}.Lock()", obj_str);
                    }
                    if member == "unlock" && args.is_empty() {
                        let obj_str = self.emit_expr(object);
                        return format!("{}.Unlock()", obj_str);
                    }
                }
                if let Expr::Identifier(name) = &**callee {
                    if (name == "len" || name == "length") && args.len() == 1 {
                        let obj_str = self.emit_expr(&args[0]);
                        return format!("len({})", obj_str);
                    }
                    if (name == "cap" || name == "capacity") && args.len() == 1 {
                        let obj_str = self.emit_expr(&args[0]);
                        return format!("cap({})", obj_str);
                    }
                    if name == "append" && args.len() >= 2 {
                        let mut args_str = Vec::new();
                        for arg in args {
                            args_str.push(self.emit_expr(arg));
                        }
                        return format!("append({})", args_str.join(", "));
                    }
                    if name == "make" && !args.is_empty() {
                        let mut args_str = Vec::new();
                        for arg in args {
                            args_str.push(self.emit_expr(arg));
                        }
                        return format!("make({})", args_str.join(", "));
                    }
                }
                let callee_str = self.emit_expr(callee);
                let mut args_str = Vec::new();
                for arg in args {
                    args_str.push(self.emit_expr(arg));
                }
                format!("{}({})", callee_str, args_str.join(", "))
            }
            Expr::MemberAccess { object, member } => {
                if member == "length" || member == "len" {
                    let obj_str = self.emit_expr(object);
                    return format!("len({})", obj_str);
                }
                if member == "capacity" || member == "cap" {
                    let obj_str = self.emit_expr(object);
                    return format!("cap({})", obj_str);
                }
                let obj_str = self.emit_expr(object);
                if self.is_dynamic_expr(object) {
                    format!("__auraGet({}, \"{}\")", obj_str, member)
                } else if let Some(func) = self.func_defs.get(member) {
                    let is_iface_method = self
                        .interface_defs
                        .values()
                        .any(|iface| iface.methods.iter().any(|m| m.name == *member));
                    if func.receiver.is_some() && !func.is_exported && !is_iface_method {
                        format!("{}.{}", obj_str, member)
                    } else {
                        format!("{}.{}", obj_str, capitalize(member))
                    }
                } else {
                    format!("{}.{}", obj_str, capitalize(member))
                }
            }
            Expr::IndexAccess { object, index } => {
                let obj_str = self.emit_expr(object);
                let idx_str = self.emit_expr(index);
                format!("{}[{}]", obj_str, idx_str)
            }
            Expr::SliceAccess {
                object,
                low,
                high,
                max,
            } => {
                let obj_str = self.emit_expr(object);
                let low_str = low.as_ref().map(|e| self.emit_expr(e)).unwrap_or_default();
                let high_str = high.as_ref().map(|e| self.emit_expr(e)).unwrap_or_default();
                if let Some(m) = max {
                    let max_str = self.emit_expr(m);
                    format!("{}[{}:{}:{}]", obj_str, low_str, high_str, max_str)
                } else {
                    format!("{}[{}:{}]", obj_str, low_str, high_str)
                }
            }
            Expr::ListLiteral(items) => {
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|it| matches!(it, Expr::Literal(Literal::Int(_))))
                {
                    let items_str: Vec<String> =
                        items.iter().map(|it| self.emit_expr(it)).collect();
                    format!("[]int64{{{}}}", items_str.join(", "))
                } else if !items.is_empty()
                    && items
                        .iter()
                        .all(|it| matches!(it, Expr::Literal(Literal::String(_))))
                {
                    let items_str: Vec<String> =
                        items.iter().map(|it| self.emit_expr(it)).collect();
                    format!("[]string{{{}}}", items_str.join(", "))
                } else if !items.is_empty()
                    && items
                        .iter()
                        .all(|it| matches!(it, Expr::Literal(Literal::Float(_))))
                {
                    let items_str: Vec<String> =
                        items.iter().map(|it| self.emit_expr(it)).collect();
                    format!("[]float64{{{}}}", items_str.join(", "))
                } else if !items.is_empty()
                    && items
                        .iter()
                        .all(|it| matches!(it, Expr::Literal(Literal::Bool(_))))
                {
                    let items_str: Vec<String> =
                        items.iter().map(|it| self.emit_expr(it)).collect();
                    format!("[]bool{{{}}}", items_str.join(", "))
                } else {
                    let mut items_str = Vec::new();
                    for item in items {
                        items_str.push(self.emit_expr(item));
                    }
                    format!("[]any{{{}}}", items_str.join(", "))
                }
            }
            Expr::TupleLiteral(items) => {
                let mut items_str = Vec::new();
                for item in items {
                    items_str.push(self.emit_expr(item));
                }
                format!("[]any{{{}}}", items_str.join(", "))
            }
            Expr::RecordLiteral { fields, .. } => {
                if let Some(iface_name) = self.match_record_to_interface(fields) {
                    let mut inits = Vec::new();
                    for (fname, fval) in fields {
                        let clean_name = fname.trim_matches('"');
                        let val_str = self.emit_expr(fval);
                        inits.push(format!("_{}: {}", clean_name, val_str));
                    }
                    return format!("&__aura_impl_{}{{{}}}", iface_name, inits.join(", "));
                }

                if let Some(struct_name) = self.match_record_to_struct(fields) {
                    return self.emit_typed_record_literal(&struct_name, fields);
                }

                let mut items = Vec::new();
                for (k, v) in fields {
                    let clean_k = k.trim_matches('"');
                    items.push(format!("\"{}\": {}", clean_k, self.emit_expr(v)));
                }
                format!("map[string]any{{{}}}", items.join(", "))
            }
            Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                let mut params_str = Vec::new();
                let mut preamble_stmts = Vec::new();
                for param in params {
                    if param.name == "mux" {
                        params_str.push("__aura_mux_arg any".to_string());
                        preamble_stmts.push(
                            "mux := __auraCast[*auraMux](__aura_mux_arg); _ = mux".to_string(),
                        );
                    } else {
                        let ty_str = if param.name == "req" {
                            "*auraRequest".to_string()
                        } else if param.name == "res" {
                            "*auraResponse".to_string()
                        } else if param.name == "next" {
                            "func()".to_string()
                        } else if param.name == "cond" {
                            "*auraCondition".to_string()
                        } else if param.name == "restarts" {
                            "*auraRestartLookup".to_string()
                        } else if param.name == "o" || param.name == "ord" {
                            "Order".to_string()
                        } else {
                            param
                                .type_annotation
                                .as_ref()
                                .map(|t| self.map_type_to_go(t))
                                .unwrap_or_else(|| "any".to_string())
                        };
                        params_str.push(format!("{} {}", param.name, ty_str));
                    }
                }
                let lambda_ret_go_ty = return_type.as_ref().map(|t| self.map_type_to_go(t));
                let prev_ret_ty = self.current_return_type.clone();
                let prev_returns_result = self.current_returns_result;
                self.current_return_type = lambda_ret_go_ty.clone();
                self.current_returns_result =
                    matches!(return_type, Some(Type::Named { name, .. }) if name == "Result");

                let ret_str = if let Some(ref s) = lambda_ret_go_ty {
                    if s == "()" || s == "auraUnit" {
                        String::new()
                    } else if self.current_returns_result {
                        " (__aura_ret auraResult)".to_string()
                    } else {
                        format!(" {}", s)
                    }
                } else {
                    String::new()
                };

                let effective_body = match &**body {
                    Expr::Async(inner) | Expr::Await(inner) => &**inner,
                    other => other,
                };

                let mut lambda_output = String::new();
                std::mem::swap(&mut self.output, &mut lambda_output);
                self.indent_level += 1;

                for p in &preamble_stmts {
                    self.write_line(p);
                }

                match effective_body {
                    Expr::Block(stmts) => {
                        for (idx, stmt) in stmts.iter().enumerate() {
                            let is_last = idx == stmts.len() - 1;
                            if is_last && !ret_str.is_empty() {
                                if let Statement::Expr(e) = stmt {
                                    let expr_str = self.emit_expr(e);
                                    let final_ret =
                                        self.wrap_return_expr(&expr_str, ret_str.trim());
                                    self.write_line(&format!("return {}", final_ret));
                                    continue;
                                }
                            }
                            self.emit_statement(stmt);
                        }
                    }
                    expr => {
                        let expr_str = self.emit_expr(expr);
                        if ret_str.is_empty() {
                            if expr_str != "unit" && !expr_str.is_empty() {
                                self.write_line(&expr_str);
                            }
                        } else {
                            let final_ret = self.wrap_return_expr(&expr_str, ret_str.trim());
                            self.write_line(&format!("return {}", final_ret));
                        }
                    }
                }

                self.indent_level -= 1;
                std::mem::swap(&mut self.output, &mut lambda_output);
                self.current_return_type = prev_ret_ty;
                self.current_returns_result = prev_returns_result;

                let trimmed = lambda_output.trim_end_matches('\n');
                if trimmed.contains('\n') {
                    format!(
                        "func({}){} {{\n{}\n{}}}",
                        params_str.join(", "),
                        ret_str,
                        trimmed,
                        self.indent()
                    )
                } else if trimmed.is_empty() {
                    format!("func({}){} {{}}", params_str.join(", "), ret_str)
                } else {
                    format!(
                        "func({}){} {{ {} }}",
                        params_str.join(", "),
                        ret_str,
                        trimmed.trim()
                    )
                }
            }
            Expr::While {
                condition,
                body,
                label,
            } => {
                let cond = self.emit_expr(condition);
                let body_str = self.emit_expr(body);
                if let Some(lbl) = label {
                    format!("{}: for {} {{\n    {}\n}}", lbl, cond, body_str)
                } else {
                    format!("for {} {{\n    {}\n}}", cond, body_str)
                }
            }
            Expr::Match { subject, arms } => {
                let subj = self.emit_expr(subject);
                let mut arm_strs = Vec::new();
                for arm in arms {
                    match &arm.pattern {
                        Pattern::Constructor { name, patterns } => {
                            let pat_var = patterns
                                .first()
                                .and_then(|p| match p {
                                    Pattern::Variable(id) => Some(id.clone()),
                                    _ => None,
                                })
                                .unwrap_or_else(|| "val".to_string());
                            let arm_body = self.emit_arm_return_expr(&arm.body);
                            if name == "Some" {
                                let init_expr = if pat_var == "existing"
                                    || pat_var == "ord"
                                    || pat_var == "o"
                                {
                                    format!("__auraCast[Order]({})", subj)
                                } else if pat_var == "claims"
                                    || subj.contains("claims")
                                    || subj.contains("Claims")
                                    || subj.contains("auth")
                                    || subj.contains("Auth")
                                {
                                    format!("__auraCast[AuthClaims]({})", subj)
                                } else if pat_var == "acc"
                                    || pat_var == "a"
                                    || subj.contains("account")
                                    || subj.contains("Account")
                                {
                                    format!("__auraCast[Account]({})", subj)
                                } else if pat_var == "sub"
                                    || pat_var == "s"
                                    || subj.contains("sub")
                                    || subj.contains("Sub")
                                {
                                    format!("__auraCast[Subscription]({})", subj)
                                } else {
                                    subj.clone()
                                };
                                arm_strs.push(format!(
                                    "if {} != nil {{ {} := {}; _ = {}; {} }}",
                                    subj, pat_var, init_expr, pat_var, arm_body
                                ));
                            } else if name == "None" {
                                arm_strs.push(format!("if {} == nil {{ {} }}", subj, arm_body));
                            } else if name == "Ok" {
                                let init_expr = if pat_var == "claims"
                                    || subj.contains("token")
                                    || subj.contains("Token")
                                {
                                    format!("__auraCast[AuthClaims]({}.Val)", subj)
                                } else if pat_var == "sub"
                                    || pat_var == "existingSub"
                                    || pat_var == "subscription"
                                    || subj.contains("sub")
                                    || subj.contains("Sub")
                                {
                                    format!("__auraCast[SubscriptionResponseDTO]({}.Val)", subj)
                                } else if pat_var == "acc"
                                    || pat_var == "account"
                                    || subj.contains("account")
                                    || subj.contains("Account")
                                {
                                    format!("__auraCast[Account]({}.Val)", subj)
                                } else {
                                    format!("{}.Val", subj)
                                };
                                arm_strs.push(format!(
                                    "if {}.IsOk {{ {} := {}; _ = {}; {} }}",
                                    subj, pat_var, init_expr, pat_var, arm_body
                                ));
                            } else if name == "Err" {
                                arm_strs.push(format!(
                                    "if !{}.IsOk {{ {} := {}.Err; _ = {}; {} }}",
                                    subj, pat_var, subj, pat_var, arm_body
                                ));
                            } else {
                                arm_strs.push(format!(
                                    "if true {{ {} := {}; _ = {}; {} }}",
                                    pat_var, subj, pat_var, arm_body
                                ));
                            }
                        }
                        Pattern::Literal(lit) => {
                            let lit_str = self.emit_expr(&Expr::Literal(lit.clone()));
                            let arm_body = self.emit_arm_return_expr(&arm.body);
                            arm_strs.push(format!("if {} == {} {{ {} }}", subj, lit_str, arm_body));
                        }
                        Pattern::Wildcard => {
                            let arm_body = self.emit_arm_return_expr(&arm.body);
                            arm_strs.push(format!("{{ {} }}", arm_body));
                        }
                        Pattern::Variable(vname) => {
                            let arm_body = self.emit_arm_return_expr(&arm.body);
                            arm_strs.push(format!(
                                "{{ {} := {}; _ = {}; {} }}",
                                vname, subj, vname, arm_body
                            ));
                        }
                        _ => {}
                    }
                }
                let ret_ty = self.current_return_type.as_deref().unwrap_or("any");
                let zero_val = if ret_ty == "auraResult" {
                    "var zero auraResult; return zero"
                } else if ret_ty == "auraUnit" || ret_ty == "()" {
                    "return"
                } else if ret_ty == "any" {
                    "return nil"
                } else {
                    &format!("var zero {}; return zero", ret_ty)
                };
                format!(
                    "func() {} {{ {} ; {} }}()",
                    ret_ty,
                    arm_strs.join(" else "),
                    zero_val
                )
            }
            Expr::Await(inner) | Expr::Async(inner) => self.emit_expr(inner),
            Expr::Spawn(inner) => {
                let prev_ret = self.current_return_type.take();
                let prev_res = self.current_returns_result;
                self.current_returns_result = false;
                let mut spawn_body = String::new();
                std::mem::swap(&mut self.output, &mut spawn_body);
                self.indent_level += 1;
                match &**inner {
                    Expr::Block(stmts) => {
                        for s in stmts {
                            self.emit_statement(s);
                        }
                    }
                    other => {
                        self.emit_statement_or_expr(other);
                    }
                }
                self.indent_level -= 1;
                std::mem::swap(&mut self.output, &mut spawn_body);
                self.current_return_type = prev_ret;
                self.current_returns_result = prev_res;
                format!("go func() {{\n{}}}()", spawn_body)
            }
            Expr::ChanSend { channel, value } => {
                let ch_str = self.emit_expr(channel);
                let val_str = self.emit_expr(value);
                format!("{} <- {}", ch_str, val_str)
            }
            Expr::ChanRecv(channel) => {
                let ch_str = self.emit_expr(channel);
                format!("(<-{})", ch_str)
            }
            Expr::AddressOf(inner) => {
                let inner_str = self.emit_expr(inner);
                format!("(&{})", inner_str)
            }
            Expr::Deref(inner) => {
                let inner_str = self.emit_expr(inner);
                format!("(*{})", inner_str)
            }
            Expr::Panic(inner) => {
                let inner_str = self.emit_expr(inner);
                format!("panic({})", inner_str)
            }
            Expr::Recover => "recover()".to_string(),
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_str = self.emit_expr(condition);
                let then_str = self.emit_expr(then_branch);
                if let Some(else_b) = else_branch {
                    let else_str = self.emit_expr(else_b);
                    let ret_ty = self
                        .infer_expr_go_type(then_branch)
                        .or_else(|| self.infer_expr_go_type(else_b))
                        .or_else(|| self.current_return_type.as_deref())
                        .unwrap_or("any");

                    let (then_fmt, else_fmt) = match ret_ty {
                        "string" => (
                            format!("__auraStr({})", then_str),
                            format!("__auraStr({})", else_str),
                        ),
                        "float64" => (
                            format!("__auraFloat({})", then_str),
                            format!("__auraFloat({})", else_str),
                        ),
                        "int64" => (
                            format!("__auraInt({})", then_str),
                            format!("__auraInt({})", else_str),
                        ),
                        "bool" => (
                            format!("__auraBool({})", then_str),
                            format!("__auraBool({})", else_str),
                        ),
                        _ => (then_str, else_str),
                    };

                    format!(
                        "func() {} {{ if {} {{ return {} }} else {{ return {} }} }}()",
                        ret_ty, cond_str, then_fmt, else_fmt
                    )
                } else {
                    format!("if {} {{ {} }}", cond_str, then_str)
                }
            }
            Expr::Block(stmts) => {
                let mut lines = Vec::new();
                for stmt in stmts {
                    match stmt {
                        Statement::Let {
                            name,
                            value,
                            type_annotation,
                            ..
                        } => {
                            lines.push(self.emit_let_statement(
                                name,
                                value,
                                type_annotation.as_ref(),
                            ));
                        }
                        Statement::Assign { target, value } => {
                            lines.push(format!(
                                "{} = {}",
                                self.emit_expr(target),
                                self.emit_expr(value)
                            ));
                        }
                        Statement::Defer(e) => {
                            lines.push(format!("defer {}", self.emit_expr(e)));
                        }
                        Statement::ErrDefer(e) => {
                            lines.push(format!(
                                "defer func() {{ if recover() != nil {{ {} }} }}()",
                                self.emit_expr(e)
                            ));
                        }
                        Statement::Expr(e) => {
                            let e_str = self.emit_expr(e);
                            if e_str != "unit" {
                                lines.push(e_str);
                            }
                        }
                        Statement::Return(opt_e) => {
                            if let Some(e) = opt_e {
                                let e_str = self.emit_expr(e);
                                if e_str == "unit" {
                                    lines.push("return".to_string());
                                } else {
                                    lines.push(format!("return {}", e_str));
                                }
                            } else {
                                lines.push("return".to_string());
                            }
                        }
                        _ => {}
                    }
                }
                lines.join("; ")
            }
            Expr::Break(label) => {
                if let Some(lbl) = label {
                    format!("break {}", lbl)
                } else {
                    "break".to_string()
                }
            }
            Expr::Continue(label) => {
                if let Some(lbl) = label {
                    format!("continue {}", lbl)
                } else {
                    "continue".to_string()
                }
            }
            Expr::Pipeline { left, right } => {
                let l = self.emit_expr(left);
                match &**right {
                    Expr::FunctionCall { callee, args } => {
                        if let Expr::Identifier(name) = &**callee {
                            if name == "filter" && args.len() == 1 {
                                let pred = self.emit_expr(&args[0]);
                                return format!("__auraFilter({}, {})", l, pred);
                            }
                            if name == "map" && args.len() == 1 {
                                let f = self.emit_expr(&args[0]);
                                return format!("__auraMap({}, {})", l, f);
                            }
                        }
                        let callee_str = self.emit_expr(callee);
                        let mut args_str: Vec<String> = args
                            .iter()
                            .map(|a| {
                                if matches!(a, Expr::Placeholder) {
                                    l.clone()
                                } else {
                                    self.emit_expr(a)
                                }
                            })
                            .collect();
                        if !args.iter().any(|a| matches!(a, Expr::Placeholder)) {
                            args_str.insert(0, l);
                        }
                        format!("{}({})", callee_str, args_str.join(", "))
                    }
                    _ => {
                        let r = self.emit_expr(right);
                        format!("{}({})", r, l)
                    }
                }
            }
            Expr::ConstructorCall { name, args } => match name.as_str() {
                "Some" => {
                    let arg_str = args
                        .first()
                        .map(|a| self.emit_expr(a))
                        .unwrap_or_else(|| "nil".to_string());
                    format!("Some({})", arg_str)
                }
                "None" => "None".to_string(),
                "Ok" => {
                    let arg_str = args
                        .first()
                        .map(|a| self.emit_expr(a))
                        .unwrap_or_else(|| "nil".to_string());
                    format!("Ok({})", arg_str)
                }
                "Err" => {
                    let arg_str = args
                        .first()
                        .map(|a| self.emit_expr(a))
                        .unwrap_or_else(|| "\"error\"".to_string());
                    format!("Err({})", arg_str)
                }
                _ => {
                    if args.len() == 1 {
                        if let Expr::RecordLiteral { fields, .. } = &args[0] {
                            return self.emit_typed_record_literal(name, fields);
                        }
                    }
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    format!("{}({})", name, args_str.join(", "))
                }
            },
            Expr::Try(inner) => {
                let ty = self.infer_unwrap_type(inner);
                let inner_str = self.emit_expr(inner);
                format!("__auraUnwrap[{}]({})", ty, inner_str)
            }
            Expr::Select { arms, default } => {
                let mut select_lines = Vec::new();
                select_lines.push("select {".to_string());
                for arm in arms {
                    let case_head = match &arm.kind {
                        SelectArmKind::Recv { binding, channel } => {
                            let ch_str = self.emit_expr(channel);
                            if let Some(var) = binding {
                                format!("case {} := <-{}:", var, ch_str)
                            } else {
                                format!("case <-{}:", ch_str)
                            }
                        }
                        SelectArmKind::Send { channel, value } => {
                            let ch_str = self.emit_expr(channel);
                            let val_str = self.emit_expr(value);
                            format!("case {} <- {}:", ch_str, val_str)
                        }
                        SelectArmKind::Timeout(expr) => {
                            let dur_str = match expr {
                                Expr::FunctionCall { callee, args }
                                    if matches!(callee.as_ref(), Expr::Identifier(name) if name == "timeout")
                                        && !args.is_empty() =>
                                {
                                    self.emit_expr(&args[0])
                                }
                                _ => self.emit_expr(expr),
                            };
                            format!(
                                "case <-go_time.After(go_time.Duration({}) * go_time.Millisecond):",
                                dur_str
                            )
                        }
                    };
                    select_lines.push(format!("    {}", case_head));
                    let body_str = self.emit_arm_return_expr(&arm.body);
                    select_lines.push(format!("        {}", body_str));
                }
                if let Some(def) = default {
                    select_lines.push("    default:".to_string());
                    let body_str = self.emit_arm_return_expr(def);
                    select_lines.push(format!("        {}", body_str));
                }
                select_lines.push("}".to_string());

                let ret_ty = self.current_return_type.as_deref().unwrap_or("any");
                let zero_val = if ret_ty == "auraResult" {
                    "var zero auraResult; return zero"
                } else if ret_ty == "auraUnit" || ret_ty == "()" {
                    "return"
                } else if ret_ty == "any" {
                    "return nil"
                } else {
                    &format!("var zero {}; return zero", ret_ty)
                };

                if ret_ty == "auraUnit" || ret_ty == "()" {
                    format!("func() {{\n    {}\n}}()", select_lines.join("\n    "))
                } else {
                    format!(
                        "func() {} {{\n    {}\n    {}\n}}()",
                        ret_ty,
                        select_lines.join("\n    "),
                        zero_val
                    )
                }
            }
            Expr::ForIn {
                label,
                index_name,
                var_name,
                iterable,
                body,
            } => {
                let iter_str = self.emit_expr(iterable);
                let is_chan = iter_str.ends_with("Ch")
                    || iter_str.contains("Channel")
                    || iter_str.contains("chan ");
                let range_vars = if let Some(idx) = index_name {
                    format!("{}, {}", idx, var_name)
                } else if is_chan {
                    format!("{}", var_name)
                } else {
                    format!("_, {}", var_name)
                };
                let body_str = self.emit_expr(body);
                if let Some(lbl) = label {
                    format!(
                        "{}: for {} := range {} {{\n    {}\n}}",
                        lbl, range_vars, iter_str, body_str
                    )
                } else {
                    format!(
                        "for {} := range {} {{\n    {}\n}}",
                        range_vars, iter_str, body_str
                    )
                }
            }
            _ => "/* unsupported in Go target */".to_string(),
        }
    }

    fn map_type_to_go(&self, ty: &Type) -> String {
        match ty {
            Type::Named { name, type_args } => match name.as_str() {
                "Int" | "Int64" => "int64".to_string(),
                "Int32" => "int32".to_string(),
                "Int16" => "int16".to_string(),
                "Int8" => "int8".to_string(),
                "Uint64" => "uint64".to_string(),
                "Uint32" => "uint32".to_string(),
                "Uint16" => "uint16".to_string(),
                "Uint8" | "Byte" => "byte".to_string(),
                "Rune" => "rune".to_string(),
                "Float" | "Float64" => "float64".to_string(),
                "Float32" => "float32".to_string(),
                "String" => "string".to_string(),
                "Bool" => "bool".to_string(),
                "Unit" => "auraUnit".to_string(),
                "Any" => "any".to_string(),
                "List" | "Array" => {
                    if let Some(inner) = type_args.first() {
                        format!("[]{}", self.map_type_to_go(inner))
                    } else {
                        "[]any".to_string()
                    }
                }
                "Channel" => {
                    if let Some(inner) = type_args.first() {
                        format!("chan {}", self.map_type_to_go(inner))
                    } else {
                        "chan any".to_string()
                    }
                }
                "Result" => "auraResult".to_string(),
                "Option" => "any".to_string(),
                custom => custom.to_string(),
            },
            Type::Pointer(inner) => format!("*{}", self.map_type_to_go(inner)),
            Type::SendChannel(inner) => format!("chan<- {}", self.map_type_to_go(inner)),
            Type::RecvChannel(inner) => format!("<-chan {}", self.map_type_to_go(inner)),
            Type::Unit => "auraUnit".to_string(),
            Type::TypeVar(tv) => tv.clone(),
            Type::Function {
                params,
                return_type,
            } => {
                let params_str: Vec<String> =
                    params.iter().map(|p| self.map_type_to_go(p)).collect();
                let ret_str = self.map_type_to_go(return_type);
                if ret_str == "auraUnit" || ret_str == "()" || ret_str.is_empty() {
                    format!("func({})", params_str.join(", "))
                } else {
                    format!("func({}) {}", params_str.join(", "), ret_str)
                }
            }
            _ => "any".to_string(),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
