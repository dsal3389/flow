mod connection;
mod environment;
mod keyboard;

use crate::environment::Environment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, root) = connection::create_connection().await?;
    let environment = Environment::setup_with_connection(conn, root).await?;
    Ok(environment.run().await?)
}
