#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub jwt_expiry: i64,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let database_name = std::env::var("DATABASE_NAME").expect("DATABASE_NAME is not set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is not set");
        let jwt_expiry = std::env::var("JWT_EXPIRY").expect("JWT_EXPIRY is not set");
        Config {
            database_url,
            database_name,
            jwt_secret,
            jwt_expiry: jwt_expiry.parse::<i64>().unwrap(),
        }
    }
}
