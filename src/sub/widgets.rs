use std::ops;

use crate::{Selection, Focus, FocusBuffer};

fn selection_list_inner<'a, K, C, I>(
    ui: &mut egui::Ui,
    mut selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: I,
    buffer: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = String>,
    I: Iterator<Item = K> {

    for key in collection_keys { 
        focus.show_if_valid(
            (buffer)(key), ui,
            egui::RichText::from(&collection[key]),
            |ui| {
                let mut is_selected = selection.is_selected(key);

                if ui.toggle_value(&mut is_selected, &collection[key])
                    .clicked() {

                    if is_selected {
                        selection.insert(key);
                    } else {
                        selection.remove(key);
                    }
                }
            }
        ); 
    }
}

pub fn selection_list_wrapped<'a, K, C, I>(
    ui: &mut egui::Ui,
    selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: I,
    buffer: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = String>,
    I: Iterator<Item = K> {

    ui.horizontal_wrapped(|ui| {
        selection_list_inner(ui, selection, focus, //
            collection, collection_keys, buffer);
    });
}

pub fn selection_list<'a, K, C, I>(
    ui: &mut egui::Ui,
    selection: Selection<'a, K>,
    focus: &mut Focus,
    collection: &mut C,
    collection_keys: I,
    buffer_from_key: fn(K) -> FocusBuffer,
) where
    K: slotmap::Key,
    C: ops::IndexMut<K, Output = String>,
    I: Iterator<Item = K> {

    egui::ScrollArea::vertical().show(ui, |ui| {
        selection_list_inner(ui, selection, focus, //
            collection, collection_keys, buffer_from_key);
    });
}