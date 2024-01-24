mod gen;
mod sc_builder;
mod evo;

pub mod editors;

use once_cell::unsync::OnceCell;

use crate::State;

#[repr(usize)]
#[derive(Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(enum_iterator::Sequence)]
pub enum ToolId { Gen, ScBuilder, Evo }

#[allow(clippy::from_over_into)]
impl Into<Box<dyn Tool>> for ToolId {
    fn into(self) -> Box<dyn Tool> {
        #[allow(clippy::default_constructed_unit_structs)]
        match self {
            ToolId::Gen => Box::from(gen::GenTool::default()) //
                as Box<dyn Tool + 'static>,
            ToolId::ScBuilder => Box::from(sc_builder::ScBuilderTool::default()) //
                as Box<dyn Tool + 'static>,
            ToolId::Evo => Box::from(evo::WordEvoTool::default()) //
                as Box<dyn Tool + 'static>,
        }
    }
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui);
}

pub fn tools<const T: usize>() -> [OnceCell<Box<dyn Tool + 'static>>; T] {
    let tools: [OnceCell<Box<dyn Tool + 'static>>; T] = //
        [(); T].map(|_| OnceCell::default());

    for id in enum_iterator::all::<ToolId>() {
        let _ = tools[id as usize].set(id.into());
    }

    tools
}