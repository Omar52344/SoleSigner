use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationErrors};
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    types::{AccessType, ElectionStatus},
};
use super::AppState;
use super::auth::AuthUser;

#[derive(Deserialize, Validate)]
pub struct CreateElectionRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub form_config: Value,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub access_type: AccessType,
}

#[derive(Serialize)]
pub struct ElectionStats {
    pub total_votes: i64,
    pub status: ElectionStatus,
}

pub async fn create_election(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateElectionRequest>,
) -> ApiResult<impl IntoResponse> {
    payload.validate().map_err(|e: ValidationErrors| ApiError::Validation(e.to_string()))?;
    use sqlx::Row;
    
    if payload.start_date >= payload.end_date {
        return Err(ApiError::Validation("start_date must be before end_date".to_string()));
    }
    if !payload.form_config.is_object() {
        return Err(ApiError::Validation("form_config must be a JSON object".to_string()));
    }

    let election_salt = Uuid::new_v4().to_string();
    let title = payload.title.clone();

    let rec = sqlx::query(
        r#"
        INSERT INTO elections (title, form_config, start_date, end_date, access_type, election_salt, status, admin_id)
        VALUES ($1, $2, $3, $4, $5::access_type, $6, 'DRAFT', $7)
        RETURNING id
        "#
    )
    .bind(title)
    .bind(payload.form_config)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.access_type)
    .bind(election_salt)
    .bind(auth.admin_id)
    .fetch_one(&state.db)
    .await?;

    let id: Uuid = rec.try_get("id")?;
    
    tracing::info!(
        election_id = %id,
        admin_id = %auth.admin_id,
        title = %payload.title,
        "ADMIN_ACTION: Election created"
    );
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn list_elections(
    auth: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    use sqlx::Row;
    
    let recs = sqlx::query(
        "SELECT id, title, start_date, end_date, status as \"status: ElectionStatus\" FROM elections WHERE admin_id = $1 ORDER BY start_date DESC"
    )
    .bind(auth.admin_id)
    .fetch_all(&state.db)
    .await?;

    let list: Vec<_> = recs
        .iter()
        .map(|rec| -> Result<_, sqlx::Error> {
            let id: Uuid = rec.try_get("id")?;
            let title: String = rec.try_get("title")?;
            let start: DateTime<Utc> = rec.try_get("start_date")?;
            let end: DateTime<Utc> = rec.try_get("end_date")?;
            let status: ElectionStatus = rec.try_get("status")?;

            Ok(serde_json::json!({
                "id": id,
                "title": title,
                "start_date": start,
                "end_date": end,
                "status": status.as_str(),
            }))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(list)))
}

pub async fn get_election(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let rec = sqlx::query!(
        "SELECT id, title, form_config, start_date, end_date, access_type as \"access_type: AccessType\", election_salt, status as \"status: ElectionStatus\" FROM elections WHERE id = $1",
        election_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Election not found".to_string()))?;

    Ok((
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
    ))
}

pub async fn start_election(
    auth: AuthUser,
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let res = sqlx::query!(
        "UPDATE elections SET status = 'OPEN' WHERE id = $1 AND status = 'DRAFT' AND admin_id = $2",
        election_id,
        auth.admin_id
    )
    .execute(&state.db)
    .await?;

    if res.rows_affected() > 0 {
        tracing::info!(
            election_id = %election_id,
            admin_id = %auth.admin_id,
            "ADMIN_ACTION: Election started (status OPEN)"
        );
        Ok(StatusCode::OK)
    } else {
        Err(ApiError::Validation("Election not found, not in DRAFT state, or you are not the owner".to_string()))
    }
}

pub async fn close_election(
    auth: AuthUser,
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let res = sqlx::query!(
        "UPDATE elections SET status = 'SEALED' WHERE id = $1 AND status = 'OPEN' AND admin_id = $2",
        election_id,
        auth.admin_id
    )
    .execute(&state.db)
    .await?;

    if res.rows_affected() > 0 {
        tracing::info!(
            election_id = %election_id,
            admin_id = %auth.admin_id,
            "ADMIN_ACTION: Election closed (status SEALED)"
        );
        Ok(StatusCode::OK)
    } else {
        Err(ApiError::Validation("Election not found, not in OPEN state, or you are not the owner".to_string()))
    }
}

pub async fn get_election_stats(
    Path(election_id): Path<Uuid>,
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM ballots WHERE election_id = $1",
        election_id
    )
    .fetch_one(&state.db)
    .await?;

    let status = sqlx::query!(
        "SELECT status as \"status: ElectionStatus\" FROM elections WHERE id = $1",
        election_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(ElectionStats {
            total_votes: count.count.unwrap_or(0),
            status: status.status,
        }),
    ))
}