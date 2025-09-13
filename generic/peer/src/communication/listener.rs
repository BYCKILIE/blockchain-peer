use std::io::ErrorKind;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task;
use common::memory::lazy_clients::LAZY_CLIENTS;
use crate::utils::aes_encryption::AesEncryption;
use crate::administrator::interpreter::InterpreterConstruct;
use crate::memory::manager_client::MANAGER_CLIENT;
use crate::memory::manager_server::MANAGER_SERVER;

pub async fn init_listener(aes_encryption: &AesEncryption, client_address: &str, out_server_url: &str, max_connections: u32) {
    let encryption_clone = aes_encryption.clone();
    let client_address_clone = client_address.to_string();
    let out_server_url_clone = out_server_url.to_string();
    
    task::spawn(async move {
        listen_node(encryption_clone, client_address_clone, out_server_url_clone, max_connections).await;
    });
}

pub async fn listen_node(encryption: AesEncryption, client_address: String, out_server_url: String, max_connections: u32) {
    let mut lazy_clients = LAZY_CLIENTS.lock().await;
    lazy_clients.add(client_address.to_string());
    drop(lazy_clients);
    
    let _ = event_loop(&encryption, &client_address, &out_server_url, max_connections).await;
    
    let mut manager_client = MANAGER_CLIENT.lock().await;
    manager_client.remove_client(&client_address);
    drop(manager_client);
}

async fn event_loop(encryption: &AesEncryption, client_address: &str, out_server_url: &str, max_connections: u32) -> io::Result<()> {
    let mut peer_server = MANAGER_SERVER.lock().await;
    let stream = peer_server.remove_server(client_address);
    drop(peer_server);
    if stream.is_none() {
        return Err(io::Error::new(ErrorKind::ConnectionAborted, "Error server listener"));
    }
    let mut stream = stream.unwrap();

    loop {
        stream.flush().await?;
        let message = process_incoming(&mut stream).await;
        if let Err(_) = message {
            continue;
        }
        let message = message?;

        let decrypted = encryption.decrypt(&message);

        let apply_construct = InterpreterConstruct::from_raw(decrypted);
        if let Err(_) = apply_construct {
            continue;
        }
        let apply_construct = apply_construct?;
        
        let mut distribute_construct = apply_construct.clone();
        distribute_construct.from = out_server_url.to_string();

        let _ = distribute_construct.distribute(max_connections).await;

        let _ = apply_construct.apply().await;
    }
}

async fn process_incoming(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let message_size = stream.read_u64().await? as usize;
    
    let mut buffer = vec![0u8; message_size];
    let actual_message_size =  stream.read_exact(&mut buffer).await?;
    
    if actual_message_size != message_size {
        return Err(io::Error::new(ErrorKind::UnexpectedEof, "Unexpected message size"));
    }
    
    Ok(buffer)
}
