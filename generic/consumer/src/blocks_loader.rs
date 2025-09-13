use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use crate::block_form::BlockForm;

pub struct BocksLoader {
    memory: HashMap<String, BTreeMap<usize, String>>,
    timeout: HashMap<String, Instant>
}

impl BocksLoader {
    fn new() -> Self {
        BocksLoader {
            memory: HashMap::new(),
            timeout: HashMap::new(),
        }
    }

    pub fn add_chunk(&mut self, block_chunk: BlockForm) -> io::Result<String> {
        let entry = self.memory.entry(block_chunk.hash.clone()).or_insert_with(BTreeMap::new);
        entry.insert(block_chunk.index, block_chunk.data);
        
        self.timeout.insert(block_chunk.hash.clone(), Instant::now());
        if block_chunk.total == entry.keys().len() {
            return Ok(block_chunk.hash);
        }

        Err(io::Error::new(ErrorKind::Other, "Block is not complete yet"))
    }

    pub fn get_block(&mut self, block_hash: &str) -> Option<String> {
        self.timeout.remove(block_hash);
        if let Some(chunks) = self.memory.remove(block_hash) {
            Some(chunks.into_values().collect::<String>())
        } else {
            None
        }
    }

    pub fn check_timeouts(&mut self) {
        let now = Instant::now();
        let timeout_keys: Vec<String> = self
            .timeout
            .iter()
            .filter(|(_, &last_seen)| now.duration_since(last_seen) > Duration::from_secs(60))
            .map(|(key, _)| key.clone())
            .collect();

        for key in timeout_keys {
            println!("Removing expired block: {}", key);
            self.memory.remove(&key);
            self.timeout.remove(&key);
        }
    }
}

pub static BLOCKS_LOADER: Lazy<Arc<Mutex<BocksLoader>>> =
    Lazy::new(|| Arc::new(Mutex::new(BocksLoader::new())));
