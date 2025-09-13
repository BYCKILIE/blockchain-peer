mod block_form;
mod blocks_loader;
mod architecture;
mod tls_config;

use rustls::crypto::aws_lc_rs::default_provider;
use common::config::nats_config::NatsConfig;
use tokio::{io, task};
use tokio::task::JoinHandle;
use crate::architecture::start_consumer;

pub async fn power_module_consumer(nats_config: NatsConfig) -> io::Result<Vec<JoinHandle<()>>> {
    default_provider()
        .install_default()
        .expect("failed to install default CryptoProvider");
    
    let consumer_nats_config = nats_config.clone();
    
    let consumer_task = task::spawn(async {
        start_consumer(consumer_nats_config).await.unwrap();
    });
    Ok(vec![consumer_task])
}
