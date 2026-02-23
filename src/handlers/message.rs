use crate::models::room::Room as RoomRecord;
use crate::models::{
    call::{Call, Status},
    user::User,
};
use anyhow::Result;
use anyhow::anyhow;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{
    Room, RoomState,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
};
use std::sync::Arc;
use surrealdb::{Surreal, engine::remote::ws::Client};
use surrealdb_types::ToSql;
use tracing::Instrument;
use tracing::info;
use tracing::{error, warn};

pub async fn listen(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    db: &Arc<Surreal<Client>>,
) -> Result<()> {
    if room.state() != RoomState::Joined {
        return Ok(());
    }
    let MessageType::Text(text_content) = event.content.msgtype else {
        return Ok(());
    };

    let event_id = event.event_id.to_string();
    let author_id = event.sender;
    let message = text_content.body;

    if !message.starts_with("!") {
        return Ok(());
    }

    let db = db.clone();
    let room = room.clone();

    tokio::spawn(
        async move {
            if let Err(err) = handle_command(event_id, author_id, message, room, db).await {
                error!("Command handler failed: {err:?}");
            }
        }
        .in_current_span(),
    );

    Ok(())
}

pub async fn handle_command(
    event_id: String,
    author_id: OwnedUserId,
    message: String,
    room: Room,
    db: Arc<Surreal<Client>>,
) -> Result<()> {
    let room_address = room.room_id().to_string();

    let room_record = match RoomRecord::fetch_from_address(&db, room_address.clone()).await? {
        Some(r) => r,
        None => {
            info!("No room found");
            let record = RoomRecord::insert_from_matrix(room.clone(), &db).await?;
            record
        }
    };

    let sender = match room.get_member(&author_id).await? {
        Some(member) => member,
        None => {
            error!("Failed to fetch the sender: {}", author_id.to_string());

            return Err(anyhow!("Failed to fetch the sender's record from matrix"));
        }
    };

    let sender_display_name = sender.display_name().map(|name| name.to_string());

    let user = User::fetch_or_create(
        &db,
        author_id.to_string(),
        sender_display_name,
        room_record.id()?,
    )
    .await?;

    let mut call = Call::fetch_or_create(
        &db,
        event_id,
        user.id()?,
        room_record.id()?,
        message.clone(),
    )
    .await?;

    match call.status {
        Status::Pending => {}
        Status::Completed => {
            warn!("Call is already completed: {}", call.id()?.to_sql());

            return Ok(());
        }
        Status::Rejected => {
            warn!("Call is already rejected: {}", call.id()?.to_sql());

            return Ok(());
        }
    }

    let args: Vec<&str> = message.split_whitespace().collect();

    match args[0] {
        "!invite" => {
            info!("Received invite command from {}", user.address);

            let content = RoomMessageEventContent::text_plain("Executing invite command");

            room.send(content).await?;
        }
        "!progress" => {
            info!("Received progress command from {}", user.address);

            let content = RoomMessageEventContent::text_plain("Executing progress command");

            room.send(content).await?;
        }
        "!help" => {
            info!("Received help command from {}", user.address);

            let content =
                RoomMessageEventContent::text_plain("Available commands: !invite, !progress");

            room.send(content).await?;
        }
        _ => {}
    }

    call.completed(&db).await?;

    Ok(())
}
