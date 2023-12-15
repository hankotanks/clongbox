use std::mem;

use crate::{PhonemeKey, FocusTarget, FocusBuffer};

#[derive(Default)]
pub struct ScBuilderTool {
    collected_phonemes: Vec<PhonemeKey>,
    requesting: bool,
}

impl super::Tool for ScBuilderTool {
    fn name(&self) -> &'static str { "Sound Changes" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        const REQUEST: mem::Discriminant<FocusTarget> = mem::discriminant(&FocusTarget::Sc { 
            field: crate::sc::Field::Target, 
            head: false, 
            tail: false, 
            nested: false 
        });

        if ui.toggle_value(&mut self.requesting, "Select Phonemes").clicked() {
            if self.requesting {
                state.focus.set(FocusTarget::Sc { 
                    field: crate::sc::Field::Target, 
                    head: false, 
                    tail: false, 
                    nested: false 
                });
            } else {
                state.focus.clear();
            }
        }

        if self.requesting {
            if let Some(buffer) = state.focus.take(REQUEST) {
                let FocusBuffer::Phoneme { key, .. } = buffer else { panic!(); };
                self.collected_phonemes.push(key);
            }
        }

        for key in self.collected_phonemes.iter().copied() {
            ui.label(format!("{}", state.language[key]));
        }
    }
}