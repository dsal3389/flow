use async_trait::async_trait;

use super::BindHandler;

// TODO: remove Default impl, its only for testing
#[derive(Default)]
pub struct SpawnHandler {
    program: String,
    arguments: Vec<String>,
}

#[async_trait]
impl BindHandler for SpawnHandler {
    async fn handle(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
