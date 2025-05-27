use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use flow_config::Config;
use flow_core::FlowConnection;

mod wm;

/// the main logger struct, it accepts a file
/// that all the log messages will be written to
struct Logger(Arc<Mutex<fs::File>>);

impl Logger {
    const FILENAME: &str = "flow.log";

    fn from_path<P>(path: P) -> std::io::Result<Logger>
    where
        P: AsRef<Path>,
    {
        let file = Arc::new(Mutex::new(
            fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path.as_ref())?,
        ));
        Ok(Logger(file))
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
        writeln!(
            file.lock().unwrap(),
            "[{}] {}",
            record.level(),
            record.args()
        )
        .expect("couldn't write to the log file");
    }

    fn flush(&self) {
        let file = Arc::clone(&self.0);
        let _ = file.lock().map(|mut f| f.flush());
    }
}

fn find_logger_file_path() -> PathBuf {
    std::env::var("HOME").map_or_else(
        |_| PathBuf::from("/var/log").join(Logger::FILENAME),
        |value| PathBuf::from(value).join(Logger::FILENAME),
    )
}

fn find_config_file_path() -> anyhow::Result<PathBuf> {
    std::env::var("HOME")
        .context("HOME environment variable is not set, couldn't find default config path")
        .map(|value| PathBuf::from(value).join(Config::FILENAME))
}

/// create a logger instance and set it as the main logger
#[inline]
fn install_logger() -> anyhow::Result<()> {
    let logger = Box::new(Logger::from_path(find_logger_file_path())?);
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Debug))?;
    Ok(())
}

/// installs panic hooks to log errors on panic
#[inline]
fn install_hooks() {
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("panic: {}", panic_info);
    }));
}

#[inline]
fn load_config() -> anyhow::Result<Config> {
    Config::from_path(find_config_file_path()?).map_err(|e| e.into())
}

async fn run() -> anyhow::Result<()> {
    let config = load_config()?;
    let connection = FlowConnection::connect(None).await?;
    let window_manager = wm::WindowManager::setup_with_config(connection, config).await?;

    log::info!("window manager environment is setup successfuly");

    window_manager.run().await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_logger()?;
    install_hooks();

    log::debug!("logger setup and panic hooks are installed");

    // TODO: just for development so I won't be stuck outside
    let handler = tokio::spawn(async move {
        let _ = run().await.inspect_err(|err| {
            log::error!("exit with error {}", err);
        });
    });

    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    handler.abort();
    Ok(())
}
