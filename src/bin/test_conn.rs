use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let url = "postgres://postgres:password@localhost:5432/solesigner?sslmode=disable";
    // We will use format debugging to catch the fields of the error struct
    match PgPoolOptions::new().connect(url).await {
        Ok(_) => println!("✅ Connection successful"),
        Err(e) => {
            // Debug print contains all fields
            let s = format!("{:#?}", e);
            use std::io::Write;
            let mut file = std::fs::File::create("error_details.txt").unwrap();
            writeln!(file, "{}", s).unwrap();
            println!("Logged to error_details.txt");
        }
    }
}
