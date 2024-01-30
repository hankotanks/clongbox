mod sc_field_editor;

use once_cell::sync::{Lazy, OnceCell};

use crate::{layout, sc, Focus};
use crate::app::fonts;

#[derive(Default)]
pub struct ScaTool {
    active: Option<usize>,
    active_scroll_to_bottom: bool,
}

impl ScaTool {
    fn show_sc_selector(&mut self, state: &mut crate::State, ui: &mut egui::Ui, idx: usize) {
        let crate::State { 
            language, 
            rep_phonemes, 
            sound_changes, .. 
        } = state;
    
        let content = sound_changes[idx].as_str(language, rep_phonemes);
        let content = egui::RichText::new(content)
            .font(fonts::FONT_ID.to_owned())
            .extra_letter_spacing(ui.painter().round_to_pixel(4.));

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

        let response = layout::button_context_line(ui, [
            layout::BtnContextElem::Label("Insert"),
            layout::BtnContextElem::Enabled("[  ]", true),
            layout::BtnContextElem::Label("or"),
            layout::BtnContextElem::Enabled("#", true),
            layout::BtnContextElem::Label("at the selected location")
        ]);

        if let Some(response) = response.get(0) {
            if response.clicked() {
                println!("brackets");
            }
        }

        if let Some(response) = response.get(1) {
            if response.clicked() {
                println!("boundary");
            }
        }

        ui.separator();

        let height = ui.text_style_height(&egui::TextStyle::Body) * 2. + //
            fonts::FONT_ID.size * 2. + //
            ui.spacing().button_padding.y * 4. + //
            ui.spacing().item_spacing.y * 10.;

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

                    match self.active {
                        Some(idx) => {
                            let crate::State {
                                sound_changes,
                                rep_phonemes,
                                language, 
                                focus, ..
                            } = state;

                            let sound_change = &mut sound_changes[idx];
                            let sound_change = sound_change.as_mut(language, rep_phonemes);
                            
                            layout::hungry_frame_bottom_up(ui, |ui| {
                                sc_field_editor::show_sc_editor(ui, sound_change, focus);
                            });
                        },
                        None => {
                            ui.centered_and_justified(|ui| {
                                ui.heading("Select a sound change to edit it");
                            });
                        },
                    }
                });
            });
    }
}