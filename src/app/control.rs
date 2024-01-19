use crate::{ToolId, editors};

pub struct Control<'a> {
    pub(super) tool_active: &'a mut usize,
    pub(super) editors_active: &'a mut Option<editors::EditorKey>,
}

impl<'a> Control<'a> {
    pub fn set_tool(&mut self, id: ToolId) {
        *(self.tool_active) = id as usize;
    }

    pub fn show_editor(&mut self, editor_key: editors::EditorKey) {
        let _ = self.editors_active.insert(editor_key);
    }
}