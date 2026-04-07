use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use uuid::Uuid;
use validator::{Validate, ValidationErrors};
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::{crypto, error::{ApiError, ApiResult}, types::AccessType};
use super::AppState;

#[derive(Deserialize)]
pub struct SubmitVoteRequest {
    pub election_id: Uuid,
    pub choices: Value,
    pub nullifier: String,
    pub request_id: Uuid,
}

#[derive(Serialize)]
pub struct VoteReceipt {
    pub ballot_hash: String,
    pub merkle_path: Vec<String>,
    pub election_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub public_key: String,
    pub signed_data: String,
    pub signature: String,
}

#[derive(Deserialize, Validate)]
pub struct ValidateIdentityRequest {
    pub election_id: Uuid,
    #[validate(length(min = 3, max = 50, message = "Document number must be between 3 and 50 characters"))]
    pub document_number: String,
}

#[derive(Deserialize, Validate)]
pub struct CheckEligibilityRequest {
    pub election_id: Uuid,
    #[validate(length(min = 3, max = 50, message = "Document number must be between 3 and 50 characters"))]
    pub document_number: String,
}

#[derive(Serialize, Deserialize)]
pub struct IdentityClaims {
    pub sub: String, // nullifier
    pub election_id: Uuid,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct ValidateIdentityResponse {
    pub identity_token: String,
    pub nullifier: String,
}

pub async fn check_eligibility(
    State(state): State<AppState>,
    Json(payload): Json<CheckEligibilityRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input
    payload.validate().map_err(|e: ValidationErrors| ApiError::Validation(e.to_string()))?;

    let election = sqlx::query!(
        "SELECT access_type as \"access_type: AccessType\" FROM elections WHERE id = $1",
        payload.election_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Election not found".to_string()))?;

    if election.access_type != AccessType::Private {
        return Ok(StatusCode::OK);
    }

    let doc_hash = crypto::hash_data(&payload.document_number);

    let exists = sqlx::query!(
        "SELECT id FROM whitelist WHERE election_id = $1 AND document_id_hash = $2",
        payload.election_id,
        doc_hash
    )
    .fetch_optional(&state.db)
    .await?;

    if exists.is_some() {
        Ok(StatusCode::OK)
    } else {
        Err(ApiError::Forbidden("Not in whitelist".to_string()))
    }
}

pub async fn validate_identity(
    State(state): State<AppState>,
    Json(payload): Json<ValidateIdentityRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input
    payload.validate().map_err(|e: ValidationErrors| ApiError::Validation(e.to_string()))?;

    // 1. Fetch election details
    let election = sqlx::query!(
        "SELECT election_salt, access_type as \"access_type: AccessType\" FROM elections WHERE id = $1",
        payload.election_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Election not found".to_string()))?;

    // 2. CHECK WHITELIST IF PRIVATE
    let doc_hash = crypto::hash_data(&payload.document_number);

    if election.access_type == AccessType::Private {
        let whitelisted = sqlx::query!(
            "SELECT id FROM whitelist WHERE election_id = $1 AND document_id_hash = $2",
            payload.election_id,
            doc_hash
        )
        .fetch_optional(&state.db)
        .await?;

        if whitelisted.is_none() {
            return Err(ApiError::Forbidden(
                "Identity not in whitelist for this private election".to_string(),
            ));
        }
    }

    // 3. Generate Nullifier
    let nullifier = crypto::generate_nullifier(&payload.document_number, &election.election_salt);
    tracing::info!(
        election_id = %payload.election_id,
        document_number_hash = %crypto::hash_data(&payload.document_number),
        "Identity validated, nullifier generated"
    );

    // 4. Generate JWT token
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(5))
        .ok_or_else(|| ApiError::Internal("Invalid timestamp".to_string()))?
        .timestamp() as usize;

    let claims = IdentityClaims {
        sub: nullifier.clone(),
        election_id: payload.election_id,
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
        Json(ValidateIdentityResponse {
            identity_token: token,
            nullifier,
        }),
    ))
}

pub async fn submit_vote(
    State(state): State<AppState>,
    Json(payload): Json<SubmitVoteRequest>,
) -> ApiResult<impl IntoResponse> {
    let receipt = state.vote_service.submit_vote(payload).await?;
    Ok((StatusCode::OK, Json(receipt)))
}

#[derive(Deserialize)]
pub struct VerifyReceiptRequest {
    pub signed_data: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct VerifyReceiptResponse {
    pub valid: bool,
    pub message: String,
}

pub async fn verify_receipt(
    Json(payload): Json<VerifyReceiptRequest>,
) -> ApiResult<impl IntoResponse> {
    use ed25519_dalek::VerifyingKey;
    
    // Decode public key
    let public_key_bytes = match hex::decode(&payload.public_key) {
        Ok(bytes) => bytes,
        Err(_) => return Ok((StatusCode::OK, Json(VerifyReceiptResponse {
            valid: false,
            message: "Invalid public key hex".to_string(),
        }))),
    };
    
    if public_key_bytes.len() != 32 {
        return Ok((StatusCode::OK, Json(VerifyReceiptResponse {
            valid: false,
            message: "Public key must be 32 bytes".to_string(),
        })));
    }
    
    let public_key_array: [u8; 32] = public_key_bytes.try_into().unwrap();
    let verifying_key = VerifyingKey::from_bytes(&public_key_array)
        .map_err(|_| ApiError::Validation("Invalid public key".to_string()))?;
    
    // Verify signature
    let valid = crate::crypto::verify_signature(&verifying_key, &payload.signed_data, &payload.signature);
    
    Ok((StatusCode::OK, Json(VerifyReceiptResponse {
        valid,
        message: if valid {
            "Signature is valid".to_string()
        } else {
            "Signature is invalid".to_string()
        },
    })))
}