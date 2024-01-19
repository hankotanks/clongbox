use std::mem;

use crate::FocusTarget;

#[derive(Default)]
pub struct GroupEditor;

impl super::Editor for GroupEditor {
    fn target(&self) -> std::mem::Discriminant<crate::FocusTarget> {
        // TODO
        const DISC: mem::Discriminant<FocusTarget> = //
            mem::discriminant(&FocusTarget::GroupEditorSelect);

        DISC
    }

    fn set(&mut self, _state: &mut crate::State, _buffer: crate::FocusBuffer) {
        // TODO
    }

    fn show(&mut self, _state: &mut crate::State, _ui: &mut egui::Ui) {
        // TODO
    }
}