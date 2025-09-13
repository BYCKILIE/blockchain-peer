use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PeerConfig {
    pub host: String,
    pub port: u32,

    pub thread_pool: u32,
    
    pub out_name: String,
    pub out_sub_name: u32,
    pub out_port: u32,
    
    pub min_ref: u32,
    pub max_ref: u32,
    
    pub peer_connections: u32,
    pub committee_size: u32,
    
    pub keys_path: String,
}