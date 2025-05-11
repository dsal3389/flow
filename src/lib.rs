mod connection;
mod keys;
mod wm;

/// basic public api to create connection and
/// a new window manager instance
pub use connection::XConnection;
pub use wm::WindowManager;

/// allow public access so outside modules can
/// insert key maps into the window manager
pub use keys::{Key, KeyAction, KeyMap};
