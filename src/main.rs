mod api;
mod crypto;
mod identity;
mod scheduler;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load environment variables
    dotenv().ok();

    // 2. Initialize logging
    tracing_subscriber::fmt::init();

    // 3. Connect to Database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✅ Connected to Database");

    // 4. Run Migrations (Optional, simplifies setup)
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("✅ Migrations Applied");

    // 5. Start Scheduler
    let pool_for_scheduler = pool.clone();
    tokio::spawn(async move {
        scheduler::start_scheduler(pool_for_scheduler).await;
    });

    println!("✅ Scheduler Started");

    // 6. Start API Server
    let router = api::router(pool);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("🚀 Server listening on {}", addr);
    axum::serve(listener, router).await?;

    Ok(())
}
