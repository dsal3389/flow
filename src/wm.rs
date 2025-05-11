use futures::StreamExt;
use x11rb_async::errors::ReplyError;
use x11rb_async::protocol::xkb::ConnectionExt as _;
use x11rb_async::protocol::xproto::{self, ConnectionExt, EventMask};
use x11rb_async::protocol::{ErrorKind, Event};
use xkbcommon::xkb as xkbc;

use crate::XConnection;
use crate::keys;

pub struct WindowManager {
    conn: XConnection,
    key_state: keys::KeyState,
}

impl WindowManager {
    pub async fn new_and_setup<'a, T>(conn: XConnection, keys_map: T) -> anyhow::Result<Self>
    where
        T: IntoIterator<Item = &'a keys::KeyMap>,
    {
        let cookie = conn
            .change_window_attributes(
                conn.root(),
                &xproto::ChangeWindowAttributesAux::default().event_mask(
                    EventMask::SUBSTRUCTURE_NOTIFY
                        | EventMask::SUBSTRUCTURE_REDIRECT
                        | EventMask::STRUCTURE_NOTIFY
                        | EventMask::PROPERTY_CHANGE
                        | EventMask::KEY_PRESS
                        | EventMask::KEY_RELEASE
                        | EventMask::BUTTON_PRESS
                        | EventMask::BUTTON_RELEASE,
                ),
            )
            .await?;

        // check if the response is access, it means we don't have access
        // to request the events probably because another WM is already
        // registered for them, so log an appropriate message
        cookie.check().await.inspect_err(|err| {
            if let ReplyError::X11Error(err) = err {
                if err.error_kind == ErrorKind::Access {
                    log::error!(
                        "window manager already running, couldn't request events from x11 server"
                    );
                }
            }
        })?;

        conn.xkb_use_extension(1, 0).await?.reply().await?;
        let key_state = keys::KeyState::from_connection(conn.inner()).await?;

        conn.ungrab_key(0, conn.root(), xproto::ModMask::ANY)
            .await?
            .check()
            .await?;

        // request the hot keys events for each key map
        // some key maps may not register if we can't find a matching keycode
        // for a mapped key, in both case a log message will be written
        // to notify the user
        for map in keys_map {
            match map.key().keycode(&key_state) {
                Some(keycode) => {
                    conn.grab_key(
                        false,
                        conn.root(),
                        xproto::ModMask::ANY,
                        keycode,
                        xproto::GrabMode::ASYNC,
                        xproto::GrabMode::ASYNC,
                    )
                    .await?
                    .check()
                    .await?;
                    log::info!("register mapping {}", map);
                }
                None => {
                    log::warn!("couldn't register mapping {}", map);
                }
            }
        }

        Ok(WindowManager { conn, key_state })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Some(result) = self.conn.next().await {
            let event = result?;

            match event {
                Event::KeyPress(press_event) => {
                    log::debug!(
                        "press event details {} with state {:?}",
                        press_event.detail,
                        press_event.state
                    );
                }
                _ => {}
            }

            log::debug!("recv event: {:?}", event);
        }
        Ok(())
    }
}
