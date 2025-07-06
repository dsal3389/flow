/// record keycode combinations, when a key pressed X11 doesn't
/// give all the keys that were pressed, we need to remember them, thus
/// the record functionality
///
use std::fmt;
use xkbcommon::xkb;

/// user pressed combination can change very fast
/// and we want to take a snapshot of the current pressed combinations
/// to do stuff with it, but we don't want to block the user from pressing more
/// combination while we handle a specific one
///
/// so snapshot takes the current pressed combinations state and
/// allows us to handle snapped combination while the user continue
/// with different combinations
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

#[derive(Debug)]
#[repr(transparent)]
pub struct ComboRecord(Vec<xkb::Keycode>);

impl ComboRecord {
    /// pushes the given keycode to the end of the combo
    /// since this the given keycode is
    pub fn add(&mut self, keycode: xkb::Keycode) {
        if !self.0.contains(&keycode) {
            self.0.push(keycode);
        }
    }

    /// remove the given keycode from the combo, if the keycode somehow
    /// does not exists nothing will be done
    pub fn remove(&mut self, keycode: xkb::Keycode) {
        let index = self.0.iter().enumerate().find_map(|(i, k)| {
            if *k == keycode {
                return Some(i);
            }
            None
        });

        if let Some(index) = index {
            if index == 0 {
                // if the first key needs to be deleted (the root key)
                // then all the tree should be deleted because a combination must start
                // with the root key and X11 won't send keyrelease event for the other keys
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

impl Default for ComboRecord {
    fn default() -> Self {
        Self(Vec::with_capacity(8))
    }
}
