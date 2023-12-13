use crate::{PhonemeKey, GroupKey};

pub enum Element {
    Phoneme(PhonemeKey),
    Group(GroupKey),
    Boundary,
    Any(Vec<Element>),
}

#[derive(Clone, Copy, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(enum_map::Enum, enum_iterator::Sequence)]
pub enum Field {
    Target,
    Replacement,
    EnvStart,
    EnvEnd,
}

pub struct SoundChange(enum_map::EnumMap<Field, Vec<Element>>);