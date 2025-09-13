use std::cmp::max;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use base64::{engine::general_purpose, Engine};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    stake: i32,
    reward: i32,
    penalty: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManagerNetwork {
    nodes: BTreeMap<String, NodeInfo>,

    #[serde(skip)]
    maintain: HashSet<String>,

    #[serde(skip)]
    committee: BTreeSet<String>,

    #[serde(skip)]
    received: BTreeSet<String>,

    #[serde(serialize_with = "serialize_seed", deserialize_with = "deserialize_seed")]
    seed: [u8; 32],
}

fn serialize_seed<S>(seed: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = general_purpose::STANDARD.encode(seed);
    serializer.serialize_str(&encoded)
}

fn deserialize_seed<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = general_purpose::STANDARD
        .decode(&s)
        .map_err(serde::de::Error::custom)?;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| serde::de::Error::custom("Invalid length for seed"))?;
    Ok(array)
}

impl ManagerNetwork {
    fn new() -> ManagerNetwork {
        ManagerNetwork {
            nodes: BTreeMap::new(),
            maintain: HashSet::new(),
            committee: BTreeSet::new(),
            received: BTreeSet::new(),
            seed: *blake3::hash("DefaultSeed".as_bytes()).as_bytes(),
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<ManagerNetwork> {
        serde_json::from_str::<ManagerNetwork>(json)
    }


    pub fn insert_node(&mut self, url: String, stake: i32, reward: i32, penalty: i32) {
        self.nodes.insert(url.clone(), NodeInfo {
            stake,
            reward,
            penalty,
        });
    }

    pub fn perform_maintenance(&mut self, received_network: ManagerNetwork) {
        for (id, received_node) in received_network.nodes {
            match self.nodes.get_mut(&id) {
                Some(node_info) => {
                    node_info.stake = max(node_info.stake, received_node.stake);
                    node_info.reward = max(node_info.reward, received_node.reward);
                    node_info.penalty = max(node_info.penalty, received_node.penalty);
                }
                None => {
                    self.nodes.insert(id, received_node);
                }
            }
        }

        self.seed = max(self.seed, received_network.seed);
    }

    pub fn choose_committee(&mut self, size: u32) {
        self.committee.clear();

        let total_stake: i32 = self.nodes.iter().map(|(_, s)| s.stake).sum();

        let mut candidates: Vec<(String, f64)> = self.nodes.iter().map(|(id, peer)| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&self.seed);
            hasher.update(id.as_bytes());
            let hash = hasher.finalize();

            let hash_value = u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap());
            let normalized = (hash_value as f64) / (u64::MAX as f64);

            let threshold = (peer.stake as f64) / (total_stake as f64);
            let score = normalized / threshold;

            (id.clone(), score)
        }).collect();

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (id, _) in candidates.into_iter().take(size as usize) {
            self.committee.insert(id);
        }

        self.refresh_stakes();
        self.update_seed();
        self.received.clear();
    }
    
    fn refresh_stakes(&mut self) {
        let committee_size      = self.committee.len().max(1);
        let total_reward_pool   = 30;
        let total_penalty_pool  = 50;

        let base_reward  = total_reward_pool  / committee_size;
        let base_penalty = total_penalty_pool / committee_size;

        for (url, peer) in self.nodes.iter_mut() {
            let (new_reward, new_penalty) = if self.committee.contains(url) {
                (0, peer.penalty + base_penalty as i32)
            } else {
                (peer.reward + base_reward as i32, 0)
            };

            let mut new_stake = peer.stake + new_reward - new_penalty;
            new_stake = new_stake.clamp(1, 100);

            peer.reward  = new_reward;
            peer.penalty = new_penalty;
            peer.stake   = new_stake;
        }
    }

    fn update_seed(&mut self) {
        let mut hasher = blake3::Hasher::new();

        for url in &self.committee {
            hasher.update(url.clone().as_bytes());
        }

        self.seed = *hasher.finalize().as_bytes();
    }

    pub fn prepare_for_maintenance(&mut self) {
        self.maintain.clear();
    }

    pub fn mark_maintenance(&mut self, from: String) {
        self.maintain.insert(from);
    }

    pub fn was_maintained_by(&self, from: &str) -> bool {
        self.maintain.contains(from)
    }

    pub fn mark_received(&mut self, node_url: String) {
        self.received.insert(node_url);
    }
    pub fn has_received(&self, node_url: &str) -> bool {
        self.received.contains(node_url)
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn exists(&self, url: &str) -> bool {
        self.nodes.contains_key(url)
    }

    pub fn missing_nodes(&self) -> BTreeSet<String> {
        &self.committee - &self.received
    }

    pub fn get_committee(&self) -> BTreeSet<String> {
        self.committee.clone()
    }

    pub fn get_all_nodes(&self) -> Vec<(String, i32)> {
        self.nodes.iter().map(|(url, peer)| {(url.clone(), peer.stake.clone())}).collect()
    }

    pub fn remove_node(&mut self, url: &str) {
        self.nodes.remove(url);
    }
}

pub static MANAGER_NETWORK: Lazy<Arc<Mutex<ManagerNetwork>>> =
    Lazy::new(|| Arc::new(Mutex::new(ManagerNetwork::new())));
