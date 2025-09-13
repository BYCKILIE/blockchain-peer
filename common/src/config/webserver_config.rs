use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u32,
    pub thread_pool: u32,
}