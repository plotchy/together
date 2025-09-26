use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Attestation {
    pub id: Uuid,
    pub cast_hash: String,
    pub creator_fid: Option<i64>,
    pub settled: bool,
    pub winner_address: Option<String>,
    pub updated_at: DateTime<Utc>,
}
