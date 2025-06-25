use xkbcommon::xkb;

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct ComboRecord(Vec<xkb::Keycode>);

impl ComboRecord {
    pub fn press<T>(&mut self, keycode: T)
    where
        T: Into<xkb::Keycode>
    {
        let keycode = keycode.into();
        if !self.0.contains(&keycode) {
            self.0.push(keycode);
        }
    }

    pub fn release<T>(&mut self, keycode: T)
    where
        T: Into<xkb::Keycode>
    {
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

    pub fn combo(&self) -> &[xkb::Keycode] {
        &self.0
    }
}
