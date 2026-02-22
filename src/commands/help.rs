use crate::commands::prelude::*;

impl Commands {
    pub async fn help() -> Result<()> {
        info!("Received help command");
        Ok(())
    }
}
