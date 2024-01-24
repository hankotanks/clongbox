pub mod focus;

use std::{borrow, io, sync};

use egui::ahash::HashMap;

use slotmap::{SlotMap, SecondaryMap};

use crate::Focus;
use crate::{PhonemeKey, Phoneme};
use crate::sc::SoundChange;
use crate::language::{Language, LanguageRaw};

#[derive(Default)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct State {
    pub language: Language,

    pub rep_phonemes: SlotMap<PhonemeKey, Phoneme>,
    pub rep_phoneme_usages: SecondaryMap<PhonemeKey, usize>,
    
    pub sound_changes: Vec<SoundChange>,
    pub sound_changes_broken: Vec<sync::Arc<str>>,

    pub lexicon: Vec<sync::Arc<str>>,

    #[serde(skip)]
    pub focus: Focus,
}

impl State {
    pub fn parse_from_str<'a, I>(content: I) -> anyhow::Result<Self> 
        where I: Into<borrow::Cow<'a, str>> + Clone {

        let content: borrow::Cow<'_, str> = content.clone().into();
        let content: &str = content.as_ref();

        StateParser::init_from_str(content).map(|parser| {
            Into::<State>::into(parser)
        })
    }
}

#[derive(Debug)]
pub struct StateParser<'a> {
    categories: Vec<(&'a str, &'a str)>,
    sound_changes: Vec<&'a str>,
    rewrite_rules: bimap::BiHashMap<&'a str, &'a str>,
    lexicon: Vec<&'a str>,
    romanization: HashMap<&'a str, &'a str>,
}

fn collect_matches<'a, T>(
    re: &regex::Regex, 
    haystack: &'a str, 
    collection: T, 
    push: fn(&mut T, &'a str) -> bool) -> Result<T, io::Error> {

    let mut collection = collection;

    let mut push_result = false;
    re
        .find_iter(haystack)
        .map(|re_match| re_match.as_str())
        .for_each(|m| {
            push_result |= (push)(&mut collection, m);
        });

    match push_result {
        true => Result::Err(io::Error::from(io::ErrorKind::InvalidData)),
        false => Ok(collection),
    }
}

impl<'a> StateParser<'a> {
    pub fn init_from_str(content: &'a str) -> anyhow::Result<Self> {
        let categories_re = regex::Regex::new(r"\S=\S*")?;
        let categories = collect_matches(&categories_re, content, 
            Vec::new(), 
        |categories, m| {
                match m.split_once('=') {
                    Some(category) => {
                        categories.push(category);
                        false
                    },
                    None => true,
                }
            }
        )?;

        let rewrite_rules_re = regex::Regex::new(r"\S+\|\S+")?;
        let rewrite_rules = collect_matches(&rewrite_rules_re, content, 
            bimap::BiHashMap::default(), 
        |rules, m| {
                match m.split_once('|') {
                    Some((from, to)) => {
                        rules.insert(from, to);
                        false
                    },
                    None => true,
                }
            }
        )?;

        let sound_changes_re = regex::Regex::new("\\S*[\\/\u{2192}]\\S*\\/\\S*_\\S*")?;
        let sound_changes = collect_matches(&sound_changes_re, content, 
            Vec::new(), 
            |sound_changes, m| {
                sound_changes.push(m);
            
                false
            }
        )?;

        let lexicon_re = regex::Regex::new(r"^[^\|]\S*").unwrap();
        let lexicon = content.lines().filter(|line| {
            let a = categories_re.is_match(line);
            let b = rewrite_rules_re.is_match(line);
            let c = sound_changes_re.is_match(line);

            !(a || b || c) && lexicon_re.is_match(line)
        }).collect();

        let romanization_re = regex::Regex::new(r"[\n^]\|(\S+)\s*\[(\S+)\]")?;
        let romanization = romanization_re
            .captures_iter(content)
            .filter_map(|capture| {
                if let Some(grapheme) = capture.get(2).map(|m| m.as_str()) {
                    let phoneme = capture.get(1).unwrap().as_str();

                    Some((phoneme, grapheme))
                } else {
                    None
                }
            }
        ).collect();

        let state_parser = Self {
            categories,
            sound_changes,
            rewrite_rules,
            lexicon,
            romanization,
        };

        Ok(state_parser)
    }
}

#[allow(clippy::from_over_into)]
impl<'a> Into<State> for StateParser<'a> {
    fn into(self) -> State {
        let language: Language = {
            let Self { categories, romanization, rewrite_rules, .. } = &self;

            LanguageRaw::parse(categories, romanization, rewrite_rules).into()
        };

        let mut rep_phonemes = SlotMap::with_key();
        let mut rep_phoneme_usages = slotmap::SecondaryMap::default();

        let mut broken = Vec::default();

        let sound_changes = self.sound_changes
            .iter()
            .copied()
            .filter_map(|sound_change| {
                let parsed = SoundChange::parse(
                    &language, 
                    &mut rep_phonemes, 
                    &mut rep_phoneme_usages, 
                    &self.rewrite_rules, sound_change
                );

                match parsed {
                    Ok(sound_change) => Some(sound_change),
                    Err(err) => {
                        let err = format!("{}", err);

                        broken.push(sync::Arc::from(err.as_str()));

                        None
                    },
                }
            }).collect();

        let lexicon = self.lexicon
            .into_iter()
            .map(sync::Arc::from)
            .collect();
        
        State {
            language,
            rep_phonemes,
            rep_phoneme_usages,
            sound_changes,
            sound_changes_broken: broken,
            lexicon,
            focus: Focus::default(),
        }
    }
}