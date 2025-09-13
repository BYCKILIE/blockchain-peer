use std::collections::HashSet;
use crate::architecture::peer_client::start_client;
use crate::memory::manager_client::MANAGER_CLIENT;
use common::config::peer_config::PeerConfig;
use rand::Rng;
use crate::utils::aes_encryption::AesEncryption;

pub async fn constructor_job(
    aes_encryption: &AesEncryption,
    peer_config: &PeerConfig,
    out_server_url: &str,
) {
    let connected_clients_mutex = MANAGER_CLIENT.lock().await;
    let connected_clients = connected_clients_mutex.get_connected_clients();
    drop(connected_clients_mutex);

    if connected_clients.len() < peer_config.peer_connections as usize {
        let mut clients: HashSet<String> = HashSet::new();
        for _ in connected_clients.len()..peer_config.peer_connections as usize {
            let client_url = random_client_url(peer_config);
            if connected_clients.contains(&client_url) || client_url == out_server_url {
                continue;
            }
            clients.insert(client_url);
        }
        for client in clients {
            let _ = start_client(
                peer_config.peer_connections,
                aes_encryption,
                &client,
                out_server_url,
            ).await;
        }
    }
}

fn random_client_url(peer_config: &PeerConfig) -> String {
    let mut rng = rand::rng();
    let choice = rng.random_range(peer_config.min_ref as usize..peer_config.max_ref as usize + 1);
    let port = 7000 + choice;
    
    if peer_config.out_sub_name != 0 {
        format!(
            "{}.{}:{}",
            choice, &peer_config.out_name, port
        )
    } else {
        format!("{}:{}", &peer_config.out_name, port)
    }
}
