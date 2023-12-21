use std::collections::BTreeSet;
use std::mem;

use once_cell::sync::OnceCell;

use crate::{widgets, FocusTarget, FocusBuffer, layout};
use crate::{Phoneme, PhonemeKey, PhonemeSrc};
use crate::GroupKey;
use crate::language::Language;
use crate::Selection;
use crate::app::fonts;

pub enum NewPhonemeTool {
    Editing { 
        phoneme_key: PhonemeKey, 
        phoneme_editor_state: widgets::EditorState<PhonemeKey>, 
        groups: BTreeSet<GroupKey>,
        group_editor_state: widgets::EditorState<GroupKey>,
        rm: bool,
        clear: bool,
    },
    New {
        content: String,
        complete: OnceCell<PhonemeKey>,
        failed: bool,
    },
}

impl Default for NewPhonemeTool {
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

impl NewPhonemeTool {
    fn selection_row(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        match self {
            NewPhonemeTool::Editing { 
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

                if ui.button("Clear Selection").clicked() {
                    *clear = true;
                }
            },
            NewPhonemeTool::New { content, complete, failed, .. } => {
                let phoneme_editor_width = fonts::ipa_text_width(content.as_str());
                let phoneme_editor_width = phoneme_editor_width.max(fonts::ipa_text_width("_ [ _ ]")) + //;
                    ui.spacing().button_padding.x * 2.;

                let phoneme_editor = egui::TextEdit::singleline(content)
                    .font(fonts::FONT_ID.to_owned())
                    .hint_text("_ [ _ ]")
                    .desired_width(phoneme_editor_width);

                if ui.add(phoneme_editor).lost_focus() {
                    match Phoneme::parse(content.as_str()) {
                        Ok(phoneme) => {
                            let phoneme_key = state.language.phoneme_add(phoneme);
                            
                            let _ = complete.set(phoneme_key);                                
                        },
                        Err(_) => {
                            *failed = true;
                        },
                    }
                }

                if content.trim().is_empty() {
                    ui.label("Add a new phoneme, or...");
                } else if Phoneme::parse(content.as_str()).is_err() {
                    ui.label("Invalid phoneme...");
                    ui.separator();
                }

                let mut selected = state.focus.needs(DISC_SELECT);

                if ui.toggle_value(&mut selected, "Select").clicked() {
                    if selected {
                        state.focus.set(ui.id(), FocusTarget::PhonemeEditorSelect);
                    } else {
                        state.focus.clear();
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
            groups, 
            group_editor_state, .. 
        } = self else { 
            panic!(); 
        };

        for key in groups.clone().into_iter() {
            if let Some(group) = state.language.group_ref_mut(key) {
                let mut should_remove = false;

                widgets::group_editor(
                    ui, 
                    &mut state.focus, 
                    group, 
                    group_editor_state, 
                    &mut Selection::Flag { 
                        flag: &mut should_remove, 
                        message: "remove selected phoneme from this group"
                    },
                );
    
                if should_remove {
                    groups.remove(&key);
    
                    state.language[key].keys.remove(&phoneme_key);
                }
            }
        }
    }

    fn group_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        layout::hungry_frame(ui, |ui| {
            match self {
                NewPhonemeTool::Editing { groups, .. } => {
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

                    self.group_panel_inner(state, ui);
                },
                NewPhonemeTool::New { .. } => {
                    ui.add_enabled_ui(false, |ui| {
                        ui.label("Complete phoneme addition to modify groups...");
                    });
                },
            }
        });
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

impl super::Tool for NewPhonemeTool {
    fn name(&self) -> &'static str { "New Phoneme" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        // Handle a Phoneme selection from the previous frame
        if let Some(buffer) = state.focus.take_if_matches(DISC_SELECT) {
            let FocusBuffer::Phoneme { key, src } = buffer else { panic!(); };

            if matches!(src, PhonemeSrc::Language) {
                *self = Self::Editing { 
                    phoneme_key: key, 
                    phoneme_editor_state: widgets::EditorState::None, 
                    groups: get_groups(&state.language, key),
                    group_editor_state: widgets::EditorState::None,
                    rm: false,
                    clear: false,
                };
            }

            state.focus.clear();
        }

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
            NewPhonemeTool::New { complete, .. } 
                if complete.get().is_some() => complete.get().copied(),
            _ => None,
        };

        if let Some(phoneme_key) = complete {
            *self = Self::Editing { 
                phoneme_key,
                phoneme_editor_state: widgets::EditorState::None,
                groups: get_groups(&state.language, phoneme_key),
                group_editor_state: widgets::EditorState::None,
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
            NewPhonemeTool::Editing { phoneme_key, rm, .. } 
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

        self.group_panel(state, ui);
    }
}