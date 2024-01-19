mod phoneme_editor;
mod group_editor;

use std::{mem, fmt};

use once_cell::unsync::OnceCell;

use crate::{FocusTarget, FocusBuffer};

#[derive(Clone, Copy, PartialEq)]
#[derive(enum_map::Enum, enum_iterator::Sequence)]
pub enum EditorKey {
    Phoneme,
    Group,
}

impl fmt::Display for EditorKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            EditorKey::Phoneme => "Phoneme",
            EditorKey::Group => "Group",
        })
    }
}

pub trait Editor {
    fn target(&self) -> mem::Discriminant<FocusTarget>;
    fn set(&mut self, state: &mut crate::State, buffer: FocusBuffer);
    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui);
}

pub fn editor_update(editor: &mut (impl Editor + ?Sized), state: &mut crate::State) {
    if let Some(buffer) = state.focus.take_if_matches(editor.target()) {
        editor.set(state, buffer);

        state.focus.clear();
    }
}

pub fn editors() -> enum_map::EnumMap<EditorKey, OnceCell<Box<dyn Editor>>> {
    let helper = |editor| {
        let cell = OnceCell::default();

        let _ = cell.set(editor);

        cell
    };

    enum_map::enum_map! {
        // TODO: This might want to be a macro
        EditorKey::Phoneme => helper(Box::new(phoneme_editor::PhonemeEditor::default()) as Box<dyn Editor>),
        EditorKey::Group => helper(Box::new(group_editor::GroupEditor::default()) as Box<dyn Editor>),
    }
}