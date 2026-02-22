use crate::models::user::User;
use anyhow::Result;
use surrealdb::types::Uuid;
use surrealdb::types::{Datetime, RecordId};
use surrealdb::{Surreal, engine::remote::ws::Client};

impl User {
    pub async fn new(address: String, display_name: Option<String>, room: RecordId) -> Self {
        Self {
            id: RecordId::new("users", Uuid::new_v7()),
            address,
            display_name,
            rooms: vec![room],
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
        }
    }

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<()> {
        let _: Option<Self> = db.create("users").content::<Self>(self.to_owned()).await?;

        Ok(())
    }

    pub async fn fetch_from_address(db: &Surreal<Client>, address: String) -> Result<Option<Self>> {
        let mut res = db
            .query("SELECT * FROM users WHERE address = $address LIMIT 1")
            .bind(("address", address))
            .await?;

        let mut calls: Vec<Self> = res.take(0)?;
        Ok(calls.pop())
    }

    pub async fn fetch_from_id(db: &Surreal<Client>, id: RecordId) -> Result<Option<Self>> {
        let mut res = db
            .query("SELECT * FROM users WHERE id = $id LIMIT 1")
            .bind(("id", id))
            .await?;

        let mut calls: Vec<Self> = res.take(0)?;
        Ok(calls.pop())
    }
}
