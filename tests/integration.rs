use axum::{
    body::{Body, to_bytes},
    http::{self, Request, StatusCode},
    Router,
};
use hyper::body::Bytes;
use tower::ServiceExt;
use dotenvy::dotenv;
use serde_json::{json, Value};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use uuid::Uuid;
use chrono::{Utc, Duration};

use solesigner::{
    api,
    config::Config,
};

async fn setup_test_db() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test_user:test_pass@localhost:5432/test_db".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    // Clean up any existing test data (optional)
    sqlx::query("TRUNCATE TABLE admins, elections, whitelist, voter_registry, ballots CASCADE")
        .execute(&pool)
        .await
        .ok(); // ignore errors if tables don't exist
    
    pool
}

fn create_test_app(pool: PgPool) -> Router {
    let config = Config {
        database_url: "".to_string(), // not used because pool is passed directly
        jwt_secret: "test-jwt-secret".to_string(),
        bcrypt_cost: 10,
    };
    
    api::router(pool, config)
}

#[tokio::test]
async fn test_admin_registration() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());
    
    let payload = json!({
        "username": "testadmin",
        "password": "password123"
    });
    
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let bytes: Bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("id").is_some());
}

#[tokio::test]
async fn test_admin_login() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());
    
    // First register
    let register_payload = json!({
        "username": "testadmin2",
        "password": "password123"
    });
    
    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();
    
    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    // Then login
    let login_payload = json!({
        "username": "testadmin2",
        "password": "password123"
    });
    
    let login_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();
    
    let login_response = app.oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
    
    let bytes = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("token").is_some());
    assert_eq!(json["username"], "testadmin2");
}

#[tokio::test]
async fn test_create_election() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());
    
    // Register and login to get token
    let register_payload = json!({
        "username": "admin_election",
        "password": "password123"
    });
    
    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();
    
    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    let login_payload = json!({
        "username": "admin_election",
        "password": "password123"
    });
    
    let login_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    let bytes = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_json: Value = serde_json::from_slice(&bytes).unwrap();
    let token = login_json["token"].as_str().unwrap();
    
    // Create election
    let start_date = (Utc::now() - Duration::days(1)).to_rfc3339();
    let end_date = (Utc::now() + Duration::days(1)).to_rfc3339();
    let election_payload = json!({
        "title": "Test Election",
        "form_config": {
            "questions": [
                {
                    "id": "q1",
                    "text": "Choose option",
                    "type": "radio",
                    "options": ["Yes", "No"]
                }
            ]
        },
        "start_date": start_date,
        "end_date": end_date,
        "access_type": "PUBLIC"
    });
    
    let election_request = Request::builder()
        .method(http::Method::POST)
        .uri("/elections/create")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(election_payload.to_string()))
        .unwrap();
    
    let election_response = app.oneshot(election_request).await.unwrap();
    assert_eq!(election_response.status(), StatusCode::CREATED);
    
    let bytes = to_bytes(election_response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("id").is_some());
}

#[tokio::test]
async fn test_vote_flow() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());
    
    // Create admin, login, create election
    let register_payload = json!({
        "username": "admin_vote",
        "password": "password123"
    });
    
    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();
    
    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    let login_payload = json!({
        "username": "admin_vote",
        "password": "password123"
    });
    
    let login_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    let bytes = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_json: Value = serde_json::from_slice(&bytes).unwrap();
    let token = login_json["token"].as_str().unwrap();
    
    // Create election
    let start_date = (Utc::now() - Duration::days(1)).to_rfc3339();
    let end_date = (Utc::now() + Duration::days(1)).to_rfc3339();
    let election_payload = json!({
        "title": "Vote Test Election",
        "form_config": {
            "questions": [
                {
                    "id": "q1",
                    "text": "Choose option",
                    "type": "radio",
                    "options": ["Yes", "No"]
                }
            ]
        },
        "start_date": start_date,
        "end_date": end_date,
        "access_type": "PUBLIC"
    });
    
    let election_request = Request::builder()
        .method(http::Method::POST)
        .uri("/elections/create")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(election_payload.to_string()))
        .unwrap();
    
    let election_response = app.clone().oneshot(election_request).await.unwrap();
    assert_eq!(election_response.status(), StatusCode::CREATED);
    let bytes: Bytes = to_bytes(election_response.into_body(), usize::MAX).await.unwrap();
    let election_json: Value = serde_json::from_slice(&bytes).unwrap();
    let election_id = election_json["id"].as_str().unwrap();
    
    // Start election via API
    let start_request = Request::builder()
        .method(http::Method::POST)
        .uri(format!("/elections/{}/start", election_id))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(""))
        .unwrap();
    
    let start_response = app.clone().oneshot(start_request).await.unwrap();
    assert_eq!(start_response.status(), StatusCode::OK);
    
    // Step 1: Validate identity
    let identity_payload = json!({
        "election_id": election_id,
        "document_number": "123456789"
    });
    
    let identity_request = Request::builder()
        .method(http::Method::POST)
        .uri("/vote/validate-identity")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(identity_payload.to_string()))
        .unwrap();
    
    let identity_response = app.clone().oneshot(identity_request).await.unwrap();
    let status = identity_response.status();
    if status != StatusCode::OK {
        let bytes = to_bytes(identity_response.into_body(), usize::MAX).await.unwrap();
        panic!("Identity validation failed: {} body: {:?}", status, String::from_utf8_lossy(&bytes));
    }
    let bytes: Bytes = to_bytes(identity_response.into_body(), usize::MAX).await.unwrap();
    let identity_json: Value = serde_json::from_slice(&bytes).unwrap();
    let nullifier = identity_json["nullifier"].as_str().unwrap();
    let identity_token = identity_json["identity_token"].as_str().unwrap();
    
    // Step 2: Submit vote
    let vote_payload = json!({
        "election_id": election_id,
        "choices": {
            "q1": "Yes"
        },
        "nullifier": nullifier,
        "request_id": Uuid::new_v4().to_string()
    });
    
    let vote_request = Request::builder()
        .method(http::Method::POST)
        .uri("/vote/submit")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(vote_payload.to_string()))
        .unwrap();
    
    let vote_response = app.clone().oneshot(vote_request).await.unwrap();
    let status = vote_response.status();
    if status != StatusCode::OK {
        let bytes = to_bytes(vote_response.into_body(), usize::MAX).await.unwrap();
        panic!("Vote submission failed: {} body: {:?}", status, String::from_utf8_lossy(&bytes));
    }
    
    let bytes: Bytes = to_bytes(vote_response.into_body(), usize::MAX).await.unwrap();
    let vote_json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(vote_json.get("ballot_hash").is_some());
    assert!(vote_json.get("election_id").is_some());
}

#[tokio::test]
async fn test_replay_vote_protection() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());
    
    // Create admin, login, create election, start election
    let register_payload = json!({
        "username": "admin_replay",
        "password": "password123"
    });
    
    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();
    
    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    let login_payload = json!({
        "username": "admin_replay",
        "password": "password123"
    });
    
    let login_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    let bytes = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_json: Value = serde_json::from_slice(&bytes).unwrap();
    let token = login_json["token"].as_str().unwrap();
    
    // Create election
    let start_date = (Utc::now() - Duration::days(1)).to_rfc3339();
    let end_date = (Utc::now() + Duration::days(1)).to_rfc3339();
    let election_payload = json!({
        "title": "Replay Test Election",
        "form_config": {
            "questions": [
                {
                    "id": "q1",
                    "text": "Choose option",
                    "type": "radio",
                    "options": ["Yes", "No"]
                }
            ]
        },
        "start_date": start_date,
        "end_date": end_date,
        "access_type": "PUBLIC"
    });
    
    let election_request = Request::builder()
        .method(http::Method::POST)
        .uri("/elections/create")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(election_payload.to_string()))
        .unwrap();
    
    let election_response = app.clone().oneshot(election_request).await.unwrap();
    assert_eq!(election_response.status(), StatusCode::CREATED);
    let bytes: Bytes = to_bytes(election_response.into_body(), usize::MAX).await.unwrap();
    let election_json: Value = serde_json::from_slice(&bytes).unwrap();
    let election_id = election_json["id"].as_str().unwrap();
    
    // Start election via API
    let start_request = Request::builder()
        .method(http::Method::POST)
        .uri(format!("/elections/{}/start", election_id))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(""))
        .unwrap();
    
    let start_response = app.clone().oneshot(start_request).await.unwrap();
    assert_eq!(start_response.status(), StatusCode::OK);
    
    // Step 1: Validate identity
    let identity_payload = json!({
        "election_id": election_id,
        "document_number": "REPLAY_DOC"
    });
    
    let identity_request = Request::builder()
        .method(http::Method::POST)
        .uri("/vote/validate-identity")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(identity_payload.to_string()))
        .unwrap();
    
    let identity_response = app.clone().oneshot(identity_request).await.unwrap();
    assert_eq!(identity_response.status(), StatusCode::OK);
    let bytes: Bytes = to_bytes(identity_response.into_body(), usize::MAX).await.unwrap();
    let identity_json: Value = serde_json::from_slice(&bytes).unwrap();
    let nullifier = identity_json["nullifier"].as_str().unwrap();
    
    // Step 2: Submit vote first time (should succeed)
    let vote_payload = json!({
        "election_id": election_id,
        "choices": {
            "q1": "Yes"
        },
        "nullifier": nullifier,
        "request_id": Uuid::new_v4().to_string()
    });
    
    let vote_request = Request::builder()
        .method(http::Method::POST)
        .uri("/vote/submit")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(vote_payload.to_string()))
        .unwrap();
    
    let vote_response = app.clone().oneshot(vote_request).await.unwrap();
    assert_eq!(vote_response.status(), StatusCode::OK);
    
    // Step 3: Submit same vote again (should fail with 409 Conflict)
    let vote_payload2 = json!({
        "election_id": election_id,
        "choices": {
            "q1": "No"
        },
        "nullifier": nullifier,
        "request_id": Uuid::new_v4().to_string()
    });
    
    let vote_request2 = Request::builder()
        .method(http::Method::POST)
        .uri("/vote/submit")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(vote_payload2.to_string()))
        .unwrap();
    
    let vote_response2 = app.clone().oneshot(vote_request2).await.unwrap();
    assert_eq!(vote_response2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_sql_injection_attempt() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone());
    
    // Create admin, login, create election, start election
    let register_payload = json!({
        "username": "admin_sql",
        "password": "password123"
    });
    
    let register_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/register")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();
    
    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    let login_payload = json!({
        "username": "admin_sql",
        "password": "password123"
    });
    
    let login_request = Request::builder()
        .method(http::Method::POST)
        .uri("/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();
    
    let login_response = app.clone().oneshot(login_request).await.unwrap();
    let bytes = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_json: Value = serde_json::from_slice(&bytes).unwrap();
    let token = login_json["token"].as_str().unwrap();
    
    // Create election
    let start_date = (Utc::now() - Duration::days(1)).to_rfc3339();
    let end_date = (Utc::now() + Duration::days(1)).to_rfc3339();
    let election_payload = json!({
        "title": "SQL Injection Test Election",
        "form_config": {
            "questions": [
                {
                    "id": "q1",
                    "text": "Choose option",
                    "type": "radio",
                    "options": ["Yes", "No"]
                }
            ]
        },
        "start_date": start_date,
        "end_date": end_date,
        "access_type": "PUBLIC"
    });
    
    let election_request = Request::builder()
        .method(http::Method::POST)
        .uri("/elections/create")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(election_payload.to_string()))
        .unwrap();
    
    let election_response = app.clone().oneshot(election_request).await.unwrap();
    assert_eq!(election_response.status(), StatusCode::CREATED);
    let bytes: Bytes = to_bytes(election_response.into_body(), usize::MAX).await.unwrap();
    let election_json: Value = serde_json::from_slice(&bytes).unwrap();
    let election_id = election_json["id"].as_str().unwrap();
    
    // Start election via API
    let start_request = Request::builder()
        .method(http::Method::POST)
        .uri(format!("/elections/{}/start", election_id))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(""))
        .unwrap();
    
    let start_response = app.clone().oneshot(start_request).await.unwrap();
    assert_eq!(start_response.status(), StatusCode::OK);
    
    // Attempt SQL injection in document_number (should not crash)
    let payload = json!({
        "election_id": election_id,
        "document_number": "123' OR '1'='1"
    });
    
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/vote/validate-identity")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    // Should return either validation error or not found, but not crash
    assert!(response.status().is_client_error() || response.status().is_success());
    // Ensure no 5xx error
    assert!(!response.status().is_server_error());
}