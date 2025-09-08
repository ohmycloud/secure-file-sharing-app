use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_maxage: i64,
    pub port: u64,
}

impl Config {
    pub fn init() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_maxage = env::var("JWT_MAXAGE")
            .expect("JWT_MAXAGE must be set")
            .parse::<i64>()
            .expect("JWT_MAXAGE must be a number");

        Self {
            database_url,
            jwt_secret,
            jwt_maxage,
            port: 8000,
        }
    }
}
