use serde::{Deserialize, Serialize};
use surrealdb::Error;
use surrealdb::types::{Datetime, RecordId, SurrealValue};
use surrealdb_types::{Kind, Value as SValue};

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

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Pending,
    Completed,
    Rejected,
}

impl SurrealValue for Status {
    fn into_value(self) -> SValue {
        match self {
            Status::Pending => SValue::String("pending".into()),
            Status::Completed => SValue::String("completed".into()),
            Status::Rejected => SValue::String("rejected".into()),
        }
    }

    fn kind_of() -> Kind {
        Kind::String
    }

    fn from_value(v: SValue) -> Result<Self, Error> {
        match v {
            SValue::String(s) => match s.as_str() {
                "pending" => Ok(Status::Pending),
                "completed" => Ok(Status::Completed),
                "rejected" => Ok(Status::Rejected),
                _ => Err(Error::serialization(
                    "Failed to serialize".to_string(),
                    None,
                )),
            },
            _ => Err(Error::serialization(
                "Failed to serialize".to_string(),
                None,
            )),
        }
    }
}
