use crate::commands::prelude::*;

impl Commands {
    pub async fn donate() -> Result<()> {
        info!("Received donate command");
        Ok(())
    }
}
