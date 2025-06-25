use async_trait::async_trait;
use super::ComboHandler;

#[derive(Debug, Default)]
pub struct Spawn {
    program: String,
    arguments: Vec<String>
}

#[async_trait]
impl ComboHandler for Spawn {
    async fn handle(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
