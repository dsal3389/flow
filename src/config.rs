/// the global window manager config
///
/// the configuration is used across the window-manager, it must have
/// a default implementation if the user didn't configure a specific section
/// or the user doesn't have config at all
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
    M5,
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

/// a combo definition contains the `keys` (key combination)
/// and the action to take when the combination is executed
///
/// all combination has a "hidden" key at the start, that key is the modifier
/// defined in the `FlowConfig`, so if a combination need to be executed
/// the user must first press the `modifier` key and then the combination
#[derive(Debug, Deserialize, Default)]
pub struct ConfigCombo {
    keys: Vec<String>,
    spawn: Vec<String>,
}

impl ConfigCombo {
    #[inline]
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    #[inline]
    pub fn spawn(&self) -> &[String] {
        &self.spawn
    }
}

/// defines the global fields that the flow window manger uses
/// those fields are too generic to be in specific sections
#[derive(Debug, Deserialize, Default)]
pub struct FlowConfig {
    modifier: Modifier,
}

impl FlowConfig {
    #[inline]
    pub fn modifier(&self) -> Modifier {
        self.modifier
    }
}

/// represent the fields and sections that the config file
/// should contain, the config must always implement the `Default`
/// trait in cases user doesn't implement a section or doesn't
/// have config file at all
///
/// the guarantee to implement the `Default` trait result in
/// easier / cleaner config file, not requiring to hussle with config
/// options and just start
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    flow: FlowConfig,

    /// the defined combos with names in the config file
    /// the hashkey is the name, the value is the combo information
    /// ```toml
    /// [combos.name]
    /// ...
    /// ```
    #[serde(rename(deserialize = "combo"))]
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
