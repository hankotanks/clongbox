use std::{mem, ops};

use crate::{app::fonts, types::syllable::SyllabicElement, Focus, FocusBuffer, FocusTarget, Syllable, SyllableRefMut};

pub struct GenTool {
    prob_mono: f64,
    prob_dropoff: f64,
}

impl Default for GenTool {
    fn default() -> Self {
        Self {
            prob_mono: 2.5,
            prob_dropoff: 2.5,
        }
    }
}

impl GenTool {
    fn syllable_selector(syllable: SyllableRefMut<'_>, ui: &mut egui::Ui, focus: &mut Focus) {
        let id = ui.id();

        let SyllableRefMut { syllable, language } = syllable;

        let show_invalid = |ui: &mut egui::Ui| {
            ui.label("\u{2205}");
        };

        ui.horizontal(|ui| {
            for element in syllable.elems.iter_mut() {
                match element {
                    SyllabicElement::Phoneme(_) => todo!(),
                    SyllabicElement::Group(key) => {
                        match language.group_ref(*key) {
                            Some(group) => {
                                let content = format!("{}", group.name);
                                let content = fonts::ipa_rt(content);
                                ui.label(content);
                            },
                            None => {
                                let _ = mem::replace(element, SyllabicElement::Invalid);
    
                                (show_invalid)(ui);
                            },
                        }
                    },
                    SyllabicElement::Invalid => (show_invalid)(ui),
                }
            }

            if let Some(buffer) = focus.take(id) {
                if let FocusBuffer::Group(key) = buffer {
                    syllable.elems.push(SyllabicElement::Group(key));
                }
            }
    
            if focus.get_id() == id {
                if ui.toggle_value(&mut true, "+").clicked() {
                    focus.clear();
                }
            } else {
                if ui.button("+").clicked() {
                    focus.set(id, FocusTarget::SyllableGroup);
                }
            }
        });
        

        
    }
}

impl super::Tool for GenTool {
    fn name(&self) -> &'static str { "Word Generation" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let Self {
            prob_mono,
            prob_dropoff, ..
        } = self;

        let prob_mono_slider = egui::Slider::new(
            prob_mono, 
            ops::RangeInclusive::new(0., 6.)
        ).custom_formatter(|n, _| {
            fn contains(start: f64, end: f64, n: f64) -> bool {
                ops::RangeInclusive::new(start, end).contains(&n)
            }

            let content = if contains(0., 1., n) {
                "Never"
            } else if contains(1., 2., n) {
                "Rare"
            } else if contains(2., 3., n) {
                "Less Frequent"
            } else if contains(3., 4., n) {
                "Frequent"
            } else if contains(4., 5., n) {
                "Mostly"
            } else if contains(5., 6., n) {
                "Always"
            } else {
                unreachable!();
            };
            
            String::from(content)
        });

        ui.label("Monosyllables");

        ui.add(prob_mono_slider);

        let prob_dropoff_slider = egui::Slider::new(
            prob_dropoff,
            ops::RangeInclusive::new(0., 4.)
        ).custom_formatter(|n, _| {
            fn contains(start: f64, end: f64, n: f64) -> bool {
                ops::RangeInclusive::new(start, end).contains(&n)
            }

            let content = if contains(0., 1., n) {
                "Equiprobable"
            } else if contains(1., 2., n) {
                "Slow"
            } else if contains(2., 3., n) {
                "Medium"
            } else if contains(3., 4., n) {
                "Fast"
            } else {
                unreachable!();
            };

            String::from(content)
        });

        ui.label("Dropoff");

        ui.vertical_centered_justified(|ui| {
            ui.add(prob_dropoff_slider);
        });

        ui.separator();

        if ui.button("Add").clicked() {
            state.phonotactics.push(Syllable::default());
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let crate::State { phonotactics, language, focus, .. } = state;

            for syllable in phonotactics.iter_mut() {
                let syllable = SyllableRefMut {
                    syllable,
                    language,
                };
        
                Self::syllable_selector(syllable, ui, focus);
            }
        });
    }
}