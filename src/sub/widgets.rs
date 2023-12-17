use std::mem;

use crate::{Selection, Phoneme, PhonemeSrc, GroupKey, GroupName};
use crate::{Focus, FocusBuffer};
use crate::PhonemeKey;
use crate::language::{PhonemeRefMut, GroupsMut, GroupRefMut};
use crate::app::fonts;

#[allow(dead_code)]
pub enum EditorState<K: slotmap::Key> {
    Active { key: K, content: String },
    None,
}

impl<K: slotmap::Key> Default for EditorState<K> {
    fn default() -> Self {
        Self::None
    }
}

#[allow(dead_code)]
fn phoneme_editor(
    ui: &mut egui::Ui,
    focus: &mut Focus,
    mut phoneme: PhonemeRefMut<'_>,
    state: &mut EditorState<PhonemeKey>,
    src: PhonemeSrc,
    selection: &mut Selection<'_, PhonemeKey>,
) {
    match state {
        EditorState::Active { key, content } if *key == phoneme.key => {
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
                        Err(_) => { /*  */ },
                    }
                }

                let _ = mem::replace(state, EditorState::None);
            }
        },
        _ => {
            let PhonemeRefMut { key, .. } = phoneme;

            focus.show_if_valid(FocusBuffer::Phoneme { key, src }, ui, |ui| {
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
                    *state = EditorState::Active { 
                        key, 
                        content: format!("{}", phoneme),
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
    phoneme_editor_state: &mut EditorState<PhonemeKey>,
    phoneme_src: PhonemeSrc,
    mut selection: Selection<'_, PhonemeKey>,
) where P: Iterator<Item = PhonemeRefMut<'a>> {
    ui.horizontal_wrapped(|ui| {
        for phoneme in phonemes {
            phoneme_editor(
                ui, focus, 
                phoneme, 
                phoneme_editor_state, 
                phoneme_src,
                &mut selection,
            );
        }
    });
}

fn group_editor_inner(
    ui: &mut egui::Ui,
    focus: &mut Focus,
    font: egui::FontId,
    mut group: GroupRefMut<'_>,
    state: &mut EditorState<GroupKey>,
    selection: &mut Selection<'_, GroupKey>,
) {
    match state {
        EditorState::Active { key, content } if *key == group.key => {
            let group_editor = egui::TextEdit::singleline(content)
                .font(font.clone());

            if ui.add(group_editor).lost_focus() {
                if content.trim().is_empty() {
                    group.delete();
                } else {
                    match GroupName::parse(content.as_str()) {
                        Ok(group_name) =>
                            *(group.name) = group_name,
                        Err(_) => { /*  */ },
                    }
                }

                let _ = mem::replace(state, EditorState::None);
            }
        },
        _ => {
            let GroupRefMut { key, name, .. } = group;

            focus.show_if_valid(FocusBuffer::Group(key), ui, |ui| {
                let content = egui::RichText::from(format!("{}", name))
                    .font(font.clone());

                ui.toggle_value(
                    &mut selection.is_selected(key), 
                    content,
                )
            }).map(|response| {
                if response.clicked() {
                    selection.toggle(key);
                } else if response.secondary_clicked() {
                    *state = EditorState::Active { 
                        key, 
                        content: format!("{}", name),
                    };
                }
            });
        },
    }
}

pub fn group_editor(
    ui: &mut egui::Ui,
    focus: &mut Focus,
    group: GroupRefMut<'_>,
    state: &mut EditorState<GroupKey>,
    selection: &mut Selection<'_, GroupKey>,
) {
    let font = ui.text_style_height(&egui::TextStyle::Button);
    let font = egui::FontId::proportional(font);

    group_editor_inner(ui, focus, font, group, state, selection);
}

pub fn group_editor_heading(
    ui: &mut egui::Ui,
    focus: &mut Focus,
    group: GroupRefMut<'_>,
    state: &mut EditorState<GroupKey>,
    selection: &mut Selection<'_, GroupKey>,
) {
    let font = ui.text_style_height(&egui::TextStyle::Heading);
    let font = egui::FontId::proportional(font);

    group_editor_inner(ui, focus, font, group, state, selection);
}

#[allow(dead_code, unused_variables, unused_mut)]
pub fn group_selection_list(
    ui: &mut egui::Ui,
    focus: &mut Focus,
    groups: GroupsMut<'_, impl Iterator<Item = GroupKey>>,
    group_editor_state: &mut EditorState<GroupKey>,
    mut selection: Selection<'_, GroupKey>,
) {
    let group_selection_list_size = egui::Vec2 { 
        x: ui.spacing().text_edit_width * 0.6,
        y: ui.available_height(),
    };

    egui_extras::TableBuilder::new(ui)
        .column(egui_extras::Column::exact(group_selection_list_size.x))
        .header(group_selection_list_size.y, |mut row| { row.col(|ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().button_padding.y = ui.spacing().item_spacing.y;
                    ui.spacing_mut().item_spacing.y *= 2.;
                    
                    for group in groups {
                        group_editor(ui, focus, group, group_editor_state, &mut selection);
                    }
                });
            });
        }); });
}