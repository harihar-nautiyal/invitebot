use crate::models::member::Member;
use anyhow::Result;
use nanoid::nanoid;
use surrealdb::types::{Datetime, RecordId, Uuid};
use surrealdb::{Surreal, engine::remote::ws::Client};
impl Member {
    pub fn new(address: String, display_name: Option<String>) -> Self {
        Self {
            id: RecordId::new("members", nanoid!()),
            address,
            display_name,
            created_at: Datetime::now(),
        }
    }

    pub async fn fetch_id(db: &Surreal<Client>, address: &String) -> Result<Option<RecordId>> {
        let mut res = db
            .query("SELECT id FROM members WHERE address = $address LIMIT 1")
            .bind(("address", address.clone()))
            .await?;

        let mut members_ids: Vec<RecordId> = res.take(0)?;
        Ok(members_ids.pop())
    }

    pub async fn fetch(db: &Surreal<Client>, id: RecordId) -> Result<()> {
        db.query("SELECT * FROM $id").bind(("id", id)).await?;
        Ok(())
    }

    pub async fn fetch_from_address(db: &Surreal<Client>, address: String) -> Result<Option<Self>> {
        let mut res = db
            .query("SELECT * FROM members WHERE address = $address LIMIT 1")
            .bind(("address", address))
            .await?;

        let mut members: Vec<Self> = res.take(0)?;
        Ok(members.pop())
    }

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<()> {
        db.query("CREATE members CONTENT $member")
            .bind(("member", self.clone()))
            .await?;

        Ok(())
    }

    pub async fn delete(&self, db: &Surreal<Client>) -> Result<()> {
        db.query("DELETE $id").bind(("id", self.id.clone())).await?;

        Ok(())
    }

    pub async fn update(&self, db: &Surreal<Client>) -> Result<()> {
        db.query("UPDATE $id MERGE $member")
            .bind(("id", self.id.clone()))
            .bind(("member", self.clone()))
            .await?;

        Ok(())
    }
}
