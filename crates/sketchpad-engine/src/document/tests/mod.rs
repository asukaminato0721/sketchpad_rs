use super::geometry::rotate_point_about;
use super::records::{f64_at, put_record, records, u32_at};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeTransformFlavor, NumericExpr, Transform,
    TransformDefinition,
};
use super::{Document, ImagePlacement, IterationDepth, Kind, LineMode, Point, TextStyleCommand};
use crate::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, PresentationOptions, PresentationStop,
    ScrollPlacement, VisibilityAction,
};
use crate::coordinate_measure_records::{CoordinateMeasureKind, CoordinateMeasureValue};
use crate::function_records::SimpleFunction;
use crate::graph::GridForm;
use crate::plot_points_records;
use crate::plot_points_records::PlotPointsMode;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::table_records::{CALCULATOR_PI, NumericUnit, ParameterCalculationGroup};
use crate::value_equation_records::{EquationValue, ValueEquationKind};
use std::fs;
use std::path::Path;

fn transform_is(kind: &Kind, flavor: NativeTransformFlavor, elements: &[ElementType]) -> bool {
    kind.transform_definition().is_some_and(|definition| {
        definition.wire_flavor() == flavor && elements.contains(&definition.element())
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

mod animation_buttons;
mod calculated_iteration;
mod calculation_units;
mod carried_iteration;
mod circle_and_ray_iteration;
mod clipboard;
mod curve_styles;
mod editing;
mod iteration_mappings;
mod locus;
mod mapped_function_plot69;
mod measurement_precision;
mod measurements;
mod multi_page_actions;
mod multi_page_editing;
mod multi_page_geometry;
mod numeric_definition_edits;
mod numeric_iteration;
mod numeric_precision;
mod numeric_transforms;
mod page_links;
mod pages;
mod plot_points;
mod point_iteration;
mod polygons;
mod quantity_functions;
mod referenced_function72;
mod roundtrip;
mod text_and_style;
mod tracing_and_coordinates;
