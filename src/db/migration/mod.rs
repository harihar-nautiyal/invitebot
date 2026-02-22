pub mod calls;
pub mod invites;
pub mod members;
pub mod rooms;
pub mod users;

use anyhow::Result;
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::db::migration::calls::CALLS;
use crate::db::migration::invites::INVITES;
use crate::db::migration::members::MEMBERS;
use crate::db::migration::rooms::ROOMS;
use crate::db::migration::users::USERS;

use tracing::info;

pub async fn start(db: &Surreal<Client>) -> Result<()> {
    info!("Initializing users on database");

    db.query(USERS.as_str()).await?;

    info!("Initialized users on database");

    info!("Initializing rooms on database");

    db.query(ROOMS.as_str()).await?;

    info!("Initialized rooms on database");

    info!("Initializing members on database");

    db.query(MEMBERS.as_str()).await?;

    info!("Initialized members on database");

    info!("Initializing users on database");

    db.query(USERS.as_str()).await?;

    info!("Initialized users on database");

    info!("Initializing invites on database");

    db.query(INVITES.as_str()).await?;

    info!("Initialized invites on database");

    info!("Initializing calls on database");

    db.query(CALLS.as_str()).await?;

    info!("Initialized calls on database");

    Ok(())
}
