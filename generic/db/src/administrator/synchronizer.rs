use chrono::Utc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use common::logger::Logger;
use common::memory::db_queue::DB_QUEUE;
use common::memory::lazy_clients::LAZY_CLIENTS;
use crate::service::alter_service::AlterService;

pub struct Synchronizer {
    pub(crate) alter_service: AlterService,
    pub(crate) feedback_sender: UnboundedSender<Vec<(String, String)>>,
}

impl Synchronizer {
    
    pub async fn power_synchronizer(&mut self) {
        let mut last_seconds: i64 = 0;
        loop {
            let now = Utc::now();
            let seconds = now.timestamp() % 12;
            let nanos = now.timestamp_subsec_nanos();

            let remaining_secs = (12 - seconds) % 12;
            let total_nanos = remaining_secs * 1_000_000_000 - nanos as i64;
            let wait_duration = Duration::from_nanos(total_nanos.max(0) as u64);

            let safe_sleep = wait_duration.saturating_sub(Duration::from_millis(200));
            sleep(safe_sleep).await;

            loop {
                let now = Utc::now();
                if now.timestamp() % 12 == 0 && last_seconds < now.timestamp() {
                    last_seconds = now.timestamp();
                    break;
                }
            }

            let epoch = now.timestamp() % 60;
            if epoch == 0 {
                let mut db_queue = DB_QUEUE.lock().await;
                let db_data = db_queue.get_and_clear();
                drop(db_queue);
                
                let print_data: Vec<String> = db_data.iter().map(|((hash, _, _), _)| {
                    hash.clone()
                }).collect();
                Logger::console("db", &format!("{:?}", print_data));
                
                let created_hashes = self.alter_service.create_blocks(db_data).await;
                
                let mut lazy_clients = LAZY_CLIENTS.lock().await;
                let network = lazy_clients.get();
                drop(lazy_clients);
                
                self.alter_service.update_connections(network).await;

                self.feedback_sender.send(created_hashes).unwrap();
            }

            sleep(Duration::from_millis(1)).await;
        }
    }
    
    
}