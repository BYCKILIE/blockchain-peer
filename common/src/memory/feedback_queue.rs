use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct FeedbackQueue {
    memory: HashMap<String, String>,
}

impl FeedbackQueue {
    fn new() -> Self {
        FeedbackQueue {
            memory: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, hash: String, feedback: String) {
        self.memory.insert(hash, feedback);
    }

    pub fn get_block(&mut self, hash: &str) -> Option<String> {
        self.memory.remove(hash)
    }
}

pub static FEEDBACK_QUEUE: Lazy<Arc<Mutex<FeedbackQueue>>> =
    Lazy::new(|| Arc::new(Mutex::new(FeedbackQueue::new())));
