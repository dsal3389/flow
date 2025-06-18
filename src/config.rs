use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigBind {
    keys: Vec<String>,
    spawn: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigBinds {
    binds: HashMap<String, ConfigBind>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    binds: ConfigBinds,
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
}
