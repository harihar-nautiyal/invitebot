use crate::models::member::Member;
use crate::models::room::Room as RoomRecord;
use anyhow::Result;
use matrix_sdk::ruma::events::room::member::{MembershipState, SyncRoomMemberEvent};
use matrix_sdk::{Client, Room};
use std::sync::Arc;
use surrealdb::{Surreal, engine::remote::ws::Client as SurrealClient};
use tracing::Instrument;
use tracing::{error, info};

pub async fn listen(
    event: SyncRoomMemberEvent,
    client: Arc<Client>,
    room: Room,
    db: Arc<Surreal<SurrealClient>>,
) -> Result<()> {
    let original_event = match event {
        SyncRoomMemberEvent::Original(e) => e,
        SyncRoomMemberEvent::Redacted(_) => return Ok(()),
    };

    let user_id = original_event.state_key.to_string();
    let display_name = original_event.content.displayname.clone();
    let membership = original_event.content.membership.clone();

    if let Some(bot_id) = client.user_id() {
        if user_id == bot_id.to_string() {
            return Ok(());
        }
    }

    tokio::spawn(
        async move {
            if let Err(err) = handle_member_event(db, room, user_id, display_name, membership).await
            {
                error!("Error handling member state update: {err:?}");
            }
        }
        .in_current_span(),
    );

    Ok(())
}

async fn handle_member_event(
    db: Arc<Surreal<SurrealClient>>,
    room: Room,
    user_address: String,
    display_name: Option<String>,
    membership: MembershipState,
) -> Result<()> {
    let room_address = room.room_id().to_string();

    let room_record_id = match RoomRecord::fetch_id(&db, &room_address).await? {
        Some(id) => id,
        None => RoomRecord::insert_from_matrix(room.clone(), &db)
            .await?
            .id()?,
    };

    match membership {
        MembershipState::Join => {
            info!(
                "User {} joined/updated in room {}",
                user_address, room_address
            );

            let _member =
                Member::fetch_or_create(&db, user_address.clone(), display_name.clone()).await?;

            let join_query = r#"
                UPDATE users SET
                    display_name = $display_name,
                    updated_at = time::now()
                WHERE address = $address;

                UPDATE users SET
                    rooms += $room_id
                WHERE address = $address AND $room_id NOTINSIDE rooms;

                UPDATE members SET
                    display_name = $display_name
                WHERE address = $address;
            "#;

            db.query(join_query)
                .bind(("display_name", display_name))
                .bind(("address", user_address))
                .bind(("room_id", room_record_id))
                .await?;
        }

        MembershipState::Leave | MembershipState::Ban => {
            info!(
                "User {} left/banned from room {}",
                user_address, room_address
            );

            let leave_query = r#"
                UPDATE users SET rooms -= $room_id WHERE address = $address;

                let $member_id = (SELECT VALUE id FROM members WHERE address = $address LIMIT 1)[0];
                IF $member_id {
                    UPDATE $room_id SET members -= $member_id;
                };
            "#;

            db.query(leave_query)
                .bind(("room_id", room_record_id))
                .bind(("address", user_address))
                .await?;
        }

        _ => {
            info!(
                "User {} is now {:?} in room {}",
                user_address, membership, room_address
            );
        }
    }

    Ok(())
}
