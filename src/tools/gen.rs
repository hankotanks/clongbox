use std::{mem, ops, sync};

use once_cell::sync::OnceCell;
use rand::seq::{IteratorRandom, SliceRandom};

use crate::app::fonts;
use crate::language::{Language, PhonemeRef};
use crate::{widgets, Focus, FocusBuffer, FocusTarget};
use crate::{Syllable, SyllableRefMut, SyllabicElement};
use crate::CONFIG;

#[derive(Clone, Copy)]
pub struct GenToolSettings {
    prob_mono: f64,
    prob_dropoff: f64,
    batch_size: usize,
}

impl Default for GenToolSettings {
    fn default() -> Self {
        Self {
            prob_mono: 0.15,
            prob_dropoff: 0.,
            batch_size: 50,
        }
    }
}

#[derive(Default)]
pub struct GenTool {
    syllable_temp: Syllable,
    settings: GenToolSettings,
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
            ui.label(fonts::ipa_rt("\u{2205}").color(ui.visuals().error_fg_color))
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

    fn syllable_selection_panel(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
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

                        let mut idx = 0;

                        while idx < phonotactics.len() {
                            if phonotactics[idx].is_empty() {
                                phonotactics.remove(idx);
                            } else {
                                let activate = activate && idx == phonotactics.len() - 1;

                                let syllable = SyllableRefMut {
                                    idx,
                                    syllable: &mut phonotactics[idx],
                                    language,
                                };
                        
                                ui.horizontal(|ui| {
                                    Self::syllable_selector(syllable, ui, focus, activate);
                                });

                                idx += 1;
                            }
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

impl super::Tool for GenTool {
    fn name(&self) -> &'static str { "Word Generation" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        let prob_mono_slider = egui::Slider::new(
            &mut self.settings.prob_mono, 
            ops::RangeInclusive::new(0., 1.)
        ).custom_formatter(|n, _| {
            fn contains(start: f64, end: f64, n: f64) -> bool {
                ops::RangeInclusive::new(start, end).contains(&n)
            }

            let content = if n == 0. {
                "Never"
            } else if contains(0., 0.25, n) {
                "Rare"
            } else if contains(0.25, 0.50, n) {
                "Less Frequent"
            } else if contains(0.50, 0.75, n) {
                "Frequent"
            } else if contains(0.75, 1., n) {
                "Mostly"
            } else if n == 1. {
                "Always"
            } else {
                unreachable!();
            };
            
            String::from(content)
        });

        ui.label("Monosyllables");

        ui.add(prob_mono_slider);

        let prob_dropoff_slider = egui::Slider::new(
            &mut self.settings.prob_dropoff,
            ops::RangeInclusive::new(0., 0.3)
        ).custom_formatter(|n, _| {
            fn contains(start: f64, end: f64, n: f64) -> bool {
                ops::RangeInclusive::new(start, end).contains(&n)
            }

            let content = if n == 0. {
                "Equiprobable"
            } else if contains(0., 0.1, n) {
                "Slow"
            } else if contains(0.1, 0.2, n) {
                "Medium"
            } else if contains(0.2, 0.3, n) {
                "Fast"
            } else {
                unreachable!();
            };

            String::from(content)
        });

        ui.label("Dropoff");

        ui.vertical_centered_justified(|ui| {
            // TODO: Enable this widget when phoneme re-ordering is implemented
            ui.add_enabled(false, prob_dropoff_slider)
                .on_disabled_hover_text("Phoneme dropoff not yet implemented");
        });

        ui.separator();

        static BOTTOM_PANEL_HEIGHT: OnceCell<f32> = OnceCell::new();

        let _ = BOTTOM_PANEL_HEIGHT.set({
            ui.text_style_height(&egui::TextStyle::Button) + //
            ui.spacing().button_padding.y * 2. + //
            ui.spacing().item_spacing.y * 2.
        });

        egui_extras::StripBuilder::new(ui)
            .size(egui_extras::Size::remainder())
            .size(egui_extras::Size::exact(*BOTTOM_PANEL_HEIGHT.get().unwrap()))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    self.syllable_selection_panel(state, ui);
                });

                strip.cell(|ui| {
                    let crate::State { 
                        phonotactics, 
                        word_gen_batch,
                        language, .. 
                    } = state;

                    // If there's at least one invalid
                    let invalid = phonotactics
                        .iter()
                        .any(|s| !s.is_valid());

                    // If there's at least one valid
                    let enabled = phonotactics
                        .iter()
                        .any(Syllable::is_valid);

                    ui.horizontal(|ui| {
                    let response = ui.add_enabled(
                        enabled,
                        egui::Button::new("Generate Batch")
                    );

                    let warning = if invalid && enabled {
                        "Invalid syllables will be skipped"
                    } else if phonotactics.is_empty() {
                        "Can't generate words without rules"
                    } else if !enabled && invalid && !phonotactics.is_empty() {
                        "Must have at least one valid rule"
                    } else {
                        ""
                    };

                    let warning = egui::RichText::new(warning)
                        .color(ui.visuals().warn_fg_color);

                    ui.add_enabled(false, egui::Label::new(warning));
                    //ui.label(warning);

                    if response.clicked() {
                        generate_batch(self.settings, word_gen_batch, phonotactics, language);
                    }
                });
            });
        });
    }
}

fn generate_syllable(
    _settings: GenToolSettings,
    word: &mut String,
    phonotactics: &[Syllable],
    language: &Language,
) {
    match phonotactics.choose(&mut rand::thread_rng()) {
        Some(syllable) => {
            if !syllable.is_valid() {
                generate_syllable(_settings, word, phonotactics, language);
            }

            let Syllable { elems, .. } = syllable;

            // TODO: We don't need to be creating String instances here
            fn phoneme_content(phoneme_ref: PhonemeRef<'_>) -> String {
                let PhonemeRef {
                    phoneme, 
                    grapheme, .. 
                } = phoneme_ref;

                let content = match grapheme {
                    Some(grapheme) => grapheme,
                    None => phoneme,
                };

                format!("{}", content)
            }

            for elem in elems.iter().copied() {
                // TODO: Reason about whether these unwraps are safe
                let elem_raw = match elem {
                    SyllabicElement::Phoneme(key) => {
                        let phoneme = language.phoneme_ref(key).unwrap();

                        phoneme_content(phoneme)
                    },
                    SyllabicElement::Group(key) => {
                        let group = language.group_ref(key).unwrap();

                        match group.phonemes.choose(&mut rand::thread_rng()) {
                            Some(phoneme) => phoneme_content(phoneme),
                            None => String::from(""),
                        }
                    },
                    SyllabicElement::Invalid => unreachable!(),
                };

                word.push_str(&elem_raw);
            }
        },
        None => unreachable!(),
    }
}

fn generate_word(
    settings: GenToolSettings,
    phonotactics: &[Syllable],
    language: &Language
) -> sync::Arc<str> {
    // TODO: Rudimentary

    let GenToolSettings { prob_mono, .. } = settings;

    let mut word = String::from("");

    if rand::random::<f64>() < prob_mono {
        generate_syllable(settings, &mut word, phonotactics, language)
    } else {
        loop {
            generate_syllable(settings, &mut word, phonotactics, language);

            // TODO: Magic number, maybe add a slider?
            if rand::random::<f64>() < 0.5 {
                break;
            }
        }
    }

    sync::Arc::from(word)
}

fn generate_batch(
    settings: GenToolSettings, 
    batch: &mut Vec<sync::Arc<str>>,
    phonotactics: &[Syllable],
    language: &Language,
) {
    batch.clear();

    let GenToolSettings { batch_size, .. } = settings;

    for _ in 0..batch_size {
        let word = generate_word(settings, phonotactics, language);

        if !word.is_empty() { 
            batch.push(word);
        }
    }
}