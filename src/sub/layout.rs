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

pub enum BtnContextElem<'a> {
    Label(&'a str),
    Button(&'a str),
    Toggle(&'a str, bool),
    Enabled(&'a str, bool),
}

pub fn button_context_line<'a, I>(ui: &mut egui::Ui, elems: I) -> Option<egui::Response>
    where I: IntoIterator<Item = BtnContextElem<'a>> {

    let mut response = None;

    ui.spacing_mut().button_padding.y = 0.;
    ui.spacing_mut().item_spacing.x = 0.;
    ui.horizontal(|ui| {
        for elem in elems {
            match elem {
                BtnContextElem::Label(content) => {
                    ui.label(content);
                },
                BtnContextElem::Button(content) => {
                    let content = egui::RichText::new(content)
                        .underline();

                    let button = egui::Button::new(content)
                        .fill(egui::Color32::TRANSPARENT);

                    let temp = ui.add(button);

                    if response.is_none() {
                        let _ = response.insert(temp);
                    } else if let Some(response) = response.as_mut() {
                        response.union(temp);
                    }
                },
                BtnContextElem::Toggle(content, mut toggle) => {
                    let content = egui::RichText::new(content)
                        .underline();

                    let temp = ui.toggle_value(&mut toggle, content);

                    if response.is_none() {
                        let _ = response.insert(temp);
                    } else if let Some(response) = response.as_mut() {
                        response.union(temp);
                    }
                },
                BtnContextElem::Enabled(content, enabled) => {
                    let content = egui::RichText::new(content)
                        .underline();

                    let content = egui::Button::new(content)
                        .fill(egui::Color32::TRANSPARENT);

                    let temp = ui.add_enabled(enabled, content);

                    if response.is_none() {
                        let _ = response.insert(temp);
                    } else if let Some(response) = response.as_mut() {
                        response.union(temp);
                    }
                },
            }
        }
    });

    response
}