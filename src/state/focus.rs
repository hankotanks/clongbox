use std::collections::BTreeSet;
use std::mem;

use crate::{sc, CONFIG, status};
use crate::{GroupKey, PhonemeKey, PhonemeSrc};

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum FocusTarget {
    Sc { field: sc::Field, head: bool, tail: bool, nested: bool },
    // NOTE: This Option will always be Some, 
    // this allows us to construct a const Discriminant<FocusTarget>
    PhonemeEditorGroups { selected: Option<BTreeSet<GroupKey>> },
    PhonemeEditorSelect,
    GroupEditorSelect,
}

impl FocusTarget {
    pub fn is_valid(&self, buffer: &FocusBuffer) -> bool {
        match self {
            FocusTarget::Sc { field, nested, head, tail } => {
                match field {
                    sc::Field::Target => match *buffer {
                        FocusBuffer::Phoneme { src: PhonemeSrc::Language, .. } => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        _ => false,
                    },
                    sc::Field::Replacement => match *buffer {
                        FocusBuffer::Phoneme { .. } => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        _ => false,
                    },
                    sc::Field::EnvStart { has_boundary } => match *buffer {
                        FocusBuffer::Phoneme { src: PhonemeSrc::Language, .. } => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        FocusBuffer::Boundary if *head && !has_boundary => true,
                        _ => false,
                    },
                    sc::Field::EnvEnd { has_boundary } => match *buffer {
                        FocusBuffer::Phoneme { src: PhonemeSrc::Language, .. } => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        FocusBuffer::Boundary if *tail && !has_boundary => true,
                        _ => false,
                    },
                }
            },
            FocusTarget::PhonemeEditorGroups { selected } //
                if let FocusBuffer::Group(key) = buffer => //
                    !selected.as_ref().unwrap().contains(key),
            FocusTarget::PhonemeEditorGroups { .. } => false,
            FocusTarget::PhonemeEditorSelect //
                if matches!(buffer, FocusBuffer::Phoneme { .. }) => true,
            FocusTarget::PhonemeEditorSelect => false,
            FocusTarget::GroupEditorSelect //
                if matches!(buffer, FocusBuffer::Group(_)) => true,
            FocusTarget::GroupEditorSelect => false,
        }
    }
}

#[derive(Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum FocusBuffer {
    Phoneme { key: PhonemeKey, src: PhonemeSrc },
    Group(GroupKey),
    Any,
    Boundary,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Focus {
    Active { 
        id: egui::Id, 
        target: FocusTarget, 
        buffer: Option<FocusBuffer>,
        fst: bool,
    },
    None,
}

impl Default for Focus {
    fn default() -> Self {
        Self::None
    }
}

impl Focus {
    pub fn needs(&self, disc: mem::Discriminant<FocusTarget>) -> bool {
        match self {
            Focus::Active { target, buffer, .. } //
                if disc == mem::discriminant(target) => buffer.is_none(),
            _ => false,
        }
    }

    pub fn set(&mut self, id: egui::Id, target: FocusTarget) {
        let focus = Self::Active { id, target, buffer: None, fst: true };

        let _ = mem::replace(self, focus);
    }

    pub fn set_buffer(&mut self, id: egui::Id, buffer: FocusBuffer) {
        if let Self::Active { id: id_curr, buffer: buffer_curr, .. } = self {
            if id == *id_curr {
                let _  = buffer_curr.insert(buffer);
            }
        }
    }

    pub fn clear(&mut self) {
        let _ = mem::take(self);
    }

    pub fn take(&mut self, from: egui::Id) -> Option<FocusBuffer> {
        match self {
            Focus::Active { id, buffer, .. } if *id == from => 
                buffer.take(),
            _ => None,
        }
    }

    pub fn take_if_matches(&mut self, disc: mem::Discriminant<FocusTarget>) -> Option<FocusBuffer> {
        if let Self::Active { target, buffer, .. } = self {
            if disc == mem::discriminant(target) {
                buffer.take()
            } else { None }
        } else { None }
    }

    /// This function requires a contract that the caller will not attach
    /// any event handlers to the items added in `widgets`. 
    /// If no selection is ongoing, the response is returned, allowing the user
    /// to add their own handlers
    pub fn show_if_valid(
        &mut self,
        buffer_temp: FocusBuffer,
        ui: &mut egui::Ui,
        mut widget: impl FnMut(&mut egui::Ui) -> egui::Response,
    ) -> Option<egui::Response> {
        let response = (widget)(ui);

        match self {
            Focus::Active { target, buffer, fst, .. } => {
                if target.is_valid(&buffer_temp) {
                    if !*fst {
                        ui.painter().rect_stroke(
                            response.rect, 
                            CONFIG.selection_rounding, 
                            CONFIG.selection_stroke
                        );

                        let status_message = format!("Click to {}", 
                            match (target, buffer_temp) {
                                (FocusTarget::Sc { .. }, FocusBuffer::Phoneme { .. }) => //
                                    "add this phoneme to the selected sound change",
                                (FocusTarget::Sc { .. }, FocusBuffer::Group(_)) => //
                                    "add this group to the selected sound change",
                                (FocusTarget::Sc { .. }, FocusBuffer::Any) => //
                                    "add a pair of brackets [ ] to the selected sound change",
                                (FocusTarget::Sc { .. }, FocusBuffer::Boundary) => //
                                    "add a word boundary # to the selected sound change",
                                (FocusTarget::PhonemeEditorGroups { .. }, FocusBuffer::Phoneme { .. }) => unreachable!(),
                                (FocusTarget::PhonemeEditorGroups { .. }, FocusBuffer::Group(_)) => //
                                    "insert the selected phoneme into this group",
                                (FocusTarget::PhonemeEditorGroups { .. }, FocusBuffer::Any) => unreachable!(),
                                (FocusTarget::PhonemeEditorGroups { .. }, FocusBuffer::Boundary) => unreachable!(),
                                (FocusTarget::PhonemeEditorSelect, FocusBuffer::Phoneme { .. }) => //
                                    "edit this phoneme",
                                (FocusTarget::PhonemeEditorSelect, FocusBuffer::Group(_)) => unreachable!(),
                                (FocusTarget::PhonemeEditorSelect, FocusBuffer::Any) => unreachable!(),
                                (FocusTarget::PhonemeEditorSelect, FocusBuffer::Boundary) => unreachable!(),
                                (FocusTarget::GroupEditorSelect, FocusBuffer::Phoneme { .. }) => //
                                    "edit this group",
                                (FocusTarget::GroupEditorSelect, FocusBuffer::Group(_)) => unreachable!(),
                                (FocusTarget::GroupEditorSelect, FocusBuffer::Any) => unreachable!(),
                                (FocusTarget::GroupEditorSelect, FocusBuffer::Boundary) => unreachable!(),
                        });

                        status::set_on_hover(&response, status_message);
                    }
        
                    if response.interact(egui::Sense::click()).clicked() {
                        let _ = buffer.insert(buffer_temp);
                    }
        
                    None
                } else {
                    Some(response)
                }
            },
            Focus::None => Some(response),
        }
    }
}