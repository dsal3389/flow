use flow_core::Key;

pub struct Keybind {
    key: Key,
}

impl Keybind {
    pub fn new<K>(key: K) -> Keybind
    where
        K: Into<Key>
    {
        Keybind { key: key.into() }
    }
}

pub struct Profile {
    keybinds: Vec<Keybind>
}

impl Profile {
    // TODO: remove, this is here just for tests
    pub fn new(keybinds: Vec<Keybind>) -> Profile {
        Profile { keybinds }
    }

    /// returns an iterator for configured keybindings
    pub fn keybinds(&self) -> impl Iterator<Item = &Keybind> {
        self.keybinds.iter()
    }
}
