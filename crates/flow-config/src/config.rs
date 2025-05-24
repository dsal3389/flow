use std::io;
use std::path::Path;


pub struct Config {
}

impl Config {
    pub const FILENAME: &str = "flow.cfg";

    pub fn from_path<P>(path: P) -> io::Result<Config>
    where
        P: AsRef<Path>,
    {
        todo!()
    }
}
