use std::mem;

use once_cell::sync::OnceCell;

use crate::{Focus, FocusBuffer, FocusTarget};
use crate::app::fonts;
use crate::sc;
use crate::CONFIG;

#[derive(Hash, Debug)]
struct ScTarget<'a> {
    field: &'a mut sc::Field,
    head: bool,
    tail: bool,
    nested: bool,
}

#[derive(Clone, Copy)]
enum ScElemAction { Remove, None, }

fn show_sc_element_inner(
    ui: &mut egui::Ui, 
    elem: sc::ScElemRefMut<'_>, 
    focus: &mut Focus,
    target: ScTarget<'_>,
) {
    let sc::ScElemRefMut {
        elem,
        rep_phonemes,
        language,
    } = elem;

    let ScTarget {
        field,
        head,
        tail, ..
    } = target;

    match elem {
        sc::Element::Phoneme { key, rep } => {
            let content = match *rep {
                true => &rep_phonemes[*key],
                false => &language[*key],
            };

            let content = fonts::ipa_rt(format!("{}", content));
            
            ui.label(content);
        },
        sc::Element::Group(key) => {
            let content = fonts::ipa_rt(format!("{}", language[*key].name));
            
            ui.label(content);
        },
        sc::Element::Boundary => {
            ui.label(fonts::ipa_rt("#"));
        },
        sc::Element::Any(elems) => {
            ui.label(fonts::ipa_rt("["));

            let mut elem_to_remove = None;

            for (idx, elem) in elems.iter_mut().enumerate() {
                let elem = sc::ScElemRefMut {
                    elem,
                    rep_phonemes,
                    language,
                };

                let target = ScTarget {
                    field,
                    head,
                    tail,
                    nested: true,
                };

                match show_sc_element(ui, elem, focus, target) {
                    ScElemAction::Remove => {
                        let _ = elem_to_remove.insert(idx);
                    },
                    ScElemAction::None => { /*  */ },
                }
            }

            if let Some(elem_to_remove) = elem_to_remove.take() {
                elems.remove(elem_to_remove);
            }

            let target = ScTarget {
                field,
                head,
                tail,
                nested: true,
            };

            show_sc_element_addition(ui, elems, focus, target);

            ui.label(fonts::ipa_rt("]"));
        },
        sc::Element::Invalid(content) => {
            let content = format!("{}", content);
            let content = fonts::ipa_rt(content).color(ui.visuals().error_fg_color);

            // TODO: This could be a button that allows the user to select a replacement
            ui.label(content);
        },
    }
}

fn show_sc_element(
    ui: &mut egui::Ui, 
    elem: sc::ScElemRefMut<'_>, 
    focus: &mut Focus,
    target: ScTarget<'_>,
) -> ScElemAction {
    let response = egui::Frame::default().show(ui, |ui| {
        show_sc_element_inner(ui, elem, focus, target);
    }).response;

    let rect = response.rect.expand2({
        egui::Vec2 { x: ui.spacing().button_padding.x, y: 0. }
    });

    let draw_border = || {
        ui.painter().rect_stroke(rect, CONFIG.selection_rounding, ui.visuals().window_stroke);
    };

    if response.hovered() {
        draw_border();
    }
    
    let mut action = ScElemAction::None;

    response.context_menu(|ui| {
        draw_border();

        if ui.button("Remove").clicked() {
            action = ScElemAction::Remove;

            ui.close_menu();
        }
    });

    action
}

fn show_sc_element_addition(
    ui: &mut egui::Ui, 
    elems: &mut Vec<sc::Element>, 
    focus: &mut Focus, 
    target: ScTarget<'_>
) {
    static SIZE: OnceCell<egui::Vec2> = OnceCell::new();
    
    let size = SIZE.get_or_init(|| {
        let temp = ui.text_style_height(&egui::TextStyle::Button);

        egui::Vec2::splat(temp)
    });

    let id = ui.id().with(&target);

    let ScTarget {
        field,
        head,
        tail,
        nested,
    } = target;

    match focus.take(id) {
        Some(buffer) => {
            let elem = match buffer {
                FocusBuffer::Phoneme { key, src } => {
                    let rep = match src {
                        crate::PhonemeSrc::Language => false,
                        crate::PhonemeSrc::Rep => true,
                    };

                    sc::Element::Phoneme { key, rep }
                },
                FocusBuffer::Group(key) => sc::Element::Group(key),
                FocusBuffer::Any => sc::Element::Any(Vec::default()),
                FocusBuffer::Boundary => {
                    match field {
                        sc::Field::EnvStart { has_boundary } | 
                        sc::Field::EnvEnd { has_boundary } => {
                            *has_boundary = true;
                        },
                        _ => { /*  */ }
                    }

                    sc::Element::Boundary 
                },
            };

            if head {
                elems.insert(0, elem);
            } else {
                elems.push(elem);
            }

            focus.clear();
        },
        None => { /*  */ },
    }

    let response = if focus.get_id() == id {
        ui.toggle_value(&mut true, "+")
    } else {
        ui.add_sized(*size, egui::Button::new("+"))
    };

    if response.clicked() {
        let target = FocusTarget::Sc {
            field: *field,
            head,
            tail,
            nested,
        };

        focus.set(id, target);
    }
}

pub fn show_sc_field(
    ui: &mut egui::Ui,
    mut sound_change: sc::ScRefMut<'_>, 
    disc: mem::Discriminant<sc::Field>,
    focus: &mut Focus,
) {
    let elements_len = sound_change.sc[disc].len();
    let elements_is_empty = elements_len == 0;

    let (mut head, mut tail, nested) = (true, false, false);

    let should_show_addition = !matches!(
        sound_change.field_mut(disc).0, 
        sc::Field::EnvStart { has_boundary: true }
    );

    if should_show_addition && !elements_is_empty {
        let (field, elements) = sound_change.field_mut(disc);

        let target = ScTarget {
            field,
            head,
            tail,
            nested,
        };

        show_sc_element_addition(ui, elements, focus, target);
    }

    {
        let sc::ScRefMut {
            sc,
            rep_phonemes,
            language,
        } = &mut sound_change;

        let (field, elements) = sc.field_mut(disc);

        let mut elem_to_remove = None;

        for (idx, elem) in elements.iter_mut().enumerate() {
            if idx != 0 {
                head = false;
            }
    
            // The RHS here just fails the check if `elements_len` is 0
            if idx == elements_len.checked_sub(1).unwrap_or(usize::MAX) {
                tail = true;
            }

            let target = ScTarget {
                field, 
                head,
                tail,
                nested,
            };

            let elem = sc::ScElemRefMut {
                elem,
                rep_phonemes,
                language,
            };

            match show_sc_element(ui, elem, focus, target) {
                ScElemAction::Remove => {
                    let _ = elem_to_remove.insert(idx);
                },
                ScElemAction::None => { /*  */ },
            }
        }

        if let Some(elem_to_remove) = elem_to_remove.take() {
            let boundary = match &elements[elem_to_remove] {
                sc::Element::Boundary => true,
                sc::Element::Any(elems) => elems.contains(&sc::Element::Boundary),
                _ => false,
            };

            if boundary {
                match field {
                    sc::Field::EnvStart { has_boundary } | 
                    sc::Field::EnvEnd { has_boundary } => {
                        *has_boundary = false;
                    },
                    _ => { /*  */ }
                }
            }

            elements.remove(elem_to_remove);
        }
    }

    if !elements_is_empty {
        head = false;
    }

    tail = true;

    let should_show_addition = !matches!(
        sound_change.field_mut(disc).0, 
        sc::Field::EnvEnd { has_boundary: true }
    );

    if should_show_addition {
        let (field, elements) = sound_change.field_mut(disc);

        let target = ScTarget {
            field,
            head,
            tail,
            nested,
        };

        show_sc_element_addition(ui, elements, focus, target);
    }
}