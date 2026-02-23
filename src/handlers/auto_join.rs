use crate::models::room::Room as RoomRecord;
use anyhow::Result;
use matrix_sdk::{Client, Room, ruma::events::room::member::StrippedRoomMemberEvent};
use std::sync::Arc;
use surrealdb::{Surreal, engine::remote::ws::Client as SurrealClient};
use surrealdb_types::ToSql;
use tokio::time::{Duration, sleep};
use tracing::Instrument;
use tracing::{error, info};

pub async fn listen(
    room_member: StrippedRoomMemberEvent,
    client: Arc<Client>,
    room: Room,
    db: Arc<Surreal<SurrealClient>>,
) -> Result<()> {
    info!(
        "Invite received for room {} from {}",
        room.room_id(),
        room_member.sender
    );

    if room_member.state_key != client.user_id().unwrap() {
        return Ok(());
    }

    let room = room.clone();

    tokio::spawn(
        async move {
            if let Err(err) = handle_auto_join(db, &room).await {
                error!("Auto-join handler failed: {err:?}");
            }
        }
        .in_current_span(),
    );

    todo!("Implement auto join");
}

pub async fn handle_auto_join(db: Arc<Surreal<SurrealClient>>, room: &Room) -> Result<()> {
    info!("Autojoining room {}", room.room_id());
    let mut delay = 2;

    while let Err(err) = room.join().await {
        error!(
            "Failed to join room {} ({err:?}), retrying in {delay}s",
            room.room_id()
        );

        sleep(Duration::from_secs(delay)).await;
        delay *= 2;

        if delay > 3600 {
            error!("Can't join room {} ({err:?})", room.room_id());
            break;
        }
    }

    let room_address = room.room_id().to_string();

    let room_id = match RoomRecord::fetch_id(&db, &room_address).await? {
        Some(id) => id,
        None => RoomRecord::insert_from_matrix(room.clone(), &db)
            .await?
            .id()?,
    };

    info!(
        "Successfully joined room {}, id: {}",
        room_address,
        room_id.to_sql()
    );

    return Ok(());
}
