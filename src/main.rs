use std::path::Path;
use std::sync::Arc;

use tokio::sync::Mutex;
use x11rb_async::connection::Connection;
use x11rb_async::rust_connection::RustConnection;

#[inline]
fn setup_logger<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let logger = flow::Logger::from_path(path)?;
    // safe to unwrap, a logger should not have been
    // set at this point
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
        .unwrap();
    Ok(())
}

/// bofore panicing and existing from the window-manager
/// we log the critical error
#[inline]
fn setup_hooks() {
    std::panic::set_hook(Box::new(|info| {
        log::error!("panied {}", info);
    }));
}

async fn run() -> anyhow::Result<()> {
    let config = flow::find_config_path("flow.toml").and_then(|cfg_path| {
        log::debug!("found config file at `{}`", cfg_path.display());
        flow::Config::from_path(cfg_path)
    })?;

    log::info!("starting rust connection to x11 server");
    let (connection, display, derive) = RustConnection::connect(None).await?;
    let root = connection.setup().roots[display].root;

    // create a background task to put the derive
    // on the execution loop task
    tokio::spawn(async move {
        match derive.await {
            Err(err) => log::error!("connection error {}", err),
            _ => unreachable!(),
        }
    });

    let wm = Arc::new(flow::WindowManager::from_connection(connection, root, config).await?);
    wm.run().await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger("flow.log")?;
    setup_hooks();

    // after successfull logger and panic hooks setups
    // we call the run function which holds the real functionality
    // and we log the errors returned if any
    run().await.inspect_err(|err| {
        log::error!("exit with error, {}", err);
    })
}
