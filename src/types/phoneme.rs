use std::{sync::Arc, fmt};

slotmap::new_key_type! { pub struct PhonemeKey; }

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Phoneme {
    pub phoneme: Arc<str>,
    pub grapheme: Option<Arc<str>>,
}

impl fmt::Display for Phoneme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.grapheme {
            Some(grapheme) => {
                write!(f, "{} [{}]", self.phoneme.as_ref(), grapheme.as_ref())
            },
            None => write!(f, "{}", self.phoneme.as_ref()),
        }
    }
}