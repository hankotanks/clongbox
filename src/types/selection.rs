use std::collections::BTreeSet;

pub enum Selection<'a, K: slotmap::Key> {
    Single(&'a mut Option<K>),
    Multiple(&'a mut BTreeSet<K>),
    None
}

impl<'a, K: slotmap::Key> Selection<'a, K> {
    pub fn insert(&mut self, key: K) {
        match self {
            Selection::Single(key_slot) => {
                let _ = key_slot.insert(key);
            },
            Selection::Multiple(key_set) => {
                key_set.insert(key);
            },
            Selection::None => { /*  */ },
        }
    }

    pub fn remove(&mut self, key: K) {
        match self {
            Selection::Single(key_slot) if let Some(selected_key) = **key_slot => {
                if selected_key == key {
                    let _ = key_slot.take();
                }
            },
            Selection::Multiple(key_set) => {
                key_set.remove(&key);
            },
            _ => { /*  */ },
        }
    }

    pub fn is_selected(&self, key: K) -> bool {
        match self {
            Selection::Single(key_slot) //
                if let Some(selected_key) = key_slot => *selected_key == key,
            Selection::Single(_) => false,
            Selection::Multiple(key_set) => key_set.contains(&key),
            Selection::None => false,
        }
    }
}
