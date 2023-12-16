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
    pub fn matches(&self, other: &str) -> bool {
        match self {
            GroupName::Full { name, abbrev } 
                if (other.starts_with(name.as_ref()) || //
                    other.starts_with(*abbrev)) => true,
            GroupName::Abbrev(abbrev) if other.starts_with(*abbrev) => true,
            _ => false,
        }
    }

    pub fn parse<'a, I>(_content: I) -> Result<Self, io::Error>
        where I: Into<borrow::Cow<'a, str>> {

        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Group {
    pub name: GroupName,
    pub keys: BTreeSet<PhonemeKey>,
}