use super::geometry::{intersect_straight, reflect_point, rotate_point_about};
use super::kind::{is_segment_kind, straight_kind_mode};
use super::records::{f64_at, header, put_record, records, u32_at};
use super::transform_model::{ElementType, NativeLiteralRole, NativeTransformFlavor};
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

fn transform_is(kind: &Kind, flavor: NativeTransformFlavor, elements: &[ElementType]) -> bool {
    kind.transform_definition().is_some_and(|definition| {
        definition.wire_flavor() == flavor && elements.contains(&definition.element())
    })
}

fn transform_literal_is(kind: &Kind, role: NativeLiteralRole, value: f64) -> bool {
    kind.transform_definition().is_some_and(|definition| {
        definition
            .native_literals()
            .iter()
            .any(|literal| literal.role == role && literal.value == value)
    })
}

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
mod arc_plot_paths;
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
mod embedded_tools;
mod equations;
mod fixed_straight_dependents;
mod functions;
mod iteration_mappings;
mod labeled_literal_paths;
mod legacy;
mod locus;
mod numeric_locus;
mod locus_metadata54;
mod iteration_endpoint;
mod mapped_geometry;
mod mapped_iteration;
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
mod plot_path_header_flags;
mod plot_paths_and_tables;
mod plot_points;
mod point_iteration;
mod polygon_boundary;
mod polygons;
mod quantity_functions;
mod roundtrip;
mod single_seed_iteration;
mod text_and_style;
mod tracing_and_coordinates;
mod transformed_plot_paths;
mod transforms;
mod translations;

mod mixed_straight_chains;

mod tool_structure;

mod tool_execution;
mod tool_numeric_execution;
mod tool_numeric_units;
mod tool_path_decoding;
mod tool_segment_inputs;

mod tool_creation;

mod tool_numeric_placement;

mod numeric_point_clipboard;

mod movement_controls;

mod iteration_bindings;

mod transformed_arcs;

mod visibility_options;

mod polygon_point_values;
mod referenced_captions;
mod referenced_function72;
mod referenced_function75;
mod standalone_custom_point73;
mod transformed_source_custom_point74;
mod fixedpolar_source_custom_point76;
mod markedpolar_source_custom_point77;
mod coordinate_marker78;
mod sector_transform79;
mod sector_chain80;
mod sector_rotation_chain81;
mod referenced_token71;
mod referenced_shared64;

mod mapped_elements;

mod axis_point_constraints;
mod derived_line_point_constraints;
mod polygon_constraint_metadata;

mod scalar_iteration_depth;

mod colorized;

mod point_numeric_labels;

mod mapped_numeric_labels;
mod mapped_rich_labels65;
mod nested_iteration66;
mod scoped_locus67;
mod sampled_collection68;
mod mapped_function_plot69;
mod scoped_point_locus70;

mod coordinate_points;
mod bisector_parameters;

mod inherited_point_transforms;

mod straight_transform_sources54;

mod arc_intersections55;

mod polygon_boundary56;

mod inherited_numeric_transforms;

mod calculation_tokens;

mod curve_intersections57;

mod round_intersections57;

mod point_curves58;

mod polygon_identity;
mod polygon_walk82;
mod circular_segment83;
mod angle_values84;
mod angle_alias85;
mod colorized_disk86;
mod colorized_iteration88;
mod colorized_polygon89;
mod image_objects91;
mod rich_style92;
mod angle_marker87;

mod plot_value_flags;

mod arc_circle59;

mod sector_boundary;

mod value_straight_paths59;

mod function_points60;

mod circle_interior60;

mod sampled_mapped61;

mod mapped_prefix61;

mod locus_drivers61;
mod locus_point_sources62;
mod locus_generic_straights63;

mod circle_interior_iteration65;
