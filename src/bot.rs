use crate::handlers::auto_join::listen as invite_listener;
use crate::handlers::message::listen as message_listener;
use crate::handlers::recovery::run_startup_sync;
use crate::handlers::update::listen as update_listener;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::UserId;
use matrix_sdk::ruma::events::room::member::SyncRoomMemberEvent;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::{Room, ruma::events::room::member::StrippedRoomMemberEvent};
use std::sync::Arc;
use surrealdb::Surreal;
use tracing::{Instrument, error, info};

pub async fn sync(
    server: String,
    username: String,
    password: String,
    db: Arc<Surreal<surrealdb::engine::remote::ws::Client>>,
    label: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_id_str = format!("@{}:{}", username, server);
    let user_id = UserId::parse(&user_id_str)?;
    let sqlite_path = format!("state_store_{}.db", label);

    let client = Client::builder()
        .server_name(user_id.server_name())
        .sqlite_store(sqlite_path, None)
        .build()
        .await?;

    client
        .matrix_auth()
        .login_username(&username, &password)
        .device_id(&label)
        .initial_device_display_name(format!("InviteBot-{}", label).as_str())
        .send()
        .await?;

    let client = Arc::new(client);
    info!("Logged in as {}", user_id);

    info!("Registering auto-invite listener");
    let c = client.clone();
    let d = db.clone();
    client.add_event_handler(move |ev: StrippedRoomMemberEvent, room: Room| {
        let c = c.clone();
        let d = d.clone();

        async move {
            if let Err(err) = invite_listener(ev, c, room, d).await {
                error!("Invite error: {}", err);
            }
        }
        .in_current_span()
    });

    info!("Registering update listener");
    let c = client.clone();
    let d = db.clone();
    client.add_event_handler(move |ev: SyncRoomMemberEvent, room: Room| {
        let c = c.clone();
        let d = d.clone();

        async move {
            if let Err(err) = update_listener(ev, c, room, d).await {
                error!("Update error: {}", err);
            }
        }
        .in_current_span()
    });

    info!("Registering commands listener");
    if label == "MAIN_BOT" {
        let d = db.clone();
        client.add_event_handler(move |ev: OriginalSyncRoomMessageEvent, room: Room| {
            let d = d.clone();

            async move {
                if let Err(err) = message_listener(ev, room, &d).await {
                    error!("Message error: {}", err);
                }
            }
            .in_current_span()
        });
    }

    let sync_client = client.clone();
    let sync_db = db.clone();

    tokio::spawn(
        async move {
            run_startup_sync(sync_client, sync_db).await;
        }
        .in_current_span(),
    );

    info!("Starting sync loop...");
    client.sync(SyncSettings::default()).await?;

    Ok(())
}
