# Microservices in Aura Language (Golang Backend Model)

This directory contains a full suite of **production microservices** implemented in the **Aura** language, utilizing **Structs with Struct Tags**, **CSP Channels (Go-style)**, **Asynchronous Fibers (`spawn`)**, **Mutual Exclusion (`Mutex`)**, and the standard HTTP engine (`net/http`).

All microservices are compiled by `aurac build` into **standalone native executable binaries** (without requiring external runtimes or dependencies in the execution environment).

---

## Microservices Index

| Microservice | File | Default Port | Key Features |
|---|---|---|---|
| **Orders Service** | [`orders_service.aura`](./orders_service.aura) | `:8081` | Purchase order lifecycle, tax calculations, asynchronous dispatch via CSP queue, real-time metrics. |
| **Auth Service** | [`auth_service.aura`](./auth_service.aura) | `:8082` | Registration, authentication, Bearer session tokens, background audit logging channel with concurrency protection. |
| **Telemetry Service** | [`telemetry_service.aura`](./telemetry_service.aura) | `:8083` | High-throughput IoT ingestion, concurrent worker pool with typed channels, real-time anomaly detection and alert emission. |

---

## Data Modeling with `struct` and Struct Tags

Aura supports structure declarations identical to Go's model, allowing developers to define data types with struct tags for JSON serialization and validation:

```aura
struct Order {
    id: String `json:"id"`,
    customer: CustomerInfo `json:"customer"`,
    items: List<OrderItem> `json:"items"`,
    subtotal: Float `json:"subtotal"`,
    taxAmount: Float `json:"tax_amount"`,
    total: Float `json:"total"`,
    status: String `json:"status"`,
    createdAt: String `json:"created_at"`
};
```

---

## Standalone Binary Compilation and Execution

### 1. Static Type Checking
```bash
aurac check examples/microservices/orders_service.aura
aurac check examples/microservices/auth_service.aura
aurac check examples/microservices/telemetry_service.aura
```

### 2. Compile to Standalone Native Binary
```bash
# Compile each microservice to its respective binary
aurac build examples/microservices/orders_service.aura -o dist/orders_service
aurac build examples/microservices/auth_service.aura -o dist/auth_service
aurac build examples/microservices/telemetry_service.aura -o dist/telemetry_service
```

### 3. Run Generated Binaries
```bash
# Start Orders Microservice on port 8081
PORT=8081 ./dist/orders_service

# Start Auth Microservice on port 8082
PORT=8082 ./dist/auth_service

# Start Telemetry Microservice on port 8083
PORT=8083 ./dist/telemetry_service
```

---

## Endpoints Reference and cURL Testing

### 1. Orders Microservice (`:8081`)

- **Healthcheck:**
  ```bash
  curl http://localhost:8081/api/health
  ```
- **List orders:**
  ```bash
  curl http://localhost:8081/api/orders
  ```
- **Create new order (dispatches event to asynchronous channel):**
  ```bash
  curl -X POST http://localhost:8081/api/orders \
    -H "Content-Type: application/json" \
    -d '{
      "customer": {
        "name": "Carlos Mendoza",
        "email": "carlos@company.com",
        "address": "400 Las Condes Ave"
      },
      "items": [
        { "productId": "prod-10", "title": "ARM64 Server", "quantity": 1, "unitPrice": 850.0 }
      ]
    }'
  ```
- **Aggregate metrics:**
  ```bash
  curl http://localhost:8081/api/orders/metrics
  ```

---

### 2. Auth Microservice (`:8082`)

- **Healthcheck:**
  ```bash
  curl http://localhost:8082/healthz
  ```
- **Register user:**
  ```bash
  curl -X POST http://localhost:8082/api/auth/register \
    -H "Content-Type: application/json" \
    -d '{
      "username": "martin_dev",
      "email": "martin@cloud.io",
      "password": "SuperSecretPass2026",
      "role": "admin"
    }'
  ```
- **Log in (Get token):**
  ```bash
  curl -X POST http://localhost:8082/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{
      "username": "admin",
      "password": "secret"
    }'
  ```
- **View security audit logs:**
  ```bash
  curl http://localhost:8082/api/auth/audit
  ```

---

### 3. Telemetry Microservice (`:8083`)

- **Healthcheck & Worker Status:**
  ```bash
  curl http://localhost:8083/healthz
  ```
- **Ingest single sensor reading:**
  ```bash
  curl -X POST http://localhost:8083/api/telemetry/ingest \
    -H "Content-Type: application/json" \
    -d '{
      "deviceId": "sensor-temp-factory-1",
      "sensorType": "temperature",
      "value": 82.5,
      "unit": "celsius"
    }'
  ```
  *(Note: Exceeding 75.0 °C causes the worker fiber to emit a critical alert to the anomalies channel).*

- **Query anomaly alerts:**
  ```bash
  curl http://localhost:8083/api/telemetry/alerts
  ```
- **Aggregate statistics:**
  ```bash
  curl http://localhost:8083/api/telemetry/stats
  ```

---

## Native Golang Transpilation (`aurac emit-go`)

Each microservice can also be transpiled directly to pure Go code:

```bash
aurac emit-go examples/microservices/orders_service.aura -o dist/orders_service.go
```
