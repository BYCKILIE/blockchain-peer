use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Instant;

pub struct ManagerServer {
    peers: HashMap<String, (TcpStream, Instant)>,
}

impl ManagerServer {
    fn new() -> Self {
        ManagerServer {
            peers: HashMap::new(),
        }
    }

    pub fn add_server(&mut self, id: String, stream: TcpStream) {
        self.peers.insert(id.clone(), (stream, Instant::now()));
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(60);

        self.peers
            .retain(|_, (_, created_at)| now.duration_since(*created_at) <= timeout);
    }

    pub fn contains(&self, id: &str) -> bool {
        self.peers.contains_key(id)
    }

    pub fn remove_server(&mut self, id: &str) -> Option<TcpStream> {
        let now = Instant::now();
        let timeout = Duration::from_secs(60);
        let peer = self.peers.remove(id);
        if peer.is_none() {
            return None;
        }
        let (stream, created_at) = peer.unwrap();
        if now.duration_since(created_at) > timeout {
            return None;
        }
        Some(stream)
    }
}

pub static MANAGER_SERVER: Lazy<Arc<Mutex<ManagerServer>>> =
    Lazy::new(|| Arc::new(Mutex::new(ManagerServer::new())));
