use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DbQueue {
    memory: HashMap<String, Vec<(Option<(String, String, String)>, DateTime<Utc>)>>,
}

impl DbQueue {
    fn new() -> Self {
        DbQueue {
            memory: HashMap::new(),
        }
    }

    pub fn add_block(
        &mut self,
        creator: String,
        data: Option<(String, String, String)>,
        created_at: DateTime<Utc>,
    ) {
        if self.memory.contains_key(&creator) {
            self.memory
                .get_mut(&creator)
                .unwrap()
                .push((data, created_at));
        } else {
            let mut stack: Vec<(Option<(String, String, String)>, DateTime<Utc>)> = Vec::new();
            stack.push((data, created_at));
            self.memory.insert(creator, stack);
        }
    }

    pub fn get_last(&self, creator: &str) -> Option<&(Option<(String, String, String)>, DateTime<Utc>)> {
        if let Some(data) = self.memory.get(creator) {
            return data.last();
        }
        None
    }

    pub fn get_and_clear(&mut self) -> Vec<((String, String, String), DateTime<Utc>)> {
        let memory_clone = self.memory.clone();
        self.memory.clear();

        let mut ordered_blocks: Vec<((String, String, String), DateTime<Utc>)> = memory_clone
            .values()
            .flat_map(|vec| vec.iter())
            .filter_map(|(data_opt, created_at)| {
                data_opt
                    .as_ref()
                    .map(|data| (data.clone(), *created_at))
            })
            .collect();
        
        ordered_blocks.sort_by(|((hash_a, _, _), created_at_a), ((hash_b, _, _), created_at_b)| {
            created_at_a
                .cmp(created_at_b)
                .then_with(|| hash_a.cmp(hash_b))
        });
        ordered_blocks
    }
}

pub static DB_QUEUE: Lazy<Arc<Mutex<DbQueue>>> = Lazy::new(|| Arc::new(Mutex::new(DbQueue::new())));