use crate::models::room::Room as RoomRecord;
use anyhow::Result;
use matrix_sdk::{Client, Room, ruma::events::room::member::StrippedRoomMemberEvent};
use std::sync::Arc;
use surrealdb::{Surreal, engine::remote::ws::Client as SurrealClient};
use surrealdb_types::ToSql;
use tokio::time::{Duration, sleep};
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

    tokio::spawn(async move {
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

        let room_id = match RoomRecord::fetch_id(&db, &room_address).await {
            Ok(Some(id)) => id,
            Ok(None) => match RoomRecord::insert_from_matrix(room, &db).await {
                Ok(room) => room.id,
                Err(err) => {
                    error!(
                        "Failed to insert room record for room {} ({err:?})",
                        room_address
                    );
                    return;
                }
            },
            Err(err) => {
                error!(
                    "Failed to fetch room record for room {} ({err:?})",
                    room.room_id()
                );

                return;
            }
        };

        info!(
            "Successfully joined room {}, id: {}",
            room_address,
            room_id.to_sql()
        );
    });

    Ok(())
}
