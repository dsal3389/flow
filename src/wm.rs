use x11rb_async::protocol::xproto::Window;
use x11rb_async::rust_connection::RustConnection;


pub(crate) struct WindowManager {
    connection: RustConnection,
    root: Window
}

impl WindowManager {
    pub(crate) fn new(connection: RustConnection, root: Window) -> Self {
        WindowManager { connection, root }
    }

    pub(crate) fn run(self) -> anyhow::Result<()> {
        todo!()
    }
}
