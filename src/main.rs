use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
                .truncate(true)
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
        let _ = file.lock().map(|mut f| f.flush());
    }
}

/// create a logger instance and set it as the main logger
#[inline]
fn install_logger() -> anyhow::Result<()> {
    let logger = Box::new(Logger::new(Logger::get_path_path())?);
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

async fn run() -> anyhow::Result<()> {
    let xconn = flow::XConnection::connect(None).await?;
    let window_manager = flow::WindowManager::new_and_setup(xconn).await?;

    log::info!("window manager environment is setup successfuly");

    window_manager.run().await?;
    Ok(())
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
