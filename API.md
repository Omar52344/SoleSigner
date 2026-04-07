# SoleSigner API Documentation

This document describes the REST API for the SoleSigner voting system.

## Base URL

`http://localhost:8080` (development)

## Authentication

Most endpoints require a JWT token obtained via `/auth/login`. Include the token in the `Authorization` header:

```
Authorization: Bearer <token>
```

## Endpoints

### Authentication

#### `POST /auth/register`

Register a new admin user.

**Request Body:**
```json
{
  "username": "string (3-50 chars)",
  "password": "string (6-100 chars)"
}
```

**Response:**
- `201 Created`: `{"id": "uuid"}`
- `400 Bad Request`: Validation error
- `409 Conflict`: Username already exists

#### `POST /auth/login`

Login and obtain a JWT token.

**Request Body:** Same as register.

**Response:**
```json
{
  "token": "jwt-string",
  "username": "string"
}
```

### Elections

#### `POST /elections/create`

Create a new election (protected).

**Request Body:**
```json
{
  "title": "string (1-200 chars)",
  "form_config": { "questions": [...] },
  "start_date": "ISO 8601 datetime",
  "end_date": "ISO 8601 datetime",
  "access_type": "PUBLIC" or "PRIVATE"
}
```

**Response:**
- `201 Created`: `{"id": "uuid"}`

#### `GET /elections`

List elections for the authenticated admin (protected).

**Response:**
```json
[
  {
    "id": "uuid",
    "title": "string",
    "start_date": "datetime",
    "end_date": "datetime",
    "status": "DRAFT|OPEN|CLOSING|SEALED"
  }
]
```

#### `GET /elections/:id`

Get election details (public).

**Response:**
```json
{
  "id": "uuid",
  "title": "string",
  "form_config": {...},
  "start_date": "datetime",
  "end_date": "datetime",
  "status": "DRAFT|OPEN|CLOSING|SEALED",
  "access_type": "PUBLIC|PRIVATE",
  "election_salt": "string"
}
```

#### `POST /elections/:id/start`

Start an election (change status from DRAFT to OPEN). Protected.

**Response:**
- `200 OK` on success.

#### `POST /elections/:id/close`

Close an election (change status from OPEN to SEALED). Protected.

**Response:**
- `200 OK` on success.

#### `GET /elections/:id/stats`

Get election statistics (total votes, status). Public.

**Response:**
```json
{
  "total_votes": 42,
  "status": "OPEN"
}
```

#### `GET /elections/:id/results`

Get election results (tally of choices). Public.

**Response:**
```json
{
  "Yes": 30,
  "No": 12
}
```

### Whitelist Management

#### `GET /elections/:id/whitelist`

Get whitelisted document hashes for a private election. Protected.

**Response:**
```json
["hash1", "hash2"]
```

#### `POST /elections/:id/whitelist`

Add document hashes to the whitelist. Protected.

**Request Body:**
```json
{
  "document_hashes": ["hash1", "hash2"]
}
```

### Voting

#### `POST /vote/check-eligibility`

Check if a document is eligible to vote in a private election.

**Request Body:**
```json
{
  "election_id": "uuid",
  "document_number": "string (3-50 chars)"
}
```

**Response:**
- `200 OK`: Eligible (no body)
- `403 Forbidden`: Not in whitelist
- `404 Not Found`: Election not found

#### `POST /vote/validate-identity`

Validate voter identity and generate a nullifier + short-lived JWT.

**Request Body:** Same as check‑eligibility.

**Response:**
```json
{
  "identity_token": "jwt-string",
  "nullifier": "sha256 hash"
}
```

#### `POST /vote/submit`

Submit a vote (requires identity token in header? currently not required).

**Request Body:**
```json
{
  "election_id": "uuid",
  "choices": { "question_id": "selected_option" },
  "nullifier": "string",
  "request_id": "uuid"
}
```

**Response:**
```json
{
  "ballot_hash": "string",
  "merkle_path": [],
  "election_id": "uuid",
  "timestamp": "datetime",
  "public_key": "hex",
  "signed_data": "string",
  "signature": "hex"
}
```

#### `POST /vote/verify-receipt`

Verify a vote receipt signature.

**Request Body:**
```json
{
  "signed_data": "string",
  "public_key": "hex",
  "signature": "hex"
}
```

**Response:**
```json
{
  "valid": true,
  "message": "Signature is valid"
}
```

### Audit

#### `GET /audit/:election_id/verify`

Get the Merkle root of a sealed election.

**Response:**
```json
{
  "merkle_root": "string"
}
```

### OpenAPI Specification

- `GET /openapi.json` – OpenAPI spec in JSON format.
- `GET /openapi.yaml` – OpenAPI spec in YAML format.

## Error Handling

All errors follow the same structure:

```json
{
  "error": "Human-readable error message"
}
```

Common HTTP status codes:

- `400` – Validation error (missing fields, invalid data)
- `401` – Missing or invalid authentication token
- `403` – Forbidden (not authorized for this election)
- `404` – Resource not found
- `409` – Conflict (e.g., duplicate vote)
- `500` – Internal server error

## Security Notes

- All passwords are hashed with bcrypt.
- JWT tokens expire after 24 hours (admin) or 5 minutes (voter identity).
- Document numbers are never stored; only their SHA‑256 hashes.
- Nullifiers are SHA256(document + election_salt) and ensure one vote per identity per election.
- Vote receipts are signed with Ed25519 using a per‑election derived key.
- Private elections require a whitelist of pre‑hashed document IDs.

## Database Schema

See `migrations/` folder for the complete SQL schema.

## Environment Variables

See `.env.example` for required configuration.

## Deployment

The application is designed for low‑cost serverless deployment (Fly.io, Railway, etc.). A `Dockerfile` is provided for containerization.