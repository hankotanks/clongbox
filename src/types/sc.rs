use std::io;

use bimap::BiHashMap;
use slotmap::{SlotMap, SecondaryMap};

use crate::{PhonemeKey, GroupKey, language::Language, Phoneme};

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Element {
    Phoneme(PhonemeKey),
    Group(GroupKey),
    Boundary,
    Any(Vec<Element>),
}

#[derive(Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(enum_map::Enum, enum_iterator::Sequence)]
pub enum Field {
    Target,
    Replacement,
    EnvStart,
    EnvEnd,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SoundChange(enum_map::EnumMap<Field, Vec<Element>>);

impl SoundChange {
    #[allow(unused_variables)]
    pub fn parse(
        language: &Language, 
        rep_phonemes: &mut SlotMap<PhonemeKey, Phoneme>, 
        rep_phoneme_usages: &mut SecondaryMap<PhonemeKey, usize>,
        rewrite_rules: &BiHashMap<&str, &str>, 
        raw: &str
    ) -> Result<SoundChange, io::Error> {
        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}