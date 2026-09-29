//! Desktop editor and renderer over the headless document engine.
pub use sketchpad_engine::*;
mod editor;
pub mod render;

pub fn run() {
    editor::run();
}
