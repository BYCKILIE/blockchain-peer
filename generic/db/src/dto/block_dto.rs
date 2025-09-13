use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDTO {
    pub hash: String,
    pub previous_hash: String,
    pub organization: String,
    pub payload: String,
    pub created_at: DateTime<Utc>,
}


impl BlockDTO {
    pub fn from_row(row: Row) -> Self {
        BlockDTO {
            hash: row.get("hash"),
            previous_hash: row.get("previous_hash"),
            organization: row.get("organization"),
            payload: row.get("payload"),
            created_at: row.get("created_at"),
        }
    }
}