use std::{collections::BTreeSet, sync};

use once_cell::sync::OnceCell;

use crate::{app::fonts, layout, status};

#[derive(Clone, Copy, PartialEq)]
enum LexiconTool { Apply, Batch, }

impl Default for LexiconTool {
    fn default() -> Self { Self::Apply }
}

#[derive(Default)]
pub struct LexiconPane {
    selection: BTreeSet<usize>,
    tool: LexiconTool,
}

impl LexiconPane {
    fn batch_word_list(&mut self, ui: &mut egui::Ui, batch: &[sync::Arc<str>]) {
        for (idx, word) in batch.iter().enumerate() {
            let word = fonts::ipa_rt(&**word);

            if self.selection.contains(&idx) {
                if ui.toggle_value(&mut true, word).clicked() {
                    self.selection.remove(&idx);
                }
            } else {
                let response = ui.toggle_value(&mut false, word);

                status::set_on_hover(&response, "Select words you like, then commit them to the lexicon");

                if response.clicked() {
                    self.selection.insert(idx);
                }
            }
        }
    }

    fn batch_panel(
        &mut self, 
        ui: &mut egui::Ui, 
        lexicon: &mut Vec<sync::Arc<str>>, 
        batch: &mut Vec<sync::Arc<str>>
    ) {
        ui.add_space(ui.spacing().item_spacing.y);

        ui.horizontal(|ui| {
            if ui.add_enabled(
                !batch.is_empty(), 
                egui::Button::new("Commit All")
            ).clicked() {
                lexicon.append(batch);

                self.selection.clear();
            }

            if ui.add_enabled(
                !self.selection.is_empty(), 
                egui::Button::new("Commit Selection"),
            ).clicked() {
                let mut indices = self.selection
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();

                indices.sort();

                self.selection.clear();

                for idx in indices.into_iter().rev() {    
                    let word = batch.remove(idx);

                    lexicon.push(word);
                }
            }
            
            if ui.add_enabled(
                !batch.is_empty(), 
                egui::Button::new("Discard")
            ).clicked() {
                self.selection.clear();

                batch.clear();
            }
        });

        ui.separator();
        
        if batch.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Generate a batch using the Word Gen tool");
            });
        } else {
            egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                self.batch_word_list(ui, batch);
            });
        }
    }
}

impl super::Pane for LexiconPane {
    fn name(&self) -> &'static str { "Lexicon" }

    fn show(
        &mut self, 
        _control: crate::Control<'_>, 
        state: &mut crate::State, 
        ui: &mut egui::Ui
    ) {
        let crate::State {
            lexicon,
            word_gen_batch, ..
        } = state;

        egui_extras::StripBuilder::new(ui)
            .sizes(egui_extras::Size::remainder(), 2)
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        for word in lexicon.iter() {
                            let word = format!("{}", word);
                            
                            ui.label(fonts::ipa_rt(word));
                        }
                    });
                });

                strip.cell(|ui| {
                    static MARGIN_OUTER: OnceCell<egui::Margin> = OnceCell::new();
                    static MARGIN_INNER: OnceCell<egui::Margin> = OnceCell::new();

                    let _ = MARGIN_OUTER.set({
                        let x = ui.spacing().item_spacing.x * 0.5;

                        let mut margin = egui::Margin::symmetric(x, 0.);

                        margin.bottom += ui.spacing().item_spacing.y;
                        margin
                    });

                    let _ = MARGIN_INNER.set({
                        egui::Margin::from(ui.spacing().item_spacing)
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(MARGIN_OUTER.get().unwrap().left);

                        ui.selectable_value(
                            &mut self.tool, 
                            LexiconTool::Apply, "Apply",
                        ); 
    
                        ui.selectable_value(
                            &mut self.tool, 
                            LexiconTool::Batch, "Current Batch",
                        );
                    });

                    ui.add_space(ui.spacing().item_spacing.y);

                    egui::Frame::default()
                        .stroke(ui.visuals().window_stroke)
                        .outer_margin(*MARGIN_OUTER.get().unwrap())
                        .inner_margin(*MARGIN_INNER.get().unwrap())
                        .show(ui, |ui| {
                            layout::hungry_frame(ui, |ui| {
                                match self.tool {
                                    LexiconTool::Apply => {
                                        ui.centered_and_justified(|ui| {
                                            ui.label("TODO");
                                        });
                                    },
                                    LexiconTool::Batch => self.batch_panel(
                                        ui, 
                                        lexicon, 
                                        word_gen_batch
                                    ),
                                }
                            });
                        });
                });
            });
    }
}