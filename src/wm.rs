use x11rb_async::errors::ReplyError;
use x11rb_async::connection::Connection;
use x11rb_async::rust_connection::RustConnection;
use x11rb_async::protocol::xkb::ConnectionExt as _;
use x11rb_async::protocol::xproto::{self, ConnectionExt as _, EventMask, Window};
use x11rb_async::protocol::{ErrorKind, Event};

use crate::key::{Key, KeyState};

pub(crate) struct WindowManager {
    connection: RustConnection,
    keystate: Option<KeyState>,
    root: Window,
}

impl WindowManager {
    pub(crate) fn new(connection: RustConnection, root: Window) -> Self {
        WindowManager {
            connection,
            root,
            keystate: None,
        }
    }

    pub(crate) async fn setup(mut self) -> anyhow::Result<Self> {
        let attributes = xproto::ChangeWindowAttributesAux::new()
            .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT);
        self.connection
            .change_window_attributes(self.root, &attributes)
            .await?
            .check()
            .await
            .inspect_err(|err| {
                if let ReplyError::X11Error(err) = err {
                    if err.error_kind == ErrorKind::Access {
                        log::error!(
                            "window manager already runnig, couldn't request event from x11 server"
                        );
                    }
                }
            })?;

        self.connection.xkb_use_extension(1, 0).await?;
        self.keystate = Some(KeyState::from_connection(&self.connection).await?);
        Ok(self)
    }

    pub(crate) async fn run(self) -> anyhow::Result<()> {
        self.connection
            .grab_key(
                true,
                self.root,
                xproto::ModMask::ANY,
                Key::from('q').keycode(&self.keystate.unwrap()).expect("ss"),
                xproto::GrabMode::ASYNC,
                xproto::GrabMode::ASYNC,
            )
            .await?
            .check()
            .await?;
        loop {
            match self.connection.wait_for_event().await? {
                Event::KeyPress(event) => {
                    log::info!("key press event {:?}", event.detail)
                }
                _ => {}
            }
        }
        Ok(())
    }
}
