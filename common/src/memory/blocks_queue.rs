use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

pub struct BlocksQueue {
    memory: VecDeque<(String, String, String, String)>,
}

impl BlocksQueue {
    fn new() -> Self {
        BlocksQueue {
            memory: VecDeque::new(),
        }
    }

    pub fn add_block(&mut self, hash: String, organization: String, feedback: String, data: String) {
        self.memory.push_back((hash, organization, feedback, data));
    }

    pub fn get_block(&mut self) -> Option<(String, String, String, String)> {
        self.memory.pop_front()
    }
}

pub static BLOCKS_QUEUE: Lazy<Arc<Mutex<BlocksQueue>>> =
    Lazy::new(|| Arc::new(Mutex::new(BlocksQueue::new())));
