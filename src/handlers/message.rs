use crate::models::call::{Call, Status};
use matrix_sdk::{
    Room, RoomState,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
};
use surrealdb::{Surreal, engine::remote::ws::Client};
use tracing::error;

pub async fn listen(event: OriginalSyncRoomMessageEvent, room: Room, db: &Surreal<Client>) {
    if room.state() != RoomState::Joined {
        return;
    }
    let MessageType::Text(text_content) = event.content.msgtype else {
        return;
    };

    let event_id = event.event_id.to_string();
    let author_id = event.sender.to_string();
    let message = text_content.body;

    if !message.starts_with("!") {
        return;
    }

    let call = match Call::fetch_event(db, event_id).await {
        Ok(call) => call,
        Err(err) => {
            error!("Failed to fetch call: {}", err);

            return;
        }
    };

    match call {
        Some(call) => match call.status {
            Status::Pending => {}
            _ => {
                return;
            }
        },
        None => {
            let new_call = Call::new(event_id)
        }
    }

    let args: Vec<&str> = message.split_whitespace().collect();

    match args[0] {
        "!invite" => {
            let content = RoomMessageEventContent::text_plain("Executing invite command");

            room.send(content).await.unwrap();
        }
        "!progress" => {
            let content = RoomMessageEventContent::text_plain("Executing progress command");

            room.send(content).await.unwrap();
        }
        "!help" => {
            let content =
                RoomMessageEventContent::text_plain("Available commands: !invite, !progress");

            room.send(content).await.unwrap();
        }
        _ => {}
    }
}
