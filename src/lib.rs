//! Compatibility facade for the workspace crates.
#[cfg(feature = "desktop")]
pub use sketchpad_editor::render;
pub use sketchpad_engine::*;
