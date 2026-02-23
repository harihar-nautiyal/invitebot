#![recursion_limit = "512"]

use dotenv::dotenv;
use invitebot::bot::sync;
use invitebot::db::migration::start;
use invitebot::env::*;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use tracing::subscriber::set_global_default;
use tracing::{Instrument, error, info, info_span};
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

    let bot_configs = vec![
        (
            SERVER.as_str(),
            USERNAME.as_str(),
            PASSWORD.as_str(),
            "main",
        ),
        (
            SCRAPER_1_SERVER.as_str(),
            SCRAPER_1_USERNAME.as_str(),
            SCRAPER_1_PASSWORD.as_str(),
            "scraper-1",
        ),
        (
            SCRAPER_2_SERVER.as_str(),
            SCRAPER_2_USERNAME.as_str(),
            SCRAPER_2_PASSWORD.as_str(),
            "scraper-2",
        ),
    ];

    let mut handles = vec![];

    for (server, user, pass, label) in bot_configs {
        let db = db.clone();
        let server = server.to_string();
        let user_name = user.to_string();
        let password = pass.to_string();
        let label = label.to_string();
        let span = info_span!("bot", id = %label);

        let handle = tokio::spawn(
            async move {
                info!("Initializing client...");
                if let Err(e) = sync(server, user_name, password, db, label.clone()).await {
                    error!("Critical failure: {}", e);
                }
            }
            .instrument(span),
        );
        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    Ok(())
}
