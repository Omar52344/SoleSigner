use axum::{
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    RequestPartsExt, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

use crate::crypto;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// --- Auth DTOs ---
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // admin_id
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}

pub struct AuthUser {
    pub admin_id: Uuid,
}

pub fn router(pool: PgPool) -> Router {
    let state = AppState { db: pool };
    Router::new()
        // Auth Routes
        .route("/auth/register", post(register_admin))
        .route("/auth/login", post(login_admin))
        // Protected Routes (handled by extractors in handlers)
        .route("/elections/create", post(create_election))
        .route("/elections", get(list_elections))
        // Public Routes
        .route("/elections/:id", get(get_election))
        .route("/elections/:id/stats", get(get_election_stats)) // Could be protected
        .route("/elections/:id/start", post(start_election)) // Should be protected, but for now Public
        .route("/elections/:id/close", post(close_election)) // Should be protected
        .route("/elections/:id/results", get(get_election_results)) // Public results
        .route(
            "/elections/:id/whitelist",
            get(get_whitelist).post(add_whitelist),
        )
        .route("/vote/validate-identity", post(validate_identity))
        .route("/vote/submit", post(submit_vote))
        .route("/audit/:election_id/verify", get(verify_election))
        .with_state(state)
}

// --- Auth Extractor ---
#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing Bearer Token".to_string()))?;

        // Decode the user data
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token".to_string()))?;

        Ok(AuthUser {
            admin_id: Uuid::parse_str(&token_data.claims.sub).unwrap(),
        })
    }
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

// --- Auth Handlers ---

async fn register_admin(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    use sqlx::Row;
    // 1. Check if user exists
    let exists = sqlx::query("SELECT id FROM admins WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await;

    if let Ok(Some(_)) = exists {
        return (StatusCode::CONFLICT, "Username already exists").into_response();
    }

    // 2. Hash password
    let password_hash = hash(payload.password, DEFAULT_COST).unwrap();

    // 3. Insert User
    let insert =
        sqlx::query("INSERT INTO admins (username, password_hash) VALUES ($1, $2) RETURNING id")
            .bind(&payload.username)
            .bind(password_hash)
            .fetch_one(&state.db)
            .await;

    match insert {
        Ok(rec) => {
            let id: Uuid = rec.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn login_admin(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    use sqlx::Row;
    let result = sqlx::query("SELECT id, password_hash FROM admins WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(rec)) => {
            let id: Uuid = rec.try_get("id").unwrap();
            let pwd: String = rec.try_get("password_hash").unwrap();

            if verify(payload.password, &pwd).unwrap_or(false) {
                // Generate JWT
                let exp = chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::hours(24))
                    .expect("valid timestamp")
                    .timestamp() as usize;

                let claims = Claims {
                    sub: id.to_string(),
                    exp,
                };

                let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(secret.as_bytes()),
                )
                .unwrap();

                (
                    StatusCode::OK,
                    Json(AuthResponse {
                        token,
                        username: payload.username,
                    }),
                )
                    .into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
            }
        }
        _ => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    }
}

// --- Handlers ---

async fn create_election(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateElectionRequest>,
) -> impl IntoResponse {
    use sqlx::Row;
    let election_salt = Uuid::new_v4().to_string();

    // Using runtime check query to avoid compile error if DB not migrated yet
    let result = sqlx::query(
        r#"
        INSERT INTO elections (title, form_config, start_date, end_date, access_type, election_salt, status, admin_id)
        VALUES ($1, $2, $3, $4, $5::access_type, $6, 'DRAFT', $7)
        RETURNING id
        "#
    )
    .bind(payload.title)
    .bind(payload.form_config)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.access_type) // This might need explicit cast handling if using simple bind
    .bind(election_salt)
    .bind(auth.admin_id)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(rec) => {
            let id: Uuid = rec.try_get("id").unwrap();
            (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Removed duplicate DTOs...

// ...

async fn list_elections(auth: AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    use sqlx::Row;
    let result = sqlx::query(
        "SELECT id, title, start_date, end_date, status::text as status FROM elections WHERE admin_id = $1 ORDER BY start_date DESC"
    )
    .bind(auth.admin_id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(recs) => {
            let list: Vec<_> = recs
                .iter()
                .map(|rec| {
                    let id: Uuid = rec.try_get("id").unwrap_or_default();
                    let title: String = rec.try_get("title").unwrap_or_default();
                    let start: chrono::DateTime<chrono::Utc> =
                        rec.try_get("start_date").unwrap_or_default(); // Might panic if null? Schema says NOT NULL usually?
                    let end: chrono::DateTime<chrono::Utc> =
                        rec.try_get("end_date").unwrap_or_default();
                    let status: String = rec.try_get("status").unwrap_or_default();

                    serde_json::json!({
                        "id": id,
                        "title": title,
                        "start_date": start,
                        "end_date": end,
                        "status": status,
                    })
                })
                .collect();
            (StatusCode::OK, Json(list)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Missing DTOs ---

#[derive(Serialize)]
pub struct ElectionStats {
    pub total_votes: i64,
    pub status: String,
}

#[derive(Deserialize)]
pub struct AddWhitelistRequest {
    pub document_hashes: Vec<String>,
}

#[derive(Deserialize)]
pub struct ValidateIdentityRequest {
    pub election_id: Uuid,
    pub selfie_base64: String,
    pub document_base64: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize)]
pub struct ValidateIdentityResponse {
    pub identity_token: String,
    pub nullifier: String,
}

// --- Missing Handlers ---

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
    let doc_id = "DOC123456";
    let doc_hash = crypto::hash_data(doc_id);

    if election.access_type.as_deref() == Some("PRIVATE") {
        let whitelisted = sqlx::query!(
            "SELECT id FROM whitelist WHERE election_id = $1 AND document_id_hash = $2",
            payload.election_id,
            doc_hash
        )
        .fetch_optional(&state.db)
        .await;

        match whitelisted {
            Ok(Some(_)) => {}
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

    (
        StatusCode::OK,
        Json(ValidateIdentityResponse {
            identity_token: "VALID_TOKEN_PLACEHOLDER".to_string(),
            nullifier,
        }),
    )
        .into_response()
}

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

async fn add_whitelist(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<AddWhitelistRequest>,
) -> impl IntoResponse {
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

async fn get_election_results(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    use sqlx::Row;
    use std::collections::HashMap;

    let ballots = sqlx::query("SELECT encrypted_choices FROM ballots WHERE election_id = $1")
        .bind(election_id)
        .fetch_all(&state.db)
        .await;

    match ballots {
        Ok(rows) => {
            let mut tally: HashMap<String, i32> = HashMap::new();

            for row in rows {
                let choices_val: Value = row
                    .try_get("encrypted_choices")
                    .unwrap_or(serde_json::json!({}));

                if let Some(obj) = choices_val.as_object() {
                    for (_question_id, answer) in obj {
                        let answer_str = answer.as_str().unwrap_or("Unknown").to_string();
                        *tally.entry(answer_str).or_insert(0) += 1;
                    }
                }
            }

            (StatusCode::OK, Json(tally)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
