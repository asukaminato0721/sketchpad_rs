//! Compatibility facade for the workspace crates.
pub use sketchpad_engine::*;
#[cfg(feature = "desktop")]
pub use sketchpad_editor::render;
