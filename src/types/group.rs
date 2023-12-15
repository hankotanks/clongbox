use std::sync::Arc;

use std::collections::BTreeSet;

use crate::PhonemeKey;

slotmap::new_key_type! { pub struct GroupKey; }

#[derive(serde::Deserialize, serde::Serialize)]
pub enum GroupName {
    Full { name: Arc<str>, abbrev: char },
    Abbrev(char),
}

impl GroupName {
    pub fn matches(&self, other: &str) -> bool {
        match self {
            GroupName::Full { name, abbrev } 
                if (other.starts_with(name.as_ref()) || //
                    other.starts_with(*abbrev)) => true,
            GroupName::Abbrev(abbrev) if other.starts_with(*abbrev) => true,
            _ => false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Group {
    pub name: GroupName,
    pub keys: BTreeSet<PhonemeKey>,
}