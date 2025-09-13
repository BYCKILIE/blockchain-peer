use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub thread_pool: u32,
}
