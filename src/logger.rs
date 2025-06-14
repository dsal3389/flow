use std::path::Path;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use log::{Record, Level, Metadata};


pub(crate) struct Logger<W: Write + Sync + Send> {
    stream: Arc<Mutex<W>>
}

impl Logger<std::fs::File> {
    pub(crate) fn from_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>
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
    W: Write + Sync + Send
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let stream = Arc::clone(&self.stream);
        write!(stream.lock().unwrap(), "[{}] {}", record.metadata().level(), record.args())
            .expect("could not write to log stream");
    }

    fn flush(&self) {}
}
