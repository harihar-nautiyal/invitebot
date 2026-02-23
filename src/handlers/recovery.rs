use crate::models::room::Room as RoomRecord;
use matrix_sdk::Client;
use std::sync::Arc;
use surrealdb::{Surreal, engine::remote::ws::Client as SurrealClient};
use tokio::time::{Duration, sleep};
use tracing::{error, info};

pub async fn run_startup_sync(client: Arc<Client>, db: Arc<Surreal<SurrealClient>>) {
    info!("Waiting for initial Matrix sync to complete before DB recovery...");

    sleep(Duration::from_secs(15)).await;

    let joined_rooms = client.joined_rooms();
    info!(
        "Bot is currently in {} rooms. Verifying with Database...",
        joined_rooms.len()
    );

    for room in joined_rooms {
        let room_address = room.room_id().to_string();

        match RoomRecord::fetch_id(&db, &room_address).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                info!(
                    "Room {} is missing from DB. Reconstructing state...",
                    room_address
                );

                if let Err(e) = RoomRecord::insert_from_matrix(room.clone(), &db).await {
                    error!("Failed to recover room {}: {}", room_address, e);
                } else {
                    info!("Successfully recovered room {}", room_address);
                }
            }
            Err(e) => {
                error!("DB error checking room {}: {}", room_address, e);
            }
        }
    }

    info!("Startup database reconciliation complete!");
}
