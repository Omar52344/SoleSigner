use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the 'postgres' database directly
    let app_url = "postgres://postgres:admin@localhost:5432/postgres";

    println!("Connecting to {}...", app_url);
    let pool = PgPoolOptions::new().connect(app_url).await?;
    println!("✅ Connected successfully.");

    println!("Running migrations...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("✅ Migrations applied successfully."),
        Err(e) => println!("❌ Failed to apply migrations: {}", e),
    }

    Ok(())
}
