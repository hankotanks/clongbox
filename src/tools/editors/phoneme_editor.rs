use std::collections::BTreeSet;
use std::mem;

use once_cell::sync::OnceCell;

use crate::{layout, widgets, FocusBuffer, FocusTarget, CONFIG};
use crate::{Phoneme, PhonemeKey, PhonemeSrc};
use crate::GroupKey;
use crate::language::Language;
use crate::Selection;
use crate::app::fonts;

pub enum PhonemeEditor {
    Editing { 
        phoneme_key: PhonemeKey, 
        phoneme_editor_state: widgets::EditorState<PhonemeKey>, 
        groups: BTreeSet<GroupKey>,
        rm: bool,
        clear: bool,
    },
    New {
        content: String,
        complete: OnceCell<PhonemeKey>,
        failed: bool,
    },
}

impl Default for PhonemeEditor {
    fn default() -> Self {
        Self::New { 
            content: String::from(""),
            complete: OnceCell::default(),
            failed: false,
        }
    }
}

// The discriminant of the PhonemeEditorSelection focus target
const DISC_SELECT: mem::Discriminant<FocusTarget> = //
    mem::discriminant(&FocusTarget::PhonemeEditorSelect);

const DISC_GROUPS: mem::Discriminant<FocusTarget> = //
    mem::discriminant(&FocusTarget::PhonemeEditorGroups { selected: None });

fn parse_new_phoneme(
    content: &str, 
    language: &mut Language, 
    failed: &mut bool,
    complete: &OnceCell<PhonemeKey>, 
) {
    match Phoneme::parse(content) {
        Ok(phoneme) => {
            let phoneme_key = language.phoneme_add(phoneme);
            
            let _ = complete.set(phoneme_key);                                
        },
        Err(_) => {
            *failed = true;
        },
    }
}

impl PhonemeEditor {
    fn selection_row(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        match self {
            PhonemeEditor::Editing { 
                phoneme_key, 
                phoneme_editor_state,
                clear, ..
            } if let Some(phoneme) = state.language.phoneme_ref_mut(*phoneme_key) => {
                egui::Frame::default()
                    .outer_margin(egui::Margin::same(0.))
                    .inner_margin({
                        if matches!(phoneme_editor_state, widgets::EditorState::None) {
                            egui::Margin::symmetric(0., ui.visuals().window_stroke.width)
                        } else {
                            egui::Margin::same(0.)
                        }
                    })
                    .show(ui, |ui| {
                        widgets::phoneme_editor(
                            ui, 
                            &mut state.focus, 
                            phoneme, 
                            phoneme_editor_state, 
                            PhonemeSrc::Language, 
                            &mut Selection::None,
                        );
                    });

                if let Some(response) = layout::button_context_line(ui,[
                    layout::BtnContextElem::Button("Clear"),
                    layout::BtnContextElem::Label("the current selection")
                ]).get(0) {
                    if response.clicked() {
                        *clear = true;
                    }
                }
            },
            PhonemeEditor::New { content, complete, failed, .. } => {
                let phoneme_editor_width = fonts::ipa_text_width(content.as_str());
                let phoneme_editor_width = phoneme_editor_width.max(fonts::ipa_text_width("_ [ _ ]")) + //;
                    ui.spacing().button_padding.x * 2.;

                let phoneme_editor = egui::TextEdit::singleline(content)
                    .font(fonts::FONT_ID.to_owned())
                    .hint_text("_ [ _ ]")
                    .desired_width(phoneme_editor_width);

                let phoneme_editor_response = ui.add(phoneme_editor);

                if phoneme_editor_response.lost_focus() {
                    let language = &mut state.language;

                    parse_new_phoneme(content, language, failed, complete);
                }

                if content.trim().is_empty() {
                    if let Some(response) = layout::button_context_line(ui, [
                        layout::BtnContextElem::Label("Add a new phoneme, or"),
                        layout::BtnContextElem::Toggle("Select", state.focus.needs(DISC_SELECT)),
                    ]).get(0) {
                        if response.clicked() {
                            if state.focus.needs(DISC_SELECT) {
                                state.focus.clear();
                            } else {
                                state.focus.set(ui.id(), FocusTarget::PhonemeEditorSelect);
                            }
                        }
                    }
                } else if Phoneme::parse(content.as_str()).is_err() {
                    ui.label("Invalid phoneme, try following the hint text");
                } else if let Some(response) = layout::button_context_line(ui, [
                    layout::BtnContextElem::Button("Confirm"),
                    layout::BtnContextElem::Label("addition of phoneme")
                ]).get(0) {
                    if response.clicked() {
                        phoneme_editor_response.surrender_focus();
                    }
                }
            },
            _ => {
                let _ = mem::take(self);
            }
        }
    }

    fn group_panel_inner(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let Self::Editing {
            phoneme_key,
            groups, ..
        } = self else {
            panic!();
        };

        for key in groups.clone().into_iter() {
            if let Some(group) = state.language.group_ref_mut(key) {
                let content = fonts::ipa_rt(format!("{}", group.name));

                let response = ui.label(content);

                if response.hovered() {
                    let rect = response.rect.expand2(egui::Vec2 {
                        x: ui.spacing().button_padding.x,
                        y: 0.,
                    });

                    ui.painter().rect_stroke(rect, CONFIG.selection_rounding, ui.visuals().window_stroke);
                }
                
                let response = widgets::deletion_overlay_corner(&response, ui);

                if response.clicked {
                    groups.remove(&key);

                    if state.focus.needs(DISC_GROUPS) {
                        let focus = FocusTarget::PhonemeEditorGroups { 
                            selected: Some(groups.clone()) 
                        };
        
                        state.focus.set(ui.id(), focus);
                    }

                    state.language[key].keys.remove(phoneme_key);
                }
            }
        }
    }

    fn group_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        match self {
            PhonemeEditor::Editing { groups, .. } => {
                let is_selected = state.focus.needs(DISC_GROUPS);

                let response = ui.toggle_value(
                    &mut state.focus.needs(DISC_GROUPS), 
                    "Add Groups"
                );

                if response.clicked() {
                    if is_selected {
                        state.focus.clear();
                    } else {
                        let focus = FocusTarget::PhonemeEditorGroups { 
                            selected: Some(groups.clone()),
                        };

                        state.focus.set(response.id, focus);
                    }
                }

                ui.horizontal_wrapped(|ui| {
                    self.group_panel_inner(state, ui);
                });
            },
            PhonemeEditor::New { .. } => {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("Complete phoneme addition to modify groups...");
                });
            },
        }
    }
}

fn get_groups(language: &Language, key: PhonemeKey) -> BTreeSet<GroupKey> {
    language.groups().filter_map(|group| {
        if group.phonemes
            .map(|phoneme| phoneme.key)
            .collect::<BTreeSet<_>>()
            .contains(&key) {
                Some(group.key)
            } else { 
                None 
            }
    }).collect()
}

impl super::Editor for PhonemeEditor {
    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        if let Some(buffer) = state.focus.take_if_matches(DISC_GROUPS) {
            let FocusBuffer::Group(key) = buffer else { panic!(); };

            if let Self::Editing { phoneme_key, groups, .. } = self {
                state.language[key].keys.insert(*phoneme_key);

                groups.insert(key);

                let focus = FocusTarget::PhonemeEditorGroups { 
                    selected: Some(groups.clone()) 
                };

                state.focus.set(ui.id(), focus);
            }
        }

        // Handle the cleanup after a new Phoneme has been added
        let complete = match self {
            PhonemeEditor::New { complete, .. } 
                if complete.get().is_some() => complete.get().copied(),
            _ => None,
        };

        if let Some(phoneme_key) = complete {
            *self = Self::Editing { 
                phoneme_key,
                phoneme_editor_state: widgets::EditorState::None,
                groups: get_groups(&state.language, phoneme_key),
                rm: false,
                clear: false,
            };
        }

        // Clear the selection if the user requested it
        if matches!(self, Self::Editing { clear, .. } if *clear) {
            *self = Self::default();
        }

        // Handle the removal of the edited phoneme
        let rm = match self {
            PhonemeEditor::Editing { phoneme_key, rm, .. } 
                if *rm => Some(*phoneme_key),
            _ => None,
        };

        if let Some(phoneme_key) = rm {
            state.language.phoneme_remove(phoneme_key);

            let _ = mem::take(self);
        }
       
        // Draw the selection row
        ui.horizontal(|ui| {
            self.selection_row(state, ui);
        });

        ui.separator();
        
        self.group_panel(state, ui);
    }

    fn target(&self) -> mem::Discriminant<FocusTarget> {
        const DISC: mem::Discriminant<FocusTarget> = //
            mem::discriminant(&FocusTarget::PhonemeEditorSelect);

        DISC
    }

    fn set(&mut self, state: &mut crate::State, buffer: FocusBuffer) {
        let FocusBuffer::Phoneme { key, src } = buffer else { panic!(); };

        if matches!(src, PhonemeSrc::Language) {
            *self = Self::Editing { 
                phoneme_key: key, 
                phoneme_editor_state: widgets::EditorState::None, 
                groups: get_groups(&state.language, key),
                rm: false,
                clear: false,
            };
        }
    }
}