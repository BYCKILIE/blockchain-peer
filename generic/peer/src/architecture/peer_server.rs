use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;

use crate::memory::manager_client::MANAGER_CLIENT;
use crate::memory::manager_server::MANAGER_SERVER;

use crate::communication::listener::init_listener;
use crate::architecture::peer_client::start_client;
use crate::utils::rsa_encryption::RsaEncryption;
use common::logger::Logger;
use crate::utils::aes_encryption::AesEncryption;

pub async fn start_server(
    server_address: String,
    max_connections: u32,
    notify: Arc<Notify>,
    rsa_encryption: RsaEncryption,
    aes_encryption: AesEncryption,
    out_server_url: String,
) -> io::Result<()> {
    let listener = TcpListener::bind(&server_address).await?;

    Logger::console(
        "server",
        &format!("Server listening on {}", &server_address),
    );

    notify.notify_one();

    loop {
        let (mut socket, _) = listener.accept().await?;

        let identification = process_handshake(&mut socket, &rsa_encryption, max_connections).await;
        if let Err(_) = identification {
            continue;
        }
        let (client_url, key, iv) = identification?;

        // Logger::console(
        //     "server",
        //     &format!("Server identification complete for {}", &client_url),
        // );

        let mut peer_server = MANAGER_SERVER.lock().await;
        peer_server.add_server(client_url.clone(), socket);
        drop(peer_server);

        let mut peer_client = MANAGER_CLIENT.lock().await;
        peer_client.add_key(client_url.clone(), key, iv);
        if peer_client.contains(&client_url) {
            drop(peer_client);
            init_listener(&aes_encryption, &client_url, &out_server_url, max_connections).await;
        } else {
            drop(peer_client);
            let _ = start_client(
                max_connections,
                &aes_encryption,
                &client_url,
                &out_server_url,
            )
            .await;
        }

        // Logger::console(
        //     "server",
        //     &format!("Server added successfully {}", &client_url),
        // );
    }
}

async fn process_handshake(
    stream: &mut TcpStream,
    rsa_encryption: &RsaEncryption,
    max_connections: u32,
) -> io::Result<(String, [u8; 32], [u8; 16])> {
    let manager_client = MANAGER_CLIENT.lock().await;
    let actual_connections = manager_client.len() as u32;
    drop(manager_client);
    if actual_connections >= max_connections {
        stream.write_u8(2).await?;
        return Err(io::Error::new(ErrorKind::Other, "Maximum number of connections reached"));
    }
    stream.write_u8(1).await?;
    
    let pkey = rsa_encryption.get_public_pkey();
    let send_size = pkey.len();

    stream.write_u64(send_size as u64).await?;
    stream.write_all(&pkey).await?;
    stream.flush().await?;
    
    let encrypted_key_size = stream.read_u64().await?;
    let mut encrypted_key = vec![0; encrypted_key_size as usize];
    let key_bytes_read = stream.read_exact(&mut encrypted_key).await?;
    if key_bytes_read != encrypted_key_size as usize {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected key size",
        ));
    }
    let key = rsa_encryption.decrypt(&encrypted_key[..key_bytes_read]);
    if key.len() != 32 {
        return Err(io::Error::new(ErrorKind::InvalidData, "Invalid key"));
    }
    let key: [u8; 32] = key.try_into().unwrap();

    let encrypted_iv_size = stream.read_u64().await?;
    let mut encrypted_iv = vec![0; encrypted_iv_size as usize];
    let iv_bytes_read = stream.read_exact(&mut encrypted_iv).await?;
    if iv_bytes_read != encrypted_iv_size as usize {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected iv size",
        ));
    }
    let iv = rsa_encryption.decrypt(&encrypted_iv[..iv_bytes_read]);
    if iv.len() != 16 {
        return Err(io::Error::new(ErrorKind::InvalidData, "Invalid iv"));
    }
    let iv: [u8; 16] = iv.try_into().unwrap();

    let encrypted_url_size = stream.read_u64().await?;
    let mut encrypted_url = vec![0u8; encrypted_url_size as usize];
    let url_bytes_read = stream.read_exact(&mut encrypted_url).await?;
    if url_bytes_read != encrypted_url_size as usize {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected url size",
        ));
    }
    let url = rsa_encryption.decrypt(&encrypted_url[..url_bytes_read]);

    Ok((String::from_utf8_lossy(&url).to_string(), key, iv))
}
