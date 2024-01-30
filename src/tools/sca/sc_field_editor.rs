use std::mem;

use once_cell::sync::OnceCell;

use crate::{app::fonts, sc, Focus, FocusBuffer, FocusTarget};

#[derive(Clone, Copy, Hash)]
struct ScTarget {
    field: sc::Field,
    head: bool,
    tail: bool,
    nested: bool,
}

fn show_sc_element(
    ui: &mut egui::Ui, 
    elem: sc::ScElemRefMut<'_>, 
    focus: &mut Focus,
    mut target: ScTarget,
) {
    let sc::ScElemRefMut {
        elem,
        rep_phonemes,
        language,
    } = elem;

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

            for elem in elems.iter_mut() {
                target.nested = true;

                let elem = sc::ScElemRefMut {
                    elem,
                    rep_phonemes,
                    language,
                };

                show_sc_element(ui, elem, focus, target);
            }

            show_sc_element_addition(ui, elems, focus, target);

            ui.label(fonts::ipa_rt("]"));
        },
    }
}

fn show_sc_element_addition(
    ui: &mut egui::Ui, 
    elems: &mut Vec<sc::Element>, 
    focus: &mut Focus, 
    target: ScTarget) {
    static SIZE: OnceCell<egui::Vec2> = OnceCell::new();
    
    let size = SIZE.get_or_init(|| {
        let temp = ui.text_style_height(&egui::TextStyle::Button);

        egui::Vec2::splat(temp)
    });

    let id = ui.id().with(target);

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
                FocusBuffer::Boundary => sc::Element::Boundary,
            };

            if target.head {
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
        let ScTarget {
            field,
            head,
            tail,
            nested,
        } = target;

        let target = FocusTarget::Sc {
            field,
            head,
            tail,
            nested,
        };

        focus.set(id, target);
    }
}

fn show_sc_field(
    ui: &mut egui::Ui,
    sound_change: &mut sc::ScRefMut<'_>, 
    field_disc: mem::Discriminant<sc::Field>,
    focus: &mut Focus,
) {
    let field = *sound_change.field(field_disc);

    let elements = &mut sound_change.sc[field_disc];
    let elements_is_empty = elements.is_empty();
    let elements_len = elements.len();

    let mut target = ScTarget {
        field,
        head: true,
        tail: false,
        nested: false,
    };

    let should_show_addition = !matches!(field, sc::Field::EnvStart { 
        has_boundary: true 
    });

    if !elements_is_empty && should_show_addition {
        show_sc_element_addition(ui, elements, focus, target);
    }

    for (idx, elem) in elements.iter_mut().enumerate() {
        if idx != 0 {
            target.head = false;
        }

        if idx == elements_len - 1 {
            target.tail = true;
        }

        let elem = sc::ScElemRefMut {
            elem,
            rep_phonemes: sound_change.rep_phonemes,
            language: sound_change.language,
        };

        show_sc_element(ui, elem, focus, target);
    }

    if !elements.is_empty() {
        target.head = false;
    }

    target.tail = true;

    let should_show_addition = !matches!(field, sc::Field::EnvEnd { 
        has_boundary: true 
    });

    if elements_is_empty || should_show_addition {
        show_sc_element_addition(ui, elements, focus, target);
    }
}

pub fn show_sc_editor(ui: &mut egui::Ui, mut sound_change: sc::ScRefMut<'_>, focus: &mut Focus) {
    ui.add_space(ui.spacing().item_spacing.y * 2.);

    let rect = ui.available_rect_before_wrap();

    ui.horizontal(|ui| {
        // NOTE: This is to preserve `egui::Align::Center`
        ui.label(fonts::ipa_rt("")); 

        show_sc_field(ui, &mut sound_change, sc::ENV_START, focus);

        let content = egui::RichText::new("_")
            .font(fonts::FONT_ID.to_owned());

        ui.label(content);

        show_sc_field(ui, &mut sound_change, sc::ENV_END, focus);
    });
    
    ui.label("Environment");

    ui.add_space(ui.spacing().item_spacing.y * 2.);

    ui.horizontal(|ui| {
        // NOTE: This is to preserve `egui::Align::Center`
        ui.label(fonts::ipa_rt(""));
        
        show_sc_field(ui, &mut sound_change, sc::TARGET, focus);

        let content = egui::RichText::new("\u{2192}")
            .font(fonts::FONT_ID.to_owned());

        ui.label(content);

        show_sc_field(ui, &mut sound_change, sc::REPLACEMENT, focus);
    });

    egui_extras::StripBuilder::new(ui)
        .sizes(egui_extras::Size::remainder(), 2)
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.label("Target & Replacement");
            });

            strip.cell(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.button("#");

                    ui.button("[  ]");
                });
                
            });
        });
}