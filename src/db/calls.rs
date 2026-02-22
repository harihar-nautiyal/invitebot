use crate::models::call::{Call, Status};
use anyhow::Result;
use surrealdb::types::Uuid;
use surrealdb::types::{Datetime, RecordId};
use surrealdb::{Surreal, engine::remote::ws::Client};

impl Call {
    pub fn new(event_id: String, user: RecordId, room: RecordId, command: String) -> Self {
        Self {
            id: RecordId::new("calls", Uuid::new_v7()),
            user,
            room,
            event_id,
            command,
            status: Status::default(),
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
        }
    }

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<()> {
        let _: Option<Self> = db.create("calls").content::<Self>(self.to_owned()).await?;

        Ok(())
    }

    pub async fn fetch_event(db: &Surreal<Client>, event_id: &String) -> Result<Option<Self>> {
        let mut res = db
            .query("SELECT * FROM calls WHERE event_id = $event_id LIMIT 1")
            .bind(("event_id", event_id.clone()))
            .await?;

        let mut calls: Vec<Self> = res.take(0)?;
        Ok(calls.pop())
    }

    pub async fn completed(&mut self, db: &Surreal<Client>) -> Result<()> {
        let mut res = db
            .query("UPDATE $id SET status = 'completed'")
            .bind(("id", self.id.clone()))
            .await?;

        let mut calls: Vec<Self> = res.take(0)?;
        let call = calls.pop();

        if call.is_some() {
            self.status = Status::Completed;
            self.updated_at = call.unwrap().updated_at;
        }

        Ok(())
    }
}
