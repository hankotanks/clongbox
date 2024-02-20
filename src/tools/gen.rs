use std::{mem, ops};

use once_cell::sync::OnceCell;

use crate::app::fonts;
use crate::{widgets, Focus, FocusBuffer, FocusTarget};
use crate::{Syllable, SyllableRefMut, SyllabicElement};
use crate::CONFIG;

pub struct GenTool {
    syllable_temp: Syllable,
    prob_mono: f64,
    prob_dropoff: f64,
}

impl Default for GenTool {
    fn default() -> Self {
        Self {
            syllable_temp: Syllable::default(),
            prob_mono: 2.5,
            prob_dropoff: 2.5,
        }
    }
}

impl GenTool {
    fn syllable_selector_elems(
        syllable: SyllableRefMut<'_>, 
        ui: &mut egui::Ui, 
        focus: &mut Focus, id: egui::Id
    ) {
        let SyllableRefMut { syllable, language, .. } = syllable;

        // TODO: Placeholder. Should match the Invalid buttons in `ScaTool`
        let show_invalid = |ui: &mut egui::Ui| {
            ui.label(fonts::ipa_rt("\u{2205}"))
        };

        let mut element_to_delete = None;
        
        for (idx, element) in syllable.elems.iter_mut().enumerate() {
            let response = match element {
                SyllabicElement::Phoneme(_) => todo!(),
                SyllabicElement::Group(key) => {
                    match language.group_ref(*key) {
                        Some(group) => {
                            let content = group.name.abbrev().to_string();
                            let content = fonts::ipa_rt(content);

                            ui.label(content)
                        },
                        None => {
                            let _ = mem::replace(element, SyllabicElement::Invalid);

                            (show_invalid)(ui)
                        },
                    }
                },
                SyllabicElement::Invalid => (show_invalid)(ui),
            };

            if widgets::deletion_overlay(&response, ui).clicked {
                let _ = element_to_delete.insert(idx);
            }
        }

        if let Some(idx) = element_to_delete {
            syllable.elems.remove(idx);
        }

        if let Some(FocusBuffer::Group(key)) = focus.take(id) {
            syllable.elems.push(SyllabicElement::Group(key));
        }
    }

    fn syllable_selector(
        syllable: SyllableRefMut<'_>, 
        ui: &mut egui::Ui, 
        focus: &mut Focus, activate: bool
    ) {
        let is_empty = syllable.syllable.elems.is_empty();

        let rect_full = ui.available_rect_before_wrap();

        let response = egui::Frame::default().show(ui, |ui| {
            let id = ui.id().with(syllable.idx);

            Self::syllable_selector_elems(syllable, ui, focus, id);

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

        if activate {
            focus.set(inner, FocusTarget::SyllableGroup);
        }

        let is_focused = activate || focus.get_id() == inner;

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

        let crate::State { 
            phonotactics, 
            language, 
            focus, .. 
        } = state;

        let activate = !self.syllable_temp.is_empty();
        
        if activate {
            let temp = mem::take(&mut self.syllable_temp);

            phonotactics.push(temp);
        }

        //
        static PADDING: OnceCell<f32> = OnceCell::new();
        //
        let _ = PADDING.set(ui.spacing().item_spacing.x * 0.);
        //

        egui_extras::StripBuilder::new(ui)
            .size(egui_extras::Size::exact(*PADDING.get().unwrap()))
            .size(egui_extras::Size::remainder())
            .horizontal(|mut strip| {
                strip.empty();

                strip.cell(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let phonotactics_len = phonotactics.len();

                        for (idx, syllable) in phonotactics.iter_mut().enumerate() {
                            let syllable = SyllableRefMut {
                                idx,
                                syllable,
                                language,
                            };

                            let activate = activate && idx == phonotactics_len - 1;
                    
                            ui.horizontal(|ui| {
                                Self::syllable_selector(syllable, ui, focus, activate);
                            });
                        }

                        let syllable = SyllableRefMut {
                            idx: phonotactics.len(),
                            syllable: &mut self.syllable_temp,
                            language,
                        };

                        if phonotactics_len > 0 {
                            ui.separator();
                        }

                        ui.horizontal(|ui| {
                            Self::syllable_selector(syllable, ui, focus, false);

                            ui.label("Begin building a syllable");
                        });
                    });
                });
            });
    }
}