use std::{ops, sync, error, fmt};

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

impl Element {
    fn as_str(
        &self, 
        language: &Language, 
        rep_phonemes: &SlotMap<PhonemeKey, Phoneme>
    ) -> String {

        match self {
            Element::Phoneme { key, rep } => {
                let phoneme = match *rep {
                    true => &rep_phonemes[*key],
                    false => &language[*key],
                };

                format!("{}", phoneme)
            },
            Element::Group(key) => format!("{}", &language[*key].name),
            Element::Boundary => String::from("#"),
            Element::Any(elements) => {
                let mut content = String::from("[");

                for element in elements.iter() {
                    content.push_str(&element.as_str(language, rep_phonemes));
                }

                content.push(']');
                content
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(enum_map::Enum, enum_iterator::Sequence)]
pub enum Field {
    Target,
    Replacement,
    EnvStart { has_boundary: bool },
    EnvEnd { has_boundary: bool },
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Field::Target => "Target",
            Field::Replacement => "Replacement",
            Field::EnvStart { .. } => "EnvStart",
            Field::EnvEnd { .. } => "EnvEnd",
        })
    }
}

impl From<Field> for usize {
    fn from(value: Field) -> Self {
        unsafe { *(&value as *const Field as *const usize) }
    }
}

impl From<usize> for Field {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Target,
            1 => Self::Replacement,
            2 => Self::EnvStart { has_boundary: false },
            3 => Self::EnvEnd { has_boundary: false },
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct FieldParseError {
    field: Field,
    msg: &'static str,
}

impl fmt::Display for FieldParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} `{}` field", self.msg, self.field)
    }
}

impl error::Error for FieldParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
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
        mut head: bool, tail: bool, nested: bool,
    ) -> Result<(), FieldParseError> {
        let (mut idx, mut idx_prev) = (0, std::usize::MAX);
        while idx != idx_prev {
            let idx_temp = idx;

            let tail_curr = !raw.is_empty();
            let tail_curr = tail_curr && idx == raw.len() - 1;

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

                    if !nested {
                        head = false;
                    }

                    idx += start_match_len;
                } else {
                    for phoneme in group.phonemes {
                        let PhonemeRef { key, phoneme, .. } = phoneme;

                        if raw[idx..].starts_with(phoneme.as_ref()) {
                            elements.push(Element::Phoneme { key, rep: false });

                            if !nested {
                                head = false;
                            }

                            idx += phoneme.len();
                        }
                    }
                }
            }

            match (raw[idx..].chars().next(), idx == idx_temp) {
                (Some('['), true) => {
                    if nested {
                        let err = FieldParseError {
                            field: *self,
                            msg: "Found nested brackets '[]'",
                        };

                        return Err(err);
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

                    let mut any_elements = Vec::default();

                    self.parse(
                        language, 
                        rep_phonemes, 
                        rep_phoneme_usages, 
                        rewrite_rules, 
                        &mut any_elements, 
                        raw_any,
                        head, tail_curr, 
                        true
                    )?;

                    elements.push(Element::Any(any_elements));

                    idx += raw_any.len() + 2;
                },
                (Some('#'), true) => {
                    let tail = (nested && tail) || (!nested && tail_curr);

                    match self {
                        Field::EnvStart { has_boundary } | //
                        Field::EnvEnd { has_boundary } //
                            if *has_boundary && !nested => {

                            let err = FieldParseError {
                                field: *self,
                                msg: "Multiple word boundary symbols present",
                            };

                            Err(err)
                        },
                        Field::EnvStart { has_boundary } //
                            if !*has_boundary && !head => {

                            let err = FieldParseError {
                                field: *self,
                                msg: "Word boundary symbol was not at the beginning of",
                            };

                            Err(err)
                        },
                        Field::EnvEnd { has_boundary } //
                            if !*has_boundary && !tail => {

                            let err = FieldParseError {
                                field: *self,
                                msg: "Word boundary symbol was not at the end of",
                            };

                            Err(err)
                        }
                        Field::EnvStart { has_boundary } //
                            if !*has_boundary && head => {
                                
                            *has_boundary = true;

                            Ok(())
                        },
                        Field::EnvEnd { has_boundary } //
                            if !*has_boundary && tail => {

                            *has_boundary = true;

                            Ok(())
                        },
                        Field::Target | Field::Replacement => {
                            let err = FieldParseError {
                                field: *self,
                                msg: "Word boundaries cannot be placed in",
                            };

                            Err(err)
                        },
                        _ => {
                            let err = FieldParseError {
                                field: *self,
                                msg: "Failed to parse",
                            };

                            Err(err)
                        }
                    }?;

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

                    if !nested {
                        head = false;
                    }
                }
            }

            idx_prev = idx_temp;
        }

        Ok(())
    }
}

#[derive(Default)]
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

#[derive(Debug)]
pub enum SoundChangeParseError<'a> {
    Field(Vec<FieldParseError>, &'a str),
    Format(&'a str),
}

impl<'a> fmt::Display for SoundChangeParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SoundChangeParseError::Field(errors, raw) => {
                let err_count = errors.len();

                writeln!(f, "{raw} => ")?;
                for (idx, err) in errors.iter().enumerate() {
                    write!(f, "    ({idx}) {err}{}",
                        if idx == err_count - 1 { "" } else { "\n" }
                    )?;
                }

                Ok(())
            },
            SoundChangeParseError::Format(raw) => //
                write!(f, concat!(
                    "{} =>\n    ",
                    "Failed to parse sound change, ensure it matches either ",
                    "of the following (permitting whitespace):\n        ",
                    "Target / Replacement / EnvStart _ EnvEnd\n        ",
                    "Target \u{2192} Replacement / EnvStart _ EnvEnd",
                ), raw),
        }
    }
}

impl<'a> error::Error for SoundChangeParseError<'a> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }
}

impl SoundChange {
    #[allow(unused_variables)]
    pub fn parse<'a>(
        language: &Language, 
        rep_phonemes: &mut SlotMap<PhonemeKey, Phoneme>, 
        rep_phoneme_usages: &mut SecondaryMap<PhonemeKey, usize>,
        rewrite_rules: &bimap::BiHashMap<&str, &str>, 
        raw: &'a str
    ) -> Result<SoundChange, SoundChangeParseError<'a>> {

        let re = regex::Regex::new(r"(\S*)(?:/)(\S*)(?:/)(\S*)(?:_)(\S*)").unwrap();

        if let Some(capture) = re.captures(raw) {
            let capture: [&str; 4] = capture.extract().1;

            let mut sc: [Vec<Element>; 4] = [(); 4].map(|_| Vec::default());
            let mut sc_err = Vec::with_capacity(4);

            for idx in 0..4 {
                let elements = &mut sc[idx];

                let result = Field::from(idx).parse(
                    language,
                    rep_phonemes,
                    rep_phoneme_usages,
                    rewrite_rules,
                    elements,
                    capture[idx],
                    true, false,
                    false,
                );

                if let Err(err) = result {
                    sc_err.push(err);
                }
            }

            if sc_err.is_empty() {
                Ok(SoundChange(sc))
            } else {
                Err(SoundChangeParseError::Field(sc_err, raw))
            }
        } else {
            Err(SoundChangeParseError::Format(raw))
        }
    }

    pub fn as_str(
        &self, 
        language: &Language, 
        rep_phonemes: &SlotMap<PhonemeKey, Phoneme>
    ) -> String {
        let mut content = String::from("");

        let field_as_str = |content: &mut String, field: &[Element]| {
            for element in field.iter() {
                content.push_str(&element.as_str(language, rep_phonemes));
            }
        };

        field_as_str(&mut content, &self.0[0]);
        content.push('\u{2192}');
        field_as_str(&mut content, &self.0[1]);
        content.push('/');
        field_as_str(&mut content, &self.0[2]);
        content.push('_');
        field_as_str(&mut content, &self.0[3]);

        content
    }
}