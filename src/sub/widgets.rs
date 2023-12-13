use std::sync::Arc;
use std::{ops, mem};

use crate::{Selection, Phoneme};
use crate::{Focus, FocusBuffer};
use crate::PhonemeKey;
use crate::language::PhonemeRefMut;
use crate::app::fonts;

#[allow(dead_code)]
enum EditorState<K: slotmap::Key, T> {
    Active { key: K, content: String, original: T },
    None,
}

#[allow(dead_code)]
fn phoneme_editor(
    ui: &mut egui::Ui,
    mut phoneme: PhonemeRefMut<'_>,
    state: &mut EditorState<PhonemeKey, Phoneme>,
    focus: &mut Focus,
    mut selection: Selection<'_, PhonemeKey>,
    rep: bool,
) {
    match state {
        EditorState::Active { key, content, original } if *key == phoneme.key => {
            let phoneme_editor_width = fonts::ipa_text_width(content.as_str());

            let phoneme_editor = egui::TextEdit::singleline(content)
                .font(fonts::FONT_ID.to_owned())
                .desired_width(phoneme_editor_width);

            if ui.add(phoneme_editor).lost_focus() {
                if content.trim().is_empty() {
                    phoneme.delete();
                } else {
                    let re = regex::Regex::new(r"([^\s\[\]]+)(\s*\[\s*\S+\s*\])?").unwrap();

                    if let Some(capture) = re.captures(content.trim()) {
                        let PhonemeRefMut { phoneme, grapheme, .. } = phoneme;
                        
                        *phoneme = Arc::from(capture.get(0).unwrap().as_str());

                        if let Some(capture) = capture.get(1) {
                            let _ = grapheme.insert(Arc::from(capture.as_str()));
                        } else {
                            let _ = grapheme.take();
                        }
                    } else {
                        let original = original.to_owned();

                        *(phoneme.phoneme) = original.phoneme;
                        *(phoneme.grapheme) = original.grapheme;
                    }
                }

                let _ = mem::replace(state, EditorState::None);
            }
        },
        _ => {
            let PhonemeRefMut { key, .. } = phoneme;

            let buffer = FocusBuffer::Phoneme { key, rep };

            let content = egui::RichText::new(format!("{}", phoneme))
                .font(fonts::FONT_ID.to_owned());

            focus.show_if_valid(buffer, ui, content, |ui| {
                let content = format!("{}", phoneme);

                let response = ui.toggle_value(&mut selection.is_selected(key), &content);

                if response.clicked() {
                    selection.toggle(key);
                } else if response.secondary_clicked() {
                    let original = phoneme.to_owned_phoneme();

                    *state = EditorState::Active { key, content, original };
                }
            });
        },
    }
}

fn selection_list_inner<'a, K, C, I>(
    ui: &mut egui::Ui,
    mut selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: I,
    buffer: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = String>,
    I: Iterator<Item = K> {

    for key in collection_keys { 
        focus.show_if_valid(
            (buffer)(key), ui,
            egui::RichText::from(&collection[key]),
            |ui| {
                if ui.toggle_value(&mut selection.is_selected(key), &collection[key]).clicked() {
                    selection.toggle(key);
                }
            }
        ); 
    }
}

pub fn selection_list_wrapped<'a, K, C, I>(
    ui: &mut egui::Ui,
    selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: I,
    buffer: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = String>,
    I: Iterator<Item = K> {

    ui.horizontal_wrapped(|ui| {
        selection_list_inner(ui, selection, focus, //
            collection, collection_keys, buffer);
    });
}

pub fn selection_list<'a, K, C, I>(
    ui: &mut egui::Ui,
    selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: I,
    buffer_from_key: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = String>,
    I: Iterator<Item = K> {

    egui::ScrollArea::vertical().show(ui, |ui| {
        selection_list_inner(ui, selection, focus, //
            collection, collection_keys, buffer_from_key);
    });
}