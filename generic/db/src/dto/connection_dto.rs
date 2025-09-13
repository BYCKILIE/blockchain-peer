use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionDTO {
    pub url: String,
    pub is_active: bool,
}


impl ConnectionDTO {
    pub fn from_row(row: Row) -> Self {
        ConnectionDTO {
            url: row.get("url"),
            is_active: true,
        }
    }
}