// Clippy 1.98's `as_chunks` rewrite leaves array references incompatible with
// the existing byte-decoding closures. `chunks_exact` is explicit and correct.
#![allow(clippy::chunks_exact_to_as_chunks)]

pub mod action_records;
pub mod coordinate_measure_records;
pub mod demo;
pub mod document;
pub mod function_records;
pub mod graph;
pub mod image_records;
pub mod plot_axis_records;
pub mod plot_points_records;
pub mod sac;
pub mod style_records;
pub mod table_records;
pub mod text_object;
pub mod text_records;
pub mod text_style_records;
pub mod value_equation_records;

#[cfg(test)]
#[path = "../../sketchpad-editor/src/render.rs"]
pub mod render;
