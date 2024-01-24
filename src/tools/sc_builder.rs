use crate::{layout, sc};
use crate::app::fonts;

#[derive(Default)]
pub struct ScBuilderTool {
    active: Option<usize>,
}

#[allow(dead_code)]
fn show_sound_change_element(state: &crate::State, ui: &mut egui::Ui, elem: &sc::Element) {
    match elem {
        sc::Element::Phoneme { key, rep } => {
            let content = match *rep {
                true => &state.rep_phonemes[*key],
                false => &state.language[*key],
            };

            let content = format!("{}", content);
            
            ui.label(content);
        },
        sc::Element::Group(key) => {
            ui.label(format!("{}", state.language[*key].name));
        },
        sc::Element::Boundary => {
            ui.label("#");
        },
        sc::Element::Any(elems) => {
            ui.label("[");

            for elem in elems {
                show_sound_change_element(state, ui, elem);
            }

            ui.label("]");
        },
    }
}

impl ScBuilderTool {
    fn show_sc_selector(&mut self, state: &mut crate::State, ui: &mut egui::Ui, idx: usize) {
        let crate::State { 
            language, 
            rep_phonemes, 
            sound_changes, .. 
        } = state;
    
        let content = sound_changes[idx].as_str(language, rep_phonemes);
        let content = egui::RichText::new(content)
            .font(fonts::FONT_ID.to_owned());

        match self.active {
            Some(idx_curr) if idx_curr == idx => {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut true, content);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
        
                        let up = egui::Button::new(fonts::ipa_rt(" \u{2191} "));
                        let up_enabled = idx != 0;
                        let up_decr = idx_curr.saturating_sub(1);
                        
                        if ui.add_enabled(up_enabled, up).clicked() && up_decr != idx_curr {
                            let _ = self.active.insert(up_decr);

                            sound_changes.swap(idx_curr, up_decr);
                        }

                        let down = egui::Button::new(fonts::ipa_rt(" \u{2193} "));
                        let down_enabled = idx != sound_changes.len().saturating_sub(1);

                        if ui.add_enabled(down_enabled, down).clicked() {
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

    fn show_sc_field(&mut self, ui: &mut egui::Ui, _field: &mut [sc::Element]) {
        ui.label(egui::RichText::new("TODO").font(fonts::FONT_ID.to_owned()).background_color(egui::Color32::GOLD));
    }

    fn show_sc_editor(&mut self, ui: &mut egui::Ui, sound_change: &mut sc::SoundChange) {
        ui.add_space(ui.spacing().item_spacing.y * 2.);

        ui.horizontal(|ui| {
            let field = sc::Field::EnvStart { has_boundary: false };

            self.show_sc_field(ui,&mut sound_change[field] );

            let content = egui::RichText::new("_")
                .font(fonts::FONT_ID.to_owned());

            ui.label(content);

            let field = sc::Field::EnvEnd { has_boundary: false };

            self.show_sc_field(ui, &mut sound_change[field]);
        });

        ui.label("Environment");

        ui.add_space(ui.spacing().item_spacing.y * 2.);

        ui.horizontal(|ui| {
            self.show_sc_field(ui, &mut sound_change[sc::Field::Target]);

            let content = egui::RichText::new("\u{2192}")
                .font(fonts::FONT_ID.to_owned());

            ui.label(content);

            self.show_sc_field(ui, &mut sound_change[sc::Field::Replacement]);
        });

        ui.label("Target & Replacement");
    }
}

impl super::Tool for ScBuilderTool {
    fn name(&self) -> &'static str { "Sound Changes" }

    fn show(&mut self, state: &mut crate::State, ui: &mut egui::Ui) {
        if let Some(response) = layout::button_context_line(ui, [
            layout::BtnContextElem::Button("Add"),
            layout::BtnContextElem::Label("a new sound change")
        ]).get(0) {
            if response.clicked() {
                self.active = Some(state.sound_changes.len());
    
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
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
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
                            let sound_change = &mut state.sound_changes[idx];
                            
                            layout::hungry_frame_bottom_up(ui, |ui| {
                                self.show_sc_editor(ui, sound_change);
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