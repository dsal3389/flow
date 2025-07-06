use std::sync::Arc;
use x11rb_async::connection::Connection;
use x11rb_async::protocol::xproto::Window as X11Window;

pub struct Window<C>
where
    C: Connection
{
    connection: Arc<C>,
    x11_window: X11Window
}

impl<C> Window<C>
where
    C: Connection
{
    pub fn new(connection: Arc<C>, window: X11Window) -> Self {
        Self {
            connection,
            x11_window: window
        }
    }
}
