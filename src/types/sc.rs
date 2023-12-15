use std::{io, ops, sync};

use slotmap::{SlotMap, SecondaryMap};

use crate::{Phoneme, PhonemeKey, GroupKey, GroupName};
use crate::language::Language;
use crate::language::PhonemeRef;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Element {
    Phoneme { key: PhonemeKey, rep: bool },
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
    EnvStart { has_boundary: bool },
    EnvEnd { has_boundary: bool },
}

impl From<Field> for usize {
    fn from(value: Field) -> Self {
        unsafe { *(&value as *const Field as *const usize) }
    }
}

impl From<usize> for Field {
    fn from(value: usize) -> Self {
        for (idx, field) in enum_iterator::all::<Field>().enumerate() {
            if idx == value { return field; }
        }

        panic!();
    }
}

impl Field {
    fn parse(
        &mut self,
        language: &Language,
        rep_phonemes: &mut SlotMap<PhonemeKey, Phoneme>,
        rep_phoneme_usages: &mut slotmap::SecondaryMap<PhonemeKey, usize>,
        rewrite_rules: &bimap::BiHashMap<&str, &str>,
        elements: &mut Vec<Element>,
        raw: &str,
        nested: bool,
    ) -> Result<(), io::Error> {
        let (mut idx, mut idx_prev) = (0, std::usize::MAX);
        while idx != idx_prev {
            let idx_temp = idx;

            for group in language.groups() {
                let start_match_len = match group.name {
                    GroupName::Full { name, .. } //
                        if raw[idx..].starts_with(name.as_ref()) => name.len(),
                    GroupName::Full { abbrev, .. } => {
                        match raw[idx..].chars().next() {
                            Some(head) if head == *abbrev => 1,
                            _ => 0,
                        }
                    },
                    GroupName::Abbrev(abbrev) => {
                        match raw[idx..].chars().next() {
                            Some(head) if head == *abbrev => 1,
                            _ => 0,
                        }
                    },
                };

                if start_match_len != 0 {
                    elements.push(Element::Group(group.key));

                    idx += start_match_len;
                } else {
                    for phoneme in group.phonemes {
                        let PhonemeRef { key, phoneme, .. } = phoneme;

                        if raw[idx..].starts_with(phoneme.as_ref()) {
                            elements.push(Element::Phoneme { key, rep: false });

                            idx += phoneme.len();
                        }
                    }
                }
            }

            match (raw[idx..].chars().next(), idx == idx_temp) {
                (Some('['), true) => {
                    if nested {
                        return Err(io::Error::from(io::ErrorKind::InvalidData));
                    }

                    let raw_any = regex::Regex::new(r"(?:\[)(.*)(?:\])")
                        .unwrap()
                        .captures(&raw[idx..])
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .as_str();

                    let raw_any = {
                        let mut temp = raw_any.chars();

                        temp.next();
                        temp.next_back();
                        temp.as_str()
                    };

                    let mut sub = Vec::default();

                    self.parse(
                        language, 
                        rep_phonemes, 
                        rep_phoneme_usages, 
                        rewrite_rules, 
                        &mut sub, 
                        raw_any,
                        true
                    )?;

                    elements.push(Element::Any(sub));

                    idx += raw_any.len() + 2;
                },
                (Some('#'), true) => {
                    // TODO: This should involve checking to see 
                    // if the boundary is in a valid position
                    // EVERY rule applied to sound change creation in the UI 
                    // should be reflected here
                    let invalid = match self {
                        Field::EnvStart { has_boundary } if !*has_boundary => {
                            *has_boundary = true;

                            false
                        },
                        Field::EnvEnd { has_boundary } if !*has_boundary => {
                            *has_boundary = true;

                            false
                        },
                        _ => true,
                    };

                    if invalid {
                        return Err(io::Error::from(io::ErrorKind::InvalidData));
                    }

                    elements.push(Element::Boundary);

                    idx += 1;
                },
                _ => { /*  */ },
            }

            if idx == idx_temp {
                let mut rep_phoneme = None;

                'rep: for (from, _) in rewrite_rules.iter() {
                    let a = language.phoneme_table
                        .iter()
                        .map(|p| p.phoneme.as_ref())
                        .collect::<Vec<_>>()
                        .contains(from);

                    let b = language
                        .groups()
                        .fold(false, |acc, g| {
                            acc | g.name.matches(&raw[idx..])
                        });

                    // If it isn't a known phoneme or a group name/abbrev
                    if !(a || b || raw[idx..].is_empty()) {
                        if raw[idx..].starts_with(from) {
                            idx += from.len();
    
                            let _ = rep_phoneme.insert(&raw[idx_temp..idx]);
    
                            break 'rep;
                        }
                    }
                }
    
                if rep_phoneme.is_none() {
                    let mut temp = raw[idx..].char_indices().peekable();
    
                    rep_phoneme = match temp.next() {
                        Some((idx_next, _)) => match temp.peek() {
                            Some((idx_final, _)) => {
                                idx += idx_final - idx_next;
    
                                Some(&raw[idx_temp..idx])
                            },
                            None => {
                                idx = raw.len();
    
                                Some(&raw[idx_temp..])
                            },
                        },
                        None => None,
                    };
                }
    
                if let Some(rep_phoneme) = rep_phoneme {
                    let rep_phoneme = Phoneme {
                        phoneme: sync::Arc::from(rep_phoneme),
                        grapheme: None,
                    };

                    let key = rep_phonemes.insert(rep_phoneme);
    
                    if let Some(usages) = rep_phoneme_usages.get_mut(key) {
                        *usages += 1;
                    } else {
                        rep_phoneme_usages.insert(key, 1);
                    }

                    elements.push(Element::Phoneme { key, rep: true });
                }
            }

            idx_prev = idx_temp;
        }

        Ok(())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SoundChange([Vec<Element>; 4]);

impl ops::Index<Field> for SoundChange {
    type Output = Vec<Element>;

    fn index(&self, index: Field) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl ops::IndexMut<Field> for SoundChange {
    fn index_mut(&mut self, index: Field) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}

impl SoundChange {
    #[allow(unused_variables)]
    pub fn parse(
        language: &Language, 
        rep_phonemes: &mut SlotMap<PhonemeKey, Phoneme>, 
        rep_phoneme_usages: &mut SecondaryMap<PhonemeKey, usize>,
        rewrite_rules: &bimap::BiHashMap<&str, &str>, 
        raw: &str
    ) -> Result<SoundChange, io::Error> {

        let re = regex::Regex::new(r"(\S*)(?:/)(\S*)(?:/)(\S*)(?:_)(\S*)").unwrap();

        if let Some(capture) = re.captures(raw) {
            let capture: [&str; 4] = capture.extract().1;

            let mut sound_change: [Vec<Element>; 4] = [(); 4]
                .map(|_| Vec::default());

            for idx in 0..4 {
                let elements = &mut sound_change[idx];

                Field::from(idx).parse(
                    language,
                    rep_phonemes,
                    rep_phoneme_usages,
                    rewrite_rules,
                    elements,
                    capture[idx],
                    false
                )?;
            }

            Ok(SoundChange(sound_change))
        } else {
            Err(io::Error::from(io::ErrorKind::InvalidData))
        }
    }
}