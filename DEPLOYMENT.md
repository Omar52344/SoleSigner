# Deployment Guide

SoleSigner is designed for serverless deployment with low computational cost. This guide covers deployment to Fly.io, Railway, and Heroku.

## Prerequisites

- Docker (optional, for containerized deployment)
- PostgreSQL database (managed or self‑hosted)
- Rust toolchain (if building from source)

## Environment Variables

Copy `.env.example` to `.env` and set the following:

```bash
DATABASE_URL=postgres://user:password@host:port/database
JWT_SECRET=your-secret-key-change-in-production
BCRYPT_COST=10
JSON_LOGS=false   # set to true for structured JSON logs
RUST_LOG=info     # logging level
```

## 1. Fly.io

### Create a Fly app

```bash
fly launch --no-deploy
```

Set secrets:

```bash
fly secrets set DATABASE_URL=postgres://...
fly secrets set JWT_SECRET=...
fly secrets set BCRYPT_COST=10
```

Deploy:

```bash
fly deploy
```

The `Dockerfile` uses a multi‑stage build to produce a slim image. The application will automatically run migrations on startup.

## 2. Railway

### Connect your repository

1. Create a new project on Railway.
2. Connect your GitHub repository.
3. Add a PostgreSQL service and link it to your app.
4. Set environment variables in the Railway dashboard.

Railway will detect the `Dockerfile` and deploy automatically.

## 3. Heroku

### Using Container Registry

```bash
heroku create
heroku addons:create heroku-postgresql:hobby-dev
heroku config:set JWT_SECRET=...
heroku config:set BCRYPT_COST=10
```

Push the Docker image:

```bash
heroku container:push web
heroku container:release web
```

## 4. Manual Docker Deployment

Build the image:

```bash
docker build -t solesigner .
```

Run with environment variables:

```bash
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://... \
  -e JWT_SECRET=... \
  -e BCRYPT_COST=10 \
  solesigner
```

## Database Migrations

Migrations are executed automatically when the application starts (see `src/main.rs`). Ensure the database user has permission to create tables.

## Health Check

The application exposes a minimal health endpoint at `GET /health` (returns 200). Use this for load‑balancer health checks.

## Backup Strategy

A backup script is provided in `scripts/backup_db.sh`. Run it periodically (e.g., via cron) to create compressed SQL dumps.

Example cron entry (daily at 2 AM):

```
0 2 * * * /home/omar/proyects/SoleSigner/scripts/backup_db.sh
```

## Monitoring

- Structured logs are written to stdout (JSON format if `JSON_LOGS=true`).
- Use `tracing` spans for request‑level observability.
- Integrate with your platform's logging service (Fly Logs, Railway Logs, Heroku Logs).

## Scaling

The application is stateless; scale horizontally by running multiple instances. Ensure all instances share the same PostgreSQL database and JWT_SECRET.

## Security Checklist

Refer to `SECURITY_CHECKLIST.md` for production hardening steps.

## Troubleshooting

- **Database connection errors**: Verify DATABASE_URL and network access.
- **Migration failures**: Check that the database user has CREATE TABLE privileges.
- **High latency**: Ensure the database is in the same region as your app.