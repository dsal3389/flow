use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

mod connection;
mod environment;
mod keyboard;

use crate::environment::Environment;

struct Logger(Arc<Mutex<fs::File>>);

impl Logger {
    fn new<T>(filename: T) -> std::io::Result<Logger>
    where
        T: AsRef<str>,
    {
        let file = Arc::new(Mutex::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .write(true)
                .open(filename.as_ref())?,
        ));
        Ok(Logger(file))
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let file = Arc::clone(&self.0);
        let message = format!("[{}] {}", record.level(), record.args());

        write!(file.lock().unwrap(), "{}", message).unwrap();
        println!("{}", message);
    }

    fn flush(&self) {}
}

/// create a logger instance and set it as the main logger
fn install_logger() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Box::new(Logger::new("/home/dsal3389/flow.log")?);
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Info))?;
    Ok(())
}

/// installs panic hooks to log errors on panic
fn install_hooks() {
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("panic: {}", panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_logger()?;
    install_hooks();

    let (conn, root) = connection::create_connection().await?;
    let environment = Environment::setup_with_connection(conn, root).await?;
    environment.run().await.map_err(|err| {
        log::error!("exit with error {}", err);
        err.into()
    })
}
