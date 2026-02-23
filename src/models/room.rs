use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Serialize, Deserialize, Clone, SurrealValue)]
pub struct Room {
    pub id: Option<RecordId>,
    pub title: String,
    pub description: String,
    pub address: String,
    pub founder: RecordId,
    pub members: Vec<RecordId>,
    pub invited_members: Vec<RecordId>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}
