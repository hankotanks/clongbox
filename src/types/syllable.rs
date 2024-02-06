use std::sync;

use crate::{language::Language, GroupKey, PhonemeKey};

#[derive(Clone, Copy)]
pub enum SyllabicElement {
    Phoneme(PhonemeKey),
    Group(GroupKey),
}

pub struct Syllable(Vec<SyllabicElement>);

pub struct Phonotactics(Vec<Syllable>);

pub struct GenConfig {
    mono: f32,
    dropoff: f32,
}