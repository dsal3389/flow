use std::env;
use std::path::PathBuf;

use anyhow::Context;

mod config;
mod key;
mod logger;
mod wm;

pub use config::Config;
pub use logger::Logger;
pub use wm::WindowManager;

/// tries to find the given config filename at predefined location
/// the priority and locations
///     `./flow.cfg`
///     `$HOME/flow.cfg`
///     `$HOEM/.config/flow.cfg`
pub fn find_config_path(filename: &str) -> anyhow::Result<PathBuf> {
    // if the config file is in the current directory
    // it has the highest priority so we need to return it
    let current = PathBuf::from(filename);
    if current.exists() {
        return Ok(current);
    }

    // find the user home directory based
    // on the `HOME` environment variable
    let home: PathBuf = env::var("HOME")
        .context("couldn't resolve `HOME` environment variable")?
        .into();

    let path = home.join(filename);
    if path.exists() {
        return Ok(path);
    }

    let path = home.join(".config").join(filename);
    if path.exists() {
        return Ok(path);
    }

    Err(anyhow::anyhow!("couldn't find config file anywhere"))
}
