use x11rb_async::connection::Connection;
use x11rb_async::protocol::xproto::{self, ConnectionExt};
use xkbcommon::xkb;

pub struct KeyState {
    min_keycode: u8,
    max_keycode: u8,
    keysyms_per_keycode: u8,
    keysyms: Vec<u32>,
}

impl KeyState {
    pub fn new(
        min_keycode: u8,
        max_keycode: u8,
        keysyms_per_keycode: u8,
        keysyms: Vec<u32>,
    ) -> KeyState {
        KeyState {
            min_keycode,
            max_keycode,
            keysyms_per_keycode,
            keysyms,
        }
    }

    /// creates a keystate instance from given conneection, will
    /// perform the required requests
    pub async fn from_connection<C>(connection: &C) -> anyhow::Result<Self>
    where
        C: Connection,
    {
        let &xproto::Setup {
            min_keycode,
            max_keycode,
            ..
        } = connection.setup();
        let xproto::GetKeyboardMappingReply {
            keysyms_per_keycode,
            keysyms,
            ..
        } = connection
            .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)
            .await?
            .reply()
            .await?;

        Ok(KeyState::new(
            min_keycode,
            max_keycode,
            keysyms_per_keycode,
            keysyms,
        ))
    }

    /// takes Keysym and returns the equivelent Keycode
    /// based on the provided keysyms map
    pub fn keysym_to_keycode(&self, keysym: xkb::Keysym) -> Option<xkb::Keycode> {
        self.keysyms
            .chunks(self.keysyms_per_keycode as usize)
            .enumerate()
            .find_map(|(i, syms)| {
                syms.contains(&keysym.raw())
                    .then_some(xkb::Keycode::new(self.min_keycode as u32 + i as u32))
            })
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Key(char);

impl Key {
    /// returns Keysym representation for the current key
    #[inline]
    pub fn keysym(&self) -> xkb::Keysym {
        xkb::utf32_to_keysym(self.0 as u32)
    }

    /// returns the Keycode for the current char, returns None if
    /// couldn't find Keycode for current key in given KeyState
    #[inline]
    pub fn keycode(&self, state: &KeyState) -> Option<xkb::Keycode> {
        let keysym = self.keysym();
        state.keysym_to_keycode(keysym)
    }
}

impl From<char> for Key {
    fn from(value: char) -> Key {
        Key(value)
    }
}

#[derive(Debug)]
pub(crate) struct Keyboard;
