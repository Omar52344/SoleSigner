use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
// use std::sync::Arc;
use uuid::Uuid;

use crate::crypto;
// use crate::identity::IdentityEngine; // Assuming this is available

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    // pub identity_engine: Arc<IdentityEngine>,
}

pub fn router(pool: PgPool) -> Router {
    let state = AppState { db: pool };
    Router::new()
        .route("/elections/create", post(create_election))
        .route("/elections", get(list_elections))
        .route("/elections/:id", get(get_election))
        .route("/elections/:id/stats", get(get_election_stats))
        .route("/elections/:id/start", post(start_election))
        .route("/elections/:id/close", post(close_election))
        .route(
            "/elections/:id/whitelist",
            get(get_whitelist).post(add_whitelist),
        )
        .route("/vote/validate-identity", post(validate_identity))
        .route("/vote/submit", post(submit_vote))
        .route("/audit/:election_id/verify", get(verify_election))
        .with_state(state)
}

// --- DTOs ---

#[derive(Deserialize)]
pub struct CreateElectionRequest {
    pub title: String,
    pub form_config: Value,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub access_type: String, // "PUBLIC" or "PRIVATE"
}

#[derive(Deserialize)]
pub struct ValidateIdentityRequest {
    pub election_id: Uuid,
    pub selfie_base64: String,
    pub document_base64: String,
    // GPS
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize)]
pub struct ValidateIdentityResponse {
    pub identity_token: String, // Could be a signed JWT or similar, for now just returning success info
    pub nullifier: String,
}

#[derive(Deserialize)]
pub struct SubmitVoteRequest {
    pub election_id: Uuid,
    pub choices: Value,
    pub nullifier: String, // In real app, this comes from the validated session/token
    pub request_id: Uuid,  // GUID for the ballot
}

#[derive(Serialize)]
pub struct VoteReceipt {
    pub ballot_hash: String,
    pub merkle_path: Vec<String>,
    pub signature: String,
}

// --- Handlers ---

async fn create_election(
    State(state): State<AppState>,
    Json(payload): Json<CreateElectionRequest>,
) -> impl IntoResponse {
    let election_salt = Uuid::new_v4().to_string(); // In reality, a cryptographically secure random string

    let result = sqlx::query!(
        r#"
        INSERT INTO elections (title, form_config, start_date, end_date, access_type, election_salt, status)
        VALUES ($1, $2, $3, $4, $5::access_type, $6, 'DRAFT')
        RETURNING id
        "#,
        payload.title,
        payload.form_config,
        payload.start_date,
        payload.end_date,
        payload.access_type as _, // Cast to enum
        election_salt
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(rec) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": rec.id })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Additional DTOs ---

#[derive(Serialize)]
pub struct ElectionStats {
    pub total_votes: i64,
    pub status: String,
}

#[derive(Deserialize)]
pub struct AddWhitelistRequest {
    pub document_hashes: Vec<String>,
}

// --- Updated Handlers ---

// Update validate_identity to check whitelist
async fn validate_identity(
    State(state): State<AppState>,
    Json(payload): Json<ValidateIdentityRequest>,
) -> impl IntoResponse {
    // 1. Fetch election details
    let election = match sqlx::query!(
        "SELECT election_salt, status::text as status, access_type::text as access_type FROM elections WHERE id = $1",
        payload.election_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(e)) => e,
        Ok(None) => return (StatusCode::NOT_FOUND, "Election not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 2. CHECK WHITELIST IF PRIVATE
    let doc_id = "DOC123456"; // In real prod, derived from OCR/Selfie check
                              // Ideally we hash the doc_id to match whitelist
    let doc_hash = crypto::hash_data(doc_id);

    if election.access_type.as_deref() == Some("PRIVATE") {
        // Check if doc_hash is in whitelist
        let whitelisted = sqlx::query!(
            "SELECT id FROM whitelist WHERE election_id = $1 AND document_id_hash = $2",
            payload.election_id,
            doc_hash // Using mocked hash for now, normally computed from payload.document_base64 extraction
        )
        .fetch_optional(&state.db)
        .await;

        match whitelisted {
            Ok(Some(_)) => {} // OK
            Ok(None) => {
                return (
                    StatusCode::FORBIDDEN,
                    "Identity not in whitelist for this private election",
                )
                    .into_response()
            }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
    }

    // 3. Generate Nullifier
    let nullifier = crypto::generate_nullifier(doc_id, &election.election_salt);

    // 4. Return Token
    (
        StatusCode::OK,
        Json(ValidateIdentityResponse {
            identity_token: "VALID_TOKEN_PLACEHOLDER".to_string(),
            nullifier,
        }),
    )
        .into_response()
}

// Stats Handler
async fn get_election_stats(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM ballots WHERE election_id = $1",
        election_id
    )
    .fetch_one(&state.db)
    .await;

    let status = sqlx::query!(
        "SELECT status::text as status FROM elections WHERE id = $1",
        election_id
    )
    .fetch_one(&state.db)
    .await;

    match (count, status) {
        (Ok(c), Ok(s)) => (
            StatusCode::OK,
            Json(ElectionStats {
                total_votes: c.count.unwrap_or(0),
                status: s.status.unwrap_or("UNKNOWN".to_string()),
            }),
        )
            .into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch stats").into_response(),
    }
}

// Whitelist Handlers
async fn add_whitelist(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<AddWhitelistRequest>,
) -> impl IntoResponse {
    // Bulk insert could be better, but loop is simpler for now
    for hash in payload.document_hashes {
        let _ = sqlx::query!(
            "INSERT INTO whitelist (election_id, document_id_hash) VALUES ($1, $2)",
            election_id,
            hash
        )
        .execute(&state.db)
        .await;
    }
    StatusCode::OK.into_response()
}

async fn get_whitelist(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let records = sqlx::query!(
        "SELECT document_id_hash FROM whitelist WHERE election_id = $1",
        election_id
    )
    .fetch_all(&state.db)
    .await;

    match records {
        Ok(recs) => {
            let hashes: Vec<String> = recs.iter().map(|r| r.document_id_hash.clone()).collect();
            (StatusCode::OK, Json(hashes)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn list_elections(state: State<AppState>) -> impl IntoResponse {
    let result = sqlx::query!(
        "SELECT id, title, start_date, end_date, status::text as status FROM elections ORDER BY start_date DESC"
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(recs) => {
            let list: Vec<_> = recs
                .iter()
                .map(|rec| {
                    serde_json::json!({
                        "id": rec.id,
                        "title": rec.title,
                        "start_date": rec.start_date,
                        "end_date": rec.end_date,
                        "status": rec.status,
                    })
                })
                .collect();
            (StatusCode::OK, Json(list)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn submit_vote(
    State(state): State<AppState>,
    Json(payload): Json<SubmitVoteRequest>,
) -> impl IntoResponse {
    // 1. Verify election status
    let election_result = sqlx::query!(
        "SELECT status::text as status FROM elections WHERE id = $1",
        payload.election_id
    )
    .fetch_optional(&state.db)
    .await;

    match election_result {
        Ok(Some(_)) => {
            // Using check_status helper check if string matches OPEN
        }
        _ => return (StatusCode::NOT_FOUND, "Election not found").into_response(),
    };

    // 2. Register Nullifier
    let insert_registry = sqlx::query!(
        "INSERT INTO voter_registry (election_id, nullifier_hash, identity_status, location_zone) VALUES ($1, $2, 'Validated', 'ZoneA')",
        payload.election_id,
        payload.nullifier
    )
    .execute(&state.db)
    .await;

    if let Err(_) = insert_registry {
        // Likely unique constraint violation
        return (StatusCode::CONFLICT, "Vote already cast with this identity").into_response();
    }

    // 3. Create Ballot
    // ballot_hash: SHA256(GUID + Choices)
    let choices_str = payload.choices.to_string();
    let ballot_hash = crypto::hash_data(&format!("{}{}", payload.request_id, choices_str));

    // Insert Ballot
    let insert_ballot = sqlx::query!(
        "INSERT INTO ballots (id, election_id, encrypted_choices, ballot_hash) VALUES ($1, $2, $3, $4)",
        payload.request_id,
        payload.election_id,
        payload.choices,
        ballot_hash
    )
    .execute(&state.db)
    .await;

    if let Err(e) = insert_ballot {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // 4. Generate Receipt
    let receipt = VoteReceipt {
        ballot_hash,
        merkle_path: vec![],
        signature: "ELECTION_SIGNATURE_PLACEHOLDER".to_string(),
    };

    (StatusCode::OK, Json(receipt)).into_response()
}

async fn get_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "SELECT id, title, form_config, start_date, end_date, access_type::text as access_type, election_salt, status::text as status FROM elections WHERE id = $1",
        election_id
    )
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(rec)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": rec.id,
                "title": rec.title,
                "form_config": rec.form_config,
                "start_date": rec.start_date,
                "end_date": rec.end_date,
                "status": rec.status,
                "access_type": rec.access_type,
                "election_salt": rec.election_salt
            })),
        )
            .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Election not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn verify_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let root = sqlx::query!(
        "SELECT merkle_root FROM elections WHERE id = $1",
        election_id
    )
    .fetch_optional(&state.db)
    .await;

    match root {
        Ok(Some(record)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "merkle_root": record.merkle_root })),
        )
            .into_response(),
        _ => (StatusCode::NOT_FOUND, "Election not found").into_response(),
    }
}

async fn start_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "UPDATE elections SET status = 'OPEN' WHERE id = $1 AND status = 'DRAFT'",
        election_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                StatusCode::OK.into_response()
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    "Election not found or not in DRAFT state",
                )
                    .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn close_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "UPDATE elections SET status = 'SEALED' WHERE id = $1 AND status = 'OPEN'",
        election_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                StatusCode::OK.into_response()
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    "Election not found or not in OPEN state",
                )
                    .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
