use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse},
    Json,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use bcrypt::{hash, verify};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use uuid::Uuid;


use crate::error::{ApiError, ApiResult};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Claims {
    pub sub: String, // admin_id
    pub exp: usize,
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6, max = 100))]
    pub password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RegisterResponse {
    pub id: Uuid,
}

pub struct AuthUser {
    pub admin_id: Uuid,
}

#[axum::async_trait]
impl axum::extract::FromRequestParts<crate::api::AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &crate::api::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing Bearer Token".to_string()))?;

        // Decode the user data
        let secret = &state.config.jwt_secret;
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token".to_string()))?;

        let admin_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID in token".to_string()))?;

        Ok(AuthUser { admin_id })
    }
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = LoginRequest,
    responses(
        (status = 201, description = "Admin registered successfully", body = RegisterResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Username already exists")
    )
)]
pub async fn register_admin(
    State(state): State<crate::api::AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    payload.validate().map_err(|e: ValidationErrors| ApiError::Validation(e.to_string()))?;
    use sqlx::Row;
    // 1. Check if user exists
    let exists = sqlx::query("SELECT id FROM admins WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?;

    if exists.is_some() {
        return Err(ApiError::Conflict("Username already exists".to_string()));
    }

    // 2. Hash password
    let password_hash = hash(payload.password, state.config.bcrypt_cost)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // 3. Insert User
    let insert =
        sqlx::query("INSERT INTO admins (username, password_hash) VALUES ($1, $2) RETURNING id")
            .bind(&payload.username)
            .bind(password_hash)
            .fetch_one(&state.db)
            .await?;

    let id: Uuid = insert.try_get("id")?;
    
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse { id }),
    ))
}

pub async fn login_admin(
    State(state): State<crate::api::AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    payload.validate().map_err(|e: ValidationErrors| ApiError::Validation(e.to_string()))?;
    use sqlx::Row;
    
    let rec = sqlx::query("SELECT id, password_hash FROM admins WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?;

    let rec = rec.ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    let id: Uuid = rec.try_get("id")?;
    let pwd: String = rec.try_get("password_hash")?;

    let valid = verify(payload.password, &pwd)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    if !valid {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate JWT
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .ok_or_else(|| ApiError::Internal("Invalid timestamp".to_string()))?
        .timestamp() as usize;

    let claims = Claims {
        sub: id.to_string(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token,
            username: payload.username,
        }),
    ))
}