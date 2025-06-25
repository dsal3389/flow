use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ConfigBind {
    keys: Vec<String>,
    spawn: Vec<String>,
}

impl ConfigBind {
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    pub fn spawn(&self) -> &[String] {
        &self.spawn
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    binds: HashMap<String, ConfigBind>,
}

impl Config {
    /// read the file content located at the given path and using
    /// a toml parser to parse the data to rust representation
    pub fn from_path<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let content = read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// returns the configured binds as hashmaps, the key is the bind name
    /// while the value is the config bind
    #[inline]
    pub fn binds(&self) -> &HashMap<String, ConfigBind> {
        &self.binds
    }
}
