use crate::commands::prelude::*;

impl Commands {
    pub async fn invite() -> Result<()> {
        info!("Received invite command");
        Ok(())
    }
}
