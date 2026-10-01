//! Desktop editor state. Commands, UI, animation, and the window runtime
//! are implemented in private modules alongside this file.

mod action_dialogs;
mod actions;
mod animation;
mod calculation_dialogs;
mod canvas;
mod colorized;
mod construction;
mod custom_tools;
mod custom_transforms;
mod document_dialogs;
mod files;
pub(crate) mod fonts;
mod function_dialogs;
mod measurements;
mod menu;
mod panels;
mod picking;
mod plot_dialogs;
mod preferences;
mod properties;
mod referenced_captions;
mod runtime;
mod selection;
mod sound;
mod transform_dialogs;
mod transforms;
mod ui;

fn is_circle_construction_host(kind: &Kind) -> bool {
    use sketchpad_engine::document::transform_model::{ElementType, NativeTransformFlavor};
    matches!(kind, Kind::Circle(..) | Kind::CircleBySegment(..))
        || kind.transform_definition().is_some_and(|definition| {
            definition.element() == ElementType::Circle
                && matches!(
                    definition.wire_flavor(),
                    NativeTransformFlavor::Reflection
                        | NativeTransformFlavor::FixedDilation
                        | NativeTransformFlavor::PointRatioDilation
                )
        })
}

use crate::render::{TraceSample, Viewport};
use sketchpad_engine::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, PresentationOptions, ScrollPlacement,
    VisibilityAction, VisibilityOptions,
};
use sketchpad_engine::document::{Clipboard, Document, Kind, Point};
use sketchpad_engine::function_records::SimpleFunction;
use sketchpad_engine::plot_points_records::PlotPointsMode;
use sketchpad_engine::table_records::ParameterAnimationSettings;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Point,
    Segment,
    Circle,
    Text,
    Custom(custom_tools::ToolSource),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MarkedAngle {
    Points([usize; 3]),
    Value(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TranslationMode {
    FixedPolar,
    FixedRectangular,
    MarkedDistance,
    RectangularMarked,
    RectangularHorizontalFixed,
    RectangularVerticalFixed,
    MarkedVector,
}
enum PendingAction {
    New,
    Open(PathBuf),
    Quit,
}
#[derive(Clone, Copy)]
enum OutputAction {
    Svg,
    Pdf,
    Print,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaperSize {
    A4,
    Letter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionMagnetism {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Preferences {
    double_click_deselection: bool,
    selection_magnetism: SelectionMagnetism,
    update_colors_automatically: bool,
    randomize_interior_colors: bool,
    background_color: [u8; 3],
    selection_color: [u8; 3],
    point_color: [u8; 3],
    straight_color: [u8; 3],
    round_color: [u8; 3],
    interior_color: [u8; 3],
    point_locus_color: [u8; 3],
    plot_color: [u8; 3],
    fade_traces: bool,
    trace_fade_seconds: f32,
    object_style_defaults: sketchpad_engine::document::ObjectStyleDefaults,
    new_polygon_opacity: u16,
    angle_precision: u16,
    distance_precision: u16,
    other_precision: u16,
    show_text_palette_for_new_captions: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            double_click_deselection: false,
            selection_magnetism: SelectionMagnetism::Medium,
            update_colors_automatically: true,
            randomize_interior_colors: false,
            background_color: [245, 247, 251],
            selection_color: [240, 82, 26],
            point_color: [255, 0, 0],
            straight_color: [0, 0, 128],
            round_color: [0, 128, 0],
            interior_color: [255, 128, 0],
            point_locus_color: [128, 0, 0],
            plot_color: [128, 0, 128],
            fade_traces: false,
            trace_fade_seconds: 5.,
            object_style_defaults: Default::default(),
            new_polygon_opacity: 32767,
            angle_precision: 2,
            distance_precision: 2,
            other_precision: 2,
            show_text_palette_for_new_captions: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PageSetup {
    paper_size: PaperSize,
    landscape: bool,
    margin_mm: f32,
    fit_to_page: bool,
    scale_percent: u16,
}

impl Default for PageSetup {
    fn default() -> Self {
        Self {
            paper_size: PaperSize::A4,
            landscape: false,
            margin_mm: 12.7,
            fit_to_page: true,
            scale_percent: 100,
        }
    }
}

#[derive(Clone, Copy)]
enum ShapeMeasure {
    Circumference,
    Radius,
    Area,
}
#[derive(Clone, Copy)]
struct AnimationTrack {
    id: usize,
    allow_hidden: bool,
    transient_point: bool,
    phase: f64,
    straight_range: Option<(f64, f64)>,
    parameter_domain: Option<(f64, f64)>,
    discrete_remainder: f64,
    direction: f64,
    free_velocity: Option<Point>,
    free_turn_remaining: f64,
    random_state: u64,
    button_settings: Option<AnimationSettings>,
    parameter_settings: Option<ParameterAnimationSettings>,
    start_phase: f64,
    distance_travelled: f64,
}
#[derive(Clone)]
struct AnimationButtonDraft {
    targets: Vec<usize>,
    settings: Vec<AnimationSettings>,
    selected: usize,
    label: Option<String>,
}
#[derive(Clone, Copy)]
struct MovementTrack {
    button: usize,
    source: usize,
    destination: usize,
    initial_destination: Option<sketchpad_engine::document::MovementValue>,
    speed: MovementSpeed,
}
#[derive(Clone)]
struct MovementButtonDraft {
    pairs: Vec<(usize, usize)>,
    speed: MovementSpeed,
    follow: bool,
}
#[derive(Clone, Copy)]
struct ScrollButtonDraft {
    target: usize,
    placement: ScrollPlacement,
}
#[derive(Clone)]
struct PresentationButtonDraft {
    targets: Vec<usize>,
    mode: PresentationMode,
    options: PresentationOptions,
}
#[derive(Clone)]
struct LinkButtonDraft {
    use_page: bool,
    target_page: usize,
    target_button_ordinal: Option<usize>,
    url: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CustomTransform {
    name: String,
    seed: usize,
    output: usize,
}
#[derive(Clone)]
struct PresentationTrack {
    id: usize,
    targets: Vec<usize>,
    mode: PresentationMode,
    options: PresentationOptions,
    next: usize,
    active: Vec<usize>,
    elapsed_seconds: f64,
    pause_remaining: f64,
}
struct Editor {
    doc: Document,
    clipboard: Option<Clipboard>,
    image_textures: HashMap<usize, egui::TextureHandle>,
    tool: Tool,
    pending: Option<usize>,
    selection_order: Vec<usize>,
    marked_center: Option<usize>,
    marked_mirror: Option<usize>,
    marked_vector: Option<(usize, usize)>,
    marked_distance: Option<usize>,
    marked_rectangular_distances: Option<(usize, usize)>,
    marked_angle: Option<MarkedAngle>,
    marked_ratio: Option<[usize; 3]>,
    marked_ratio_value: Option<usize>,
    trace_marks: Vec<TraceSample>,
    animations: Vec<AnimationTrack>,
    movements: Vec<MovementTrack>,
    presentations: Vec<PresentationTrack>,
    sounds: sound::SoundState,
    animation_speed: f64,
    animation_paused: bool,
    animation_last_tick: Instant,
    animation_bounds: Option<(Point, Point)>,
    viewport_rect: Option<egui::Rect>,
    document_options_dialog: bool,
    document_options_page: usize,
    page_name_draft: String,
    remove_page_confirmation: Option<usize>,
    page_setup_dialog: bool,
    print_preview_dialog: bool,
    page_setup: PageSetup,
    print_preview_texture: Option<egui::TextureHandle>,
    print_preview_error: Option<String>,
    preferences_dialog: bool,
    preferences: Preferences,
    preferences_draft: Preferences,
    show_motion_controller: bool,
    show_toolbox: bool,
    show_text_palette: bool,
    create_tool_dialog: bool,
    tool_name_draft: String,
    custom_tool_inputs: Vec<usize>,
    custom_tool_path_start: Option<custom_tools::PathStart>,
    tool_folder: Option<custom_tools::ToolFolder>,
    custom_transforms: Vec<CustomTransform>,
    define_custom_transform_dialog: Option<(usize, usize)>,
    custom_transform_name_draft: String,
    edit_custom_transforms_dialog: bool,
    custom_transform_edit_selected: usize,
    animation_button_dialog: Option<AnimationButtonDraft>,
    movement_button_dialog: Option<MovementButtonDraft>,
    scroll_button_dialog: Option<ScrollButtonDraft>,
    presentation_button_dialog: Option<PresentationButtonDraft>,
    link_dialog: Option<LinkButtonDraft>,
    dragging: Option<usize>,
    drag_offset: egui::Vec2,
    text_draft: String,
    font_definitions: egui::FontDefinitions,
    font_context: Option<egui::Context>,
    font_definitions_applied: bool,
    loaded_fonts: HashMap<String, bool>,
    text_font_dialog: bool,
    text_font_input: String,
    text_selected_range: Option<(usize, usize, usize)>,
    inspected: Option<usize>,
    edit_buffer: String,
    scale: f32,
    center: Point,
    button_view_center: Point,
    status: String,
    pending_action: Option<PendingAction>,
    pending_output: Option<OutputAction>,
    translate_dialog: bool,
    translation_angle: f64,
    translation_distance: f64,
    translation_horizontal: f64,
    translation_vertical: f64,
    translation_mode: TranslationMode,
    dilate_dialog: bool,
    dilation_factor: f64,
    dilation_use_marked: bool,
    rotate_dialog: bool,
    rotation_angle: f64,
    rotation_use_marked: bool,
    function_dialog: Option<bool>,
    function_edit_id: Option<usize>,
    function_draft: String,
    function_draft_sources: Vec<usize>,
    function_angle_dialog: Option<(SimpleFunction, bool, Option<usize>)>,
    derivative_angle_dialog: Option<usize>,
    parametric_dialog: bool,
    parametric_x: usize,
    parametric_y: usize,
    parametric_mode: PlotPointsMode,
    parametric_min_draft: String,
    parametric_max_draft: String,
    plot_range_dialog: Option<usize>,
    plot_min_draft: String,
    plot_max_draft: String,
    plot_samples_draft: String,
    parameter_value_dialog: Option<usize>,
    parameter_value_draft: String,
    parameter_precision_draft: usize,
    parameter_display_mode_draft: u16,
    parameter_original_display_mode: bool,
    parameter_edit_box_draft: bool,
    parameter_discrete_draft: bool,
    parameter_rate_units_draft: String,
    parameter_rate_seconds_draft: String,
    parameter_min_draft: String,
    parameter_max_draft: String,
    parameter_keyboard_step_draft: String,
    named_parameter_dialog: bool,
    parameter_name_draft: String,
    calculation_dialog: bool,
    calculation_edit_id: Option<usize>,
    calculation_draft: String,
    properties_dialog: Option<usize>,
    properties_draft_for: Option<usize>,
    properties_label_draft: String,
    properties_label_visible_draft: bool,
    properties_opacity_draft: u16,
    properties_visibility_action_draft: VisibilityAction,
    properties_visibility_options_draft: VisibilityOptions,
    properties_animation_settings_draft: Vec<AnimationSettings>,
    properties_animation_target_draft: usize,
    properties_movement_speed_draft: MovementSpeed,
    properties_movement_follow_draft: bool,
    properties_scroll_placement_draft: ScrollPlacement,
    iterate_dialog: bool,
    iteration_depth_draft: String,
    plot_axis_dialog: Option<Option<usize>>,
    plot_axis_draft: String,
    plot_points_dialog: Option<PlotPointsMode>,
    plot_points_first_draft: String,
    plot_points_second_draft: String,
    plot_table_dialog: Option<usize>,
    plot_table_mode: PlotPointsMode,
    exit_requested: bool,
}
impl Editor {
    fn new(doc: Document) -> Self {
        let mut e = Self {
            doc,
            clipboard: None,
            image_textures: HashMap::new(),
            tool: Tool::Select,
            pending: None,
            selection_order: Vec::new(),
            marked_center: None,
            marked_mirror: None,
            marked_vector: None,
            marked_distance: None,
            marked_rectangular_distances: None,
            marked_angle: None,
            marked_ratio: None,
            marked_ratio_value: None,
            trace_marks: Vec::new(),
            animations: Vec::new(),
            movements: Vec::new(),
            presentations: Vec::new(),
            sounds: sound::SoundState::default(),
            animation_speed: 1.,
            animation_paused: false,
            animation_last_tick: Instant::now(),
            animation_bounds: None,
            viewport_rect: None,
            document_options_dialog: false,
            document_options_page: 0,
            page_name_draft: String::new(),
            remove_page_confirmation: None,
            page_setup_dialog: false,
            print_preview_dialog: false,
            page_setup: PageSetup::default(),
            print_preview_texture: None,
            print_preview_error: None,
            preferences_dialog: false,
            preferences: Preferences::default(),
            preferences_draft: Preferences::default(),
            show_motion_controller: false,
            show_toolbox: true,
            show_text_palette: false,
            create_tool_dialog: false,
            tool_name_draft: String::new(),
            custom_tool_inputs: Vec::new(),
            custom_tool_path_start: None,
            tool_folder: None,
            custom_transforms: Vec::new(),
            define_custom_transform_dialog: None,
            custom_transform_name_draft: String::new(),
            edit_custom_transforms_dialog: false,
            custom_transform_edit_selected: 0,
            animation_button_dialog: None,
            movement_button_dialog: None,
            scroll_button_dialog: None,
            presentation_button_dialog: None,
            link_dialog: None,
            dragging: None,
            drag_offset: egui::Vec2::ZERO,
            text_draft: "Text".into(),
            font_definitions: egui::FontDefinitions::default(),
            font_context: None,
            font_definitions_applied: false,
            loaded_fonts: HashMap::new(),
            text_font_dialog: false,
            text_font_input: "Tahoma".into(),
            text_selected_range: None,
            inspected: None,
            edit_buffer: String::new(),
            scale: 1.,
            center: Point { x: 400., y: 300. },
            button_view_center: Point { x: 400., y: 300. },
            status: String::new(),
            pending_action: None,
            pending_output: None,
            translate_dialog: false,
            translation_angle: 90.,
            translation_distance: 1.,
            translation_horizontal: 1.,
            translation_vertical: 1.,
            translation_mode: TranslationMode::FixedPolar,
            dilate_dialog: false,
            dilation_factor: 0.5,
            dilation_use_marked: false,
            rotate_dialog: false,
            rotation_angle: 60.,
            rotation_use_marked: false,
            function_dialog: None,
            function_edit_id: None,
            function_draft: "x".into(),
            function_draft_sources: Vec::new(),
            function_angle_dialog: None,
            derivative_angle_dialog: None,
            parametric_dialog: false,
            parametric_x: 0,
            parametric_y: 0,
            parametric_mode: PlotPointsMode::Cartesian,
            parametric_min_draft: "0".into(),
            parametric_max_draft: "1".into(),
            plot_range_dialog: None,
            plot_min_draft: String::new(),
            plot_max_draft: String::new(),
            plot_samples_draft: String::new(),
            parameter_value_dialog: None,
            parameter_value_draft: "1".into(),
            parameter_precision_draft: 2,
            parameter_display_mode_draft: 3,
            parameter_original_display_mode: false,
            parameter_edit_box_draft: true,
            parameter_discrete_draft: false,
            parameter_rate_units_draft: "1".into(),
            parameter_rate_seconds_draft: "1".into(),
            parameter_min_draft: "-100".into(),
            parameter_max_draft: "100".into(),
            parameter_keyboard_step_draft: "0.01".into(),
            named_parameter_dialog: false,
            parameter_name_draft: "a".into(),
            calculation_dialog: false,
            calculation_edit_id: None,
            calculation_draft: "2 + 3".into(),
            properties_dialog: None,
            properties_draft_for: None,
            properties_label_draft: String::new(),
            properties_label_visible_draft: false,
            properties_opacity_draft: 32767,
            properties_visibility_action_draft: VisibilityAction::Hide,
            properties_visibility_options_draft: VisibilityOptions::default(),
            properties_animation_settings_draft: Vec::new(),
            properties_animation_target_draft: 0,
            properties_movement_speed_draft: MovementSpeed::Medium,
            properties_movement_follow_draft: true,
            properties_scroll_placement_draft: ScrollPlacement::UpperLeft,
            iterate_dialog: false,
            iteration_depth_draft: "3".into(),
            plot_axis_dialog: None,
            plot_axis_draft: "1".into(),
            plot_points_dialog: None,
            plot_points_first_draft: "1".into(),
            plot_points_second_draft: "1".into(),
            plot_table_dialog: None,
            plot_table_mode: PlotPointsMode::Cartesian,
            exit_requested: false,
        };
        e.sync_document_preferences();
        e.fit();
        e
    }

    fn fit(&mut self) {
        let mut pts: Vec<_> = self
            .doc
            .objects
            .iter()
            .filter(|o| self.doc.is_displayed(o.id))
            .filter_map(|o| self.doc.position(o.id))
            .collect();
        for object in self
            .doc
            .objects
            .iter()
            .filter(|o| self.doc.is_displayed(o.id))
        {
            if let Kind::PlainText { rect, .. } = &object.kind {
                pts.push(Point {
                    x: rect.left.into(),
                    y: rect.top.into(),
                });
                pts.push(Point {
                    x: rect.right.into(),
                    y: rect.bottom.into(),
                });
            }
            if let Some((anchor, _)) = crate::render::number_lines_for_object(&self.doc, object.id)
            {
                pts.push(Point {
                    x: anchor.0 as f64,
                    y: anchor.1 as f64,
                });
            }
            if let Kind::Image { png, .. } = &object.kind
                && let Some(transform) = self.doc.image_transform(object.id)
                && let Some(corners) = crate::render::picture_screen_quad(
                    &Viewport {
                        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO),
                        scale: 1.,
                        center: Point { x: 0., y: 0. },
                    },
                    png,
                    transform,
                )
            {
                // The zero-sized viewport places world positions at the origin.
                pts.extend(corners.map(|corner| Point {
                    x: corner.x as f64,
                    y: corner.y as f64,
                }));
            }
        }
        if pts.is_empty() {
            return;
        }
        let (mut minx, mut miny, mut maxx, mut maxy) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in pts {
            minx = minx.min(p.x);
            miny = miny.min(p.y);
            maxx = maxx.max(p.x);
            maxy = maxy.max(p.y);
        }
        self.center = Point {
            x: (minx + maxx) / 2.,
            y: (miny + maxy) / 2.,
        };
        self.button_view_center = self.center;
        self.scale = (650. / (maxx - minx).max(100.) as f32)
            .min(480. / (maxy - miny).max(100.) as f32)
            .clamp(0.2, 3.);
    }
}

#[cfg(test)]
mod tests;

pub(crate) use runtime::run;
