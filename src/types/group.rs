use std::{sync::Arc, collections::BTreeSet};

use crate::PhonemeKey;

slotmap::new_key_type! { pub struct GroupKey; }

#[derive(serde::Deserialize, serde::Serialize)]
pub enum GroupName {
    Full { name: Arc<str>, abbrev: char },
    Abbrev(char),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Group {
    pub name: GroupName,
    pub keys: BTreeSet<PhonemeKey>,
}