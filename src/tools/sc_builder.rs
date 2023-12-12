#[derive(Default)]
pub struct ScBuilderTool;

impl super::Tool for ScBuilderTool {
    fn name(&self) -> &'static str { "Sound Changes" }
    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}