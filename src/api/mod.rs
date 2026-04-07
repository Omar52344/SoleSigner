use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::{config::Config, repositories::election::PostgresElectionRepository, services::vote::VoteService};

pub mod auth;
pub mod elections;
pub mod vote;
pub mod whitelist;
pub mod results;
pub mod openapi;
pub mod health;


#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub election_repo: PostgresElectionRepository,
    pub vote_service: VoteService,
}



pub fn router(pool: PgPool, config: Config) -> Router {
    let election_repo = PostgresElectionRepository::new(pool.clone());
    let vote_service = VoteService::new(pool.clone(), election_repo.clone(), config.clone());
    let state = AppState { db: pool, config, election_repo, vote_service };
    Router::new()
        // Auth Routes
        .route("/health", get(health::health))
        .route("/auth/register", post(auth::register_admin))
        .route("/auth/login", post(auth::login_admin))
        // Protected Routes (handled by extractors in handlers)
        .route("/elections/create", post(elections::create_election))
        .route("/elections", get(elections::list_elections))
        // Public Routes
        .route("/elections/:id", get(elections::get_election))
        .route("/elections/:id/stats", get(elections::get_election_stats)) // Could be protected
        .route("/elections/:id/start", post(elections::start_election)) // Should be protected, but for now Public
        .route("/elections/:id/close", post(elections::close_election)) // Should be protected
        .route("/elections/:id/results", get(results::get_election_results)) // Public results
        .route(
            "/elections/:id/whitelist",
            get(whitelist::get_whitelist).post(whitelist::add_whitelist),
        )
        .route("/vote/validate-identity", post(vote::validate_identity))
        .route("/vote/check-eligibility", post(vote::check_eligibility))
        .route("/vote/submit", post(vote::submit_vote))
        .route("/vote/verify-receipt", post(vote::verify_receipt))
        .route("/audit/:election_id/verify", get(results::verify_election))
        .route("/openapi.json", get(openapi::serve_openapi))
        .route("/openapi.yaml", get(openapi::serve_openapi_yaml))
        .with_state(state)
}



// --- DTOs ---









// --- Handlers ---




// --- Missing DTOs ---











// --- Missing Handlers ---





















