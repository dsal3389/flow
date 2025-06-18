use std::collections::HashMap;

use x11rb_async::connection::Connection;
use x11rb_async::errors::ReplyError;
use x11rb_async::protocol::xkb::ConnectionExt as _;
use x11rb_async::protocol::xproto::{self, ConnectionExt as _, EventMask, Window};
use x11rb_async::protocol::{ErrorKind, Event};
use xkbcommon::xkb;

use crate::Config;
use crate::key::{Key, KeyState};

pub struct WindowManager<C> {
    config: Config,
    connection: C,
    keybinds: HashMap<xkb::Keycode, ()>,
    keystate: KeyState,
    root: Window,
}

impl<C> WindowManager<C>
where
    C: Connection
{
    /// creates a new window manager from the given connection
    /// will try to change the root window attributes to register
    /// as window manager
    pub async fn from_connection(
        connection: C,
        root: Window,
        config: Config,
    ) -> anyhow::Result<Self> {
        let attributes = xproto::ChangeWindowAttributesAux::new()
            .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT);
        connection
            .change_window_attributes(root, &attributes)
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

        connection.xkb_use_extension(1, 0).await?;
        let keystate = KeyState::from_connection(&connection).await?;

        Ok(WindowManager {
            config,
            connection,
            root,
            keystate,
            keybinds: HashMap::default(),
        })
    }

    /// running the window manager will register for keybinds defined in the
    /// configuration and listen/handle events from X11
    pub async fn run(mut self) -> anyhow::Result<()> {
        self.setup_binds().await?;
        loop {
            match self.connection.wait_for_event().await? {
                Event::KeyPress(event) => {
                    log::info!(
                        "key press event {:?}, {:?}",
                        event.detail,
                        self.keybinds.get(&xkb::Keycode::new(event.detail as u32))
                    )
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn setup_binds(&mut self) -> anyhow::Result<()> {
        let keys = vec![
            Key::from('q').keycode(&self.keystate).unwrap(),
            Key::from('w').keycode(&self.keystate).unwrap(),
        ];
        let mut map = HashMap::new();

        for keycode in keys {
            map.insert(keycode, ());
            self.connection
                .grab_key(
                    true,
                    self.root,
                    xproto::ModMask::ANY,
                    keycode,
                    xproto::GrabMode::ASYNC,
                    xproto::GrabMode::ASYNC,
                )
                .await?
                .check()
                .await?;
        }
        self.keybinds = map;
        Ok(())
    }
}
