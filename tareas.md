🚀 Plan MVP Minimalista: Serverless y Bajo Costo

## Cambios Clave:
1. **Eliminación de validación biométrica ONNX**: Se mantiene el frontend pero se reemplaza con validación simple por documento.
2. **Enfoque serverless**: Optimización para bajo consumo computacional mensual.
3. **Mantener todas las mejoras de robustez**: Manejo de errores, modularización, transacciones, tipado fuerte.

---

## Día 1-2: Cimentación y Errores
**Objetivo**: Establecer base sólida de manejo de errores y configuración.

### Tareas:
- [ ] Crear `src/error.rs` con `ApiError` enum e implementar `IntoResponse`
- [ ] Reemplazar primeros 50 `unwrap()`/`expect()` por operador `?` y `ApiError`
- [ ] Configurar variables obligatorias: `JWT_SECRET`, `DATABASE_URL` (fallar al inicio si faltan)
- [ ] Implementar costo de bcrypt configurable desde entorno (`BCRYPT_COST`)
- [ ] Eliminar/commentar dependencias no usadas: `ort`, `image` (mantener en Cargo.toml pero no usarlas)
- [ ] Crear `config.rs` para centralizar configuración

---

## Día 3-4: Modularización Estructural
**Objetivo**: Dividir monolito en módulos mantenibles.

### Tareas:
- [ ] Dividir `src/api/mod.rs` en:
  - `auth.rs` (registro, login, JWT)
  - `elections.rs` (crear, listar, iniciar, cerrar, stats)
  - `vote.rs` (validación, elegibilidad, submit)
  - `whitelist.rs` (gestión lista blanca)
  - `results.rs` (resultados, auditoría)
- [ ] Crear `ElectionRepository` para centralizar consultas SQL de elecciones
- [ ] Crear `VoteService` para lógica de votación
- [ ] Refactorizar router principal (`src/api/mod.rs`) para orquestar módulos

---

## Día 5: Transacciones e Integridad
**Objetivo**: Garantizar atomicidad en operaciones críticas.

### Tareas:
- [ ] Implementar transacción DB en `submit_vote` (nullifier + ballot atómicos)
- [ ] Validar estado de elección: solo permitir votos si `status = 'OPEN'` y `NOW() BETWEEN start_date AND end_date`
- [ ] Validar unicidad de nullifier por elección (ya existe en DB)
- [ ] Crear función helper `check_election_open(election_id)` reutilizable

---

## Día 6: Tipado Fuerte y Validación
**Objetivo**: Mejorar seguridad con tipos y validación.

### Tareas:
- [ ] Crear enums `AccessType` y `ElectionStatus` con `sqlx::Type`
- [ ] Actualizar DTOs (`CreateElectionRequest`, etc.) para usar enums
- [ ] Validar `start_date < end_date` en creación de elección
- [ ] Validar formato básico de `form_config` (JSON con campos mínimos)
- [ ] Usar `validator` crate para validar longitud de username/password

---

## Día 7: Identidad Simplificada
**Objetivo**: Reemplazar validación biométrica por flujo simple.

### Tareas:
- [ ] Modificar `validate_identity` para aceptar solo `document_number` (eliminar selfie_base64, document_base64)
- [ ] Generar nullifier con `crypto::generate_nullifier(document_number, election_salt)`
- [ ] Mantener validación de whitelist para elecciones privadas
- [ ] Opcional: validar formato de documento (ej: DNI regex)
- [ ] Actualizar frontend para enviar solo documento (ocultar/mantener campos selfie/documento)

---

## Día 8: Tokens y Verificabilidad
**Objetivo**: Implementar tokens reales y verificabilidad.

### Tareas:
- [ ] Reemplazar `"VALID_TOKEN_PLACEHOLDER"` por JWT real que firme nullifier + timestamp
- [ ] Implementar recibo firmado con clave de elección (usar Ed25519)
- [ ] Crear endpoint para verificar recibo (`/verify-receipt`)
- [ ] Test unitario: verificar que voto alterado rompe Merkle root

---

## Día 9: Frontend Sync
**Objetivo**: Adaptar frontend a cambios de API.

### Tareas:
- [ ] Actualizar hooks de API para manejar nuevo formato de `ApiError`
- [ ] Implementar visualización de recibo de votación (Merkle proof)
- [ ] Refactorizar UI para usar nuevos enums de estado de elección
- [ ] Ocultar/deshabilitar componentes de captura biométrica
- [ ] Añadir campo simple de "Número de documento" en flujo de votación

---

## Día 10: Observabilidad y Logging
**Objetivo**: Implementar monitoreo básico.

### Tareas:
- [ ] Configurar `tracing-subscriber` con salida estructurada (JSON para producción)
- [ ] Añadir logs críticos: `ADMIN_ACTION`, `VOTE_CAST`, `ELECTION_SEALED`
- [ ] Middleware de trazabilidad: asignar `request_id` a cada petición
- [ ] Loggear métricas básicas (tiempos de respuesta, conteo de votos)

---

## Día 11-12: Infraestructura Minimalista
**Objetivo**: Preparar para despliegue de bajo costo.

### Tareas:
- [ ] Crear Dockerfile multi-stage optimizado (<50MB imagen final)
- [ ] Configurar GitHub Actions para CI: tests, clippy, security audit
- [ ] Configurar migraciones automáticas al iniciar app
- [ ] Crear script de despliegue serverless (ej: para Fly.io o Railway)
- [ ] Configurar variables de entorno para producción

---

## Día 13: Pruebas y Seguridad
**Objetivo**: Validar robustez y seguridad básica.

### Tareas:
- [ ] Tests de integración para flujos principales (registro, creación elección, voto)
- [ ] Script de estrés básico (100 votos simultáneos)
- [ ] Revisar logs para asegurar que no se exponen datos sensibles
- [ ] Probar ataques básicos: inyección SQL, replay de votos
- [ ] Verificar que transacciones no causan deadlocks

---

## Día 14: Documentación y Preparación
**Objetivo**: Documentar y preparar para producción.

### Tareas:
- [ ] Generar OpenAPI spec básica con `utoipa`
- [ ] Documentar endpoints principales en `API.md`
- [ ] Crear `.env.example` con todas las variables
- [ ] Checklist final de seguridad
- [ ] Configurar backup automático de DB (ej: pg_dump diario)

---

## Día 15: MVP GO LIVE 🚀
**Objetivo**: Desplegar versión mínima funcional.

### Tareas:
- [ ] Despliegue en entorno de producción (Fly.io/Heroku/Railway)
- [ ] Monitoreo inicial (logs, errores, performance)
- [ ] Prueba de flujo completo con datos reales
- [ ] Revisar métricas de costo computacional
- [ ] Plan de escalabilidad (si aplica)

---

## Notas Técnicas:
1. **Validación de Identidad Simplificada**:
   - Frontend mantiene UI pero solo envía `document_number`
   - Backend genera nullifier con `SHA256(document_number + election_salt)`
   - Para elecciones privadas, verificar hash en whitelist
   - Eliminar procesamiento de imágenes (ONNX, image)

2. **Dependencias**:
   ```toml
   # Mantener comentadas para futuro
   # ort = { version = "2.0.0-rc.11", features = ["load-dynamic"] }
   # image = "0.24"
   ```

3. **Costos Optimizados**:
   - Sin inferencia de modelos de IA
   - Bajo consumo de CPU/memoria
   - Posible despliegue en tier gratuito de plataformas serverless

4. **Mantener Frontend Existente**:
   - No eliminar componentes de captura biométrica
   - Deshabilitarlos o ocultarlos con feature flags
   - Mantener código para futura expansión

---

## Prioridades Críticas:
1. **Transacciones en votación** (Día 5) - Imprescindible para integridad
2. **Manejo de errores** (Día 1-2) - Evitar pánicos en producción
3. **Validación de entrada** (Día 6) - Prevenir datos corruptos
4. **Tokens reales** (Día 8) - Seguridad básica

---

## Métricas de Éxito MVP:
- ✅ Votación funcional con nullifiers
- ✅ Elecciones públicas/privadas con whitelist
- ✅ Recibos verificables con Merkle proofs
- ✅ Admin puede crear/iniciar/cerrar elecciones
- ✅ Despliegue funcionando con < $10/mes
- ✅ Frontend básico funcional

---

> **Nota**: Este plan mantiene el 80% del valor con 20% de la complejidad.
> La validación biométrica puede añadirse posteriormente como módulo premium.