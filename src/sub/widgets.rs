use std::mem;

use crate::{status, FocusTarget, GroupKey, GroupName, Phoneme, PhonemeSrc, Selection, CONFIG};
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
pub fn phoneme_editor(
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

            let response = focus.show_if_valid(FocusBuffer::Phoneme { key, src }, ui, |ui| {
                let content = egui::RichText::new(format!("{}", phoneme))
                    .font(fonts::FONT_ID.to_owned());

                ui.toggle_value(
                    &mut selection.is_selected(key), 
                    content
                )
            });
            
            if let Some(response) = response {
                if response.clicked() {
                    if let Selection::Flag { flag, .. } = selection {
                        let _ = mem::replace(*flag, true);

                        focus.set(response.id, FocusTarget::PhonemeEditorSelect);
                        focus.set_buffer(response.id, FocusBuffer::Phoneme { key, src });
                    } else {
                        selection.toggle(key);
                    }
                } else if response.secondary_clicked() {
                    *state = EditorState::Active {
                        key, 
                        content: format!("{}", phoneme),
                    };
                }

                let status_message = format!("{}Right-click to edit in place. Clearing the name deletes the phoneme", match selection {
                    Selection::Single(_) | Selection::Multiple(_) => //
                        "Click to select this phoneme. ",
                    Selection::Flag { message, .. } => {
                        if message.is_empty() {
                            ""
                        } else {
                            Box::leak(format!("Click to {message}. ").into_boxed_str())
                        }
                    },
                    Selection::None => "",
                });

                status::set_on_hover(&response, status_message);
            }
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

            let response = focus.show_if_valid(FocusBuffer::Group(key), ui, |ui| {
                let content = egui::RichText::from(format!("{}", name))
                    .font(font.clone());

                ui.toggle_value(
                    &mut selection.is_selected(key), 
                    content,
                )
            });
            
            if let Some(response) = response {
                if response.clicked() {
                    selection.toggle(key);
                } else if response.secondary_clicked() {
                    *state = EditorState::Active { 
                        key, 
                        content: format!("{}", name),
                    };
                }

                let status_message = format!("{}Right-click to edit group name. Clearing the name deletes the group", match selection {
                    Selection::Single(_) | Selection::Multiple(_) => //
                        "Click to select. ",
                    Selection::Flag { message, .. } => {
                        if message.is_empty() {
                            ""
                        } else {
                            Box::leak(format!("Click to {message}. ").into_boxed_str())
                        }
                    },
                    Selection::None => "",
                });

                status::set_on_hover(&response, status_message);
            }
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
            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
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

#[derive(Default)]
pub struct FauxButtonResponse {
    // NOTE: This is a struct for clarity, nothing else
    pub clicked: bool,
}

fn faux_button_size(ui: &egui::Ui) -> egui::Vec2 {
    egui::Vec2::splat(ui.text_style_height(&egui::TextStyle::Small))
}

fn faux_button(rect: egui::Rect, ui: &egui::Ui) -> FauxButtonResponse {
    // TODO: These colors should not be hardcoded, 
    // but pulled from button styling defaults
    ui.painter().rect(
        rect, 
        CONFIG.selection_rounding,
        egui::Color32::LIGHT_GRAY,
        egui::Stroke {
            width: ui.visuals().window_stroke.width,
            color: egui::Color32::DARK_GRAY,
        },
    );

    let width = rect.width();
    let width_offset = (width - width * 0.55) * 0.5;

    ui.painter().line_segment(
        [
            rect.left_center() + egui::Vec2::new(width_offset, 0.), 
            rect.right_center() - egui::Vec2::new(width_offset, 0.),
            ], 
        egui::Stroke { width: 1., color: egui::Color32::BLACK, }
    );

    let mut response = FauxButtonResponse::default();

    ui.ctx().input(|input| {
        for event in input.events.iter() {
            if let egui::Event::PointerButton {
                pos, 
                button: egui::PointerButton::Primary, 
                pressed: false, .. 
            } = event {
                if rect.contains(*pos) {
                    response.clicked = true;
                }
            }
        }
    });

    response
}

fn faux_button_small(center: egui::Pos2, ui: &egui::Ui) -> FauxButtonResponse {
    let rect = egui::Rect::from_center_size(
        center, 
        faux_button_size(ui),
    );

    faux_button(rect, ui)
}

pub fn deletion_overlay(response: &egui::Response, ui: &egui::Ui) -> FauxButtonResponse {
    let response = response.interact(egui::Sense::hover());

    if response.hovered() {
        faux_button_small(response.rect.center(), ui)
    } else {
        FauxButtonResponse::default()
    }
}

pub fn deletion_overlay_corner(response: &egui::Response, ui: &egui::Ui) -> FauxButtonResponse {
    let response = response.interact(egui::Sense::hover());

    if response.hovered() {
        let rect = response.rect.expand2({
            egui::Vec2::new(ui.spacing().button_padding.x, 0.)
        });

        let faux_button_size = faux_button_size(ui) * 0.5;

        let offset = egui::Vec2 {
            x: faux_button_size.x * -1.,
            y: faux_button_size.y,
        };

        // NOTE: This compensates for the width of the stroke
        let offset = offset + egui::Vec2 {
            x: 0.,
            y: ui.visuals().window_stroke.width * -1.,
        };

        let center = rect.right_top() + offset;

        faux_button_small(center, ui)
    } else {
        FauxButtonResponse::default()
    }
}