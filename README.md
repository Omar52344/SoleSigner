# 🗳️ SoleSigner: Digital Voting Sovereignty

> **Sistema de Votación Digital Auditable, Soberano y Resistente a la Censura.**

![Rust](https://img.shields.io/badge/Backend-Rust-black?style=for-the-badge&logo=rust) ![Next.js](https://img.shields.io/badge/Frontend-Next.js-black?style=for-the-badge&logo=next.js) ![Postgres](https://img.shields.io/badge/DB-PostgreSQL-blue?style=for-the-badge&logo=postgresql) ![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

**SoleSigner** no es solo una app de encuestas; es una infraestructura crítica para la democracia digital. Diseñado bajo los principios de **Zero-Knowledge** y **Habeas Data**, permite elecciones seguras donde:

1.  **Nadie sabe por quién votaste** (ni siquiera el administrador).
2.  **Todos pueden verificar** que su voto fue contado (prueba matemática vía Merkle Trees).
3.  **Una persona = Un voto** (garantizado por biometría local y Nullifiers).

---

## 🏛️ Arquitectura de Alto Nivel

El sistema desacopla la **Identidad** de la **Intención de Voto** para garantizar el secreto absoluto.

### 1. El Backend (Rust Core) 🦀
El corazón del sistema. Un binario único, compilado estáticamente, sin dependencia de nubes propietarias.
*   **Framework**: `Axum` + `Tokio` para concurrencia masiva.
*   **Identidad IA**: Motor `ONNX Runtime` integrado para validación facial y liveness *in-memory* (las fotos nunca tocan el disco).
*   **Criptografía**: Implementación nativa de árboles de Merkle y firmas Ed25519.
*   **Scheduler**: Cierre y sellado automático de urnas.

### 2. El Frontend (Next.js) ⚡
Una interfaz moderna y reactiva diseñada para la transparencia.
*   **Client-Side Computing**: `js-sha256` para calcular hashes en el navegador del usuario.
*   **Biometría Web**: Captura inteligente de documentos y selfies.
*   **Auditabilidad**: Herramientas offline para verificar recibos de votación.

---

## 🔐 Pilares de Seguridad

| Característica | Descripción Técnica |
| :--- | :--- |
| **Recibos Criptográficos** | Cada votante recibe un JSON con un `ballot_hash` y un `merkle_path`. Permite probar matemáticamente que el voto es parte del `Root Hash` final. |
| **Identidad sin Rastros** | Usamos **Nullifiers** (`SHA256(Doc + Salt)`). El sistema sabe *que* votaste, pero olvida *quién* eres inmediatamente después de validar. |
| **Urnas Selladas** | Al cerrar la votación, se genera un Merkle Root inmutable. Cualquier alteración en la base de datos rompería la cadena de pruebas de todos los votantes. |
| **Geofencing** | Validación de coordenadas GPS para limitar votaciones a zonas físicas específicas. |

---

## 🚀 Quick Start

### Requisitos Previos
*   Rust (Cargo)
*   Node.js & npm
*   PostgreSQL

### 1. Configuración del Entorno
Clona el repositorio y configura las variables de entorno.
```bash
# En el root del proyecto
cp .env.example .env
# Edita DATABASE_URL=postgres://user:pass@localhost:5432/solesigner
```

### 2. Levantar el Backend (Rust)
```bash
# Instalar dependencias y preparar la base de datos
cargo sqlx migrate run

# Iniciar el servidor (Puerto 8080)
cargo run
```

### 3. Iniciar la Interfaz (Frontend)
```bash
cd frontend
npm install
npm run dev
# Abre http://localhost:3000
```

---

## 🕵️‍♂️ Cómo Auditar una Elección

SoleSigner empodera al usuario. No necesitas confiar en nosotros.

1.  Vota y descarga tu **Recibo Digital** (`receipt.json`).
2.  Ve a la sección `/verify` del frontend (o usa el script CLI).
3.  Carga tu recibo.
4.  El sistema recalculará la ruta del Árbol de Merkle localmente.
5.  **Si el hash coincide con el `Root Hash` público de la elección, tu voto es inmutable.**

> *"Democracy dies in darkness. We turn on the lights."*

---

## 📂 Estructura del Proyecto

```
SoleSigner/
├── src/
│   ├── api/        # Endpoints REST (Axum)
│   ├── crypto/     # Merkle Trees & Hashing
│   ├── identity/   # ONNX Face Matching Logic
│   └── scheduler/  # Cron jobs para sellado de urnas
├── migrations/     # Esquema SQL (SQLx)
├── frontend/       # Next.js App Router UI
└── Dockerfile      # Despliegue Distroless (Seguridad militar)
```

## 📜 Licencia
Open Source MIT. Construido para la comunidad.
