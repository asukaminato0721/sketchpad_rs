//! Desktop editor and renderer over the headless document engine.
pub use sketchpad_engine::*;
pub mod render;
mod math_text;
mod editor;

pub fn run() { editor::run(); }
