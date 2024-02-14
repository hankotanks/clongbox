use std::{mem, ops};

use once_cell::sync::{Lazy, OnceCell};

use crate::app::fonts;
use crate::{Focus, FocusBuffer, FocusTarget};
use crate::{Syllable, SyllableRefMut, SyllabicElement};
use crate::CONFIG;

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
    fn syllable_selector_inner(syllable: SyllableRefMut<'_>, ui: &mut egui::Ui, focus: &mut Focus, id: egui::Id) {
        let SyllableRefMut { syllable, language } = syllable;

        let show_invalid = |ui: &mut egui::Ui| {
            ui.label("\u{2205}");
        };
        
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
    }

    fn syllable_selector(syllable: SyllableRefMut<'_>, ui: &mut egui::Ui, focus: &mut Focus) {
        let is_empty = syllable.syllable.elems.is_empty();

        let rect_full = ui.available_rect_before_wrap();

        let response = egui::Frame::default().show(ui, |ui| {
            let id = ui.id();

            Self::syllable_selector_inner(syllable, ui, focus, id);

            id
        });

        let egui::InnerResponse::<egui::Id> { 
            response, 
            inner, ..
        } = response;

        let egui::Response { mut rect, .. } = response;

        let rect_full = egui::Rect {
            min: egui::Pos2 { x: rect_full.left(), y: rect.top(), },
            max: egui::Pos2 { x: rect_full.right(), y: rect.bottom(), },
        };

        let hovered = match ui.ctx().pointer_latest_pos() {
            Some(pos) => rect_full.contains(pos),
            None => response.hovered(),
        };

        let is_focused = focus.get_id() == inner;

        if is_empty {
            ui.label("NEW");
        }

        if hovered || is_empty || is_focused {
            let temp = if is_focused {
                let temp = ui.toggle_value(&mut true, "+");

                if temp.clicked() {
                    focus.clear();
                }

                temp
            } else {
                let temp = ui.button("+");

                if temp.clicked() {
                    focus.set(inner, FocusTarget::SyllableGroup);
                }

                temp
            };

            rect.extend_with_x(temp.rect.right());
        }

        if (hovered || is_focused) && !is_empty {
            let rect = rect.expand2({
                egui::Vec2 { x: ui.spacing().item_spacing.x, y: 0., }
            });

            let stroke = ui.visuals().window_stroke;

            ui.painter().rect_stroke(rect, CONFIG.selection_rounding, stroke);
        }
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

        static PADDING: OnceCell<f32> = OnceCell::new();

        let _ = PADDING.set(ui.spacing().item_spacing.x * 0.);

        egui_extras::StripBuilder::new(ui)
            .size(egui_extras::Size::exact(*PADDING.get().unwrap()))
            .size(egui_extras::Size::remainder())
            .horizontal(|mut strip| {
                strip.empty();

                strip.cell(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let crate::State { 
                            phonotactics, 
                            language, 
                            focus, .. 
                        } = state;
            
                        for syllable in phonotactics.iter_mut() {
                            let syllable = SyllableRefMut {
                                syllable,
                                language,
                            };
                    
                            ui.horizontal(|ui| {
                                Self::syllable_selector(syllable, ui, focus);
                            });
                        }
                    });
                });
            });
    }
}