use super::{CircleMeasure, Editor, MarkedAngle, Tool, TranslationMode};
use crate::render::Viewport;
use sketchpad_engine::action_records::{
    AnimationDirection, AnimationSettings, MovementSpeed, PresentationMode, PresentationOptions,
    PresentationStop,
};
use sketchpad_engine::document::{Document, IterationDepth, Kind, LineMode, Point};
use sketchpad_engine::function_records::SimpleFunction;
use sketchpad_engine::value_equation_records::ValueEquationKind;
mod animation_buttons;
mod animation_motion;
mod animation_sources;
mod carried_iteration;
mod colorized;
mod display_controls;
mod functions;
mod iteration;
mod marked_iteration;
mod measurements;
mod multi_page_construction;
mod page_actions;
mod precision_and_clipboard;
mod translations;

mod iteration_bindings;

mod transformed_arcs;

mod straight_transform_sources;

mod arc_intersections55;

mod polygon_boundary56;

mod curve_intersections57;

mod arc_circle59;
