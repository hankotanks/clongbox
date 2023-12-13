use std::collections::BTreeSet;
use std::{ops, mem, vec, fmt};
use std::sync::Arc;

use slotmap::SlotMap;
use slotmap::basic::Keys;

use crate::{Phoneme, PhonemeKey, GroupName};
use crate::{Group, GroupKey};

pub struct Language {
    phonemes: SlotMap<PhonemeKey, Phoneme>,
    groups: SlotMap<GroupKey, Group>,
}

impl ops::Index<PhonemeKey> for Language {
    type Output = Phoneme;

    fn index(&self, index: PhonemeKey) -> &Self::Output {
        &self.phonemes[index]
    }
}

impl ops::IndexMut<PhonemeKey> for Language {
    fn index_mut(&mut self, index: PhonemeKey) -> &mut Self::Output {
        &mut self.phonemes[index]
    }
}

impl ops::Index<GroupKey> for Language {
    type Output = Group;

    fn index(&self, index: GroupKey) -> &Self::Output {
        &self.groups[index]
    }
}

impl ops::IndexMut<GroupKey> for Language {
    fn index_mut(&mut self, index: GroupKey) -> &mut Self::Output {
        &mut self.groups[index]
    }
}

impl Language {
    pub fn phoneme_add(&mut self, phoneme: Phoneme) -> PhonemeKey {
        self.phonemes.insert(phoneme)
    }

    pub fn phoneme_remove(&mut self, key: PhonemeKey) {
        for (_, group) in self.groups.iter_mut() {
            group.keys.remove(&key);
        }

        self.phonemes.remove(key);
    }

    pub fn group_add(&mut self, name: GroupName) -> GroupKey {
        let group = Group {
            name,
            keys: BTreeSet::default(),
        };

        self.groups.insert(group)
    }

    pub fn group_remove(&mut self, key: GroupKey) {
        self.groups.remove(key);
    }
}

pub struct PhonemeRef<'a> {
    pub key: PhonemeKey,
    pub phoneme: &'a Arc<str>,
    pub grapheme: Option<&'a Arc<str>>,
}

impl<'a> fmt::Display for PhonemeRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let phoneme = Phoneme {
            phoneme: self.phoneme.clone(),
            grapheme: self.grapheme.cloned(),
        };

        write!(f, "{}", phoneme)
    }
}

impl<'a> PhonemeRef<'a> {
    pub fn to_owned_phoneme(&self) -> Phoneme {
        Phoneme {
            phoneme: self.phoneme.clone(),
            grapheme: self.grapheme.cloned(),
        }
    }
}

pub struct Phonemes<'a> {
    idx: usize,
    keys: Vec<PhonemeKey>,
    source: &'a SlotMap<PhonemeKey, Phoneme>,
}

impl<'a> From<&'a SlotMap<PhonemeKey, Phoneme>> for Phonemes<'a> {
    fn from(value: &'a SlotMap<PhonemeKey, Phoneme>) -> Self {
        Self {
            idx: 0,
            keys: value.keys().collect(),
            source: value,
        }
    }
}

impl<'a> Iterator for Phonemes<'a> {
    type Item = PhonemeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.keys.len() { return None; }

        let key = self.keys[self.idx];

        let Phoneme { phoneme, grapheme } = &self.source[key];

        self.idx += 1;

        let phoneme_ref = PhonemeRef {
            key,
            phoneme,
            grapheme: grapheme.as_ref(),
        };

        Some(phoneme_ref)
    }
}

pub struct PhonemeRefMut<'a> {
    pub key: PhonemeKey,
    pub phoneme: &'a mut Arc<str>,
    pub grapheme: &'a mut Option<Arc<str>>,
    pub rm: &'a mut bool,
}

impl<'a> fmt::Display for PhonemeRefMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let phoneme = Phoneme {
            phoneme: self.phoneme.clone(),
            grapheme: self.grapheme.clone(),
        };

        write!(f, "{}", phoneme)
    }
}

impl<'a> PhonemeRefMut<'a> {
    pub fn to_owned_phoneme(&self) -> Phoneme {
        Phoneme {
            phoneme: self.phoneme.clone(),
            grapheme: self.grapheme.clone(),
        }
    }

    pub fn delete(&mut self) {
        *self.rm = true;
    }
}

pub struct PhonemesMut<'a> {
    idx: usize,
    keys: Vec<PhonemeKey>,
    rm: bool,
    source: Result<&'a mut Language, &'a mut SlotMap<PhonemeKey, Phoneme>>,
}

impl<'a> From<&'a mut SlotMap<PhonemeKey, Phoneme>> for PhonemesMut<'a> {
    fn from(value: &'a mut SlotMap<PhonemeKey, Phoneme>) -> Self {
        Self {
            idx: 0,
            keys: value.keys().collect(),
            rm: false,
            source: Err(value),
        }
    }
}

impl<'a> Iterator for PhonemesMut<'a> {
    type Item = PhonemeRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rm {
            self.idx -= 1;

            self.rm = false;

            let key = self.keys[self.idx];

            match &mut self.source {
                Ok(language) => language.phoneme_remove(key),
                Err(phonemes) => {
                    phonemes.remove(key);
                },
            }
        }

        if self.idx == self.keys.len() { return None; }

        let key = self.keys[self.idx];

        let Phoneme { phoneme, grapheme } = match &mut self.source {
            Ok(language) => &mut language.phonemes[key],
            Err(phonemes) => &mut phonemes[key],
        };

        let phoneme_ref_mut = PhonemeRefMut {
            key,
            phoneme,
            grapheme,
            rm: &mut self.rm,
        };

        let phoneme_ref_mut = unsafe {
            mem::transmute::<PhonemeRefMut<'_>, PhonemeRefMut<'a>>(phoneme_ref_mut)
        };

        self.idx += 1;

        Some(phoneme_ref_mut)
    }
}

pub struct GroupRef<'a> {
    pub key: GroupKey,
    pub name: &'a GroupName,
    pub phonemes: Phonemes<'a>,
}

pub struct Groups<'a, Id: Iterator<Item = GroupKey>> {
    keys: Id,
    language: &'a Language,
}

impl<'a, Id: Iterator<Item = GroupKey>> Iterator for Groups<'a, Id> {
    type Item = GroupRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|key| {
            let Group { name, keys } = &self.language[key];

            let group_ref = GroupRef {
                key, name,
                phonemes: Phonemes {
                    idx: 0,
                    keys: keys.iter().copied().collect(),
                    source: &self.language.phonemes,
                },
            };

            group_ref
        })
    }
}

pub struct GroupRefMut<'a> {
    pub key: GroupKey,
    pub name: &'a mut GroupName,
    pub phonemes: PhonemesMut<'a>,
}

pub struct GroupsMut<'a, Id: Iterator<Item = GroupKey>> {
    keys: Id,
    language: &'a mut Language,
}

impl<'a, Id: Iterator<Item = GroupKey>> Iterator for GroupsMut<'a, Id> {
    type Item = GroupRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|key| {
            let Group { name, keys } = &mut self.language.groups[key];

            let group_ref_mut = GroupRefMut {
                key,
                name,
                phonemes: PhonemesMut {
                    idx: 0,
                    keys: keys.iter().copied().collect(),
                    rm: false,
                    source: Err(&mut self.language.phonemes),
                },
            };

            let group_ref_mut = unsafe {
                mem::transmute::<GroupRefMut<'_>, GroupRefMut<'a>>(group_ref_mut)
            };

            group_ref_mut
        })
    }
}

impl Language {
    pub fn phonemes(&self, key: GroupKey) -> Phonemes<'_> {
        Phonemes {
            idx: 0,
            keys: self.groups[key].keys.iter().copied().collect(),
            source: &self.phonemes,
        }
    }

    pub fn phonemes_mut(&mut self, key: GroupKey) -> PhonemesMut<'_> {
        PhonemesMut {
            idx: 0,
            keys: self.groups[key].keys.iter().copied().collect(),
            rm: false,
            source: Ok(self),
        }
    }

    pub fn groups(&self) -> Groups<'_, Keys<'_, GroupKey, Group>> {
        Groups {
            keys: self.groups.keys(),
            language: self,
        }
    }

    pub fn groups_mut(&mut self) -> GroupsMut<'_, vec::IntoIter<GroupKey>> {
        let keys = self.groups.keys().collect::<Vec<_>>().into_iter();

        GroupsMut { 
            keys, 
            language: self, 
        }
    }
}