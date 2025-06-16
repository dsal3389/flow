use x11rb_async::connection::Connection;
use x11rb_async::protocol::xkb::ConnectionExt as ConnectionXkb;
use x11rb_async::protocol::xproto::{self, ConnectionExt as ConnectionXproto};
use xkbcommon::xkb;

pub(crate) struct KeyState {
    min_keycode: u8,
    max_keycode: u8,
    keysyms_per_keycode: u8,
    keysyms: Vec<u32>,
}

impl KeyState {
    pub(crate) fn new(
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

    pub(crate) async fn from_connection<C>(
        connection: &C
    ) -> anyhow::Result<Self>
    where
        C: Connection + ConnectionXproto + ConnectionXkb
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
    pub(crate) fn keysym_to_keycode(&self, keysym: xkb::Keysym) -> Option<xkb::Keycode> {
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
pub(crate) struct Key(char);

impl Key {
    /// returns Keysym from the current char
    #[inline]
    pub(crate) fn keysym(&self) -> xkb::Keysym {
        xkb::utf32_to_keysym(self.0 as u32)
    }

    /// returns the Keycode for the current char, returns None if
    /// couldn't find Keycode for current key in given KeyState
    #[inline]
    pub(crate) fn keycode(&self, state: &KeyState) -> Option<xkb::Keycode> {
        let keysym = self.keysym();
        state.keysym_to_keycode(keysym)
    }

    pub(crate) fn from_keycode(keycode: xkb::Keycode, state: &KeyState) -> Key {
        todo!()
    }
}

impl From<char> for Key {
    fn from(value: char) -> Key {
        Key(value)
    }
}
