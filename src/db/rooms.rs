use crate::models::member::Member;
use crate::models::room::Room;
use anyhow::Result;
use anyhow::anyhow;
use matrix_sdk::{Room as MatrixRoom, RoomMemberships};
use nanoid::nanoid;
use surrealdb::types::{Datetime, RecordId, Uuid};
use surrealdb::{Surreal, engine::remote::ws::Client};
use tracing::error;

impl Room {
    pub fn new(
        address: String,
        founder: RecordId,
        members: Vec<RecordId>,
        title: String,
        description: String,
    ) -> Self {
        Self {
            id: RecordId::new("rooms", nanoid!()),
            address,
            founder,
            members,
            invited_members: vec![],
            title,
            description,
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
        }
    }

    pub async fn fetch(db: &Surreal<Client>, id: RecordId) -> Result<()> {
        db.query("SELECT * FROM rooms WHERE id = $id")
            .bind(("id", id.clone()))
            .await?;

        Ok(())
    }

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<()> {
        db.query("CREATE rooms CONTENT $room")
            .bind(("room", self.clone()))
            .await?;

        Ok(())
    }

    pub async fn update(&self, db: &Surreal<Client>) -> Result<()> {
        db.query("UPDATE $id MERGE $room")
            .bind(("id", self.id.clone()))
            .bind(("room", self.clone()))
            .await?;

        Ok(())
    }

    pub async fn fetch_from_address(db: &Surreal<Client>, address: String) -> Result<Option<Self>> {
        let mut res = db
            .query("SELECT * FROM rooms WHERE address = $address LIMIT 1")
            .bind(("address", address))
            .await?;

        let mut rooms: Vec<Self> = res.take(0)?;
        Ok(rooms.pop())
    }

    pub async fn delete(&self, db: &Surreal<Client>) -> Result<()> {
        db.query("DELETE $id").bind(("id", self.id.clone())).await?;

        Ok(())
    }

    pub async fn insert_from_matrix(room: MatrixRoom, db: &Surreal<Client>) -> Result<Room> {
        let title = room.name().unwrap_or_else(|| "".to_string());
        let description = room.topic().unwrap_or_else(|| "".to_string());

        let members = room.members(RoomMemberships::empty()).await?;

        let mut member_records: Vec<RecordId> = Vec::new();

        for member in members {
            let address = member.user_id().to_string();

            match Member::fetch_id(db, &address).await {
                Ok(Some(id)) => {
                    member_records.push(id);
                }
                Ok(None) => {
                    let display_name = member.display_name().map(|name| name.to_string());
                    let member = Member::new(address, display_name);
                    member.insert(db).await?;
                    member_records.push(member.id);
                }
                Err(err) => {
                    error!("Failed to fetch member record: {}", err);

                    continue;
                }
            };
        }

        let founder_address = room
            .creators()
            .and_then(|creators| creators.into_iter().next())
            .unwrap();

        let founder = match room.get_member(&founder_address).await? {
            Some(member) => member,
            None => {
                error!("Failed to fetch founder member record: {}", founder_address);

                return Err(anyhow!("failed to fetch founder member record"));
            }
        };

        let founder_id = match Member::fetch_id(db, &founder_address.to_string()).await? {
            Some(id) => id,
            None => {
                let display_name = founder.display_name().map(|name| name.to_string());
                let member = Member::new(founder_address.to_string(), display_name);
                member.insert(db).await?;
                member.id
            }
        };

        let room_record = Self::new(
            room.room_id().to_string(),
            founder_id,
            member_records,
            title,
            description,
        );

        room_record.insert(db).await?;

        Ok(room_record)
    }

    pub async fn fetch_id(db: &Surreal<Client>, address: &String) -> Result<Option<RecordId>> {
        let mut res = db
            .query("SELECT id FROM rooms WHERE address = $address LIMIT 1")
            .bind(("address", address.clone()))
            .await?;

        let mut rooms_id: Vec<RecordId> = res.take(0)?;
        Ok(rooms_id.pop())
    }
}
