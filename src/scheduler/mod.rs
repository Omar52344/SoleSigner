use crate::crypto::MerkleTree;
use sqlx::PgPool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler}; // Assuming exposed

pub async fn start_scheduler(pool: PgPool) {
    let sched = JobScheduler::new().await.unwrap();
    let pool = Arc::new(pool);

    // Job to close elections
    let pool_clone = pool.clone();
    let job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        // Every minute
        let pool = pool_clone.clone();
        Box::pin(async move {
            close_expired_elections(pool).await;
        })
    })
    .unwrap();

    sched.add(job).await.unwrap();
    sched.start().await.unwrap();
}

async fn close_expired_elections(pool: Arc<PgPool>) {
    // 1. Find elections to close
    // Note: sqlx query! macros might be tricky with enums if not careful, using simple string query or checking compatibility.
    // Casting status to text for comparison if needed or ensuring custom types work.
    let records = sqlx::query!(
        r#"
        SELECT id, title FROM elections
        WHERE status = 'OPEN' AND end_date <= NOW()
        "#
    )
    .fetch_all(&*pool)
    .await;

    if let Ok(elections) = records {
        for election in elections {
            println!("Closing election: {}", election.title);

            // 2. Lock / Set status to PROCESSING/CLOSING
            let _ = sqlx::query!(
                "UPDATE elections SET status = 'CLOSING' WHERE id = $1",
                election.id
            )
            .execute(&*pool)
            .await;

            // 3. Seal the election (Compute Merkle Tree)
            let ballots = sqlx::query!(
                "SELECT ballot_hash FROM ballots WHERE election_id = $1 ORDER BY created_at ASC",
                election.id
            )
            .fetch_all(&*pool)
            .await
            .unwrap_or(vec![]);

            let leaves: Vec<String> = ballots.into_iter().map(|b| b.ballot_hash).collect();
            let tree = MerkleTree::new(leaves);

            let root = tree.root;

            // 4. Update Election with Root and set to SEALED
            let _ = sqlx::query!(
                "UPDATE elections SET status = 'SEALED', merkle_root = $1 WHERE id = $2",
                root,
                election.id
            )
            .execute(&*pool)
            .await;

            println!("Election {} sealed. Root: {}", election.title, root);
        }
    }
}
