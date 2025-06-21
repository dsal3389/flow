use std::thread;
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc;

use log::{Metadata, Record};

/// the standard logger type that is used, at creation it will spawns
/// a new thread that will handle writes to the given writer, so the logging
/// operations will never block
pub struct Logger {
    tx: mpsc::Sender<String>,
}

impl Logger {
    pub fn new<W: Write + Sync + Send + 'static>(stream: W) -> Self {
        let (tx, rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            let mut stream = stream;
            while let Ok(message) = rx.recv() {
                print!("{}", message);
                let _ = stream.write_all(message.as_bytes());
            }
        });

        Logger { tx }
    }

    /// creates a new logger that writes to the given filepath
    pub fn from_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        Ok(Logger::new(file))
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        let message = format!("[{}] {}\n", record.metadata().level(), record.args());
        let _ = self.tx.send(message);
    }

    fn flush(&self) {}
}
