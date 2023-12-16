use crate::widgets;
use crate::{GroupKey, Selection};

#[derive(Default)]
pub struct GroupPane {
    group_active: Option<GroupKey>,
    group_editor_state: widgets::EditorState<GroupKey>,
}

impl super::Pane for GroupPane {
    fn name(&self) -> &'static str { "Groups" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        widgets::group_selection_list(
            ui, 
            &mut state.focus, 
            state.language.groups_mut(), 
            &mut self.group_editor_state, 
            Selection::Single(&mut self.group_active),
        );
    }
}