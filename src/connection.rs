use std::task::Poll;

use futures::Stream;
use x11rb_async::connection::Connection;
use x11rb_async::errors::{ConnectError, ConnectionError};
use x11rb_async::protocol::Event;
use x11rb_async::protocol::xproto::Window;
use x11rb_async::rust_connection::RustConnection;

/// creates a default connection to the x11 server and does nothing with it,
/// also setup the connection derive in the background
pub async fn create_connection() -> Result<(RustConnection, Window), ConnectError> {
    let (conn, _, derive) = RustConnection::connect(None).await?;
    let root = conn.setup().roots[0].root;

    // create a background task that will listen for
    // incoming data from the x11 server
    tokio::spawn(async move {
        match derive.await {
            Err(e) => eprintln!("error {}", e),
            _ => unreachable!(),
        }
    });
    Ok((conn, root))
}

pub struct ConnectionEventStreamer<'conn>(pub &'conn RustConnection);

impl Stream for ConnectionEventStreamer<'_> {
    type Item = Result<Event, ConnectionError>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let result = match self.0.poll_for_event() {
            Ok(event) => {
                if event.is_none() {
                    return Poll::Pending;
                }

                let inner = unsafe { event.unwrap_unchecked() };
                Ok(inner)
            }
            Err(e) => Err(e.into()),
        };

        cx.waker().clone().wake();
        Poll::Ready(Some(result))
    }
}
