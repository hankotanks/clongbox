use crate::{language::Language, GroupKey, PhonemeKey};

#[derive(Clone, Copy)]
pub enum SyllabicElement {
    Phoneme(PhonemeKey),
    Group(GroupKey),
    Invalid,
}

#[derive(Default)]
pub struct Syllable {
    pub elems: Vec<SyllabicElement>
}

pub struct SyllableRefMut<'a> {
    pub syllable: &'a mut Syllable,
    pub language: &'a Language,
}