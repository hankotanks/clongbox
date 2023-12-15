use std::{ops, mem, fmt};

use crate::{Selection, Phoneme, PhonemeSrc};
use crate::{Focus, FocusBuffer};
use crate::PhonemeKey;
use crate::language::PhonemeRefMut;
use crate::app::fonts;

#[allow(dead_code)]
pub enum EditorState<K: slotmap::Key, T> {
    Active { key: K, content: String, original: T },
    None,
}

impl<K: slotmap::Key, T> Default for EditorState<K, T> {
    fn default() -> Self {
        Self::None
    }
}

#[allow(dead_code)]
fn phoneme_editor(
    ui: &mut egui::Ui,
    mut phoneme: PhonemeRefMut<'_>,
    state: &mut EditorState<PhonemeKey, Phoneme>,
    focus: &mut Focus,
    selection: &mut Selection<'_, PhonemeKey>,
    src: PhonemeSrc,
) {
    match state {
        EditorState::Active { key, content, original } if *key == phoneme.key => {
            let phoneme_editor_width = fonts::ipa_text_width(content.as_str()) + //
                ui.spacing().button_padding.x * 2.;

            let phoneme_editor = egui::TextEdit::singleline(content)
                .font(fonts::FONT_ID.to_owned())
                .desired_width(phoneme_editor_width);

            if ui.add(phoneme_editor).lost_focus() {
                if content.trim().is_empty() {
                    phoneme.delete();
                } else {
                    let PhonemeRefMut { phoneme, grapheme, .. } = phoneme;

                    match Phoneme::parse(content.as_str()) {
                        Ok(new_phoneme) => {
                            *phoneme = new_phoneme.phoneme;
                            *grapheme = new_phoneme.grapheme;
                        },
                        Err(_) => {
                            let original = original.to_owned();

                            *phoneme = original.phoneme;
                            *grapheme = original.grapheme;
                        },
                    }
                }

                let _ = mem::replace(state, EditorState::None);
            }
        },
        _ => {
            let PhonemeRefMut { key, .. } = phoneme;

            let buffer = FocusBuffer::Phoneme { key, src };

            let content = egui::RichText::new(format!("{}", phoneme))
                .font(fonts::FONT_ID.to_owned());

            focus.show_if_valid(
                buffer, ui, 
                |ui| ui.button(content.clone()),
                |ui| {
                    let response = ui.toggle_value(
                        &mut selection.is_selected(key), 
                        content.clone()
                    );

                    if response.clicked() {
                        selection.toggle(key);
                    } else if response.secondary_clicked() {
                        let original = phoneme.to_owned_phoneme();

                        *state = EditorState::Active { 
                            key, 
                            content: format!("{}", phoneme), 
                            original 
                        };
                    }
                }
            );
        },
    }
}

pub fn phoneme_selection_list<'a, P>(
    ui: &mut egui::Ui,
    focus: &mut Focus,
    phonemes: P,
    phoneme_editor_state: &mut EditorState<PhonemeKey, Phoneme>,
    phoneme_src: PhonemeSrc,
    mut selection: Selection<'_, PhonemeKey>,
) where P: Iterator<Item = PhonemeRefMut<'a>> {
    ui.horizontal_wrapped(|ui| {
        for phoneme in phonemes {
            phoneme_editor(ui, phoneme, phoneme_editor_state, focus, &mut selection, phoneme_src);
        }
    });
}

#[allow(dead_code)]
fn selection_list_inner<'a, K, C, Ki, I>(
    ui: &mut egui::Ui,
    mut selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: Ki,
    buffer: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = I>,
    Ki: Iterator<Item = K>,
    I: fmt::Display {

    for key in collection_keys { 
        let content = format!("{}", collection[key]);

        focus.show_if_valid(
            (buffer)(key), ui,
            |ui| ui.button(&content),
            |ui| {
                if ui.toggle_value(&mut selection.is_selected(key), &content).clicked() {
                    selection.toggle(key);
                }
            }
        ); 
    }
}

#[allow(dead_code)]
fn selection_list_wrapped<'a, K, C, Ki, I>(
    ui: &mut egui::Ui,
    selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: Ki,
    buffer_from_key: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = I>,
    Ki: Iterator<Item = K>,
    I: fmt::Display {

    ui.horizontal_wrapped(|ui| {
        selection_list_inner(ui, selection, focus, //
            collection, collection_keys, buffer_from_key);
    });
}

#[allow(dead_code)]
fn selection_list<'a, K, C, Ki, I>(
    ui: &mut egui::Ui,
    selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: Ki,
    buffer_from_key: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = I>,
    Ki: Iterator<Item = K>,
    I: fmt::Display {

    egui::ScrollArea::vertical().show(ui, |ui| {
        selection_list_inner(ui, selection, focus, //
            collection, collection_keys, buffer_from_key);
    });
}