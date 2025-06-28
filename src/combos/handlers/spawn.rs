use async_trait::async_trait;
use super::ComboHandler;

#[derive(Debug, Default)]
pub struct Spawn {
    name: String,
    program: String,
    arguments: Vec<String>,
}

impl Spawn {
    pub fn new(name: String, program: String, arguments: Vec<String>) -> Self {
        Self { name, program, arguments }
    }
}

#[async_trait]
impl ComboHandler for Spawn {
    fn handler_name(&self) -> &str {
        &self.name
    }

    async fn handle(&self) -> anyhow::Result<()> {
        let _ = tokio::process::Command::new(&self.program)
            .args(&self.arguments)
            .output()
            .await
            .unwrap();
        Ok(())
    }
}
