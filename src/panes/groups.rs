#[derive(Default)]
pub struct GroupPane;

impl super::Pane for GroupPane {
    fn name(&self) -> &'static str { "Groups" }

    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}