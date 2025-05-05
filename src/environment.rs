use futures::StreamExt;
use x11rb_async::errors::ConnectionError;
use x11rb_async::protocol::xproto::{
    ChangeWindowAttributesAux, ConnectionExt, EventMask, GrabMode, Keycode, ModMask, Window,
};
use x11rb_async::rust_connection::RustConnection;

use crate::connection::ConnectionEventStreamer;
use crate::keyboard::KeyEvent;

pub struct Environment {
    conn: RustConnection,
}

impl Environment {
    /// setup the window manager environment with all the required events
    pub async fn setup_with_connection(
        conn: RustConnection,
        root: Window,
    ) -> Result<Self, ConnectionError> {
        log::debug!("setting up environment with created rust connection");
        log::debug!("changing window `{}` attributes", root);

        conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::default().event_mask(
                EventMask::STRUCTURE_NOTIFY
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::SUBSTRUCTURE_REDIRECT,
            ),
        )
        .await?;

        conn.ungrab_key(0, root, ModMask::ANY).await?;
        conn.grab_key(
            true,
            root,
            ModMask::SHIFT,
            'q' as u8,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .await?;

        log::debug!("grab and ungrab requet event performed successfuly");
        Ok(Self { conn })
    }

    pub async fn run(self) -> Result<(), ConnectionError> {
        log::info!("start listening for events from X server");
        let mut stream = ConnectionEventStreamer(&self.conn);

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => match event {
                    _ => {
                        log::info!("new event {:?}", event);
                        break;
                    }
                },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
