use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;
use surrealdb::types::{Datetime, RecordId};

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
pub struct Call {
    pub id: RecordId,
    pub user: RecordId,
    pub room: RecordId,
    pub event_id: String,
    pub command: String,
    pub status: Status,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Serialize, Deserialize, Default, SurrealValue, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Pending,
    Completed,
    Rejected,
}
