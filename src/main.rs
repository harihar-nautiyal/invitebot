use dotenv::dotenv;
use invitebot::db::migration::start;
use invitebot::env::*;
use invitebot::handlers::auto_join::listen as invite_listener;
use invitebot::handlers::message::listen as message_listener;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::UserId;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
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

    let db = Surreal::new::<Ws>(DB_URL.as_str()).await?;

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

    let client = Client::builder()
        .server_name(user.server_name())
        .build()
        .await?;

    client
        .matrix_auth()
        .login_username(&username, &password)
        .send()
        .await?;

    info!("Successfully logged in as {}", user);

    client.add_event_handler(invite_listener);
    info!("Listening for invites");

    client.add_event_handler(message_listener);
    info!("Listening for commands");

    client.sync(SyncSettings::default()).await?;

    Ok(())
}
