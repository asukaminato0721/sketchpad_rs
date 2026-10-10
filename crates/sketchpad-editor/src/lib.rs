//! Desktop editor and renderer over the headless document engine.
pub use sketchpad_engine::*;
mod diagnostics;
mod editor;
pub mod render;

pub fn run() {
    diagnostics::init();
    editor::run();
    log::info!("Application exited normally");
}
