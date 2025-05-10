use std::io::IoSlice;

use futures::StreamExt;
use x11rb::resource_manager::Database;
use x11rb_async::connection::RequestConnection;
use x11rb_async::errors::ReplyError;
use x11rb_async::protocol::xproto::{self, ConnectionExt, EventMask};
use x11rb_async::protocol::{ErrorKind, Event};

use crate::XConnection;

/// loads the default X database
async fn load_database(conn: &XConnection) -> anyhow::Result<Database> {
    let (bytes, fd) = Database::GET_RESOURCE_DATABASE.serialize();
    let slice = IoSlice::new(&bytes[0]);
    let reply = conn
        .send_request_with_reply(&[slice], fd)
        .await?
        .reply()
        .await?;
    Ok(Database::new_from_default(&reply, "".into()))
}

pub struct WindowManager {
    conn: XConnection,
}

impl WindowManager {
    pub async fn new_and_setup(conn: XConnection) -> anyhow::Result<Self> {
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

        conn.grab_key(
            true,
            conn.root(),
            xproto::ModMask::SHIFT,
            'q' as xproto::Keycode,
            xproto::GrabMode::ASYNC,
            xproto::GrabMode::ASYNC,
        )
        .await?
        .check()
        .await?;

        Ok(Self { conn })
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
