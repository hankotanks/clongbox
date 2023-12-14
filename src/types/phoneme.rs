use std::{sync::Arc, fmt, borrow, io};

slotmap::new_key_type! { pub struct PhonemeKey; }

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Phoneme {
    pub phoneme: Arc<str>,
    pub grapheme: Option<Arc<str>>,
}

impl Phoneme {
    pub fn parse<'a, I>(content: I) -> Result<Self, io::Error>
        where I: Into<borrow::Cow<'a, str>> {
        
        let content: borrow::Cow<'_, str> = content.into();
        let content = content.as_ref();

        regex::Regex::new(r"([^\s\[\]]+)(\s*\[\s*\S+\s*\])?")
            .unwrap()
            .captures(content.trim())
            .ok_or(io::Error::from(io::ErrorKind::InvalidData))
            .map(|capture| {
                Self {
                    phoneme: Arc::from(capture.get(0).unwrap().as_str()),
                    grapheme: capture
                        .get(1)
                        .map(|grapheme| grapheme.as_str())
                        .map(Arc::from),
                }
            })
    }
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

#[derive(Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum PhonemeSrc { Language, Rep }