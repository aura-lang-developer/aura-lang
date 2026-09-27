# Microservicios en Aura Language (Modelo Backend Golang)

Este directorio contiene una suite completa de **microservicios de producción** implementados en el lenguaje **Aura**, utilizando la sintaxis de **Structs con Struct Tags**, **Canales CSP (Go-style)**, **Fibers asíncronos (`spawn`)**, **Exclusión mutua (`Mutex`)** y el motor HTTP estándar (`net/http`).

Todos los microservicios son compilados por `aurac build` en **binarios nativos ejecutables autónomos** (sin requerir runtimes externos ni dependencias en el entorno de ejecución).

---

## Índice de Microservicios

| Microservicio | Archivo | Puerto por Defecto | Características Principales |
|---|---|---|---|
| **Orders Service** | [`orders_service.aura`](./orders_service.aura) | `:8081` | Ciclo de vida de órdenes de compra, cálculo de impuestos, despacho asíncrono con CSP queue, métricas en tiempo real. |
| **Auth Service** | [`auth_service.aura`](./auth_service.aura) | `:8082` | Registro, autenticación, tokens de sesión Bearer, canal de auditoría en segundo plano con protección de concurrencia. |
| **Telemetry Service** | [`telemetry_service.aura`](./telemetry_service.aura) | `:8083` | Ingesta masiva IoT, worker pool concurrente con canales tipados, detección y emisión de alertas de anomalías en tiempo real. |

---

## Modelado de Datos con `struct` y Struct Tags

Aura soporta la declaración de estructuras idéntica al modelo de Go, permitiendo definir tipos de datos con etiquetas (`struct tags`) para serialización JSON y validaciones:

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

## Compilación y Ejecución a Binario Autónomo

### 1. Verificar tipos estáticos
```bash
aurac check examples/microservices/orders_service.aura
aurac check examples/microservices/auth_service.aura
aurac check examples/microservices/telemetry_service.aura
```

### 2. Compilar a binario nativo ejecutable
```bash
# Compilar cada microservicio a su respectivo binario
aurac build examples/microservices/orders_service.aura -o dist/orders_service
aurac build examples/microservices/auth_service.aura -o dist/auth_service
aurac build examples/microservices/telemetry_service.aura -o dist/telemetry_service
```

### 3. Ejecutar los binarios generados
```bash
# Iniciar Orders Microservice en puerto 8081
PORT=8081 ./dist/orders_service

# Iniciar Auth Microservice en puerto 8082
PORT=8082 ./dist/auth_service

# Iniciar Telemetry Microservice en puerto 8083
PORT=8083 ./dist/telemetry_service
```

---

## Referencia de Endpoints y Pruebas con cURL

### 1. Orders Microservice (`:8081`)

- **Healthcheck:**
  ```bash
  curl http://localhost:8081/api/health
  ```
- **Listar órdenes:**
  ```bash
  curl http://localhost:8081/api/orders
  ```
- **Crear nueva orden (despacha evento a canal asíncrono):**
  ```bash
  curl -X POST http://localhost:8081/api/orders \
    -H "Content-Type: application/json" \
    -d '{
      "customer": {
        "name": "Carlos Mendoza",
        "email": "carlos@empresa.com",
        "address": "Av. Las Condes 400"
      },
      "items": [
        { "productId": "prod-10", "title": "Servidor ARM64", "quantity": 1, "unitPrice": 850.0 }
      ]
    }'
  ```
- **Métricas agregadas:**
  ```bash
  curl http://localhost:8081/api/orders/metrics
  ```

---

### 2. Auth Microservice (`:8082`)

- **Healthcheck:**
  ```bash
  curl http://localhost:8082/healthz
  ```
- **Registrar usuario:**
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
- **Iniciar sesión (Obtener token):**
  ```bash
  curl -X POST http://localhost:8082/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{
      "username": "admin",
      "password": "secret"
    }'
  ```
- **Ver logs de auditoría de seguridad:**
  ```bash
  curl http://localhost:8082/api/auth/audit
  ```

---

### 3. Telemetry Microservice (`:8083`)

- **Healthcheck & Estado de Workers:**
  ```bash
  curl http://localhost:8083/healthz
  ```
- **Ingestar lectura de sensor individual:**
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
  *(Nota: Al superar 75.0 °C, el worker fiber emitirá automáticamente una alerta crítica al canal de anomalías).*

- **Consultar alertas de anomalías:**
  ```bash
  curl http://localhost:8083/api/telemetry/alerts
  ```
- **Estadísticas agregadas:**
  ```bash
  curl http://localhost:8083/api/telemetry/stats
  ```

---

## Transpilación a Golang Nativo (`aurac emit-go`)

Cada microservicio también puede ser transpilado directamente a código Go:

```bash
aurac emit-go examples/microservices/orders_service.aura -o dist/orders_service.go
```
