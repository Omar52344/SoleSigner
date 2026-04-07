use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub bcrypt_cost: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set".to_string())?;

        let bcrypt_cost = env::var("BCRYPT_COST")
            .map(|s| s.parse().unwrap_or(10))
            .unwrap_or(10);

        Ok(Config {
            database_url,
            jwt_secret,
            bcrypt_cost,
        })
    }
}
