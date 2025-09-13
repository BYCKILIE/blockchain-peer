use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockForm {
    pub hash: String,
    pub index: usize,
    pub total: usize,
    pub organization: String,
    pub feedback: String,
    pub data: String,
}

impl BlockForm {
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
