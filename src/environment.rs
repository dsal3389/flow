use futures::StreamExt;
use x11rb_async::errors::ConnectionError;
use x11rb_async::protocol::Event;
use x11rb_async::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask, Window};
use x11rb_async::rust_connection::RustConnection;

use crate::connection::ConnectionEventStreamer;
use crate::keyboard::Key;

pub struct Environment {
    conn: RustConnection,
}

impl Environment {
    /// setup the window manager environment with all the required events
    pub async fn setup_with_connection(
        conn: RustConnection,
        root: Window,
    ) -> Result<Self, ConnectionError> {
        conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::default().event_mask(
                EventMask::STRUCTURE_NOTIFY
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::SUBSTRUCTURE_REDIRECT
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE,
            ),
        )
        .await?;
        Ok(Self { conn })
    }

    pub async fn run(mut self) -> Result<(), ConnectionError> {
        let mut stream = ConnectionEventStreamer(&self.conn);

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => match event {
                    Event::ButtonPress(_) | Event::ButtonPress(_) => {
                        let key = Key::try_from(event).expect("couldn't parse key event");
                        break;
                    }
                    _ => {}
                },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
