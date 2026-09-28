pub mod action_records;
pub mod coordinate_measure_records;
pub mod document;
pub mod function_records;
pub mod graph;
pub mod image_records;
pub mod plot_axis_records;
pub mod plot_points_records;
pub mod style_records;
pub mod table_records;
pub mod text_object;
pub mod text_records;
pub mod text_style_records;
pub mod value_equation_records;

#[cfg(test)]
#[path = "../../sketchpad-editor/src/render.rs"]
pub mod render;
