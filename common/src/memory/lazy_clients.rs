use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LazyClients {
    memory: Vec<(String, bool)>,
}

impl LazyClients {
    pub fn new() -> LazyClients {
        LazyClients {
            memory: Vec::new(),
        }
    }

    pub fn add(&mut self, url: String) {
        self.memory.push((url, true));
    }

    pub fn get(&mut self) -> Vec<(String, bool)> {
        let mem_clone = self.memory.clone();
        self.memory.clear();
        mem_clone
    }
    
    pub fn remove(&mut self, url: String) {
        self.memory.push((url, false));
    }
}

pub static LAZY_CLIENTS: Lazy<Arc<Mutex<LazyClients>>> =
    Lazy::new(|| Arc::new(Mutex::new(LazyClients::new())));
