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

impl Syllable { 
    pub fn len(&self) -> usize {
        self.elems.len()
    } 

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
}

pub struct SyllableRefMut<'a> {
    pub idx: usize,
    pub syllable: &'a mut Syllable,
    pub language: &'a Language,
}