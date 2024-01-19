#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Native
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(clongbox::CONFIG.window_min())
            .with_min_inner_size(clongbox::CONFIG.window_min()),
        ..Default::default()
    };
    
    eframe::run_native(
        "ClongBox",
        native_options,
        // NOTE: The const generics here MUST be updated if a new Pane/Tool is added
        Box::new(|cc| Box::new({
            const NUM_PANES: usize = enum_iterator::cardinality::<clongbox::PaneId>();
            const NUM_TOOLS: usize = enum_iterator::cardinality::<clongbox::ToolId>();
            
            clongbox::App::<NUM_PANES, NUM_TOOLS>::new(cc)
        })),
    )
}

// Web
#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "eframe_canvas",
                eframe::WebOptions::default(),
                // NOTE: The const generics here MUST be updated if a new Pane/Tool is added
                Box::new(|cc| Box::new({
                    const NUM_PANES: usize = enum_iterator::cardinality::<clongbox::PaneId>();
                    const NUM_TOOLS: usize = enum_iterator::cardinality::<clongbox::ToolId>();
                    
                    clongbox::App::<NUM_PANES, NUM_TOOLS>::new(cc)
                })),
            ).await.expect("Failed to start eframe");
    });
}
