use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;

use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::types::ElectionStatus;
use super::AppState;

#[derive(Deserialize)]
pub struct AddWhitelistRequest {
    pub document_hashes: Vec<String>,
}

pub async fn add_whitelist(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<AddWhitelistRequest>,
) -> ApiResult<impl IntoResponse> {
    // 1. Check election status
    let election = sqlx::query!(
        "SELECT status as \"status: ElectionStatus\" FROM elections WHERE id = $1",
        election_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Election not found".to_string()))?;

    if election.status == ElectionStatus::Sealed {
        return Err(ApiError::Forbidden("Election is closed".to_string()));
    }

    for hash in payload.document_hashes {
        sqlx::query!(
            "INSERT INTO whitelist (election_id, document_id_hash) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            election_id,
            hash
        )
        .execute(&state.db)
        .await?;
    }
    Ok(StatusCode::OK)
}

pub async fn get_whitelist(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let records = sqlx::query!(
        "SELECT document_id_hash FROM whitelist WHERE election_id = $1",
        election_id
    )
    .fetch_all(&state.db)
    .await?;

    let hashes: Vec<String> = records.iter().map(|r| r.document_id_hash.clone()).collect();
    Ok((StatusCode::OK, Json(hashes)))
}