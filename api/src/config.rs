#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let database_name = std::env::var("DATABASE_NAME").expect("DATABASE_NAME is not set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is not set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN is not set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE is not set");
        Config {
            database_url,
            database_name,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
        }
    }
}
