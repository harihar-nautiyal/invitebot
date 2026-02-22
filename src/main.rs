use dotenv::dotenv;
use invitebot::db::migration::start;
use invitebot::env::*;
use invitebot::handlers::auto_join::listen as invite_listener;
use invitebot::handlers::message::listen as message_listener;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::UserId;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::{Room, ruma::events::room::member::StrippedRoomMemberEvent};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use tracing::error;
use tracing::info;
use tracing::subscriber::set_global_default;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("warn,invitebot=info,matrix_sdk_crypto::identities::manager=error,matrix_sdk_crypto::backups=error")
    });

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(true)
        .with_env_filter(filter)
        .finish();

    set_global_default(subscriber)?;

    info!("Starting invitebot");

    info!("Connecting to Database");

    let db = Arc::new(Surreal::new::<Ws>(DB_URL.as_str()).await?);

    db.signin(Root {
        username: DB_USERNAME.to_string(),
        password: DB_PASSWORD.to_string(),
    })
    .await?;

    db.use_ns("invitebot").use_db("invitebot").await?;

    info!("Connected to Database");

    info!("Starting database migrations");

    start(&db).await?;

    info!("Completed database migrations");

    let server = SERVER.as_str();
    let username = USERNAME.as_str();
    let password = PASSWORD.as_str();

    let user_id_str = format!("@{}:{}", username, server);
    let user = UserId::parse(&user_id_str)?;

    info!("Logging as {}", user);

    let client = Arc::new(
        Client::builder()
            .server_name(user.server_name())
            .build()
            .await?,
    );

    client
        .matrix_auth()
        .login_username(&username, &password)
        .send()
        .await?;

    info!("Successfully logged in as {}", user);

    let client_clone_for_invites = client.clone();
    let db_clone_for_invites = db.clone();

    client.add_event_handler(async move |ev: StrippedRoomMemberEvent, room: Room| {
        if let Err(err) = invite_listener(
            ev,
            client_clone_for_invites.clone(),
            room,
            db_clone_for_invites.clone(),
        )
        .await
        {
            error!("Error handling invite: {}", err);
        }
    });

    info!("Listening for invites");

    let db_clone_for_messages = db.clone();

    client.add_event_handler(async move |ev: OriginalSyncRoomMessageEvent, room: Room| {
        if let Err(err) = message_listener(ev, room, &db_clone_for_messages).await {
            error!("Error handling message: {}", err);
        }
    });

    info!("Listening for commands");

    client.sync(SyncSettings::default()).await?;

    Ok(())
}
