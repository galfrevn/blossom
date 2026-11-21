use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Numbers {
    names: Vec<Box<str>>,
    numbers: HashMap<Box<str>, u32>,
}

impl Numbers {
    pub(super) fn number_of(&mut self, name: &str) -> u32 {
        if let Some(number) = self.numbers.get(name) {
            return *number;
        }
        let number = u32::try_from(self.names.len()).unwrap_or(u32::MAX);
        self.names.push(name.into());
        self.numbers.insert(name.into(), number);
        number
    }

    pub(super) fn within(&self, channel: &str) -> Vec<bool> {
        let subtree = format!("{channel}/");
        self.names
            .iter()
            .map(|name| &**name == channel || name.starts_with(&subtree))
            .collect()
    }
}
