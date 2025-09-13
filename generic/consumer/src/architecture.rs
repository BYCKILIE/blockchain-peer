use crate::block_form::BlockForm;
use crate::blocks_loader::BLOCKS_LOADER;
use common::config::nats_config::NatsConfig;
use common::logger::Logger;
use common::memory::blocks_queue::BLOCKS_QUEUE;
use futures::StreamExt;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::io;
use async_nats::{ConnectOptions, Subscriber};
use tokio::{task, time::{sleep, Duration}};
use tokio::sync::mpsc;
use crate::tls_config::build_tls_config;

pub async fn start_consumer(nats_config: NatsConfig) -> io::Result<()> {
    let (tx, rx) = mpsc::channel::<(String, String, String)>(100);
    
    let client_config = build_tls_config(&nats_config);
    
    let url = format!("{}:{}", &nats_config.host, &nats_config.port);
    
    let client = ConnectOptions::new()
        .tls_client_config(client_config)
        .connect(&url)
        .await.expect("Failed to connect to server");

    let mut subscribers: Vec<Subscriber> = Vec::new();
    for i in 1..(nats_config.no_channels + 1) {
        let subscriber = client.subscribe(format!("Channel{}", i)).await;
        if let Err(e) = subscriber {
            return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
        }
        subscribers.push(subscriber.unwrap());
    }

    for _ in 0..nats_config.no_channels {
        let sender = tx.clone();
        let sub = subscribers.pop().unwrap();
        task::spawn(async move {
            listen_channel(sub, sender).await;
        });
    }

    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(60)).await;

            let mut loader = BLOCKS_LOADER.lock().await;
            loader.check_timeouts();
            drop(loader);
        }
    });
    Logger::console(
        "consumer",
        &format!(
            "Consumer listening on {}:{}",
            nats_config.host, nats_config.port
        ),
    );

    assembler(rx).await;
    Ok(())
}

pub async fn listen_channel(mut subscriber: Subscriber, sender: Sender<(String, String, String)>) {
    while let Some(message) = subscriber.next().await {
        let parsed_message = BlockForm::from_json_bytes(&message.payload.to_vec());
        if let Err(_) = parsed_message {
            continue;
        }
        let parsed_message = parsed_message.unwrap();
        let organization = parsed_message.organization.clone();
        let feedback = parsed_message.feedback.clone();
        let mut blocks_loader = BLOCKS_LOADER.lock().await;

        if let Ok(hash) = blocks_loader.add_chunk(parsed_message) {
            drop(blocks_loader);
            sender.send((hash, organization, feedback)).await.unwrap();
        } else {
            drop(blocks_loader);
        }
    }
}

pub async fn assembler(mut block_ready_receiver: Receiver<(String, String, String)>) {
    while let Some((hash, organization, feedback)) = block_ready_receiver.recv().await {
        let mut blocks_loader = BLOCKS_LOADER.lock().await;
        let block = blocks_loader.get_block(&hash);
        drop(blocks_loader);

        if block.is_none() {
            continue;
        }
        let block = block.unwrap();

        let mut blocks_queue = BLOCKS_QUEUE.lock().await;
        blocks_queue.add_block(hash, organization, feedback, block);
        drop(blocks_queue);
    }
}