/// record keycode combinations, when a key pressed X11 doesn't
/// give all the keys that were pressed, we need to remember them, thus
/// the record functionality
///
use std::fmt;
use xkbcommon::xkb;

#[derive(Debug)]
#[repr(transparent)]
pub struct ComboSnapshot(Vec<xkb::Keycode>);

impl fmt::Display for ComboSnapshot {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let combo_string = self
            .0
            .iter()
            .map(|k| k.raw().to_string())
            .collect::<Vec<String>>()
            .join("+");
        write!(fmt, "{}", combo_string)
    }
}

impl IntoIterator for ComboSnapshot {
    type Item = xkb::Keycode;
    type IntoIter = <Vec<xkb::Keycode> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct ComboRecord(Vec<xkb::Keycode>);

impl ComboRecord {
    /// pushes the given keycode to the end of the combo
    /// since this the given keycode is
    pub fn add(&mut self, keycode: xkb::Keycode) {
        let keycode = keycode.into();
        if !self.0.contains(&keycode) {
            self.0.push(keycode);
        }
    }

    /// remove the given keycode from the combo, if the keycode somehow
    /// does not exists nothing will be done
    pub fn remove(&mut self, keycode: xkb::Keycode) {
        let keycode = keycode.into();
        let index = self.0.iter().enumerate().find_map(|(i, k)| {
            if *k == keycode {
                return Some(i);
            }
            None
        });

        if let Some(index) = index {
            if index == 0 {
                self.0.clear();
            } else {
                self.0.remove(index);
            }
        }
    }

    /// makes a clone of the current combo and returns it represented
    /// in a ComboSnapshot object
    pub fn snapshot(&self) -> ComboSnapshot {
        ComboSnapshot(self.0.clone())
    }
}
