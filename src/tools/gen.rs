#[derive(Default)]
pub struct GenTool;

impl super::Tool for GenTool {
    fn name(&self) -> &'static str { "Word Generation" }

    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}