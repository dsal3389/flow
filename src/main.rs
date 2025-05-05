use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod connection;
mod environment;
mod keyboard;

use crate::environment::Environment;

/// the main logger struct, it accepts a file
/// that all the log messages will be written to
struct Logger(Arc<Mutex<fs::File>>);

impl Logger {
    const LOGGER_FILENAME: &str = "flow.log";

    fn new<T>(filename: T) -> std::io::Result<Logger>
    where
        T: AsRef<Path>,
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

    fn get_path_path() -> PathBuf {
        std::env::var("HOME").map_or_else(
            |_| PathBuf::from("/var/log").join(Self::LOGGER_FILENAME),
            |value| PathBuf::from(value).join(Self::LOGGER_FILENAME),
        )
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let file = Arc::clone(&self.0);
        let message = format!("[{}] {}", record.level(), record.args());

        writeln!(file.lock().unwrap(), "{}", message).unwrap();
        println!("{}", message);
    }

    fn flush(&self) {
        let file = Arc::clone(&self.0);
        let _ = file.lock().unwrap().flush();
    }
}

/// create a logger instance and set it as the main logger
fn install_logger() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Box::new(Logger::new(Logger::get_path_path())?);
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Debug))?;
    Ok(())
}

/// installs panic hooks to log errors on panic
fn install_hooks() {
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("panic: {}", panic_info);
    }));
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, root) = connection::create_connection().await?;
    let environment = Environment::setup_with_connection(conn, root).await?;

    log::info!("window manager environment is setup successfuly");

    environment.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_logger()?;
    install_hooks();

    log::debug!("logger setup and panic hooks are installed");

    run().await.inspect_err(|err| {
        log::error!("exit with error {}", err);
    })
}
