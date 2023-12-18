use crate::ToolId;

pub struct Control<'a> {
    pub(super) tool_active: &'a mut usize,
}

impl<'a> Control<'a> {
    pub fn set_tool(&mut self, id: ToolId) {
        *(self.tool_active) = id as usize;
    }
}