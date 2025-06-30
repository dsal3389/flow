use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use serde::Deserialize;

use x11rb_async::protocol::xproto::ModMask;

#[derive(Debug, Deserialize, Default, Clone, Copy)]
pub enum Modifier {
    CTRL,
    SHIFT,
    LOCK,
    #[default]
    M1,
    M2,
    M3,
    M4,
    M5
}

impl From<Modifier> for ModMask {
    fn from(value: Modifier) -> ModMask {
        match value {
            Modifier::CTRL => ModMask::CONTROL,
            Modifier::SHIFT => ModMask::SHIFT,
            Modifier::LOCK => ModMask::LOCK,
            Modifier::M1 => ModMask::M1,
            Modifier::M2 => ModMask::M2,
            Modifier::M3 => ModMask::M3,
            Modifier::M4 => ModMask::M4,
            Modifier::M5 => ModMask::M5,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigCombo {
    keys: Vec<String>,
    spawn: Vec<String>,
}

impl ConfigCombo{
    #[inline]
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    #[inline]
    pub fn spawn(&self) -> &[String] {
        &self.spawn
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FlowConfig {
    modifier: Modifier
}

impl FlowConfig {
    #[inline]
    pub fn modifier(&self) -> Modifier {
        self.modifier
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    flow: FlowConfig,

    #[serde(rename(deserialize="combo"))]
    combos: HashMap<String, ConfigCombo>,
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

    #[inline]
    pub fn flow(&self) -> &FlowConfig {
        &self.flow
    }

    /// returns the configured combos as hashmaps, the key is the bind name
    /// while the value is the config bind
    #[inline]
    pub fn combos(&self) -> &HashMap<String, ConfigCombo> {
        &self.combos
    }
}
