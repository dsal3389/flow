use std::sync::Arc;
use std::collections::HashMap;
use xkbcommon::xkb;

use super::handlers::ComboHandler;

#[derive(Default)]
struct Combo {
    entries: HashMap<xkb::Keycode, Combo>,
    /// the handler for the current combination, it is inside
    /// an `Arc` so it can be returned indipendently of the lifetime
    /// of the current Combo
    handler: Option<Arc<dyn ComboHandler>>,
}

impl Combo {
    fn add<T>(&mut self, combo: &[T], handler: Arc<dyn ComboHandler>)
    where
        T: Into<xkb::Keycode> + Clone,
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

    /// drills down the `entries` to the last `Combo`, when last combo is reached
    /// the iterator will be empty and the combo should return its handler
    fn find<I, T>(&self, mut combo: I) -> Option<Arc<dyn ComboHandler>>
    where
        I: Iterator<Item = T>,
        T: Into<xkb::Keycode>,
    {
        match combo.next() {
            Some(keycode) => self
                .entries
                .get(&keycode.into())
                .and_then(|bind| bind.find(combo)),
            None => self.handler.as_ref().map(|handler| Arc::clone(handler)),
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
    pub fn add_combo<T>(&mut self, combo: &[T], handler: Arc<dyn ComboHandler>)
    where
        T: Into<xkb::Keycode> + Clone,
    {
        self.root.add(combo, handler)
    }

    /// returns the handler for the provided combo, if `None` is returned
    /// it means that the given combination wasn't registered
    #[inline]
    pub fn find_combo_handler<I, T>(&self, combo: I) -> Option<Arc<dyn ComboHandler>>
    where
        I: IntoIterator<Item = T>,
        T: Into<xkb::Keycode>,
    {
        self.root.find(combo.into_iter())
    }

    /// clears all registered combinations in the bind tree
    #[inline]
    pub fn clear(&mut self) {
        self.root.entries.drain();
    }
}
