use std::fs::read_to_string;
use std::io;
use std::path::{Path, PathBuf};

use futures::Stream;

use super::lexer::Lexer;
use super::profile::ProfileContext;

pub struct ConfigParser {
    lexer: Lexer,
}

pub struct Config {
    lex: Lexer,
}

impl Config {
    pub const FILENAME: &str = "flow.cfg";

    pub fn from_path<P>(path: P) -> io::Result<Config>
    where
        P: AsRef<Path>,
    {
        let content = read_to_string(path.as_ref())?;
        let lex = Lexer::new(content);
        Ok(Config { lex })
    }
}
