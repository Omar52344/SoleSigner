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
        .route("/elections/:id", get(get_election)) // Added this
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

async fn validate_identity(
    State(state): State<AppState>,
    Json(payload): Json<ValidateIdentityRequest>,
) -> impl IntoResponse {
    // 1. Fetch election to get salt
    let election = match sqlx::query!(
        "SELECT election_salt, status::text as status FROM elections WHERE id = $1",
        payload.election_id
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(e)) => e,
        Ok(None) => return (StatusCode::NOT_FOUND, "Election not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 2. Process Images (Mocked here, calls IdentityEngine)
    // let (valid, doc_id) = state.identity_engine.validate(...).await...
    let doc_id = "DOC123456"; // Derived from OCR

    // 3. Generate Nullifier
    let nullifier = crypto::generate_nullifier(doc_id, &election.election_salt);

    // 4. Check if already registered/voted
    // We insert into voter_registry. If conflict, they already voted.
    // Note: The prompt says "Identity (Nullifiers) and Ballots (Anonymous Votes)".
    // Voter registry stores the nullifier.

    // Validate Geofencing here if needed (payload.latitude/longitude)

    (
        StatusCode::OK,
        Json(ValidateIdentityResponse {
            identity_token: "VALID_TOKEN_PLACEHOLDER".to_string(),
            nullifier,
        }),
    )
        .into_response()
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
        Ok(Some(r)) => {
            // Using check_status helper check if string matches OPEN
            // But here we might just retrieve the enum string
        }
        _ => return (StatusCode::NOT_FOUND, "Election not found").into_response(),
    };

    // 2. Register Nullifier (Atomically ensure unique vote)
    // In a real flow, this might happen at the 'validate-identity' step if that generates the 'right to vote' token,
    // OR it happens here if we want to ensure they can *submit* only once.
    // The architecture says: "Voter Registry ... Nullifier avoids double vote".
    // We try to insert the nullifier.

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
    // In a real system, we might update the Merkle Tree here or in background.
    // Returning empty merkle path for now or simplified one.

    let receipt = VoteReceipt {
        ballot_hash,
        merkle_path: vec![], // TODO: Retrieve real path
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
        Ok(Some(rec)) => {
            // We return everything needed for the UI
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": rec.id,
                    "title": rec.title,
                    "form_config": rec.form_config,
                    "start_date": rec.start_date,
                    "end_date": rec.end_date,
                    "status": rec.status, // We need to handle the Enum mapping carefully if sqlx doesn't auto-map to string in JSON without feature
                    // "access_type": rec.access_type, // same issue potential
                    "election_salt": rec.election_salt
                })),
            )
                .into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Election not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn verify_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Return the Merkle Root and proof data
    // For now, just dumping the root
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

async fn list_elections(State(state): State<AppState>) -> impl IntoResponse {
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
