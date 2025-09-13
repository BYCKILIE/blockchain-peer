use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NatsConfig {
    pub host: String,
    pub port: u32,
    pub no_channels: u32,
    pub thread_pool: u32,
    pub cert_store_path: String,
}