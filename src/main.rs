//! Joycode — управление Claude Code с геймпада.
//! Читает Xbox-геймпад (gilrs), впрыскивает клавиши через uinput,
//! GUI на egui с настройками и инструкцией.
mod actions;
mod app;
mod config;
mod engine;
mod fonts;
mod injector;
mod keys;
mod state;
mod theme;

use app::App;
use config::Bindings;
use engine::SharedConfig;
use parking_lot::Mutex;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_path = Bindings::default_path();
    let bindings = match Bindings::load_or_create(&config_path) {
        Ok(b) => b,
        Err(e) => {
            log::error!("не удалось загрузить конфиг ({e}), беру дефолтный");
            Bindings::default_map()
        }
    };

    let state = state::new_state();
    let config: SharedConfig = Arc::new(Mutex::new(bindings));

    engine::spawn(state.clone(), config.clone());

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([560.0, 640.0]),
        ..Default::default()
    };

    let state_ui = state.clone();
    let config_ui = config.clone();
    eframe::run_native(
        "Joycode",
        native_options,
        Box::new(move |cc| {
            fonts::install(&cc.egui_ctx);
            Ok(Box::new(App::new(cc, state_ui, config_ui, config_path)))
        }),
    )
}
