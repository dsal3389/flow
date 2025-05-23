use std::fmt;

use flow_core::Key;

pub struct Keybind {
    key: Key,
}

impl Keybind {
    pub fn new<K>(key: K) -> Keybind
    where
        K: Into<Key>,
    {
        Keybind { key: key.into() }
    }
}

impl fmt::Display for Keybind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` -> todo", self.key)
    }
}

pub struct ProfileContext {
    keybinds: Vec<Keybind>,
}

impl ProfileContext {
    // TODO: remove, this is here just for tests
    pub fn new(keybinds: Vec<Keybind>) -> ProfileContext {
        ProfileContext { keybinds }
    }

    /// returns an iterator for configured keybindings
    pub fn keybinds(&self) -> impl Iterator<Item = &Keybind> {
        self.keybinds.iter()
    }
}
