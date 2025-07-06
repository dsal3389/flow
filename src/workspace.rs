use std::sync::Arc;
use x11rb_async::connection::Connection;

use crate::window::Window;

pub struct Workspace<C>
where
    C: Connection
{
    connection: Arc<C>,
    windows: Vec<Window<C>>
}

impl<C> Workspace<C>
where
    C: Connection + Send
{
    pub fn with_connection(connection: Arc<C>) -> Self {
        Self {
            connection,
            windows: Vec::with_capacity(4)
        }
    }
}
