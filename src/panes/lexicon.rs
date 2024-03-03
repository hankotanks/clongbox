use std::{collections::BTreeSet, mem, sync};

use once_cell::sync::OnceCell;

use crate::{app::fonts, layout, status};

#[derive(Clone, Copy, PartialEq)]
enum LexiconTool { Apply, Batch, }

impl Default for LexiconTool {
    fn default() -> Self { Self::Apply }
}

#[derive(Clone, Copy, PartialEq)]
enum LexiconSort { None, Alphabetical, Length }

impl Default for LexiconSort {
    fn default() -> Self { Self::None }
}

#[derive(Default)]
pub struct LexiconPane {
    selection: BTreeSet<usize>,
    sort: LexiconSort,
    sort_rev: bool,
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
        ui.horizontal_wrapped(|ui| {
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

    fn apply_panel(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Random Selection").clicked() {
                todo!();
            }

            if ui.button("All").clicked() {
                todo!();
            }

            if ui.button("Select").clicked() {
                todo!();
            }

            if ui.button("Clear").clicked() {
                todo!();
            }

            if ui.button("Apply").clicked() {
                todo!();
            }
        });

        ui.separator();

        ui.centered_and_justified(|ui| {
            ui.label("TODO");
        });
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
                    ui.push_id(0x20C69A3, |ui| {
                        egui::Frame::default()
                            .show(ui, |ui| { 
                                show_lexicon(&mut self.sort, &mut self.sort_rev, ui, lexicon); 
                            });
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
                                ui.add_space(ui.spacing().item_spacing.y);

                                match self.tool {
                                    LexiconTool::Apply => self.apply_panel(ui),
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

fn show_lexicon_inner(ui: &mut egui::Ui, lexicon: &[sync::Arc<str>]) {
    let available = ui.available_rect_before_wrap();

    let (word_count, mut word_count_temp) = (lexicon.len(), lexicon.len());

    let mut width_avg = 0.;
    for word in lexicon {
        if word.len() > 2 {
            width_avg += fonts::ipa_text_width(&**word);
        } else {
            word_count_temp -= 1;
        }
    }

    let width_avg = width_avg / word_count_temp as f32;

    let columns = ui.available_width() / width_avg;
    let columns = columns.floor() as usize;
    let columns = columns.saturating_sub(1);

    static ROW_HEIGHT: OnceCell<f32> = OnceCell::new();

    let _ = ROW_HEIGHT.set({
        fonts::FONT_ID.size + //
        ui.spacing().button_padding.y * 2. + //
        ui.spacing().item_spacing.y
    });

    egui_extras::TableBuilder::new(ui)
        .columns(egui_extras::Column::exact(width_avg), columns)
        .auto_shrink([false, true])
        .body(|mut body| {
            let mut idx = 0;

            while idx < word_count {
                let mut skip = 0;

                body.row(*ROW_HEIGHT.get().unwrap(), |mut row| {
                    let mut idx_col = 0;

                    'col: while idx_col < columns {
                        if skip > 0 {
                            skip -= 1;
    
                            row.col(|_ui| { /* */ });
                        } else {
                            let width = fonts::ipa_text_width(&*lexicon[idx]);

                            let temp = (width / width_avg).ceil() as usize;

                            if temp > columns - idx_col && temp < columns && columns > 1 {
                                break 'col;
                            }

                            skip = temp;

                            let mut advance = false;

                            row.col(|ui| {
                                let rect = ui.available_rect_before_wrap();

                                let right = rect.left() + rect.width() * skip as f32;

                                if right <= available.right() || idx_col == 0 {
                                    let rect = egui::Rect {
                                        max: egui::Pos2 {
                                            x: right.min(available.right()),
                                            y: rect.bottom().min(available.bottom()),
                                        },
                                        min: egui::Pos2 {
                                            x: rect.left(),
                                            y: rect.top().max(available.top()),
                                        }
                                    };
    
                                    ui.set_clip_rect(rect);
                                    
                                    let content = fonts::ipa_rt(&*lexicon[idx]);
                                    let content = egui::Label::new(content);
    
                                    ui.put(rect, content);
    
                                    idx += 1;
                                } else {
                                    advance = true;
                                }
                            });

                            if advance && idx_col > 0 { break 'col; }
                        }

                        idx_col += 1;

                        if idx == word_count { break 'col; }
                    }
                });
            }
        });
}

fn show_lexicon(sort: &mut LexiconSort, sort_rev: &mut bool, ui: &mut egui::Ui, lexicon: &[sync::Arc<str>]) {
    ui.horizontal_wrapped(|ui| {
        let _ = ui.add_enabled(false, egui::Button::new("Sort"));

        ui.separator();

        ui.selectable_value(sort, LexiconSort::None, "Original");
        ui.selectable_value(sort, LexiconSort::Alphabetical, "Alphabetically");
        ui.selectable_value(sort, LexiconSort::Length, "By Length");

        ui.separator();

        let content = match sort_rev {
            true => "\u{039B}",
            false => "V",
        };

        if ui.button(content).clicked() {
            let _ = mem::replace(sort_rev, !*sort_rev);
        }
    });

    ui.separator();

    show_lexicon_inner(ui, lexicon);
}