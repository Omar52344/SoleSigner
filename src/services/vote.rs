use sqlx::PgPool;
use chrono::Utc;
use hex;


use crate::{
    config::Config,
    crypto::{self, derive_election_signing_key, sign_message},
    error::{ApiError, ApiResult},
    repositories::election::{ElectionRepository, PostgresElectionRepository},
};

use crate::api::vote::{SubmitVoteRequest, VoteReceipt};

#[derive(Clone)]
pub struct VoteService {
    pool: PgPool,
    election_repo: PostgresElectionRepository,
    config: Config,
}

impl VoteService {
    pub fn new(pool: PgPool, election_repo: PostgresElectionRepository, config: Config) -> Self {
        Self { pool, election_repo, config }
    }

    pub async fn submit_vote(&self, request: SubmitVoteRequest) -> ApiResult<VoteReceipt> {
        // 1. Verify election is open
        let election_info = self.election_repo.check_election_open(request.election_id).await?;

        // 2. Start a transaction
        let mut tx = self.pool.begin().await?;

        // 3. Register Nullifier
        let insert_registry = sqlx::query!(
            "INSERT INTO voter_registry (election_id, nullifier_hash, identity_status, location_zone) VALUES ($1, $2, 'Validated', 'ZoneA')",
            request.election_id,
            request.nullifier
        )
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert_registry {
            if e.to_string().contains("unique constraint") {
                return Err(ApiError::Conflict("Vote already cast with this identity".to_string()));
            }
            return Err(ApiError::Internal(e.to_string()));
        }

        // 4. Create Ballot
        let choices_str = request.choices.to_string();
        let ballot_hash = crypto::hash_data(&format!("{}{}", request.request_id, choices_str));

        sqlx::query!(
            "INSERT INTO ballots (id, election_id, encrypted_choices, ballot_hash) VALUES ($1, $2, $3, $4)",
            request.request_id,
            request.election_id,
            request.choices,
            ballot_hash
        )
        .execute(&mut *tx)
        .await?;

        // 5. Commit transaction
        tx.commit().await?;
        tracing::info!(
            election_id = %request.election_id,
            nullifier = %request.nullifier,
            ballot_hash = %ballot_hash,
            "VOTE_CAST: Vote submitted successfully"
        );

        // 6. Generate signed receipt
        let timestamp = Utc::now();
        let signing_key = derive_election_signing_key(&self.config.jwt_secret, &election_info.election_salt);
        let public_key = hex::encode(signing_key.verifying_key().to_bytes());
        
        let signed_data = serde_json::json!({
            "ballot_hash": ballot_hash,
            "election_id": request.election_id,
            "timestamp": timestamp.to_rfc3339(),
        }).to_string();
        
        let signature = sign_message(&signing_key, &signed_data);
        
        Ok(VoteReceipt {
            ballot_hash,
            merkle_path: vec![],
            election_id: request.election_id,
            timestamp,
            public_key,
            signed_data,
            signature,
        })
    }
}