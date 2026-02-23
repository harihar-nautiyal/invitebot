use crate::models::member::Member;
use anyhow::Result;
use anyhow::anyhow;
use surrealdb::types::{Datetime, RecordId};
use surrealdb::{Surreal, engine::remote::ws::Client};

impl Member {
    pub fn new(address: String, display_name: Option<String>) -> Self {
        Self {
            id: None,
            address,
            display_name,
            created_at: Datetime::now(),
        }
    }

    pub fn id(&self) -> Result<RecordId> {
        self.id.clone().ok_or(anyhow!("Member ID not set"))
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

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<Self> {
        let mut res = db
            .query("CREATE members CONTENT $member")
            .bind(("member", self.clone()))
            .await?;

        let mut members: Vec<Self> = res.take(0)?;
        let member = members.pop();

        match member {
            Some(member) => Ok(member),
            None => Err(anyhow!("Failed to insert member: {}", self.address.clone())),
        }
    }

    pub async fn fetch_or_create(
        db: &Surreal<Client>,
        address: String,
        display_name: Option<String>,
    ) -> Result<Self> {
        let query = r#"
            let $existing = SELECT * FROM members WHERE address = $address LIMIT 1;
            IF $existing[0].id {
              RETURN $existing[0];
            } ELSE {
              CREATE members CONTENT {
                 address: $address,
                 display_name: $display_name,
                 created_at: time::now(),
                 updated_at: time::now()
              }
            }
            "#;

        let mut res = db
            .query(query)
            .bind(("address", address))
            .bind(("display_name", display_name))
            .await?;

        let mut members: Vec<Self> = res.take(1)?;
        let member = members
            .pop()
            .ok_or_else(|| anyhow!("Failed to upsert member"))?;

        Ok(member)
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
