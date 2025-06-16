use std::io;
use std::path::Path;

use x11rb_async::connection::Connection;
use x11rb_async::rust_connection::RustConnection;

use crate::logger::Logger;

mod key;
mod logger;
mod wm;

#[inline]
fn setup_logger<P>(path: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let logger = Logger::from_path(path)?;
    // safe to unwrap, a logger should not have been
    // set at this point
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .unwrap();
    Ok(())
}

#[inline]
fn setup_hooks() {
    std::panic::set_hook(Box::new(|info| {
        log::error!("panied {}", info);
    }));
}

async fn run() -> anyhow::Result<()> {
    log::info!("starting rust connection to x11 server");
    let (connection, display, derive) = RustConnection::connect(None).await?;
    let root = connection.setup().roots[display].root;

    // create a background task
    // to listen for connection error
    tokio::spawn(async move {
        match derive.await {
            Err(err) => log::error!("connection error {}", err),
            _ => unreachable!(),
        }
    });

    wm::WindowManager::new(connection, root)
        .setup().await?
        .run().await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger("flow.log")?;
    setup_hooks();
    run().await.inspect_err(|err| {
        log::error!("exit with error, {}", err);
    })
}
