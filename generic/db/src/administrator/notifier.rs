use reqwest::Client;
use serde::Serialize;
use common::memory::feedback_queue::FEEDBACK_QUEUE;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    block_hash: &'a str,
    full_hash:  &'a str,
}

pub async fn start_notifier(mut hash_receiver: UnboundedReceiver<Vec<(String, String)>>) {
    let client = Client::new();
    
    while let Some(hashes) = hash_receiver.recv().await {
        let mut feedback_queue = FEEDBACK_QUEUE.lock().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        for (block_hash, full_hash) in hashes {
            if let Some(url) = feedback_queue.get_block(&block_hash) {
                let body = Payload {
                    block_hash: &block_hash,
                    full_hash:  &full_hash,
                };
                let _ = client
                    .post(url)
                    .json(&body)
                    .send()
                    .await;
            }
        }
        drop(feedback_queue);
    }
}
