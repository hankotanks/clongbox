#[derive(Default)]
pub struct LexiconPane;

impl super::Pane for LexiconPane {
    fn name(&self) -> &'static str { "Lexicon" }

    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading(self.name());
        });
    }
}