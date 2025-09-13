use std::time::Duration;
use tokio::time::sleep;
use common::config::peer_config::PeerConfig;
use crate::architecture::constructor::constructor_job;
use crate::memory::manager_server::MANAGER_SERVER;
use crate::utils::aes_encryption::AesEncryption;

pub async fn run_memory_jobs() {
    loop {
        let mut manager_server = MANAGER_SERVER.lock().await;
        manager_server.cleanup();
        drop(manager_server);
        
        sleep(Duration::from_secs(60)).await;
    }
}

pub async fn run_constructor_job(
    encryption: AesEncryption,
    peer_config: PeerConfig,
    out_server_url: String,
) {
    loop {
        constructor_job(&encryption, &peer_config, &out_server_url).await;
        sleep(Duration::from_secs(5)).await;
    }
}