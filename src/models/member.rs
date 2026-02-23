use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId, SurrealValue};

#[derive(Serialize, Deserialize, SurrealValue, Clone)]
pub struct Member {
    pub id: Option<RecordId>,
    pub address: String,
    pub display_name: Option<String>,
    pub created_at: Datetime,
}
