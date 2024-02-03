use std::{borrow, io};

use once_cell::unsync::OnceCell;

use crate::{State, Pane, Tool, CONFIG, Control, editors};
use crate::{panes, tools};

pub mod fonts;
pub mod status;
pub mod config;
pub mod control;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum App<const P: usize, const T: usize> where 
    [OnceCell<Box<dyn Pane>>; P]: Default, 
    [OnceCell<Box<dyn Tool>>; T]: Default {

    Unloaded,
    
    Failed,

    Loading { input: String },

    #[cfg(target_arch = "wasm32")]
    Import { 
        // NOTE: This will ALWAYS be Some
        #[serde(skip)]
        promise: Option<poll_promise::Promise<io::Result<State>>>
    },

    Ready { 
        state: State,

        #[serde(skip)]
        panes: [OnceCell<Box<dyn Pane>>; P],

        #[serde(skip)]
        pane_active: usize,

        #[serde(skip)]
        tools: [OnceCell<Box<dyn Tool>>; T],

        #[serde(skip)]
        tool_active: usize,

        #[serde(skip)]
        editors: enum_map::EnumMap<editors::EditorKey, OnceCell<Box<dyn editors::Editor>>>,

        #[serde(skip)]
        editors_active: Option<editors::EditorKey>,

        #[serde(skip)]
        events_queue: Vec<egui::Event>,
    }
}

impl<const P: usize, const T: usize> Default for App<P, T> where 
    [OnceCell<Box<dyn Pane>>; P]: Default, 
    [OnceCell<Box<dyn Tool>>; T]: Default {

    fn default() -> Self {
        Self::Unloaded
    }
}

impl<const P: usize, const T: usize> App<P, T> where 
    [OnceCell<Box<dyn Pane>>; P]: Default, 
    [OnceCell<Box<dyn Tool>>; T]: Default {

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_fonts(fonts::load_fonts(fonts::Font::GentiumPlus));

        if let Some(storage) = cc.storage {
            return match eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default() {
                Self::Failed => Self::Unloaded,
                Self::Ready { state, .. } => {
                    let mut app = Self::Unloaded;

                    app.load(state);
                    app
                }
                app => app
            };
        }

        Default::default()
    }

    pub fn load(&mut self, state: State) {
        let loaded_app = Self::Ready { 
            state, 
            panes: panes::<P>(),
            pane_active: 0,
            tools: tools::<T>(),
            tool_active: 0,
            editors: editors::editors(),
            editors_active: None,
            events_queue: Vec::with_capacity(1),
        };

        *self = loaded_app;
    }

    pub fn load_handler<E>(&mut self, state: Result<State, E>)
        where E: Into<anyhow::Error> {

        match state {
            Ok(state) => self.load(state),
            _ => *self = Self::Failed,
        }
    }
}

fn show_message<'a, I>(ctx: &egui::Context, message: I) 
    where I: Into<borrow::Cow<'a, str>> {

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.centered_and_justified(|ui| {
            ui.heading(message.into());
        });
    });
}

async fn load_state_from_file() -> io::Result<State> {
    let file = rfd::AsyncFileDialog::new()
        .set_directory("/")
        .pick_file()
        .await;

    let contents = file.unwrap().read().await;
    let contents = String::from_utf8(contents);

    match contents.map(State::parse_from_str) {
        Ok(Ok(state)) => Ok(state),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}

impl<const P: usize, const T: usize> eframe::App for App<P, T> where 
    [OnceCell<Box<dyn Pane>>; P]: Default, 
    [OnceCell<Box<dyn Tool>>; T]: Default {
        
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Language", |ui| {
                    if ui.button("New").clicked() {
                        self.load(State::default());

                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Import").clicked() {
                        *self = Self::Loading { input: String::from("") };
                        
                        ui.close_menu();
                    }

                    if ui.button("Import from file").clicked() {
                        let state = load_state_from_file();
                        
                        #[cfg(not(target_arch = "wasm32"))] {
                            let state = pollster::block_on(state);

                            self.load_handler(state);
                        }

                        #[cfg(target_arch = "wasm32")] {
                            let promise = poll_promise::Promise::spawn_local(state);
                            let promise = Some(promise);

                            *self = Self::Import { promise };
                        }
                        
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Clear").clicked() {
                        *self = Self::Unloaded;

                        ui.close_menu();
                    }
                });

                let mut state = None;
                if let App::Loading { input } = self {
                    ui.separator();

                    if ui.button("Finish").clicked() {
                        let temp = State::parse_from_str(input.as_str());

                        let _ = state.insert(temp);
                    }
                }

                if let Some(state) = state {
                    self.load_handler(state);
                }
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| { 
            ui.horizontal(|ui| {
                let mut frame = egui::Frame::default();

                frame.inner_margin.top += ui.spacing().item_spacing.y;
                frame.show(ui, |ui| {
                    if let Some(status) = status::get() {
                        ui.label(format!("{}", status));
                    }
                });
            }); 
        });

        match self {
            App::Unloaded => //
                show_message(ctx, "Import a language to get started"),

            App::Failed => //
                show_message(ctx, "Failed to import language"),

            App::Loading { input } => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let size = ui.available_size_before_wrap();

                    let contents = egui::TextEdit::multiline(input)
                        .min_size(size);

                    ui.add(contents);
                });
            },

            #[cfg(target_arch = "wasm32")]
            App::Ready { .. } => {
                let inner_size = ctx.available_rect().size();

                if inner_size.min(CONFIG.window_min()) == inner_size {
                    show_message(ctx, "Window is too small to render");
                } else {
                    self.show_ready(ctx)
                }
            },

            #[cfg(not(target_arch = "wasm32"))]
            App::Ready { .. } => //
                self.show_ready(ctx),

            #[cfg(target_arch = "wasm32")]
            App::Import { promise } => {
                if promise.as_ref().unwrap().ready().is_some() {
                    let state = match promise.take().unwrap().try_take() {
                        Ok(Ok(state)) => Ok(state),
                        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
                    };

                    self.load_handler(state);
                }

                show_message(ctx, "Processing import");
            },
        }
    }
}

impl<const P: usize, const T: usize> App<P, T> where 
    [OnceCell<Box<dyn Pane>>; P]: Default, 
    [OnceCell<Box<dyn Tool>>; T]: Default {

    fn show_ready(&mut self, ctx: &egui::Context) {
        let App::Ready { 
            state, 
            panes,
            pane_active,
            tools,
            tool_active,
            editors,
            editors_active,
            events_queue,
            ..
        } = self else {
            panic!();
        };

        ctx.input(|input| {
            if input.key_released(egui::Key::Escape) {
                state.focus.clear();
            }
        });

        ctx.input_mut(|input| {
            for event in events_queue.drain(0..) {
                input.events.push(event);
            }
        });

        for (_, editor) in editors.iter_mut() {
            let editor = editor.get_mut().unwrap().as_mut();

            editors::editor_update(editor, state);
        }

        let egui::Margin { left, right, .. } = ctx.style().spacing.window_margin;

        let min = CONFIG.window_sidebar_width - left - right;
        let max = ctx.available_rect().width() - CONFIG.window_main_width;
        let max = max.min(ctx.available_rect().width() * 0.4);

        let resizable = ctx.available_rect().width() - CONFIG.window_min().x;
        let resizable = resizable.abs() > 24.;

        egui::SidePanel::right("tools")
            .width_range(egui::Rangef::new(min, max))
            .resizable(resizable)
            .show(ctx, |ui| {
                
            ui.add_space(ui.spacing().item_spacing.y * 2.);
            
            egui::ComboBox::from_label("Select Tool")
                .selected_text(tools[*tool_active].get().unwrap().name())
                .wrap(false)
                .show_ui(ui, |ui| {
                    for (idx, tool) in tools.iter().enumerate() {
                        let name = tool.get().unwrap().name();

                        if ui.selectable_value(tool_active, idx, name).clicked() {
                            state.focus.clear();
                        }
                    }
                });

            ui.separator();

            for (editor_key, editor) in editors.iter_mut() {
                let header = format!("{editor_key} Editor");
                let header = egui::CollapsingHeader::new(header);

                let header = if matches!(editors_active, Some(key) if *key == editor_key) {
                    header.open(Some(true))
                } else {
                    header
                };
                
                header.show(ui, |ui| {
                    editor.get_mut().unwrap().show(state, ui);
                });

                let _ = editors_active.take();
            }

            ui.separator();

            if let Some(tool) = tools[*tool_active].get_mut() {
                tool.show(state, ui);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (idx, pane) in panes.iter().enumerate() {
                    let name = pane.get().unwrap().name();

                    ui.selectable_value(pane_active, idx, name);
                }
            });

            ui.add_space(ui.spacing().item_spacing.y);

            if let Some(pane) = panes[*pane_active].get_mut() {
                egui::Frame::default()
                    .stroke(ui.visuals().window_stroke)
                    .inner_margin(ui.spacing().window_margin)
                    .show(ui, |ui| {

                    let control = Control {
                        tool_active,
                        editors_active,
                    };

                    pane.show(control, state, ui);
                });
            }
        });

        if let crate::Focus::Active { fst, .. } = &mut state.focus {
            *fst = false;
        }
    }
}