#[derive(Default)]
pub struct PhonemePane;

impl super::Pane for PhonemePane {
    fn name(&self) -> &'static str { "Phonemes" }

    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}