use matrix_sdk::{Client, Room, ruma::events::room::member::StrippedRoomMemberEvent};
use surrealdb::{Surreal, engine::remote::ws::Client};
use tokio::time::{Duration, sleep};
use tracing::{error, info};

pub async fn listen(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
    db: &Surreal<Client>,
) {
    info!(
        "Invite received for room {} from {}",
        room.room_id(),
        room_member.sender
    );

    if room_member.state_key != client.user_id().unwrap() {
        return;
    }

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

        info!("Successfully joined room {}", room.room_id());
    });
}
