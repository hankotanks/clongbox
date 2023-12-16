use crate::widgets;
use crate::{PhonemeKey, PhonemeSrc};
use crate::Selection;

#[derive(Default)]
pub struct PhonemePane {
    phoneme_active: Option<PhonemeKey>,
    phoneme_editor_state: widgets::EditorState<PhonemeKey>,
}

impl super::Pane for PhonemePane {
    fn name(&self) -> &'static str { "Phonemes" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        widgets::phoneme_selection_list(
            ui, 
            &mut state.focus, 
            state.language.phonemes_mut_all(), 
            &mut self.phoneme_editor_state, 
            PhonemeSrc::Language, 
            Selection::Single(&mut self.phoneme_active),
        );
    }
}