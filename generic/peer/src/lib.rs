use crate::administrator::background::{run_constructor_job, run_memory_jobs};
use crate::administrator::synchronizer::Synchronizer;
use crate::architecture::peer_server::start_server;
use crate::utils::aes_encryption::AesEncryption;
use crate::utils::rsa_encryption::RsaEncryption;
use common::config::peer_config::PeerConfig;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::{io, task};

pub mod administrator;
pub mod architecture;
pub mod communication;
pub mod memory;
pub mod utils;

pub async fn power_module_peer(peer_config: PeerConfig) -> io::Result<Vec<JoinHandle<()>>> {
    let server_url = format!("{}:{}", &peer_config.host, peer_config.port);
    let out_server_url = if peer_config.out_sub_name != 0 {
        format!(
            "{}.{}:{}",
            peer_config.out_sub_name, &peer_config.out_name, peer_config.out_port
        )
    } else {
        format!("{}:{}", &peer_config.out_name, peer_config.out_port)
    };

    let rsa_encryption = RsaEncryption::new(&peer_config.keys_path);
    let aes_encryption = AesEncryption::new();

    let module_server_url = server_url.clone();
    let module_max_connections = peer_config.peer_connections.clone();
    let module_rsa_encryption = rsa_encryption.clone();
    let module_aes_encryption = aes_encryption.clone();
    let module_out_server_url = out_server_url.clone();

    let synchronizer_server_url = out_server_url.clone();
    let synchronizer_committee_size = peer_config.committee_size.clone();
    let synchronizer_max_connections = peer_config.peer_connections.clone();

    let constructor_aes_encryption = aes_encryption.clone();
    let constructor_peer_config = peer_config.clone();
    let constructor_out_server_url = out_server_url.clone();

    let notify = Arc::new(Notify::new());
    let module_task_notify = notify.clone();
    let module_task = task::spawn(async move {
        start_server(
            module_server_url,
            module_max_connections,
            module_task_notify,
            module_rsa_encryption,
            module_aes_encryption,
            module_out_server_url,
        )
        .await
        .unwrap()
    });
    notify.notified().await;

    let mut synchronizer = Synchronizer::new(
        synchronizer_server_url,
        synchronizer_committee_size,
        synchronizer_max_connections,
    )
    .await;
    let synchronizer_task = task::spawn(async move {
        synchronizer.power_synchronizer().await;
    });

    let constructor_task = task::spawn(async move {
        run_constructor_job(
            constructor_aes_encryption,
            constructor_peer_config,
            constructor_out_server_url,
        )
        .await;
    });

    let memory_task = task::spawn(run_memory_jobs());

    Ok(vec![
        module_task,
        synchronizer_task,
        constructor_task,
        memory_task,
    ])
}
