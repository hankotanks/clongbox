use std::fmt;

use once_cell::{sync::Lazy, unsync::OnceCell};

pub fn hungry_frame<R>(
    ui: &mut egui::Ui, 
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) {
    let available = ui.available_size_before_wrap();

    egui_extras::TableBuilder::new(ui)
        .column(egui_extras::Column::exact(available.x))
        .header(available.y, |mut row| { row.col(|ui| {
            (add_contents)(ui);
        }); });
}

pub fn hungry_frame_with_layout<R>(
    ui: &mut egui::Ui, 
    layout: egui::Layout,
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) {
    let available = ui.available_size_before_wrap();

    egui_extras::TableBuilder::new(ui)
        .column(egui_extras::Column::exact(available.x))
        .header(available.y, |mut row| { row.col(|ui| {
            ui.with_layout(layout, |ui| {
                (add_contents)(ui);
            });
        }); });
}

pub fn hungry_frame_bottom_up<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) {
    static LAYOUT: Lazy<egui::Layout> = Lazy::new(|| {
        egui::Layout::bottom_up(egui::Align::LEFT)
    });

    hungry_frame_with_layout(ui, LAYOUT.to_owned(), add_contents);
}

pub enum BtnContextElem<'a> {
    Label(&'a str),
    Button(&'a str),
    Toggle(&'a str, bool),
    Enabled(&'a str, bool),
}

pub struct BtnContextResponse<const E: usize>([Option<egui::Response>; E]);

impl<const E: usize>  BtnContextResponse<E> {
    pub fn get(&self, index: usize) -> Option<&egui::Response> {
        self.0.iter().filter_map(|e| e.as_ref()).nth(index)
    }
}

pub fn button_context_line<const E: usize>(
    ui: &mut egui::Ui, 
    elems: [BtnContextElem<'_>; E]
) -> BtnContextResponse<E> {
    
    let mut response = {
        const NONE: Option<egui::Response> = None;

        BtnContextResponse([NONE; E])
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().button_padding.y = 0.;
        ui.spacing_mut().item_spacing.x = 0.;
        
        for (idx, elem) in elems.into_iter().enumerate() {
            let temp = match elem {
                BtnContextElem::Label(content) => {
                    ui.label(content);

                    None
                },
                BtnContextElem::Button(content) => {
                    let content = egui::RichText::new(content)
                        .underline();

                    let button = egui::Button::new(content)
                        .fill(egui::Color32::TRANSPARENT);

                    Some(ui.add(button))
                },
                BtnContextElem::Toggle(content, mut toggle) => {
                    let content = egui::RichText::new(content)
                        .underline();

                    Some(ui.toggle_value(&mut toggle, content))
                },
                BtnContextElem::Enabled(content, enabled) => {
                    let content = egui::RichText::new(content)
                        .underline();

                    let content = egui::Button::new(content)
                        .fill(egui::Color32::TRANSPARENT);

                    Some(ui.add_enabled(enabled, content))
                },
            };

            if let Some(temp) = temp {
                let _ = response.0[idx].insert(temp);
            }
        }
    });

    response
}

pub fn fixed_height_frame<R>(
    ui: &mut egui::Ui,
    height: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) {
    egui_extras::TableBuilder::new(ui)
        .column(egui_extras::Column::remainder())
        .header(height, |mut row| {
            row.col(|ui| {
                (add_contents)(ui);
            });
        });
}

// TODO: This doesn't work yet (for unknown reasons, possible egui bug)
// When working, should wrap all `egui::Ui::toggle_value` elements that
// are the origin of the current selection action
#[allow(dead_code)]
pub fn selection_origin<R: fmt::Debug>(
    ui: &mut egui::Ui, 
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) -> R {
    let mut response = OnceCell::default();

    ui.scope(|ui| {
        ui.visuals_mut().widgets.hovered.weak_bg_fill = egui::Color32::RED;

        response.set((add_contents)(ui)).unwrap();
    });

    response.take().unwrap()
}