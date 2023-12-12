#[derive(Default)]
pub struct WordEvoTool;

impl super::Tool for WordEvoTool {
    fn name(&self) -> &'static str { "Word Evolution" }

    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}