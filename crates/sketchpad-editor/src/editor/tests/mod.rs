use super::{CircleMeasure, Editor, MarkedAngle, Tool, TranslationMode};
use sketchpad_engine::action_records::{
    AnimationDirection, AnimationSettings, MovementSpeed, PresentationMode, ScrollPlacement,
};
use sketchpad_engine::document::{Document, IterationDepth, Kind, LineMode, Point};
use sketchpad_engine::function_records::SimpleFunction;
use sketchpad_engine::graph::GridForm;
use crate::render::{TraceMark, Viewport};
use sketchpad_engine::value_equation_records::ValueEquationKind;
mod actions;
mod colorized;
mod referenced_captions;
mod animation_buttons;
mod animation_motion;
mod animation_sources;
mod carried_iteration;
mod custom_tools;
mod functions;
mod iteration;
mod marked_iteration;
mod measurements;
mod multi_page_construction;
mod page_actions;
mod picking_and_tracing;
mod polygons;
mod precision_and_clipboard;
mod transforms;
mod translations;

mod iteration_bindings;

mod transformed_arcs;

mod visibility_actions;

mod straight_transform_sources;

mod arc_intersections55;

mod polygon_boundary56;

mod curve_intersections57;

mod round_intersections57;

mod point_curves58;

mod arc_circle59;

mod sector_boundary;

mod shared_geometry60;

mod sampled_mapped61;

mod locus_generic_straights63;
mod scoped_locus67;
mod sampled_collection68;
mod mapped_function_plot69;
mod scoped_point_locus70;
mod referenced_token71;

mod referenced_shared64;

mod circle_interior_iteration65;

mod mapped_rich_labels66;

mod nested_iteration66;
