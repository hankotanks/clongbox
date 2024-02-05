use crate::app::fonts;

#[derive(Default)]
pub struct LexiconPane;

impl super::Pane for LexiconPane {
    fn name(&self) -> &'static str { "Lexicon" }

    fn show(
        &mut self, 
        _control: crate::Control<'_>, 
        state: &mut crate::State, 
        ui: &mut egui::Ui
    ) {
        let crate::State { lexicon, .. } = state;

        ui.horizontal_wrapped(|ui| {
            for word in lexicon {
                ui.label(fonts::ipa_rt(&**word));
            }
        });
    }
}