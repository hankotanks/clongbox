use std::{fmt, io, borrow};
use std::sync::Arc;

use std::collections::BTreeSet;

use crate::PhonemeKey;

slotmap::new_key_type! { pub struct GroupKey; }

#[derive(serde::Deserialize, serde::Serialize)]
pub enum GroupName {
    Full { name: Arc<str>, abbrev: char },
    Abbrev(char),
}

impl fmt::Display for GroupName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupName::Full { name, abbrev } => //
                write!(f, "{} ({})", name.as_ref(), abbrev),
            GroupName::Abbrev(abbrev) => write!(f, "{}", abbrev),
        }
    }
}

impl GroupName {
    pub fn abbrev(&self) -> char {
        match self {
            GroupName::Full { abbrev, .. } | //
            GroupName::Abbrev(abbrev) => *abbrev,
        }
    }
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

    pub fn parse<'a, I>(content: I) -> Result<Self, io::Error>
        where I: Into<borrow::Cow<'a, str>> {

        let content: borrow::Cow<'_, str> = content.into();
        let content = content.as_ref();

        let parse_result = regex::Regex::new(r"([^\s\[\]]+)(\s*\([^\s\d]\))?")
            .unwrap()
            .captures(content.trim())
            .ok_or(io::Error::from(io::ErrorKind::InvalidData))
            .map(|capture| {
                let fst = capture.get(1).unwrap().as_str().trim();
                let fst: Arc<str> = Arc::from(fst);

                let snd = capture
                    .get(2)
                    .map(|abbrev| abbrev
                        .as_str()
                        .trim()[1..(abbrev.as_str().len() - 2)]
                        .trim()
                        .chars()
                        .next()
                    );

                match snd.flatten() {
                    Some(abbrev) => {
                        Some(Self::Full { 
                            name: fst,
                            abbrev,
                        })
                    },
                    None if fst.len() == 1 => {
                        Some(Self::Abbrev(fst.chars().next().unwrap()))
                    },
                    _ => None,
                }
            });

        match parse_result {
            Ok(Some(name)) => Ok(name),
            Ok(None) => Err(io::Error::from(io::ErrorKind::InvalidData)),
            Err(err) => Err(err),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Group {
    pub name: GroupName,
    pub keys: BTreeSet<PhonemeKey>,
}