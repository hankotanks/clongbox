mod groups;
mod lexicon;
mod phonemes;

use once_cell::unsync::OnceCell;

use crate::{State, Control};

#[repr(usize)]
#[derive(Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(enum_iterator::Sequence)]
pub enum PaneId { Groups, Phonemes, Lexicon, }

#[allow(clippy::from_over_into)]
impl Into<Box<dyn Pane>> for PaneId {
    fn into(self) -> Box<dyn Pane> {
        #[allow(clippy::default_constructed_unit_structs)]
        match self {
            PaneId::Groups => Box::from(groups::GroupPane::default()) //
                as Box<dyn Pane + 'static>,
            PaneId::Phonemes => Box::from(phonemes::PhonemePane::default()) //
                as Box<dyn Pane + 'static>,
            PaneId::Lexicon => Box::from(lexicon::LexiconPane::default()) //
                as Box<dyn Pane + 'static>,
        }
    }
}
pub trait Pane: Send + Sync {
    fn name(&self) -> &'static str;
    fn show(&mut self, control: Control<'_>, state: &mut State, ui: &mut egui::Ui);
}

pub fn panes<const P: usize>() -> [OnceCell<Box<dyn Pane + 'static>>; P] {
    let panes: [OnceCell<Box<dyn Pane + 'static>>; P] = //
        [(); P].map(|_| OnceCell::default());

    for id in enum_iterator::all::<PaneId>() {
        let _ = panes[id as usize].set(id.into());
    }

    panes
}