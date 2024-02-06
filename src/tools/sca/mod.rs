mod sc_editor;

use once_cell::sync::{Lazy, OnceCell};

use crate::{layout, sc};
use crate::FocusBuffer;
use crate::app::fonts;

#[derive(Default)]
pub struct ScaTool {
    active: Option<usize>,
    active_scroll_to_bottom: bool,
}

impl ScaTool {
    fn show_sc_selector(
        &mut self, 
        state: &mut crate::State, 
        ui: &mut egui::Ui, 
        idx: usize
    ) {
        let crate::State { 
            language, 
            rep_phonemes, 
            sound_changes, .. 
        } = state;
    
        let content = sound_changes[idx].as_str(language, rep_phonemes);
        let content = egui::RichText::new(content)
            .font(fonts::FONT_ID.to_owned())
            .extra_letter_spacing(ui.painter().round_to_pixel(4.))
            .color({
                if sound_changes[idx].invalid() {
                    ui.visuals().error_fg_color
                } else {
                    ui.visuals().text_color()
                }
            });

        match self.active {
            Some(idx_curr) if idx_curr == idx => {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut true, content);

                    static LAYOUT: Lazy<egui::Layout> = Lazy::new(|| {
                        egui::Layout::right_to_left(egui::Align::TOP)
                    });

                    ui.with_layout(*LAYOUT, |ui| {
                        static NAV_OFFSET: OnceCell<f32> = OnceCell::new();

                        ui.add_space(*NAV_OFFSET.get_or_init(|| {
                            ui.spacing().scroll.bar_width + //
                            ui.spacing().scroll.bar_inner_margin + //
                            ui.spacing().scroll.bar_outer_margin
                        }));

                        if ui.button(fonts::ipa_rt(" \u{00D7} ")).clicked() {
                            sound_changes.remove(idx_curr);

                            if idx >= sound_changes.len() {
                                if sound_changes.is_empty() || idx == 0 {
                                    let _ = self.active.take();
                                } else {
                                    let _ = self.active.insert(idx.saturating_sub(1));
                                };
                            }
                        }
        
                        let up_decr = idx_curr.saturating_sub(1);
                        
                        if ui.add_enabled(
                            idx != 0, 
                            egui::Button::new(fonts::ipa_rt(" \u{2191} "))
                        ).clicked() && up_decr != idx_curr {
                            let _ = self.active.insert(up_decr);

                            sound_changes.swap(idx_curr, up_decr);
                        }

                        if ui.add_enabled(
                            idx != sound_changes.len().saturating_sub(1), 
                            egui::Button::new(fonts::ipa_rt(" \u{2193} "))
                        ).clicked() {
                            let _ = self.active.insert(idx_curr + 1);

                            sound_changes.swap(idx_curr, idx_curr + 1);
                        }
                    });
                });
            },
            _ => {
                if ui.toggle_value(&mut false, content).clicked() {
                    self.active = Some(idx);
                }
            },
        };
    }

    fn show_sc_editor(&self, ui: &mut egui::Ui, state: &mut crate::State) {
        let Some(idx) = self.active else { unreachable!(); };

        let crate::State {
            focus,
            sound_changes,
            rep_phonemes,
            language, ..
        } = state;

        let sound_change = &mut sound_changes[idx];

        ui.add_space(ui.spacing().item_spacing.y * 2.);
    
        ui.horizontal(|ui| {
            // NOTE: This is to preserve `egui::Align::Center`
            ui.label(fonts::ipa_rt(""));

            sc_editor::show_sc_field(ui, {
                sound_change.as_mut(language, rep_phonemes)
            }, sc::ENV_START, focus);
    
            let content = egui::RichText::new("_")
                .font(fonts::FONT_ID.to_owned());
    
            ui.label(content);

            sc_editor::show_sc_field(ui, {
                sound_change.as_mut(language, rep_phonemes)
            }, sc::ENV_END, focus);
        });
        
        ui.label("Environment");
    
        ui.add_space(ui.spacing().item_spacing.y * 2.);
    
        ui.horizontal(|ui| {
            // NOTE: This is to preserve `egui::Align::Center`
            ui.label(fonts::ipa_rt(""));

            sc_editor::show_sc_field(ui, {
                sound_change.as_mut(language, rep_phonemes)
            }, sc::TARGET, focus);
    
            let content = egui::RichText::new("\u{2192}")
                .font(fonts::FONT_ID.to_owned());
    
            ui.label(content);

            sc_editor::show_sc_field(ui, {
                sound_change.as_mut(language, rep_phonemes)
            }, sc::REPLACEMENT, focus);
        });
    
        egui_extras::StripBuilder::new(ui)
            .sizes(egui_extras::Size::remainder(), 2)
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.label("Target & Replacement");
                });
    
                strip.cell(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        focus.show_if_valid(FocusBuffer::Boundary, ui, |ui| {
                            ui.button("#")
                        });
                        
                        focus.show_if_valid(FocusBuffer::Any, ui, |ui| {
                            ui.button("[  ]")
                        });
                    });
                });
            });
    }
}

impl super::Tool for ScaTool {
    fn name(&self) -> &'static str { "Sound Changes" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        if let Some(response) = layout::button_context_line(ui, [
            layout::BtnContextElem::Button("Add"),
            layout::BtnContextElem::Label("a new sound change")
        ]).get(0) {
            if response.clicked() {
                self.active = Some(state.sound_changes.len());
                self.active_scroll_to_bottom = true;

                state.sound_changes.push(sc::SoundChange::default());
            }
        }

        ui.separator();

        let height = ui.text_style_height(&egui::TextStyle::Body) * 2. + //
            fonts::FONT_ID.size * 2. + //
            ui.spacing().button_padding.y * 4. + //
            ui.spacing().item_spacing.y * 10. + //
            ui.spacing().window_margin.bottom;

        egui_extras::StripBuilder::new(ui)
            .size(egui_extras::Size::remainder())
            .size(egui_extras::Size::exact(height))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    let mut scroller = egui::ScrollArea::vertical()
                        .auto_shrink([false, true]);

                    if self.active_scroll_to_bottom {
                        scroller = scroller.vertical_scroll_offset(ui.available_height());

                        self.active_scroll_to_bottom = false;
                    }
                    
                    scroller.show(ui, |ui| {
                        let mut idx = 0;

                        while idx < state.sound_changes.len() {
                            self.show_sc_selector(state, ui, idx);

                            idx += 1;
                        }
                    });
                });

                strip.cell(|ui| {
                    ui.separator();

                    if self.active.is_some() {
                        layout::hungry_frame_bottom_up(ui, |ui| {
                            ui.add_space(ui.spacing().window_margin.bottom);
                            
                            self.show_sc_editor(ui, state);
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.heading("Select a sound change");
                        });
                    }
                });
            });
    }
}