use crate::commands::prelude::*;

impl Commands {
    pub async fn progress() -> Result<()> {
        info!("Received progress command");
        Ok(())
    }
}
