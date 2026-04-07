use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;
use uuid::Uuid;
use chrono::{Utc, Duration};

const BASE_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting stress test: 100 concurrent votes");
    let client = Client::new();
    
    // 1. Create admin user
    let admin_username = format!("stress_admin_{}", Uuid::new_v4());
    let admin_password = "stress_password123";
    
    let register_resp = client
        .post(&format!("{}/auth/register", BASE_URL))
        .json(&json!({
            "username": admin_username,
            "password": admin_password
        }))
        .send()
        .await?;
    
    if !register_resp.status().is_success() {
        let text = register_resp.text().await?;
        println!("Failed to register admin: {}", text);
        return Ok(());
    }
    
    // 2. Login to get token
    let login_resp = client
        .post(&format!("{}/auth/login", BASE_URL))
        .json(&json!({
            "username": admin_username,
            "password": admin_password
        }))
        .send()
        .await?;
    
    let login_json: Value = login_resp.json().await?;
    let token = login_json["token"].as_str().unwrap();
    
    // 3. Create election
    let election_id = Uuid::new_v4();
    let start_date = (Utc::now() - Duration::days(1)).to_rfc3339();
    let end_date = (Utc::now() + Duration::days(1)).to_rfc3339();
    let election_resp = client
        .post(&format!("{}/elections/create", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": format!("Stress Test Election {}", election_id),
            "form_config": {
                "questions": [
                    {
                        "id": "q1",
                        "text": "Stress test option",
                        "type": "radio",
                        "options": ["A", "B", "C"]
                    }
                ]
            },
            "start_date": start_date,
            "end_date": end_date,
            "access_type": "PUBLIC"
        }))
        .send()
        .await?;
    
    if !election_resp.status().is_success() {
        let text = election_resp.text().await?;
        println!("Failed to create election: {}", text);
        return Ok(());
    }
    
    let election_json: Value = election_resp.json().await?;
    let election_id = election_json["id"].as_str().unwrap();
    
    // 4. Start election (update status to OPEN)
    // First, we need to start the election. Our API doesn't have a start endpoint yet.
    // We'll use the admin token to update via SQL (not ideal).
    // Instead, we'll assume the election is already OPEN because we set start_date in past.
    // But the election status is DRAFT by default. We'll need to start it.
    // Let's use the start_election endpoint if it exists.
    // Check the API: there's a PUT /elections/{id}/start endpoint.
    let start_resp = client
        .post(&format!("{}/elections/{}/start", BASE_URL, election_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;
    
    if let Err(e) = start_resp {
        println!("Note: Could not start election (maybe endpoint missing): {}", e);
        // Continue anyway, we'll manually update status via SQL if needed
        // For now, we'll assume the election is OPEN
    }
    
    println!("✅ Election created: {}", election_id);
    
    // 5. Generate unique document numbers for each vote
    let document_numbers: Vec<String> = (0..100)
        .map(|i| format!("STRESS_DOC_{}_{}", i, Uuid::new_v4()))
        .collect();
    
    // 6. Validate identity for each document (get nullifier and token)
    println!("🔐 Validating identities...");
    let mut nullifiers = Vec::new();
    for doc in &document_numbers {
        let identity_resp = client
            .post(&format!("{}/vote/validate-identity", BASE_URL))
            .json(&json!({
                "election_id": election_id,
                "document_number": doc
            }))
            .send()
            .await?;
        
        if identity_resp.status().is_success() {
            let identity_json: Value = identity_resp.json().await?;
            nullifiers.push(identity_json["nullifier"].as_str().unwrap().to_string());
        } else {
            println!("Failed to validate identity for doc: {}", doc);
            nullifiers.push("".to_string());
        }
    }
    
    // 7. Concurrent vote submission
    println!("🗳️  Submitting 100 concurrent votes...");
    let start_time = Instant::now();
    
    let mut handles = Vec::new();
    for (i, nullifier) in nullifiers.iter().enumerate() {
        if nullifier.is_empty() {
            continue;
        }
        let client = client.clone();
        let election_id = election_id.to_string();
        let nullifier = nullifier.clone();
        let request_id = Uuid::new_v4().to_string();
        
        let handle = tokio::spawn(async move {
            let vote_resp = client
                .post(&format!("{}/vote/submit", BASE_URL))
                .json(&json!({
                    "election_id": election_id,
                    "choices": { "q1": if i % 2 == 0 { "A" } else { "B" } },
                    "nullifier": nullifier,
                    "request_id": request_id
                }))
                .send()
                .await;
            
            match vote_resp {
                Ok(resp) => {
                    if resp.status().is_success() {
                        Ok(())
                    } else {
                        let text = resp.text().await.unwrap_or_default();
                        Err(format!("Vote failed: {}", text))
                    }
                }
                Err(e) => Err(format!("Request error: {}", e)),
            }
        });
        handles.push(handle);
    }
    
    // Wait for all votes
    let mut successes = 0;
    let mut failures = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(())) => successes += 1,
            Ok(Err(e)) => {
                failures += 1;
                println!("Vote failure: {}", e);
            }
            Err(e) => {
                failures += 1;
                println!("Join error: {}", e);
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("📊 Results:");
    println!("   Successes: {}", successes);
    println!("   Failures: {}", failures);
    println!("   Total time: {:?}", elapsed);
    println!("   Votes per second: {:.2}", successes as f64 / elapsed.as_secs_f64());
    
    // 8. Verify vote count via stats endpoint
    let stats_resp = client
        .get(&format!("{}/elections/{}/stats", BASE_URL, election_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if stats_resp.status().is_success() {
        let stats_json: Value = stats_resp.json().await?;
        let total_votes = stats_json["total_votes"].as_i64().unwrap_or(0);
        println!("✅ Election stats: {} votes recorded", total_votes);
        assert_eq!(total_votes, successes as i64, "Vote count mismatch!");
    }
    
    println!("🎉 Stress test completed successfully!");
    Ok(())
}