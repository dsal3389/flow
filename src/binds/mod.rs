use std::collections::HashMap;

use async_trait::async_trait;
use xkbcommon::xkb;

mod spawn;

pub use spawn::SpawnHandler;

/// a trait that is implemented on types that can be used
/// as handlers for key combo
#[async_trait]
pub trait BindHandler: Send + Sync {
    async fn handle(&self) -> anyhow::Result<()>;
}

#[derive(Default)]
struct KeyBind {
    entries: HashMap<xkb::Keycode, KeyBind>,
    handler: Option<Box<dyn BindHandler>>,
}

impl KeyBind {
    fn add(&mut self, combo: &[xkb::Keycode], handler: Box<dyn BindHandler>) {
        match combo.first() {
            Some(key) => self
                .entries
                .entry(key.clone())
                .or_default()
                .add(&combo[1..], handler),
            // if there is not next char in the combo, it means the current
            // keybind is the last and the handler should be attached to here
            None => {
                self.handler = Some(handler);
            }
        };
    }
}

#[derive(Default)]
pub struct BindsTree {
    root: KeyBind,
}

impl BindsTree {
    #[inline]
    pub fn add_combo(&mut self, combo: &[xkb::Keycode], handler: Box<dyn BindHandler>) {
        self.root.add(combo, handler)
    }
}
