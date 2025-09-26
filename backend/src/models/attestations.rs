use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TogetherAttestation {
    pub id: Uuid,
    pub address_1: String,
    pub address_2: String,
    pub attestation_timestamp: i64,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TogetherCount {
    pub id: Uuid,
    pub address: String,
    pub total_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WatcherState {
    pub id: String,
    pub last_processed_block: i64,
    pub chunk_size: i64,
    pub updated_at: DateTime<Utc>,
}

// DTOs for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub address: String,
    pub total_connections: i64,
    pub recent_connections: Vec<ConnectionInfo>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConnectionInfo {
    pub partner_address: String,
    pub attestation_timestamp: i64,
    pub tx_hash: Option<String>,
}
