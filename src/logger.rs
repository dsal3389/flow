use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use log::{Metadata, Record};

pub struct Logger<W: Write + Sync + Send> {
    stream: Arc<Mutex<W>>,
}

impl Logger<std::fs::File> {
    pub fn from_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        let stream = Arc::new(Mutex::new(file));
        Ok(Logger { stream })
    }
}

impl<W> log::Log for Logger<W>
where
    W: Write + Sync + Send,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        let stream = Arc::clone(&self.stream);
        let message = format!("[{}] {}", record.metadata().level(), record.args());

        println!("{}", message);
        writeln!(stream.lock().unwrap(), "{}", message).expect("could not write to log stream");
    }

    fn flush(&self) {}
}
