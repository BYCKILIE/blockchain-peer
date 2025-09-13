use std::io::ErrorKind;
use crate::communication::listener::init_listener;
use crate::memory::manager_client::MANAGER_CLIENT;
use crate::memory::manager_server::MANAGER_SERVER;
use crate::utils::aes_encryption::AesEncryption;
use crate::utils::rsa_encryption::RsaEncryption;
use common::logger::Logger;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn start_client(
    selection_size: u32,
    aes_encryption: &AesEncryption,
    client_address: &str,
    out_server_url: &str,
) -> io::Result<()> {
    let mut socket = TcpStream::connect(&client_address).await?;

    process_handshake(
        &mut socket,
        out_server_url,
        aes_encryption.get_key(),
        aes_encryption.get_iv(),
    )
    .await?;

    // Logger::console(
    //     "client",
    //     &format!("Accepted connection from {}", client_address),
    // );

    let mut manager_client = MANAGER_CLIENT.lock().await;
    manager_client.add_client(client_address.to_string(), socket);
    drop(manager_client);

    let manager_server = MANAGER_SERVER.lock().await;
    if manager_server.contains(&client_address) {
        drop(manager_server);
        init_listener(aes_encryption, client_address, out_server_url, selection_size).await;
    } else {
        drop(manager_server);
    }

    // Logger::console(
    //     "client",
    //     &format!("Successfully added client {}", client_address),
    // );
    Ok(())
}

async fn process_handshake(
    stream: &mut TcpStream,
    server_address: &str,
    key: &[u8; 32],
    iv: &[u8; 16],
) -> io::Result<()> {
    let signal = stream.read_u8().await?;
    if signal == 2 {
        return Err(io::Error::new(ErrorKind::Other, "Node full"));
    }
    
    let receive_size = stream.read_u64().await?;
    let mut buffer = vec![0u8; receive_size as usize];

    let bytes_read = stream.read_exact(&mut buffer).await?;
    if bytes_read != receive_size as usize {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected message size",
        ));
    }

    let pkey = RsaEncryption::parse_public(&buffer[..bytes_read])?;

    let encrypted_key = RsaEncryption::encrypt(key, &pkey);
    stream.write_u64(encrypted_key.len() as u64).await?;
    stream.write_all(&encrypted_key).await?;

    let encrypted_iv = RsaEncryption::encrypt(iv, &pkey);
    stream.write_u64(encrypted_iv.len() as u64).await?;
    stream.write_all(&encrypted_iv).await?;

    let encrypted_server_address = RsaEncryption::encrypt(server_address.as_bytes(), &pkey);
    stream
        .write_u64(encrypted_server_address.len() as u64)
        .await?;
    stream.write_all(&encrypted_server_address).await?;

    stream.flush().await?;

    Ok(())
}
