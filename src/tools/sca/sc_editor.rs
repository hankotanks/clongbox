use std::mem;

use once_cell::sync::OnceCell;

use crate::{status, Focus, FocusBuffer, FocusTarget};
use crate::app::fonts;
use crate::sc;
use crate::CONFIG;

#[derive(Hash, Debug)]
struct ScTarget<'a> {
    field: &'a mut sc::Field,
    head: bool,
    tail: bool,
    nested: bool,
    invalid: bool,
}

fn focus_buffer_to_element(buffer: FocusBuffer, field: &mut sc::Field) -> sc::Element {
    match buffer {
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
    }
}

#[derive(Clone, Copy)]
enum ScElemAction { Remove, None, }

fn show_sc_element_inner(
    ui: &mut egui::Ui, 
    elem: sc::ScElemRefMut<'_>, 
    focus: &mut Focus,
    target: ScTarget<'_>,
) -> egui::Response {
    let sc::ScElemRefMut {
        elem,
        rep_phonemes,
        language,
    } = elem;

    let ScTarget {
        field,
        head,
        tail, 
        nested, ..
    } = target;

    match elem {
        sc::Element::Phoneme { key, rep } => {
            let content = match *rep {
                true => &rep_phonemes[*key],
                false => &language[*key],
            };

            let content = fonts::ipa_rt(format!("{}", content));
            
            ui.label(content)
        },
        sc::Element::Group(key) => {
            let content = fonts::ipa_rt(format!("{}", language[*key].name));
        
            ui.label(content)
        },
        sc::Element::Boundary => {
            ui.label(fonts::ipa_rt("#"))
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
                    invalid: false,
                };

                match show_sc_element(ui, elem, focus, target) {
                    ScElemAction::Remove => {
                        let _ = elem_to_remove.insert(idx);
                    },
                    ScElemAction::None => { /*  */ },
                }
            }

            if let Some(idx) = elem_to_remove.take() {
                elems.remove(idx);
            }

            let target = ScTarget {
                field,
                head,
                tail,
                nested: true,
                invalid: false,
            };

            show_sc_element_addition(ui, elems, focus, target);

            // NOTE: The response returned from the `sc::Element::Any` branch
            // isn't representative of all of its UI components
            ui.label(fonts::ipa_rt("]"))
        },
        sc::Element::Invalid => {
            let content = fonts::ipa_rt("\u{2205}")
                .color(ui.visuals().error_fg_color);

            let target = ScTarget {
                field,
                head,
                tail,
                nested,
                invalid: true,
            };

            let id = ui.id().with(&target);

            if let Some(buffer) = focus.take(id) {
                let _ = mem::replace(elem, focus_buffer_to_element(buffer, field));

                focus.clear();
            } 

            if focus.get_id() == id {
                let response = ui.toggle_value(&mut true, content);

                if response.clicked() {
                    focus.clear();
                }

                response
            } else {
                let response = ui.button(content);

                if response.clicked() {
                    let target = FocusTarget::Sc {
                        field: *field,
                        head,
                        tail,
                        nested,
                    };
            
                    focus.set(id, target);
                }

                response
            }
        },
    }
}

fn show_sc_element(
    ui: &mut egui::Ui, 
    elem: sc::ScElemRefMut<'_>, 
    focus: &mut Focus,
    target: ScTarget<'_>,
) -> ScElemAction {
    let invalid = {
        let sc::ScElemRefMut { elem, .. } = &elem;

        matches!(elem, sc::Element::Invalid)
    };

    let response = egui::Frame::default().show(ui, |ui| {
        show_sc_element_inner(ui, elem, focus, target)
    });

    let egui::InnerResponse::<egui::Response> { 
        response, 
        inner, .. 
    } = response;

    let mut action = ScElemAction::None;

    if invalid {
        status::set_on_hover(&inner, "Click to replace. Right-click for options");

        inner.context_menu(|ui| {
            if ui.button("Remove").clicked() {
                action = ScElemAction::Remove;
            }
        });
    } else {
        status::set_on_hover(&response, "Right-click for options");

        let rect = response.rect.expand2({
            egui::Vec2 { x: ui.spacing().button_padding.x, y: 0. }
        });

        let draw_border = || {
            ui.painter().rect_stroke(
                rect, 
                CONFIG.selection_rounding, 
                ui.visuals().window_stroke
            );
        };
    
        if response.hovered() {
            draw_border();
        }
        
        response.context_menu(|ui| {
            draw_border();
    
            if ui.button("Remove").clicked() {
                action = ScElemAction::Remove;
    
                ui.close_menu();
            }
        });
    }

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
        nested, ..
    } = target;

    match focus.take(id) {
        Some(buffer) => {
            let elem = focus_buffer_to_element(buffer, field);

            if head {
                elems.insert(0, elem);
            } else {
                elems.push(elem);
            }

            focus.clear();
        },
        None => { /*  */ },
    }

    if focus.get_id() == id {
        if ui.toggle_value(&mut true, "+").clicked() {
            focus.clear();
        }
    } else if ui.add_sized(*size, egui::Button::new("+")).clicked() {
        let target = FocusTarget::Sc {
            field: *field,
            head,
            tail,
            nested,
        };

        focus.set(id, target);
    };
}

pub fn show_sc_field(
    ui: &mut egui::Ui,
    mut sound_change: sc::ScRefMut<'_>, 
    disc: mem::Discriminant<sc::Field>,
    focus: &mut Focus,
) {
    let elements_len = {
        let (_, elements) = sound_change.sc.field(disc);

        elements.len()
    };
    
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
            invalid: false,
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
                invalid: false,
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
            invalid: false,
        };

        show_sc_element_addition(ui, elements, focus, target);
    }
}