use std::fmt;

use x11rb_async::errors::ReplyError;
use x11rb_async::connection::Connection;
use x11rb_async::protocol::xproto::{ConnectionExt, GetKeyboardMappingReply};
use xkbcommon::xkb;

pub struct KeyState {
    min_keycode: u8,
    max_keycode: u8,
    mapping: GetKeyboardMappingReply,
}

impl KeyState {
    pub async fn from_connection<C>(conn: &C) -> Result<Self, ReplyError>
    where
        C: Connection + ConnectionExt,
    {
        let min_keycode = conn.setup().min_keycode;
        let max_keycode = conn.setup().max_keycode;
        let mapping = conn
            .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)
            .await?
            .reply()
            .await?;
        Ok(KeyState {
            min_keycode,
            max_keycode,
            mapping,
        })
    }

    /// converts the given keysym to the keycode, if couldn't find a matching
    /// keycode, returns None
    pub fn keysym_to_keycode(&self, keysym: xkb::Keysym) -> Option<xkb::Keycode> {
        self.mapping
            .keysyms
            .chunks(self.mapping.keysyms_per_keycode as usize)
            .enumerate()
            .find_map(|(i, syms)| {
                syms.contains(&keysym.raw())
                    .then_some(xkb::Keycode::new(self.min_keycode as u32 + i as u32))
            })
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Key {
    c: char,
}

impl Key {
    /// returns the current key corrisponding keycode
    /// if couldn't find the keycode, returns None
    pub fn keycode(&self, state: &KeyState) -> Option<xkb::Keycode> {
        let keysym = xkb::utf32_to_keysym(self.c as u32);
        state.keysym_to_keycode(keysym)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.c)
    }
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        Key { c: value }
    }
}
