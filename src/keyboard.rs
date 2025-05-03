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

enum KeyEvent {
    Press,
    Release,
}

pub struct Key {
    c: char,
    time: u32,
    event: KeyEvent,
}

impl Key {
    fn new(c: char, time: u32, event: KeyEvent) -> Key {
        Key { c, time, event }
    }
}

impl TryFrom<Event> for Key {
    type Error = KeyError;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        let key = match value {
            Event::ButtonPress(event) => {
                Key::new(char::from(event.detail), event.time, KeyEvent::Press)
            }
            Event::ButtonRelease(event) => {
                Key::new(char::from(event.detail), event.time, KeyEvent::Release)
            }
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
