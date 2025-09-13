use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use rand::seq::IteratorRandom;
use rand::rng;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct ManagerClient {
    peers: HashMap<String, Arc<Mutex<TcpStream>>>,
    keys: HashMap<String, ([u8; 32], [u8; 16])>,
}

impl ManagerClient {
    fn new() -> Self {
        ManagerClient {
            peers: HashMap::new(),
            keys: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, id: String, stream: TcpStream) {
        let stream = Arc::new(Mutex::new(stream));
        self.peers.insert(id.clone(), stream);
    }

    pub fn add_key(&mut self, id: String, key: [u8; 32], iv: [u8; 16]) {
        self.keys.insert(id, (key, iv));
    }

    pub fn contains(&self, id: &str) -> bool {
        self.peers.contains_key(id)
    }

    pub fn use_client(&self, id: &str) -> Option<(&Arc<Mutex<TcpStream>>, (&[u8; 32], &[u8; 16]))> {
        if let Some(stream) = self.peers.get(id) {
            if let Some((key, iv)) = self.keys.get(id) {
                return Some((stream, (key, iv)))
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn remove_client(&mut self, id: &str) {
        self.peers.remove(id);
        self.keys.remove(id);
    }

    pub fn get_connected_clients(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }
    
    pub fn get_to_send_clients(&self, from: &str) -> Vec<String> {
        self.peers.keys().cloned().filter(|k| *k != from).collect()
    }

    pub fn get_random_selection(&self, size: u32, from: &str) -> Vec<String> {
        let mut rng = rng();
        self.peers
            .keys()
            .filter(|k| *k != from)
            .cloned()
            .choose_multiple(&mut rng, size as usize)
    }
}

pub static MANAGER_CLIENT: Lazy<Arc<Mutex<ManagerClient>>> =
    Lazy::new(|| Arc::new(Mutex::new(ManagerClient::new())));
