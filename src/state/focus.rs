use std::mem;

use crate::sc;
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

#[derive(Default)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Focus {
    target: FocusTarget,
    buffer: Option<FocusBuffer>,
}

impl Focus {
    pub fn set(&mut self, target: FocusTarget) {
        self.target = target;

        self.buffer = None;
    }

    pub fn clear(&mut self) {
        self.target = FocusTarget::None;

        self.buffer = None;
    }

    pub fn take(&mut self, request: mem::Discriminant<FocusTarget>) -> Option<FocusBuffer> {
        if mem::discriminant(&self.target) == request {
            self.buffer.take()
        } else {
            None
        }
    }

    pub fn show_if_valid<R>(
        &mut self, 
        buffer: FocusBuffer,
        ui: &mut egui::Ui,
        mut valid: impl FnMut(&mut egui::Ui) -> egui::Response,
        mut invalid: impl FnMut(&mut egui::Ui) -> R,
    ) {
        if self.target.is_valid(&buffer) {
            if (valid)(ui).clicked() {
                let _ = self.buffer.insert(buffer);
            }
        } else {
            (invalid)(ui);
        }
    }
}