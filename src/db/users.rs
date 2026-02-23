use crate::models::user::User;
use anyhow::Result;
use anyhow::anyhow;
use surrealdb::types::{Datetime, RecordId};
use surrealdb::{Surreal, engine::remote::ws::Client};

impl User {
    pub fn new(address: String, display_name: Option<String>, room: RecordId) -> Self {
        Self {
            id: None,
            address,
            display_name,
            rooms: vec![room],
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
        }
    }

    pub fn id(&self) -> Result<RecordId> {
        self.id.clone().ok_or(anyhow!("Member ID not set"))
    }

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<Self> {
        let mut res = db
            .query("CREATE users CONTENT $user")
            .bind(("user", self.clone()))
            .await?;

        let mut users: Vec<Self> = res.take(0)?;
        let user = users.pop();

        match user {
            Some(user) => Ok(user),
            None => Err(anyhow!("Failed to insert user: {}", self.address)),
        }
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

        let mut users: Vec<Self> = res.take(0)?;
        Ok(users.pop())
    }

    pub async fn fetch_or_create(
        db: &Surreal<Client>,
        address: String,
        display_name: Option<String>,
        room_id: RecordId,
    ) -> Result<User> {
        let query = r#"
            let $existing = SELECT * FROM users WHERE address = $address LIMIT 1;
            IF $existing[0].id {
                RETURN $existing[0];
            } ELSE {
                CREATE users CONTENT {
                    address: $address,
                    display_name: $display_name,
                    rooms: [$room_id],
                    created_at: time::now(),
                    updated_at: time::now()
                };
            };
        "#;

        let mut res = db
            .query(query)
            .bind(("address", address))
            .bind(("display_name", display_name))
            .bind(("room_id", room_id))
            .await?;

        let mut users: Vec<Self> = res.take(1)?;
        let user = users
            .pop()
            .ok_or_else(|| anyhow!("Failed to upsert user"))?;

        Ok(user)
    }
}
