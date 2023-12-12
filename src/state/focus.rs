use crate::{sc, PhonemeKey, GroupKey};

#[derive(Clone, Copy, Debug)]
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
                        FocusBuffer::Phoneme { rep, .. } if !rep => true,
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
                    sc::Field::EnvStart => match *buffer {
                        FocusBuffer::Phoneme { rep, .. } if !rep => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        FocusBuffer::Boundary if head => true,
                        _ => false,
                    },
                    sc::Field::EnvEnd => match *buffer {
                        FocusBuffer::Phoneme { rep, .. } if !rep => true,
                        FocusBuffer::Group(_) => true,
                        FocusBuffer::Any if !nested => true,
                        FocusBuffer::Boundary if tail => true,
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

#[derive(Clone, Copy, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum FocusBuffer {
    Phoneme { key: PhonemeKey, rep: bool },
    Group(GroupKey),
    Any,
    Boundary,
}

#[derive(Default, Debug)]
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

    pub fn show_if_valid<R>(
        &mut self, 
        buffer: FocusBuffer,
        ui: &mut egui::Ui,
        valid: egui::RichText,
        mut invalid: impl FnMut(&mut egui::Ui) -> R,
    ) {
        if self.target.is_valid(&buffer) {
            if ui.button(valid).clicked() {
                let _ = self.buffer.insert(buffer);
            }
        } else {
            (invalid)(ui);
        }
    }
}