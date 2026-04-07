use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{ApiError, ApiResult};
use crate::types::{ElectionStatus, AccessType};

#[derive(Debug)]
pub struct ElectionRecord {
    pub id: Uuid,
    pub title: String,
    pub form_config: serde_json::Value,
    pub status: ElectionStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub access_type: AccessType,
    pub merkle_root: Option<String>,
    pub election_salt: String,
}

#[derive(Debug)]
pub struct OpenElectionInfo {
    pub id: Uuid,
    pub title: String,
    pub election_salt: String,
}

#[allow(async_fn_in_trait)]
pub trait ElectionRepository {
    async fn find_by_id(&self, election_id: Uuid) -> ApiResult<Option<ElectionRecord>>;
    async fn check_election_open(&self, election_id: Uuid) -> ApiResult<OpenElectionInfo>;
}

#[derive(Clone)]
pub struct PostgresElectionRepository {
    pool: PgPool,
}

impl PostgresElectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ElectionRepository for PostgresElectionRepository {
    async fn find_by_id(&self, election_id: Uuid) -> ApiResult<Option<ElectionRecord>> {
        let rec = sqlx::query_as!(
            ElectionRecord,
            r#"
            SELECT id, title, form_config, status as "status: _", start_date, end_date,
                   access_type as "access_type: _", merkle_root, election_salt
            FROM elections WHERE id = $1
            "#,
            election_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rec)
    }

    async fn check_election_open(&self, election_id: Uuid) -> ApiResult<OpenElectionInfo> {
        let rec = sqlx::query!(
            r#"
            SELECT id, title, election_salt
            FROM elections
            WHERE id = $1
                AND status = 'OPEN'::election_status
                AND NOW() BETWEEN start_date AND end_date
            "#,
            election_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match rec {
            Some(rec) => Ok(OpenElectionInfo {
                id: rec.id,
                title: rec.title,
                election_salt: rec.election_salt,
            }),
            None => Err(ApiError::Validation("Election is not open for voting".to_string())),
        }
    }
}