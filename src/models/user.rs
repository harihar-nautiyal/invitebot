use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
pub struct User {
    pub id: Option<RecordId>,
    pub address: String,
    pub display_name: Option<String>,
    pub rooms: Vec<RecordId>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}
