use crate::{widgets, layout};
use crate::{GroupKey, Selection};
use crate::{PhonemeSrc, PhonemeKey};

#[derive(Default)]
pub struct GroupPane {
    group_active: Option<GroupKey>,
    group_editor_state: widgets::EditorState<GroupKey>,
    group_editor_state_heading: widgets::EditorState<GroupKey>,
    
    phoneme_editor_state: widgets::EditorState<PhonemeKey>,
}

impl GroupPane {
    fn group_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let group_active_prev = self.group_active;

        widgets::group_selection_list(
            ui, 
            &mut state.focus, 
            state.language.groups_mut(), 
            &mut self.group_editor_state, 
            Selection::Single(&mut self.group_active),
        );

        if self.group_active != group_active_prev {
            self.group_editor_state_heading = widgets::EditorState::None;
        }
    }

    fn phoneme_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let layout = egui::Layout::top_down(egui::Align::LEFT);

        layout::hungry_frame_with_layout(ui, layout, |ui| {
            match self.group_active {
                Some(key) => {
                    widgets::group_editor_heading(
                        ui, 
                        &mut state.focus, 
                        state.language.group_ref_mut(key), 
                        &mut self.group_editor_state_heading, 
                        &mut Selection::None
                    );
                },
                None => {
                    let content = ui.text_style_height(&egui::TextStyle::Heading);
                    let content = egui::RichText::new("All Phonemes")
                        .font(egui::FontId::proportional(content));

                    let header = egui::Button::new(content)
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::default())
                        .selected(false);

                    ui.add_enabled(false, header);
                },
            }

            ui.separator();

            let phonemes = match self.group_active {
                Some(key) => state.language.phonemes_mut(key),
                None => state.language.phonemes_mut_all(),
            };

            widgets::phoneme_selection_list(
                ui, 
                &mut state.focus, 
                phonemes,
                &mut self.phoneme_editor_state, 
                PhonemeSrc::Language, 
                Selection::None,
            );
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