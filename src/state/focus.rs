use std::mem;

use crate::{sc, CONFIG};
use crate::{GroupKey, PhonemeKey, PhonemeSrc};

#[derive(Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum FocusTarget {
    Sc { field: sc::Field, head: bool, tail: bool, nested: bool },
    NewPhonemeGroups,
    None,
}

impl Default for FocusTarget {
    fn default() -> Self {
        Self::None
    }
}

impl FocusTarget {
    pub fn is_valid(&self, buffer: &FocusBuffer) -> bool {
        match *self {
            FocusTarget::Sc { field, nested, head, tail } => {
                match field {
                    sc::Field::Target => match *buffer {
                        FocusBuffer::Phoneme { src, .. } //
                            if matches!(src, PhonemeSrc::Language) => true,
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
                        FocusBuffer::Phoneme { src, .. } //
                            if matches!(src, PhonemeSrc::Language) => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        FocusBuffer::Boundary if head && !has_boundary => true,
                        _ => false,
                    },
                    sc::Field::EnvEnd { has_boundary } => match *buffer {
                        FocusBuffer::Phoneme { src, .. } //
                            if matches!(src, PhonemeSrc::Language) => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        FocusBuffer::Boundary if tail && !has_boundary => true,
                        _ => false,
                    },
                }
            },
            FocusTarget::NewPhonemeGroups //
                if matches!(buffer, FocusBuffer::Group(_)) => true,
            FocusTarget::NewPhonemeGroups => false,
            FocusTarget::None => false,
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
        buffer: Option<FocusBuffer> 
    },
    None,
}

impl Default for Focus {
    fn default() -> Self {
        Self::None
    }
}

impl Focus {
    pub fn set(&mut self, id: egui::Id, target: FocusTarget) {
        let focus = Self::Active { id, target, buffer: None };

        let _ = mem::replace(self, focus);
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
            Focus::Active { target, buffer, .. } => {
                if target.is_valid(&buffer_temp) {
                    ui.painter().rect_stroke(
                        response.rect, 
                        CONFIG.selection_rounding, 
                        CONFIG.selection_stroke
                    );
        
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