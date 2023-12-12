mod groups;
mod lexicon;
mod phonemes;

use once_cell::unsync::OnceCell;

use crate::State;

pub trait Pane: Send + Sync {
    fn name(&self) -> &'static str;
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui);
}

pub fn panes<const P: usize>() -> [OnceCell<Box<dyn Pane + 'static>>; P] {
    let panes: [OnceCell<Box<dyn Pane + 'static>>; P] = //
        [(); P].map(|_| OnceCell::default());

    let _ = panes[0].set(Box::new(groups::GroupPane::default()) //
        as Box<dyn Pane + 'static>);

    let _ = panes[1].set(Box::new(lexicon::LexiconPane::default()) //
        as Box<dyn Pane + 'static>);

    let _ = panes[2].set(Box::new(phonemes::PhonemePane::default()) //
        as Box<dyn Pane + 'static>);

    panes
}