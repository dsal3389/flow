use async_trait::async_trait;

use super::ComboHandler;

// TODO: remove Default impl, its only for testing
#[derive(Default)]
pub struct SpawnHandler {
    program: String,
    arguments: Vec<String>,
}

#[async_trait]
impl ComboHandler for SpawnHandler {
    async fn handle(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
