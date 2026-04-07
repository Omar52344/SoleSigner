use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::Value;

use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use super::AppState;

pub async fn get_election_results(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    use sqlx::Row;

    let rows = sqlx::query("SELECT encrypted_choices FROM ballots WHERE election_id = $1")
        .bind(election_id)
        .fetch_all(&state.db)
        .await?;

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

    Ok((StatusCode::OK, Json(tally)))
}

pub async fn verify_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let record = sqlx::query!(
        "SELECT merkle_root FROM elections WHERE id = $1",
        election_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Election not found".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "merkle_root": record.merkle_root })),
    ))
}