use crate::models::member::Member;
use crate::models::room::Room;
use anyhow::Result;
use anyhow::anyhow;
use matrix_sdk::{Room as MatrixRoom, RoomMemberships};
use surrealdb::types::{Datetime, RecordId};
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
            id: None,
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

    pub fn id(&self) -> Result<RecordId> {
        self.id.clone().ok_or(anyhow!("Member ID not set"))
    }

    pub async fn fetch(db: &Surreal<Client>, id: RecordId) -> Result<()> {
        db.query("SELECT * FROM rooms WHERE id = $id")
            .bind(("id", id.clone()))
            .await?;

        Ok(())
    }

    pub async fn insert(&self, db: &Surreal<Client>) -> Result<Self> {
        let mut res = db
            .query("CREATE rooms CONTENT $room")
            .bind(("room", self.clone()))
            .await?;

        let mut rooms: Vec<Self> = res.take(0)?;
        let room = rooms.pop();

        match room {
            Some(room) => Ok(room),
            None => Err(anyhow!("Failed to insert room: {}", self.address.clone())),
        }
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

        let members = room.members(RoomMemberships::JOIN).await?;
        let mut member_records: Vec<RecordId> = Vec::new();

        for member in members {
            let display_name = member.display_name().map(|name| name.to_string());
            let address = member.user_id().to_string();

            match Member::fetch_or_create(db, address, display_name).await {
                Ok(member_rec) => {
                    member_records.push(member_rec.id.unwrap());
                }
                Err(err) => {
                    error!("Failed to fetch or create the member record: {}", err);
                    continue;
                }
            };
        }

        let invited = room.members(RoomMemberships::INVITE).await?;
        let mut invited_member_records: Vec<RecordId> = Vec::new();

        for member in invited {
            let display_name = member.display_name().map(|name| name.to_string());
            let address = member.user_id().to_string();

            match Member::fetch_or_create(db, address, display_name).await {
                Ok(member_rec) => {
                    invited_member_records.push(member_rec.id.unwrap());
                }
                Err(err) => {
                    error!("Failed to fetch or create invited member: {}", err);
                    continue;
                }
            };
        }

        let founder_address = room
            .creators()
            .and_then(|creators| creators.into_iter().next())
            .ok_or_else(|| anyhow!("No creator found for room"))?;

        let founder_id = match Member::fetch_id(db, &founder_address.to_string()).await? {
            Some(id) => id,
            None => {
                let member_info = room.get_member(&founder_address).await?;
                let display_name = member_info
                    .as_ref()
                    .and_then(|m| m.display_name().map(|n| n.to_string()));
                let member = Member::new(founder_address.to_string(), display_name);
                member.insert(db).await?.id()?
            }
        };

        let mut room_record_struct = Self::new(
            room.room_id().to_string(),
            founder_id,
            member_records,
            title,
            description,
        );
        room_record_struct.invited_members = invited_member_records;

        let query = r#"
            let $existing = SELECT * FROM rooms WHERE address = $address LIMIT 1;
            IF $existing[0].id {
                RETURN $existing[0];
            } ELSE {
                CREATE rooms CONTENT $content;
            };
        "#;

        let mut res = db
            .query(query)
            .bind(("address", room.room_id().to_string()))
            .bind(("content", room_record_struct))
            .await?;

        let room_record: Option<Room> = res.take(1)?;

        match room_record {
            Some(r) => Ok(r),
            None => Err(anyhow!("Failed to insert the room: {}", room.room_id())),
        }
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
