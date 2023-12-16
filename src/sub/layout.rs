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