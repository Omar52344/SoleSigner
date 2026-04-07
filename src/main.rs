

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use axum::{body::Body, http::Request};
use std::net::SocketAddr;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use std::env;
use tower_http::trace::TraceLayer;


use solesigner::{api, config, scheduler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load environment variables
    dotenv().ok();

    // 2. Initialize logging
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,solesigner=debug"));
    
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false);
    
    // JSON logs for production if JSON_LOGS env var is set
    let json_logs = env::var("JSON_LOGS").unwrap_or_default();
    if json_logs == "1" || json_logs.to_lowercase() == "true" {
        let json_layer = fmt::layer()
            .json()
            .with_target(true)
            .with_level(true)
            .with_current_span(true)
            .with_thread_ids(true);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }

    // 3. Load configuration
    let config = config::Config::from_env()
        .map_err(|e| Box::<dyn std::error::Error>::from(e))?;

    // 4. Connect to Database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
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
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            let request_id = Uuid::new_v4().to_string();
            tracing::info_span!(
                "request",
                request_id = %request_id,
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let router = api::router(pool, config)
        .layer(cors)
        .layer(trace_layer);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("🚀 Server listening on {}", addr);
    axum::serve(listener, router).await?;

    Ok(())
}
