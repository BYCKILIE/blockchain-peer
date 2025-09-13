use crate::communication::sender::write_node;
use crate::memory::manager_client::MANAGER_CLIENT;
use crate::memory::manager_network::{ManagerNetwork, MANAGER_NETWORK};
use crate::utils::task_codes;
use chrono::{DateTime, Utc};
use common::memory::db_queue::DB_QUEUE;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use tokio::{io, task};
use common::memory::lazy_clients::LAZY_CLIENTS;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterpreterConstruct {
    pub creator: String,
    pub from: String,
    pub task: u16,
    pub data: Option<(String, String, String)>,
    pub created_at: DateTime<Utc>,
}

impl InterpreterConstruct {
    pub fn from_raw(data: Vec<u8>) -> serde_json::Result<InterpreterConstruct> {
        serde_json::from_slice(data.as_slice())
    }

    pub fn from_json(data: String) -> serde_json::Result<InterpreterConstruct> {
        serde_json::from_str(&data)
    }

    pub fn serialize_to_vec(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(&self)
    }

    pub async fn apply(&self) -> io::Result<()> {
        match self.task {
            task_codes::MAINTAIN => {
                let mut manager_network = MANAGER_NETWORK.lock().await;
                manager_network.mark_received(self.creator.clone());

                let received_network = self.data.clone();
                if received_network.is_none() {
                    drop(manager_network);
                    return Err(Error::new(ErrorKind::Other, "Couldn't get network"));
                }
                let (_, _, received_network) = received_network.unwrap();

                let received_network = ManagerNetwork::from_json(&received_network)?;

                manager_network.perform_maintenance(received_network);
                manager_network.mark_maintenance(self.creator.clone());
                drop(manager_network);

                Ok(())
            }

            task_codes::BLOCK => {
                let mut manager_network = MANAGER_NETWORK.lock().await;
                if manager_network.has_received(&self.creator) {
                    return Ok(());
                }
                manager_network.mark_received(self.creator.clone());
                drop(manager_network);

                let mut db_queue = DB_QUEUE.lock().await;
                db_queue.add_block(
                    self.creator.clone(),
                    self.data.clone(),
                    self.created_at.clone(),
                );
                drop(db_queue);

                Ok(())
            }

            task_codes::SYNC => {
                let data = self.data.clone();
                if data.is_none() {
                    return Err(Error::new(ErrorKind::Other, "Couldn't sync for no data"));
                }
                let (_, _, encoded_nodes) = data.unwrap();

                let missing_nodes: HashSet<String> = serde_json::from_str(&encoded_nodes)?;

                let mut handles = Vec::new();

                let db_queue = DB_QUEUE.lock().await;
                for node in missing_nodes {
                    let node_block = db_queue.get_last(&node);
                    if node_block.is_none() {
                        continue;
                    }
                    let (node_block, created_at) = node_block.unwrap();

                    let response_construct = InterpreterConstruct {
                        creator: node,
                        from: "Response".to_string(),
                        task: task_codes::RESP_SYNC,
                        data: node_block.clone(),
                        created_at: created_at.clone(),
                    };
                    let encoded_response = response_construct.serialize_to_vec()?;

                    let client_clone = self.from.clone();
                    let handle = task::spawn(async move {
                        let _ = write_node(&client_clone, encoded_response).await;
                    });
                    handles.push(handle);
                }
                drop(db_queue);

                let _ = join_all(handles).await;
                Ok(())
            }

            task_codes::RESP_SYNC => {
                let mut manager_network = MANAGER_NETWORK.lock().await;
                if manager_network.has_received(&self.creator) {
                    return Ok(());
                }
                manager_network.mark_received(self.creator.clone());
                drop(manager_network);

                let mut db_queue = DB_QUEUE.lock().await;
                db_queue.add_block(
                    self.creator.clone(),
                    self.data.clone(),
                    self.created_at.clone(),
                );
                drop(db_queue);
                Ok(())
            }

            _ => Err(Error::new(ErrorKind::Other, "Unknown task")),
        }
    }

    pub async fn distribute(&self, max_connections: u32) -> io::Result<()> {
        match self.task {
            task_codes::MAINTAIN => {
                let manager_network = MANAGER_NETWORK.lock().await;
                if manager_network.was_maintained_by(&self.creator) {
                    return Ok(());
                }
                drop(manager_network);

                let manager_client = MANAGER_CLIENT.lock().await;
                let all_clients = manager_client.get_to_send_clients(&self.from);
                drop(manager_client);
                
                if all_clients.is_empty() {
                    return Err(Error::new(ErrorKind::Other, "No connected clients"));
                }

                let encoded_msg = self.serialize_to_vec()?;

                self.action_senders(all_clients, encoded_msg).await;
                Ok(())
            }

            task_codes::BLOCK => {
                let manager_network = MANAGER_NETWORK.lock().await;
                if manager_network.has_received(&self.creator) {
                    return Ok(());
                }
                drop(manager_network);

                let manager_client = MANAGER_CLIENT.lock().await;
                let selection_size = max((2 * max_connections) / 3, 1);
                let selection = manager_client.get_random_selection(selection_size, &self.from);
                drop(manager_client);

                let encoded_msg = self.serialize_to_vec()?;

                self.action_senders(selection, encoded_msg).await;
                Ok(())
            }

            task_codes::SYNC => {
                if self.creator != self.from {
                    return Ok(());
                }

                let manager_client = MANAGER_CLIENT.lock().await;
                let all_clients = manager_client.get_to_send_clients(&self.from);
                drop(manager_client);

                let encoded_msg = self.serialize_to_vec()?;

                self.action_senders(all_clients, encoded_msg).await;
                Ok(())
            }

            task_codes::RESP_SYNC => Ok(()),

            _ => Err(Error::new(ErrorKind::Other, "Unknown task")),
        }
    }

    async fn action_senders(&self, selection: Vec<String>, msg: Vec<u8>) {
        let mut handles = Vec::new();
        for client in selection {
            if client == self.creator || client == self.from {
                continue;
            }

            let encoded_clone = msg.clone();
            let client_clone = client.clone();

            let handle = task::spawn(async move {
                if let Err(_) = write_node(&client_clone, encoded_clone).await {
                    let mut management_client = MANAGER_CLIENT.lock().await;
                    management_client.remove_client(&client_clone);
                    drop(management_client);

                    let mut lazy_clients = LAZY_CLIENTS.lock().await;
                    lazy_clients.remove(client_clone);
                    drop(lazy_clients);
                }
            });

            handles.push(handle);
        }
        let _ = join_all(handles).await;
    }
}
