use std::collections::HashMap;
use xkbcommon::xkb;

use super::handlers::ComboHandler;


#[derive(Default)]
struct Combo {
    entries: HashMap<xkb::Keycode, Combo>,
    handler: Option<Box<dyn ComboHandler>>,
}

impl Combo {
    fn add<T>(&mut self, combo: &[T], handler: Box<dyn ComboHandler>)
    where
        T: Into<xkb::Keycode> + Clone
    {
        match combo.first().cloned() {
            Some(keycode) => self
                .entries
                .entry(keycode.clone().into())
                .or_default()
                .add(&combo[1..], handler),
            // if there is not next char in the combo, it means the current
            // keybind is the last and the handler should be attached to here
            None => {
                self.handler = Some(handler);
            }
        };
    }

    fn find<T>(&self, combo: &[T]) -> Option<&dyn ComboHandler>
    where
        T: Into<xkb::Keycode> + Clone
    {
        match combo.first().cloned() {
            Some(keycode) => {
                self.entries.get(&keycode.into())
                    .and_then(|bind| bind.find(&combo[1..]))
            },
            None => self.handler.as_ref().map(|handler| &**handler)
        }
    }
}

/// the `BindsTree` type holds key combo bind information, since x11
/// use keycodes to signal what key was pressed, `BindsTree` also uses keycodes
/// for combinations, if key we want
#[derive(Default)]
pub struct ComboTree {
    root: Combo,
}

impl ComboTree {
    /// takes a combination of keycode arguments with the handler that should be called
    /// when the combination performed
    #[inline]
    pub fn add_combo<T>(&mut self, combo: &[T], handler: Box<dyn ComboHandler>)
    where
        T: Into<xkb::Keycode> + Clone
    {
        self.root.add(combo, handler)
    }

    /// returns the handler for the provided combo, if `None` is returned
    /// it means that the given combination wasn't registered
    #[inline]
    pub fn find_combo_handler<T>(&self, combo: &[T]) -> Option<&dyn ComboHandler>
    where
        T: Into<xkb::Keycode> + Clone
    {
        self.root.find(combo)
    }

    /// clears all registered combinations in the bind tree
    #[inline]
    pub fn clear(&mut self) {
        self.root.entries.drain();
    }
}
