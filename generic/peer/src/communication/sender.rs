use crate::memory::manager_client::MANAGER_CLIENT;
use crate::utils::aes_encryption::AesEncryption;
use std::io::ErrorKind;
use tokio::io;
use tokio::io::AsyncWriteExt;

pub async fn write_node(address: &str, message: Vec<u8>) -> io::Result<()> {
    let manager = MANAGER_CLIENT.lock().await;
    let client = manager.use_client(address);
    if client.is_none() {
        drop(manager);
        return Err(io::Error::new(ErrorKind::NotFound, "No client available"));
    }
    let (sending_ark, (key, iv)) = client.unwrap();

    let encrypted = AesEncryption::encrypt(&message, key, iv);

    let mut sending_end = sending_ark.lock().await;

    sending_end.write_u64(encrypted.len() as u64).await?;
    sending_end.write_all(&encrypted).await?;
    sending_end.flush().await?;

    drop(sending_end);
    drop(manager);

    Ok(())
}
