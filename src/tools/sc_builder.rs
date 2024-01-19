use crate::{sc::SoundChange, layout};

#[derive(Default)]
pub struct ScBuilderTool {
    active: Option<usize>,
}

impl super::Tool for ScBuilderTool {
    fn name(&self) -> &'static str { "Sound Changes" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let response = layout::button_context_line(ui, [
            layout::BtnContextElem::Button("Add"),
            layout::BtnContextElem::Label("a new sound change")
        ]).unwrap();

        if response.clicked() {
            self.active = Some(state.sound_changes.len());

            state.sound_changes.push(SoundChange::default());
        }

        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().button_padding.x);

            layout::button_context_line(ui, [
                layout::BtnContextElem::Label("Insert"),
                layout::BtnContextElem::Enabled("[  ]", false),
                layout::BtnContextElem::Label("or"),
                layout::BtnContextElem::Enabled("#", false),
                layout::BtnContextElem::Label("at the selected location")
            ]);
        });
        
    }
}