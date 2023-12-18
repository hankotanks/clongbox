use std::mem;

use once_cell::sync::OnceCell;

use crate::{widgets, FocusTarget, FocusBuffer};
use crate::{Phoneme, PhonemeKey, PhonemeSrc};
use crate::GroupKey;
use crate::language::PhonemeRefMut;
use crate::Selection;
use crate::app::fonts;

pub enum NewPhonemeTool {
    Editing { 
        phoneme_key: PhonemeKey, 
        phoneme_editor_state: widgets::EditorState<PhonemeKey>, 
        rm: bool,
        clear: bool,
    },
    New { 
        content: String, 
        groups: Vec<GroupKey>, 
        complete: OnceCell<PhonemeKey>,
        failed: bool,
    },
}

impl Default for NewPhonemeTool {
    fn default() -> Self {
        Self::New { 
            content: String::from(""), 
            groups: Vec::new(), 
            complete: OnceCell::default(),
            failed: false,
        }
    }
}

// The discriminant of the PhonemeEditorSelection focus target
const DISC: mem::Discriminant<FocusTarget> = //
    mem::discriminant(&FocusTarget::PhonemeEditorSelect);

impl NewPhonemeTool {
    fn selection_row(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        match self {
            NewPhonemeTool::Editing { phoneme_key, phoneme_editor_state, rm, clear } => {
                let Phoneme { phoneme, grapheme } = &mut state.language[*phoneme_key];

                let phoneme = PhonemeRefMut {
                    key: *phoneme_key,
                    phoneme,
                    grapheme,
                    rm,
                };
                
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
            NewPhonemeTool::New { content, groups, complete, failed, } => {
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

                            for group_key in groups.iter().copied() {
                                state.language[group_key].keys.insert(phoneme_key);
                            }

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

                let mut selected = state.focus.needs(DISC);

                if ui.toggle_value(&mut selected, "Select").clicked() {
                    if selected {
                        state.focus.set(ui.id(), FocusTarget::PhonemeEditorSelect);
                    } else {
                        state.focus.clear();
                    }
                }
            },
        }
    }
}

impl super::Tool for NewPhonemeTool {
    fn name(&self) -> &'static str { "New Phoneme" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        // Handle a Phoneme selection from the previous frame
        if let Some(buffer) = state.focus.take_if_matches(DISC) {
            let FocusBuffer::Phoneme { key, src } = buffer else { panic!(); };

            if matches!(src, PhonemeSrc::Language) {
                *self = Self::Editing { 
                    phoneme_key: key, 
                    phoneme_editor_state: widgets::EditorState::None, 
                    rm: false,
                    clear: false,
                };
            }

            state.focus.clear();
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
    }
}