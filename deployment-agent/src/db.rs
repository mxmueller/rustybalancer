use redis::{Commands, Connection};
use std::env;
use dotenv::dotenv;

pub fn get_redis_connection() -> Connection {
    dotenv().ok();

    let redis_host = env::var("REDIS_HOST").expect("REDIS_HOST must be set in .env");
    let redis_port = env::var("REDIS_PORT").unwrap_or("6379".to_string());
    let redis_url = format!("redis://{}:{}", redis_host, redis_port);

    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    client.get_connection().expect("Failed to connect to Redis")
}

pub fn get_config_value<T: redis::FromRedisValue>(conn: &mut Connection, key: &str) -> Option<T> {
    conn.get(key).ok()
}

pub fn set_config_value<T: redis::ToRedisArgs>(conn: &mut Connection, key: &str, value: T) -> redis::RedisResult<()> {
    conn.set(key, value)
}

pub fn check_config_value_exists(conn: &mut Connection, key: &str) -> bool {
    conn.exists(key).unwrap_or(false)
}

pub fn init(conn: &mut Connection) {
    let key = "DEFAULT_CONTAINER";
    if !check_config_value_exists(conn, key) {
        if let Ok(value) = env::var(key).and_then(|v| v.parse::<i32>().map_err(|_| env::VarError::NotPresent)) {
            let _: () = set_config_value(conn, key, value).unwrap();
        }
    }
}