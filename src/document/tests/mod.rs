use super::geometry::{intersect_straight, reflect_point, rotate_point_about};
use super::kind::{is_segment_kind, straight_kind_mode};
use super::records::{f64_at, header, put_record, records, u32_at};
use super::{Document, IterationDepth, Kind, LineMode, Point, TextStyleCommand};
use crate::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, ScrollPlacement, VisibilityAction,
};
use crate::coordinate_measure_records::{CoordinateMeasureKind, CoordinateMeasureValue};
use crate::function_records::SimpleFunction;
use crate::graph::{GraphPoint, GridForm};
use crate::plot_points_records;
use crate::plot_points_records::PlotPointsMode;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::table_records::{NumericUnit, ParameterCalculationGroup};
use crate::value_equation_records::{EquationValue, ValueEquationKind};
use std::fs;
use std::path::Path;

// Native 0384 stores the initial page at payload offset 16. Older
// construction fixtures predate persistence of the selected view.
fn expected_page(bytes: &[u8], page: usize) -> Vec<u8> {
    let mut expected = bytes.to_vec();
    let record = records(bytes)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x0384)
        .unwrap();
    expected[record.data.start + 16..record.data.start + 18]
        .copy_from_slice(&(page as u16).to_le_bytes());
    expected
}

mod actions;
mod animation_buttons;
mod calculated_iteration;
mod calculation_units;
mod calculations;
mod carried_iteration;
mod circle_and_ray_iteration;
mod clipboard;
mod constructions;
mod coordinates;
mod curve_styles;
mod editing;
mod equations;
mod embedded_tools;
mod functions;
mod fixed_straight_dependents;
mod iteration_mappings;
mod legacy;
mod mapped_iteration;
mod locus;
mod measurement_precision;
mod measurements;
mod multi_page_actions;
mod multi_page_editing;
mod multi_page_geometry;
mod numeric_iteration;
mod numeric_precision;
mod numeric_transforms;
mod page_links;
mod pages;
mod plot_paths_and_tables;
mod transformed_plot_paths;
mod labeled_literal_paths;
mod plot_path_header_flags;
mod arc_plot_paths;
mod plot_points;
mod point_iteration;
mod polygons;
mod polygon_boundary;
mod quantity_functions;
mod roundtrip;
mod text_and_style;
mod tracing_and_coordinates;
mod transforms;
mod translations;
