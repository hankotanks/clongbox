use std::mem;

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

            focus.show_if_valid(buffer, ui, |ui| {
                let content = egui::RichText::new(format!("{}", phoneme))
                    .font(fonts::FONT_ID.to_owned());

                ui.toggle_value(
                    &mut selection.is_selected(key), 
                    content
                )
            }).map(|response| {
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
            });
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