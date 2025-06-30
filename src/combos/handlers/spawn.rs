use std::process::Stdio;

use async_trait::async_trait;
use super::ComboHandler;

/// spawn handler allows calling subprocesses to be executed
/// so binding combos will trigger subprocess that can
/// request for GUI from X11 and other
#[derive(Debug, Default)]
pub struct Spawn {
    name: String,
    program: String,
    arguments: Vec<String>,
}

impl Spawn {
    pub fn new(name: String, program: String, arguments: Vec<String>) -> Self {
        Self {
            name,
            program,
            arguments,
        }
    }
}

#[async_trait]
impl ComboHandler for Spawn {
    fn handler_name(&self) -> &str {
        &self.name
    }

    /// fire off the given program with the given argument
    /// if there is a problem with the spawning the process we return it, but we don't
    /// care about the process results itself
    async fn handle(&self) -> anyhow::Result<()> {
        tokio::process::Command::new(&self.program)
            .args(&self.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
}
