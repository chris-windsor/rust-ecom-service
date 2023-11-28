#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry: i64,
    pub web_host: String,
    pub api_port: u16,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is not set");
        let jwt_expiry = std::env::var("JWT_EXPIRY").expect("JWT_EXPIRY is not set");
        let web_host = std::env::var("WEB_HOST").expect("WEB_HOST is not set");
        let api_port = std::env::var("API_PORT").expect("API_PORT is not set");
        Config {
            database_url,
            jwt_secret,
            jwt_expiry: jwt_expiry.parse().unwrap(),
            web_host,
            api_port: api_port.parse().unwrap(),
        }
    }
}
