use crate::{widgets, layout, editors};
use crate::{PhonemeKey, PhonemeSrc};
use crate::Selection;

#[derive(Default)]
pub struct PhonemePane {
    phoneme_editor_state: widgets::EditorState<PhonemeKey>,
}

impl super::Pane for PhonemePane {
    fn name(&self) -> &'static str { "Phonemes" }

    fn show(
        &mut self, 
        mut control: crate::Control<'_>, 
        state: &mut crate::State, 
        ui: &mut egui::Ui
    ) {
        layout::hungry_frame(ui, |ui| {
            let mut flag = false;
            widgets::phoneme_selection_list(
                ui, 
                &mut state.focus, 
                state.language.phonemes_mut_all(), 
                &mut self.phoneme_editor_state, 
                PhonemeSrc::Language, 
                Selection::Flag {
                    flag: &mut flag,
                    message: "view this phoneme in the editor"
                },
            );

            if flag {
                control.show_editor(editors::EditorKey::Phoneme);
            }
        });
    }
}