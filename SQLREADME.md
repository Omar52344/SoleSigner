🚀 Configuración de Base de Datos y Migraciones (PostgreSQL + SQLx)
Para inicializar el entorno de base de datos en Linux (Ubuntu 24.04+), sigue estos pasos:

1. Acceder a PostgreSQL
Como Linux usa autenticación peer por defecto, entra como el superusuario del sistema:

Bash
sudo -u postgres psql
2. Crear Base de Datos y Usuario
Dentro de la consola de Postgres (postgres=#), ejecuta lo siguiente:

SQL
-- 1. Crear la base de datos del proyecto
CREATE DATABASE nombre_de_tu_db;

-- 2. Crear el usuario de desarrollo
CREATE USER omar_dev WITH PASSWORD 'tu_password_seguro';

-- 3. Asignar al usuario como dueño de la base de datos
ALTER DATABASE nombre_de_tu_db OWNER TO omar_dev;
3. Configurar Permisos del Esquema (PostgreSQL 15+)
Es necesario dar permisos explícitos sobre el esquema public para que sqlx pueda crear la tabla de control de migraciones:

SQL
-- Conectarse a la base de datos recién creada
\c nombre_de_tu_db

-- Otorgar permisos de creación en el esquema public
GRANT ALL ON SCHEMA public TO omar_dev;
4. Configurar Variables de Entorno
Crea o edita tu archivo .env en la raíz del proyecto de Rust:

Bash
DATABASE_URL=postgres://omar_dev:tu_password_seguro@localhost:5432/nombre_de_tu_db
5. Ejecutar Migraciones
Finalmente, desde la terminal de tu proyecto, corre las migraciones con sqlx-cli:

Bash
# Crea la base de datos si no existe (opcional)
cargo sqlx database create

# Ejecuta las migraciones pendientes
cargo sqlx migrate run
Nota para el equipo: Si recibes un error de Peer authentication failed, asegúrate de estar usando la conexión vía TCP en el .env (especificando localhost o 127.0.0.1) o ajusta el archivo pg_hba.conf para permitir conexiones locales.