use std::collections::BTreeSet;
use std::mem;

use once_cell::sync::Lazy;
use slotmap::Key;

use crate::{sc, CONFIG};
use crate::{GroupKey, PhonemeKey, PhonemeSrc};

pub enum FocusTarget {
    Sc { 
        field: sc::Field,
        head: bool, 
        tail: bool, 
        nested: bool 
    },

    // NOTE: This Option will always be Some, 
    // this allows us to construct a const Discriminant<FocusTarget>
    PhonemeEditorGroups { selected: Option<BTreeSet<GroupKey>> },

    PhonemeEditorSelect,

    GroupEditorSelect,

    // TODO: Temporary implementation of syllable constructor selection
    // In the future, this needs to support other things like Phonemes
    SyllableGroup,
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
                if matches!(buffer, FocusBuffer::Phoneme { 
                    src: PhonemeSrc::Language, .. 
                }) => true,
            FocusTarget::PhonemeEditorSelect => false,
            FocusTarget::GroupEditorSelect //
                if matches!(buffer, FocusBuffer::Group(_)) => true,
            FocusTarget::GroupEditorSelect => false,
            FocusTarget::SyllableGroup //
                if let FocusBuffer::Group(_) = buffer => true,
            FocusTarget::SyllableGroup => false,
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

// TODO: I don't like this solution because it has to be maintained 
// if new `FocusBuffer` variants are added
// This is not a problem for `FocusBuffer::as_str_plural` because
// it will fail to compile upon addition
static FOCUS_BUFFER_TESTS: Lazy<[FocusBuffer; 5]> = Lazy::new(|| [
    FocusBuffer::Phoneme { 
        key: PhonemeKey::null(), 
        src: PhonemeSrc::Language 
    },
    FocusBuffer::Phoneme { 
        key: PhonemeKey::null(), 
        src: PhonemeSrc::Rep 
    },
    FocusBuffer::Group(GroupKey::null()),
    FocusBuffer::Any,
    FocusBuffer::Boundary,
]);

impl FocusBuffer {
    // NOTE: This is not implemented as `fmt::Display` b/c it should be obvious that it's plural
    // Not for general use, just for the focus status message
    fn as_str_plural(&self) -> &'static str {
        match self {
            FocusBuffer::Phoneme { src: PhonemeSrc::Language, .. } => "phonemes",
            FocusBuffer::Phoneme { src: PhonemeSrc::Rep, .. } => "new phonemes",
            FocusBuffer::Group(_) => "groups",
            FocusBuffer::Any => "nonce categories [ ]",
            FocusBuffer::Boundary => "word boundaries",
        }
    }
}

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

    pub fn get_id(&self) -> egui::Id {
        match self {
            Focus::Active { id, .. } => *id,
            Focus::None => egui::Id::NULL,
        }
    }

    pub fn get_target(&self) -> Option<&FocusTarget> {
        if let Self::Active { target, .. } = self {
            Some(target)
        } else {
            None
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

    pub fn get_focus_status(&self) -> Option<String> {
        match self {
            Focus::Active { target, .. } => {
                let mut valid = BTreeSet::default();

                for buffer in FOCUS_BUFFER_TESTS.iter() {
                    if target.is_valid(buffer) {
                        valid.insert(buffer.as_str_plural());
                    }
                }

                let len = valid.len();

                let mut status = String::from("Selecting ");

                for (idx, valid_buffer) in valid.into_iter().enumerate() {
                    let content = if idx == 0 && idx == len - 1 {
                        format!("{}. ", valid_buffer)
                    } else if idx == 0 {
                        String::from(valid_buffer)
                    } else if idx == len - 1 {
                        format!(" & {}. ", valid_buffer)
                    } else {
                        format!(", {}", valid_buffer)
                    };

                    status.push_str(content.as_str());
                }

                status.push_str("Press ESC to cancel");

                Some(status)
            },
            Focus::None => None,
        }
    }
}