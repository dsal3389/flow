use bitflags::bitflags;
use thiserror::Error;
use x11rb_async::protocol::Event;

#[derive(Error, Debug)]
#[error("{}", self.msg)]
pub struct KeyError {
    msg: String,
    event: Event,
}

enum KeyModifier {
    SHIFT = 0x1,
    ALT = 0x2,
}

pub struct Key {
    c: char,
}

pub enum KeyEvent {
    Press(Key),
    Release(Key),
}

impl TryFrom<Event> for KeyEvent {
    type Error = KeyError;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        let key = match value {
            Event::ButtonPress(event) => KeyEvent::Press(Key {
                c: event.detail.into(),
            }),
            Event::ButtonRelease(event) => KeyEvent::Release(Key {
                c: event.detail.into(),
            }),
            _ => {
                return Err(KeyError {
                    msg: format!("unknown key event {:?}", value),
                    event: value,
                });
            }
        };
        Ok(key)
    }
}

#[derive(Default)]
pub struct Keyboard {
    active_modifiers: u32,
}
