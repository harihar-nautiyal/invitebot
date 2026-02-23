use crate::models::call::{Call, Status};
use anyhow::Result;
use anyhow::anyhow;
use surrealdb::types::{Datetime, RecordId};
use surrealdb::{Surreal, engine::remote::ws::Client};

impl Call {
    pub fn new(event_id: String, user: RecordId, room: RecordId, command: String) -> Self {
        Self {
            id: None,
            user,
            room,
            event_id,
            command,
            status: Status::default(),
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
        }
    }

    pub fn id(&self) -> Result<RecordId> {
        self.id.clone().ok_or(anyhow!("Call ID not set"))
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
            .bind(("id", self.id()?))
            .await?;

        let mut calls: Vec<Self> = res.take(0)?;
        let call = calls.pop();

        if call.is_some() {
            self.status = Status::Completed;
            self.updated_at = call.unwrap().updated_at;
        }

        Ok(())
    }

    pub async fn fetch_other_pending(
        db: &Surreal<Client>,
        room_address: String,
        event_id: String,
    ) -> Result<Vec<String>> {
        let mut res = db
            .query(
                r#"
            SELECT command FROM calls
            WHERE room = (SELECT VALUE id FROM rooms WHERE address = $room LIMIT 1)[0]
              AND event_id != $event_id
              AND status = 'pending'
            "#,
            )
            .bind(("room", room_address))
            .bind(("event_id", event_id))
            .await?;

        let calls: Vec<String> = res.take(0)?;

        Ok(calls)
    }

    pub async fn fetch_or_create(
        db: &Surreal<Client>,
        event_id: String,
        user: RecordId,
        room: RecordId,
        command: String,
    ) -> Result<Self> {
        let query = r#"
            let $existing = SELECT * FROM calls WHERE event_id = $event_id LIMIT 1;
            IF $existing[0].id {
                RETURN $existing[0];
            } ELSE {
              CREATE calls CONTENT {
                event_id: $event_id,
                user: $user,
                room: $room,
                command: $command,
                status: 'pending',
                created_at: time::now(),
                updated_at: time::now()
              }
            }
        "#;

        let mut res = db
            .query(query)
            .bind(("event_id", event_id))
            .bind(("user", user))
            .bind(("room", room))
            .bind(("command", command))
            .await?;

        let mut calls: Vec<Self> = res.take(1)?;

        let call = calls
            .pop()
            .ok_or_else(|| anyhow!("Failed to upsert user"))?;

        Ok(call)
    }
}
