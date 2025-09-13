use crate::administrator::interpreter::InterpreterConstruct;
use crate::memory::manager_network::MANAGER_NETWORK;
use crate::utils::task_codes;
use chrono::Utc;
use common::logger::Logger;
use common::memory::blocks_queue::BLOCKS_QUEUE;
use tokio::time::{sleep, Duration};
use common::memory::feedback_queue::FEEDBACK_QUEUE;

pub struct Synchronizer {
    server_url: String,
    committee_size: u32,
    selection_size: u32,
    first_maintenance: bool,
}

impl Synchronizer {
    pub async fn new(server_url: String, committee_size: u32, selection_size: u32) -> Synchronizer {
        let mut manager_network = MANAGER_NETWORK.lock().await;
        manager_network.insert_node(server_url.clone(), 50, 0, 0);
        drop(manager_network);

        Synchronizer {
            server_url,
            committee_size,
            selection_size,
            first_maintenance: true,
        }
    }

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
            match epoch {
                0 => {
                    self.maintenance_round().await;
                }
                12 => {
                    if !self.first_maintenance {
                        self.committee_round().await;
                    }
                }
                24 => {
                    if !self.first_maintenance {
                        self.sync_round().await;
                    }
                }
                36 => {
                    if !self.first_maintenance {
                        self.committee_round().await;
                    }
                }
                48 => {
                    if !self.first_maintenance {
                        self.sync_round().await;
                    }
                }
                _ => {}
            }

            sleep(Duration::from_millis(1)).await;
        }
    }

    async fn maintenance_round(&mut self) {
        let mut manager_network = MANAGER_NETWORK.lock().await;
        manager_network.prepare_for_maintenance();
        let encoded_network = manager_network.serialize();
        drop(manager_network);

        let interpreter_construct = InterpreterConstruct {
            creator: self.server_url.clone(),
            from: self.server_url.clone(),
            task: task_codes::MAINTAIN,
            data: Some(("maintenance".to_string(), "job".to_string(), encoded_network)),
            created_at: Utc::now(),
        };

        if let Ok(()) = interpreter_construct.distribute(self.selection_size).await {
            if self.first_maintenance {
                self.first_maintenance = false;
            }
        }
    }

    async fn committee_round(&self) {
        let mut manager_network = MANAGER_NETWORK.lock().await;

        manager_network.choose_committee(self.committee_size);
        let committee = manager_network.get_committee();

        Logger::console("synchronizer", &format!("committee={:?}", committee));
        Logger::console(
            "synchronizer",
            &format!("network={:?}", manager_network.get_all_nodes()),
        );
        
        drop(manager_network);

        if committee.contains(&self.server_url) {
            let mut blocks_queue = BLOCKS_QUEUE.lock().await;
            let block_data = blocks_queue.get_block();
            drop(blocks_queue);
            let data = match block_data {
                Some((hash, organization, feedback, payload)) => {
                    let mut feedback_queue = FEEDBACK_QUEUE.lock().await;
                    feedback_queue.add_block(hash.clone(), feedback);
                    drop(feedback_queue);
                    Some((hash, organization, payload))
                },
                None => None,
            };

            let interpreter_construct = InterpreterConstruct {
                creator: self.server_url.clone(),
                from: self.server_url.clone(),
                task: task_codes::BLOCK,
                data,
                created_at: Utc::now(),
            };

            if let Err(e) = interpreter_construct.distribute(self.selection_size).await {
                println!("Failed to distribute interpreter construct: {:?}", e);
            }

            if let Err(e) = interpreter_construct.apply().await {
                println!("Failed to apply interpreter construct: {:?}", e);
            }
        }
    }

    async fn sync_round(&self) {
        let manager_network = MANAGER_NETWORK.lock().await;
        let missing_nodes = manager_network.missing_nodes();
        drop(manager_network);

        let encoded_missing_nodes = serde_json::to_string(&missing_nodes).unwrap();

        let interpreter_construct = InterpreterConstruct {
            creator: self.server_url.clone(),
            from: self.server_url.clone(),
            task: task_codes::SYNC,
            data: Some(("missing".to_string(), "job".to_string(), encoded_missing_nodes)),
            created_at: Utc::now(),
        };

        if let Err(e) = interpreter_construct.distribute(self.selection_size).await {
            println!("Failed to distribute SYNC interpreter construct: {:?}", e);
        }
    }
}
