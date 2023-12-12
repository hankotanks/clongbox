#[derive(Default)]
pub struct NewPhonemeTool;

impl super::Tool for NewPhonemeTool {
    fn name(&self) -> &'static str { "New Phoneme" }
    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}