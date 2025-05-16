use std::ops::Deref;
use std::task::Poll;

use futures::Stream;
use thiserror::Error;
use x11rb_async::errors::{ConnectionError, ConnectError};
use x11rb_async::connection::Connection;
use x11rb_async::protocol::Event;
use x11rb_async::protocol::xproto::Window;
use x11rb_async::rust_connection::RustConnection;


#[derive(Error, Debug)]
pub enum FlowConnectionError {
    #[error(transparent)]
    X11ConnectionError(#[from] ConnectionError),

    #[error(transparent)]
    X11ConnectError(#[from] ConnectError)
}


#[derive(Debug)]
pub struct FlowConnection {
    conn: RustConnection,
    root: Window,
}

impl FlowConnection {
    /// creates a default connection to the x11 server and does nothing with it,
    /// also setup the connection derive in the background
    pub async fn connect(display_name: Option<&str>) -> Result<Self, FlowConnectionError> {
        let (conn, display, derive) = RustConnection::connect(display_name).await?;
        let root = conn.setup().roots[display].root;

        // create a background task that will listen for
        // incoming data from the x11 server
        tokio::spawn(async move {
            match derive.await {
                Err(e) => log::error!("connection error {}", e),
                _ => unreachable!(),
            }
        });
        Ok(Self { conn, root })
    }

    /// returns the connection root window
    pub fn root(&self) -> Window {
        self.root
    }

    /// although the type implements `Deref`, some functions expect
    /// a type that implements `Connection + ConnectionExt` and it is more
    /// nice and readable to call inner instead or referencing a deref (&*conn)
    pub fn x11_raw_connection(&self) -> &RustConnection {
        &self.conn
    }
}

/// defined so we could use all the regular `RustConnection`
/// functionalities
impl Deref for FlowConnection {
    type Target = RustConnection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl Stream for FlowConnection {
    type Item = Result<Event, ConnectionError>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let result = match self.poll_for_event() {
            Ok(event) => {
                if event.is_none() {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }

                let inner = unsafe { event.unwrap_unchecked() };
                Ok(inner)
            }
            Err(e) => Err(e.into()),
        };

        cx.waker().wake_by_ref();
        Poll::Ready(Some(result))
    }
}
