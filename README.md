# 🗳️ SoleSigner: Digital Voting Sovereignty

> **MVP Minimalista Serverless – Sistema de Votación Digital Auditable, Soberano y de Bajo Coste Computacional.**

![Rust](https://img.shields.io/badge/Backend-Rust-black?style=for-the-badge&logo=rust) ![Next.js](https://img.shields.io/badge/Frontend-Next.js-black?style=for-the-badge&logo=next.js) ![Postgres](https://img.shields.io/badge/DB-PostgreSQL-blue?style=for-the-badge&logo=postgresql) ![Serverless](https://img.shields.io/badge/Deployment-Serverless-orange?style=for-the-badge&logo=serverless) ![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

**SoleSigner MVP** es una implementación robusta, modular y tipada lista para producción, diseñada para despliegue serverless con coste mensual mínimo. Mantiene los pilares de **privacidad** y **auditabilidad** pero simplifica la validación de identidad para un MVP funcional:

1.  **Nadie sabe por quién votaste** (ni siquiera el administrador).
2.  **Todos pueden verificar** que su voto fue contado (prueba matemática vía Merkle Trees + firmas Ed25519).
3.  **Una persona = Un voto** (garantizado por nullifiers derivados del número de documento).

---

## 🏛️ Arquitectura Actual (MVP Serverless)

El sistema sigue desacoplando **Identidad** de **Intención de Voto**, pero reemplaza la validación biométrica por un flujo simplificado que reduce la complejidad y el coste computacional.

### 1. El Backend (Rust Core) 🦀
Un API modular y fuertemente tipada, lista para despliegue en Fly.io, Railway o cualquier plataforma serverless.

*   **Framework**: `Axum` + `Tokio` + `SQLx` (con tipos fuertes para PostgreSQL).
*   **Identidad Simplificada**: Validación por número de documento; nullifiers generados con `SHA256(Doc + Salt)`.
*   **Criptografía**: Árboles de Merkle, firmas Ed25519 por elección (derivadas de un secreto maestro + salt), y JWT para tokens de identidad.
*   **Transacciones Atómicas**: Inserción de nullifier y voto en una única transacción de base de datos (garantiza “una persona, un voto”).
*   **Observabilidad**: Logs estructurados con `tracing`, request‑ID, y capa `tower‑http` para trazabilidad.
*   **Scheduler**: Cierre y sellado automático de urnas, cálculo del Merkle Root final.

### 2. El Frontend (Next.js) ⚡
Interfaz moderna que oculta los componentes biométricos y guía al usuario con un flujo simplificado de dos pasos.

*   **Client-Side Hashing**: `js-sha256` para calcular hashes en el navegador.
*   **UI Adaptada**: Paso 1: capturar documento (UI presente pero deshabilitada). Paso 2: confirmar número de documento.
*   **Verificación de Recibos**: Herramienta para validar firmas Ed25519 de los recibos de votación.

### 3. Infraestructura Serverless 🚀
*   **Dockerfile multi‑etapa** basado en `distroless/cc‑debian12` (imagen mínima, ~30 MB).
*   **Despliegue en Fly.io, Railway, Heroku** con variables de entorno inyectadas.
*   **Coste mensual mínimo**: Sin procesamiento biométrico pesado; solo transacciones de base de datos y lógica de negocio ligera.

---

## 🔐 Pilares de Seguridad (MVP)

| Característica | Descripción Técnica |
| :--- | :--- |
| **Recibos Firmados con Ed25519** | Cada votante recibe un JSON con `ballot_hash`, `public_key`, `signed_data` y `signature`. La clave se deriva del secreto maestro + salt de la elección, permitiendo verificación offline. |
| **Identidad sin Rastros (Nullifiers)** | `SHA256(NúmeroDocumento + ElectionSalt)`. El sistema registra solo el hash, nunca el documento original. |
| **Urnas Selladas con Merkle Root** | Al cerrar la elección, se calcula un Merkle Root inmutable a partir de todos los `ballot_hash`. Cualquier alteración rompería la cadena de pruebas. |
| **Transacciones Atómicas** | Inserción del nullifier y del voto se hacen dentro de una misma transacción SQL: o ambos éxitos o ninguno (evita votos duplicados). |
| **Validación de Entrada Estructurada** | DTOs con `validator` y enums fuertemente tipados (`ElectionStatus`, `AccessType`). |
| **Protección contra Inyección SQL y Replay** | Consultas parametrizadas (SQLx) y registro único de nullifiers (clave única en base de datos). |

---

## 🚀 Quick Start (Desarrollo Local)

### Requisitos Previos
*   Rust (Cargo) 1.76+
*   Node.js 18+ & npm
*   PostgreSQL 15+

### 1. Configuración del Entorno
```bash
git clone https://github.com/tu‑org/SoleSigner.git
cd SoleSigner
cp .env.example .env
# Edita DATABASE_URL, JWT_SECRET, etc.
```

### 2. Base de Datos y Migraciones
```bash
# Ejecuta migraciones (las migraciones se ejecutan automáticamente al iniciar la app)
cargo sqlx migrate run
```

### 3. Levantar el Backend
```bash
cargo run
# API disponible en http://localhost:8080
# Endpoints OpenAPI: /openapi.json, /openapi.yaml
```

### 4. Iniciar el Frontend
```bash
cd frontend
npm install
npm run dev
# Abre http://localhost:3000
```

---

## 🧪 Testing & Calidad

El proyecto incluye un suite de pruebas integral y herramientas de calidad:

```bash
# Ejecutar todas las pruebas (6 pruebas de integración)
cargo test --workspace

# Ejecutar pruebas de integración específicas
cargo test --test integration

# Linter y formateo
cargo clippy --workspace -- -D warnings
cargo fmt -- --check

# Stress test (100 votos concurrentes)
cargo run --bin stress
```

**Pruebas implementadas**:
- Registro y login de administrador
- Creación y apertura de elecciones
- Flujo completo de votación (validación de identidad + emisión de voto)
- Protección contra replay (nullifier duplicado → 409 Conflict)
- Intento de inyección SQL (no debe causar crash)

---

## 📊 Flujo de Voto Simplificado (MVP)

1.  **Paso 1 – Validar Identidad**
    - Usuario introduce su número de documento.
    - Backend verifica whitelist (si la elección es privada), genera nullifier y devuelve un JWT de identidad (válido 5 minutos).

2.  **Paso 2 – Emitir Voto**
    - Usuario selecciona opciones en la papeleta.
    - Frontend envía `choices`, `nullifier` y `request_id`.
    - Backend:
        - Verifica que la elección esté abierta y que el nullifier no haya sido usado (transacción atómica).
        - Inserta nullifier en `voter_registry` y voto en `ballots`.
        - Genera recibo firmado con Ed25519 (clave derivada del secreto + salt de elección).

3.  **Post‑Voto – Verificación**
    - Usuario descarga recibo JSON (`ballot_hash`, `public_key`, `signature`, etc.).
    - Puede verificar la firma en cualquier momento con el endpoint `/vote/verify‑receipt` o con herramientas offline.

---

## 🚀 Despliegue en Producción

### Opciones Serverless (Recomendadas)
1.  **Fly.io**: `fly launch --no‑deploy`, `fly secrets set DATABASE_URL=...`, `fly deploy`.
2.  **Railway**: Conectar repositorio, añadir servicio PostgreSQL, setear variables de entorno.
3.  **Heroku**: `heroku container:push web`, `heroku container:release web`.

### Scripts de Operación
- `scripts/backup_db.sh` – Backup comprimido con retención de 7 días.
- `.github/workflows/ci.yml` – CI con PostgreSQL, tests, clippy y fmt.

### Monitoreo
- Health endpoint: `GET /health` (para load‑balancers).
- Logs estructurados (JSON si `JSON_LOGS=true`).
- Métricas de request/response con `tower‑http`.

---

## 📂 Estructura del Proyecto (Actualizada)

```
SoleSigner/
├── src/
│   ├── api/                    # Handlers modulares (auth, elections, vote, whitelist, results, health)
│   ├── crypto/                 # Merkle Trees, hashing, firmas Ed25519, nullifiers
│   ├── services/               # Lógica de negocio (VoteService)
│   ├── repositories/           # Patrón repositorio (ElectionRepository)
│   ├── scheduler/              # Cron job para sellar elecciones
│   ├── config.rs               # Configuración centralizada
│   ├── error.rs                # ApiError y ApiResult (unified error handling)
│   └── types.rs                # Enums fuertes (ElectionStatus, AccessType)
├── tests/
│   └── integration.rs          # 6 pruebas de integración (auth, elections, vote flow, replay, SQLi)
├── frontend/
│   ├── app/vote/[election_id]/ # UI de votación simplificada (sin biometría)
│   ├── lib/utils.ts            # Fetcher con manejo de ApiError
│   └── components/             # Componentes React (LanguageProvider actualizado)
├── migrations/                 # Esquema SQL (SQLx)
├── scripts/
│   └── backup_db.sh            # Backup automatizado de BD
├── .github/workflows/
│   └── ci.yml                  # CI con PostgreSQL, tests, clippy, fmt
├── Dockerfile                  # Multi‑stage build → imagen distroless
├── DEPLOYMENT.md               # Guía de despliegue (Fly.io, Railway, Heroku)
├── API.md                      # Referencia completa de endpoints
├── SECURITY_CHECKLIST.md       # Checklist de seguridad para producción
├── openapi.yaml                # Especificación OpenAPI 3.0
└── tareas.md                   # Plan de 15 días (MVP minimalista serverless)
```

---

## 📜 Estado Actual & Roadmap

### ✅ **Completado (MVP Minimalista)**
- [x] **Días 1‑10**: Error handling unificado, configuración centralizada, modularización, transacciones atómicas, tipado fuerte, identidad simplificada, tokens JWT, recibos Ed25519, sincronización frontend.
- [x] **Días 11‑12**: Infraestructura serverless (Dockerfile, CI, scripts de despliegue).
- [x] **Días 13‑14**: Testing integral (6 pruebas), stress test (100 votos concurrentes), seguridad (replay, SQLi), documentación (API.md, DEPLOYMENT.md), backup automatizado.

### 🚀 **Próximos Pasos (Post‑MVP)**
- [ ] Integración con servicios de identidad gubernamental (opcional).
- [ ] Dashboard de administración enriquecido.
- [ ] Soporte para votación por correo (QR‑code receipts).
- [ ] Plugin de verificación offline (CLI tool).

---

## 📄 Licencia
MIT – Construido para la comunidad, con transparencia y auditabilidad como principios fundamentales.

> *"Democracy dies in darkness. We turn on the lights."*