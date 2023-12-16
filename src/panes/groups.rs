use crate::{widgets, layout};
use crate::{GroupKey, Selection};
use crate::{PhonemeSrc, PhonemeKey};

#[derive(Default)]
pub struct GroupPane {
    group_active: Option<GroupKey>,
    group_editor_state: widgets::EditorState<GroupKey>,
    phoneme_editor_state: widgets::EditorState<PhonemeKey>,
}

impl GroupPane {
    fn group_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        widgets::group_selection_list(
            ui, 
            &mut state.focus, 
            state.language.groups_mut(), 
            &mut self.group_editor_state, 
            Selection::Single(&mut self.group_active),
        );
    }

    fn phoneme_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        layout::hungry_frame(ui, |ui| {
            match self.group_active {
                Some(key) => {
                    widgets::phoneme_selection_list(
                        ui, 
                        &mut state.focus, 
                        state.language.phonemes_mut(key),
                        &mut self.phoneme_editor_state, 
                        PhonemeSrc::Language, 
                        Selection::None,
                    );
                },
                None => {
                    ui.label("No group selected...");
                },
            }
        });
    }
}

impl super::Pane for GroupPane {
    fn name(&self) -> &'static str { "Groups" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let temp = egui::Layout::left_to_right(egui::Align::TOP);

        layout::hungry_frame_with_layout(ui, temp, |ui| {
            egui::Frame::default()
                .outer_margin(egui::Margin::same(0.))
                .inner_margin(egui::Margin::symmetric(0., ui.spacing().window_margin.top))
                .show(ui, |ui| { 
                    ui.push_id(0xA0B643, |ui| {
                        self.group_panel(state, ui);
                    }); 
                });

            egui::Frame::default()
                .stroke(ui.visuals().window_stroke)
                .inner_margin(ui.spacing().window_margin)
                .outer_margin(ui.spacing().window_margin)
                .show(ui, |ui| {
                    self.phoneme_panel(state, ui);
                });
        });
    }
}