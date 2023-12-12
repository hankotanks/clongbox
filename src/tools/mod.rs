mod gen;
mod new_phoneme;
mod sc_builder;
mod evo;

use once_cell::unsync::OnceCell;

use crate::State;

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui);
}

pub fn tools<const T: usize>() -> [OnceCell<Box<dyn Tool + 'static>>; T] {
    let tools: [OnceCell<Box<dyn Tool + 'static>>; T] = //
        [(); T].map(|_| OnceCell::default());

    let _ = tools[0].set(Box::from(gen::GenTool::default()) //
        as Box<dyn Tool + 'static>);

    let _ = tools[1].set(Box::from(new_phoneme::NewPhonemeTool::default()) //
        as Box<dyn Tool + 'static>);

    let _ = tools[2].set(Box::from(sc_builder::ScBuilderTool::default()) //
        as Box<dyn Tool + 'static>);

    let _ = tools[3].set(Box::from(evo::WordEvoTool::default()) //
        as Box<dyn Tool + 'static>);

    tools
}