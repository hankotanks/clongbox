use std::sync::Arc;

slotmap::new_key_type! { pub struct GroupKey; }

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Group {
    Full { name: Arc<str>, abbrev: char },
    Abbrev(char),
}