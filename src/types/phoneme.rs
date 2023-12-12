use std::sync::Arc;

slotmap::new_key_type! { pub struct PhonemeKey; }

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Phoneme {
    phoneme: Arc<str>,
    grapheme: Option<Arc<str>>,
}