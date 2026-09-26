use super::{
    CircleMeasure, Editor, LinkButtonDraft, MovementButtonDraft, OutputAction, PendingAction,
    PresentationButtonDraft, ScrollButtonDraft,
};
use sketchpad_rs::action_records::{MovementSpeed, PresentationMode, ScrollPlacement};
use sketchpad_rs::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_rs::document::{Document, Kind, TextStyleCommand};
use sketchpad_rs::graph::GridForm;
use sketchpad_rs::plot_points_records::PlotPointsMode;
use sketchpad_rs::style_records::{LinePattern, LineWidth, PointStyle, RgbColor};

impl Editor {
    pub(super) fn menu_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("menu").show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        if self.doc.dirty {
                            self.pending_action = Some(PendingAction::New);
                        } else {
                            self.replace_document(Document::new());
                        }
                        ui.close();
                    }
                    if ui.button("Close").clicked() {
                        if self.doc.dirty {
                            self.pending_action = Some(PendingAction::New);
                        } else {
                            self.replace_document(Document::new());
                        }
                        ui.close();
                    }
                    if ui.button("Open…").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("Sketchpad document", &["gsp"])
                            .pick_file()
                        {
                            if self.doc.dirty {
                                self.pending_action = Some(PendingAction::Open(p));
                            } else {
                                self.open(p);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Import PNG Picture…").clicked() {
                        self.import_picture();
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save(false);
                        ui.close();
                    }
                    if ui.button("Save As…").clicked() {
                        self.save(true);
                        ui.close();
                    }
                    if ui.button("Export SVG…").clicked() {
                        self.pending_output = Some(OutputAction::Svg);
                        ui.close();
                    }
                    if ui.button("Export PDF…").clicked() {
                        self.pending_output = Some(OutputAction::Pdf);
                        ui.close();
                    }
                    if ui.button("Print…").clicked() {
                        self.pending_output = Some(OutputAction::Print);
                        ui.close();
                    }
                    if ui.button("Document Options…").clicked() {
                        self.document_options_page = self.doc.active_page;
                        self.page_name_draft = self.doc.page_names[self.doc.active_page]
                            .clone()
                            .unwrap_or_default();
                        self.document_options_dialog = true;
                        ui.close();
                    }
                    if ui.button("Quit").clicked() {
                        if self.doc.dirty {
                            self.pending_action = Some(PendingAction::Quit);
                        } else {
                            self.exit_requested = true;
                        }
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    ui.menu_button("Action Buttons", |ui| {
                        let targets: Vec<_> = self
                            .selection_order
                            .iter()
                            .copied()
                            .filter(|id| {
                                self.doc.objects.get(id - 1).is_some_and(|object| {
                                object.page == self.doc.active_page
                                    && object.visible
                                    && !object.deleted
                                    && !matches!(
                                        object.kind,
                                        Kind::HideShowButton { .. }
                                            | Kind::AnimationButton { .. }
                                            | Kind::MovementButton { .. }
                                            | Kind::ScrollButton { .. }
                                            | Kind::PresentationButton { .. }
                                            | Kind::LinkUrlButton { .. }
                                            | Kind::LinkPageButton { .. }
                                            | Kind::Unsupported(_)
                                    )
                                })
                            })
                            .collect();
                        if ui
                            .add_enabled(
                                self.doc.can_add_hide_show_button_for_targets(&targets).is_ok(),
                                egui::Button::new("Hide/Show"),
                            )
                            .clicked()
                        {
                            match self.doc.add_hide_show_button_for_targets(&targets) {
                                Ok(id) => self.select_only(id),
                                Err(error) => self.status = error,
                            }
                            ui.close();
                        }
                        let animation_draft = self.animation_button_draft();
                        let animation_ready = animation_draft.as_ref().is_some_and(|draft| {
                            let settings = draft
                                .targets
                                .iter()
                                .copied()
                                .zip(draft.settings.iter().copied())
                                .collect::<Vec<_>>();
                            self.doc
                                .can_add_animation_button_for_targets_with_settings_and_label(
                                    &settings,
                                    draft.label.as_deref(),
                                )
                                .is_ok()
                        });
                        if ui
                            .add_enabled(
                                animation_ready,
                                egui::Button::new("Animation"),
                            )
                            .clicked()
                        {
                            self.animation_button_dialog = animation_draft;
                            ui.close();
                        }
                        let movement_pair = match self.selection_order.as_slice() {
                            [source, destination]
                                if self
                                    .doc
                                    .can_add_movement_button(*source, *destination)
                                    .is_ok() =>
                            {
                                Some((*source, *destination))
                            }
                            _ => None,
                        };
                        if ui.add_enabled(movement_pair.is_some(),
                            egui::Button::new("Movement")).clicked()
                        {
                            let (source, destination) = movement_pair.unwrap();
                            self.movement_button_dialog = Some(MovementButtonDraft {
                                source, destination, speed: MovementSpeed::Medium, follow: true,
                            });
                            ui.close();
                        }
                        let presentation_targets = self.selection_order.iter().copied()
                            .filter(|id| self.doc.objects.get(id - 1).is_some_and(|object| {
                                object.visible && !object.deleted && object.page == self.doc.active_page
                                    && object.kind.is_action_button()
                            }))
                            .collect::<Vec<_>>();
                        if ui.add_enabled(presentation_targets.len() == self.selection_order.len()
                            && self.doc.can_add_presentation_button_for_targets(&presentation_targets).is_ok(),
                            egui::Button::new("Presentation")).clicked()
                        {
                            self.presentation_button_dialog = Some(PresentationButtonDraft {
                                targets: presentation_targets,
                                mode: PresentationMode::Simultaneous,
                            });
                            ui.close();
                        }
                        if ui.add_enabled(
                            (!self.doc.partial_view && self.doc.page_count == 1)
                                || (self.doc.page_count > 1
                                    && !self.doc.objects.iter().any(|object| matches!(object.kind, Kind::Unsupported(_)))),
                            egui::Button::new("Link…")).clicked()
                        {
                            self.link_dialog = Some(LinkButtonDraft {
                                use_page: self.doc.page_count > 1,
                                target_page: usize::from(self.doc.active_page == 0),
                                target_button_ordinal: None,
                                url: "https://".into(),
                            });
                            ui.close();
                        }
                        let scroll_target = match self.selection_order.as_slice() {
                            [target] if self.doc.can_add_scroll_button(*target).is_ok() => Some(*target),
                            _ => None,
                        };
                        if ui.add_enabled(scroll_target.is_some(),
                            egui::Button::new("Scroll")).clicked()
                        {
                            self.scroll_button_dialog = Some(ScrollButtonDraft {
                                target: scroll_target.unwrap(),
                                placement: ScrollPlacement::UpperLeft,
                            });
                            ui.close();
                        }
                    });
                    if ui
                        .add_enabled(!self.doc.undo.is_empty(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.undo();
                        self.image_textures.clear();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.doc.redo.is_empty(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.redo();
                        self.image_textures.clear();
                        ui.close();
                    }
                    if ui.button("Cut").clicked() {
                        self.cut_selected();
                        ui.close();
                    }
                    if ui.button("Copy").clicked() {
                        self.copy_selected();
                        ui.close();
                    }
                    if ui.button("Paste").clicked() {
                        self.paste();
                        ui.close();
                    }
                    if ui.button("Clear").clicked() {
                        self.clear_selected();
                        ui.close();
                    }
                    if ui.button("Select All").clicked() {
                        self.doc.selected = self
                            .doc
                            .objects
                            .iter()
                            .filter(|o| o.page == self.doc.active_page && o.visible && !o.deleted)
                            .map(|o| o.id)
                            .collect();
                        self.selection_order = self.doc.selected.iter().copied().collect();
                        ui.close();
                    }
                    if ui.button("Select Parents").clicked() {
                        self.select_related(false);
                        ui.close();
                    }
                    if ui.button("Select Children").clicked() {
                        self.select_related(true);
                        ui.close();
                    }
                });
                ui.menu_button("Construct", |ui| {
                    if ui.button("Point on Object").clicked() {
                        self.point_on_object();
                        ui.close();
                    }
                    if ui.button("Segment").clicked() {
                        self.pair(0);
                        ui.close();
                    }
                    if ui.button("Ray").clicked() {
                        self.pair(4);
                        ui.close();
                    }
                    if ui.button("Line").clicked() {
                        self.pair(3);
                        ui.close();
                    }
                    if ui.button("Circle by Center and Point").clicked() {
                        self.pair(1);
                        ui.close();
                    }
                    if ui.button("Circle by Center and Radius Segment").clicked() {
                        self.circle_by_segment();
                        ui.close();
                    }
                    if ui.button("Midpoint").clicked() {
                        self.pair(2);
                        ui.close();
                    }
                    if ui.button("Intersection").clicked() {
                        self.intersection();
                        ui.close();
                    }
                    if ui.button("Angle Bisector").clicked() {
                        self.angle_bisector();
                        ui.close();
                    }
                    if ui.button("Arc through 3 Points").clicked() {
                        self.three_point_arc();
                        ui.close();
                    }
                    if ui.button("Arc on Circle").clicked() {
                        self.arc_on_circle();
                        ui.close();
                    }
                    if ui.button("Polygon Interior").clicked() {
                        self.polygon_interior();
                        ui.close();
                    }
                    if ui.button("Locus").clicked() {
                        self.construct_locus();
                        ui.close();
                    }
                    if ui.button("Parallel Line").clicked() {
                        self.derived_line(false);
                        ui.close();
                    }
                    if ui.button("Perpendicular Line").clicked() {
                        self.derived_line(true);
                        ui.close();
                    }
                });
                ui.menu_button("Measure", |ui| {
                    if ui.button("Length").clicked() {
                        self.measure_length();
                        ui.close();
                    }
                    if ui.button("Distance").clicked() {
                        self.measure_distance();
                        ui.close();
                    }
                    for (label, kind) in [
                        ("Coordinates", CoordinateMeasureKind::Coordinates),
                        ("Abscissa (x)", CoordinateMeasureKind::Abscissa),
                        ("Ordinate (y)", CoordinateMeasureKind::Ordinate),
                        (
                            "Coordinate Distance",
                            CoordinateMeasureKind::CoordinateDistance,
                        ),
                    ] {
                        if ui.button(label).clicked() {
                            self.measure_coordinate(kind);
                            ui.close();
                        }
                    }
                    if ui.button("Ratio").clicked() {
                        self.measure_ratio();
                        ui.close();
                    }
                    if ui.button("Perimeter").clicked() {
                        self.measure_perimeter();
                        ui.close();
                    }
                    if ui.button("Slope").clicked() {
                        self.measure_slope();
                        ui.close();
                    }
                    if ui.button("Value of Point").clicked() {
                        self.measure_value_equation(false);
                        ui.close();
                    }
                    if ui.button("Equation").clicked() {
                        self.measure_value_equation(true);
                        ui.close();
                    }
                    if ui.button("Circumference").clicked() {
                        self.measure_circle(CircleMeasure::Circumference);
                        ui.close();
                    }
                    if ui.button("Angle").clicked() {
                        self.measure_angle();
                        ui.close();
                    }
                    if ui.button("Radius").clicked() {
                        self.measure_circle(CircleMeasure::Radius);
                        ui.close();
                    }
                    if ui.button("Area").clicked() {
                        self.measure_circle(CircleMeasure::Area);
                        ui.close();
                    }
                    if ui.button("Arc Angle").clicked() {
                        self.measure_arc_angle();
                        ui.close();
                    }
                    if ui.button("Arc Length").clicked() {
                        self.measure_arc_length();
                        ui.close();
                    }
                    ui.separator();
                    ui.menu_button("Decimal Places", |ui| {
                        for precision in 0..=5 {
                            if ui.button(precision.to_string()).clicked() {
                                let ids = self.doc.selected.iter().copied().collect::<Vec<_>>();
                                match self.doc.set_numeric_precision(&ids, precision) {
                                    Ok(count) => self.status = format!("Updated precision for {count} numeric captions"),
                                    Err(error) => self.status = error,
                                }
                                ui.close();
                            }
                        }
                    });
                });
                ui.menu_button("Number", |ui| {
                    if ui.button("Attach Selected Value to Point").clicked() {
                        let point=self.doc.selected.iter().copied().find(|id|self.doc.position(*id).is_some());
                        let value=self.doc.selected.iter().copied().find(|id|self.doc.unitless_numeric_value(*id).is_some());
                        let result=if self.doc.selected.len()==2 {
                            point.zip(value).ok_or_else(||"Select one point and one numeric value".to_string()).and_then(|(point,value)|self.doc.add_point_numeric_label(point,value))
                        } else { Err("Select one point and one numeric value".into()) };
                        match result { Ok(id)=>{self.select_only(id);self.status=format!("Created attached numeric label {id}");},Err(error)=>self.status=error }
                        ui.close();
                    }
                    if ui.button("New Parameter (1.00)").clicked() {
                        let anchor = if self.doc.objects.len() == 1 {
                            (10, 38)
                        } else {
                            (10, 5)
                        };
                        match self.doc.add_default_parameter(anchor) {
                            Ok(id) => self.status = format!("Created parameter {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("New Named Parameter…").clicked() {
                        self.parameter_name_draft = "a".into();
                        self.parameter_value_draft = "1".into();
                        self.named_parameter_dialog = true;
                        ui.close();
                    }
                    if ui.button("Edit Selected Parameter…").clicked() {
                        let selected = self.doc.selected.iter().copied().find(|&id| {
                            self.doc
                                .objects
                                .get(id - 1)
                                .is_some_and(|object| matches!(object.kind, Kind::Parameter(_)))
                        });
                        if let Some(id) = selected {
                            if let Kind::Parameter(group) = &self.doc.objects[id - 1].kind {
                                self.parameter_value_draft = group.value().to_string();
                                self.parameter_value_dialog = Some(id);
                            }
                        } else {
                            self.status = "Select a parameter".into();
                        }
                        ui.close();
                    }
                    if ui.button("Calculate…").clicked() {
                        self.calculation_draft = if self.doc.selected.len() >= 2 {
                            "a + b".into()
                        } else if self.doc.selected.len() == 1 {
                            match self.doc.selected.iter().next().and_then(|&id| self.doc.objects.get(id - 1)).map(|object| &object.kind) {
                                Some(Kind::Parameter(group)) => {
                                    let source = *self.doc.selected.iter().next().unwrap();
                                    let name = self.doc.parameter_display_name(source).unwrap_or_else(|| "v1".into());
                                    if group.display_mode() == 0 { format!("({name})^2") } else { format!("{name}^2") }
                                },
                                Some(kind) if kind.numeric_unit() == Some(sketchpad_rs::table_records::NumericUnit::Degrees) => "v / 2".into(),
                                Some(Kind::AngleMeasurement(_,_,_) | Kind::ArcAngle(_)) => "v / 2".into(),
                                Some(Kind::ParameterCalculation(_, group)) if matches!(group.unit(),sketchpad_rs::table_records::NumericUnit::Length | sketchpad_rs::table_records::NumericUnit::Area) => "v * 2".into(),
                                Some(Kind::ParameterCalculation(_, _)) => "v + 1".into(),
                                Some(kind) if kind.is_distance_value() => "× 2".into(),
                                Some(kind) if kind.numeric_unit()==Some(sketchpad_rs::table_records::NumericUnit::Area) => "v * 2".into(),
                                Some(kind) if kind.numeric_unit().is_some() => "v + 1".into(),
                                _ => "2 + 3".into(),
                            }
                        } else { "2 + 3".into() };
                        self.calculation_dialog = true;
                        ui.close();
                    }
                    if ui.button("Tabulate Selected Parameter").clicked() {
                        let pair_selected = self.doc.selected.len() == 2
                            && ["t[1]", "t[2]"].iter().all(|name| {
                                self.doc.selected.iter().any(|id| {
                                    self.doc.objects.get(id - 1).is_some_and(|object| {
                                        matches!(&object.kind, Kind::Parameter(group) if group.name() == *name)
                                    })
                                })
                            });
                        let result =
                            if pair_selected {
                                self.doc.tabulate_default_pair((10, 71))
                            } else {
                                let source = self.doc.selected.iter().next().copied().unwrap_or(0);
                                let anchor = source
                                    .checked_sub(1)
                                    .and_then(|index| self.doc.objects.get(index))
                                    .and_then(|object| match &object.kind {
                                        Kind::Parameter(group) if group.name() != "t[1]" => {
                                            let bottom = group.bbox().1.1;
                                            bottom.checked_add(8).map(|y| (group.anchor().0, y))
                                        }
                                        _ => None,
                                    })
                                    .unwrap_or((10, 38));
                                self.doc.tabulate_default_parameter(source, anchor)
                            };
                        match result {
                            Ok(id) => self.status = format!("Created table {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Add Table Data").clicked() {
                        let id = self.doc.selected.iter().next().copied().unwrap_or(0);
                        if let Err(error) = self.doc.change_table_rows(id, true) {
                            self.status = error;
                        }
                        ui.close();
                    }
                    if ui.button("Remove Table Data").clicked() {
                        let id = self.doc.selected.iter().next().copied().unwrap_or(0);
                        if let Err(error) = self.doc.change_table_rows(id, false) {
                            self.status = error;
                        }
                        ui.close();
                    }
                    if ui.button("New Function…").clicked() {
                        self.function_dialog = Some(false);
                        ui.close();
                    }
                    let selected_function = if self.doc.selected.len() == 1 {
                        self.doc.selected.iter().copied().next().filter(|&id| {
                            self.doc.objects.get(id - 1).is_some_and(|object| {
                                !object.deleted
                                    && matches!(
                                        object.kind,
                                        Kind::FunctionDefinition(_)
                                            | Kind::DerivativeFunction(_, _)
                                    )
                            })
                        })
                    } else {
                        None
                    };
                    if ui
                        .add_enabled(
                            selected_function.is_some(),
                            egui::Button::new("Define Derivative Function"),
                        )
                        .clicked()
                    {
                        let source = selected_function.unwrap();
                        let uses_trigonometry = match self.doc.objects[source - 1].kind {
                            Kind::FunctionDefinition(expression)
                            | Kind::DerivativeFunction(_, expression) => {
                                expression.uses_trigonometry()
                            }
                            _ => false,
                        };
                        if uses_trigonometry && !self.doc.angle_unit_radians {
                            self.derivative_angle_dialog = Some(source);
                        } else {
                            match self.doc.add_derivative_function(source) {
                                Ok(id) => {
                                    self.select_only(id);
                                    self.status = format!("Created derivative function {id}");
                                }
                                Err(error) => self.status = error,
                            }
                        }
                        ui.close();
                    }
                });
                ui.menu_button("Graph", |ui| {
                    let path = self.doc.selected.iter().copied()
                        .find(|&id| self.doc.can_plot_value_on_path(id));
                    if ui
                        .button(if path.is_some() {
                            "Plot Value on Path"
                        } else {
                            "Plot Value on Axis"
                        })
                        .clicked()
                    {
                        let source = self.doc.selected.iter().copied()
                            .find(|&id| self.doc.is_scalar_plot_source(id));
                        self.status = match source {
                            Some(source) => match self.doc.plot_parameter_on_path(source, path) {
                                Ok(id) => format!("Plotted path value {id}"),
                                Err(error) => error,
                            },
                            None => {
                                self.plot_axis_dialog = Some(path);
                                self.plot_axis_draft =
                                    match path.map(|id| &self.doc.objects[id - 1].kind) {
                                        Some(Kind::Segment(_, _) | Kind::ThreePointArc(..) | Kind::ArcOnCircle(..)) => "0.5",
                                        Some(kind) if super::is_circle_construction_host(kind) => "0.25",
                                        _ => "1",
                                    }
                                    .into();
                                String::new()
                            }
                        };
                        ui.close();
                    }
                    if ui.button("Plot Points as (x, y)").clicked() {
                        self.plot_points_dialog = Some(PlotPointsMode::Cartesian);
                        self.plot_points_first_draft = "1".into();
                        self.plot_points_second_draft = "1".into();
                        ui.close();
                    }
                    if ui.button("Plot Points as (r, θ)").clicked() {
                        self.plot_points_dialog = Some(PlotPointsMode::Polar);
                        self.plot_points_first_draft = "1".into();
                        self.plot_points_second_draft = "1".into();
                        ui.close();
                    }
                    if ui.button("Plot Table Data").clicked() {
                        let table = self.doc.selected.iter().next().copied().unwrap_or(0);
                        self.plot_table_dialog = Some(table);
                        self.plot_table_mode = PlotPointsMode::Cartesian;
                        ui.close();
                    }
                    let selected_function = self.doc.selected.iter().copied().find(|&id| {
                        self.doc.objects.get(id - 1).is_some_and(|o| {
                            !o.deleted
                                && matches!(
                                    o.kind,
                                    Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                                )
                        })
                    });
                    if ui
                        .button(if selected_function.is_some() {
                            "Plot Function"
                        } else {
                            "Plot New Function…"
                        })
                        .clicked()
                    {
                        if let Some(definition) = selected_function {
                            match self.doc.plot_function(definition, self.center) {
                                Ok(id) => {
                                    self.select_only(id);
                                    self.status = format!("Plotted function {id}");
                                }
                                Err(error) => self.status = error,
                            }
                        } else {
                            self.function_dialog = Some(true);
                        }
                        ui.close();
                    }
                    if ui.button("Plot Parametric Curve…").clicked() {
                        let functions = self
                            .doc
                            .objects
                            .iter()
                            .filter(|object| {
                                object.page == self.doc.active_page
                                    && !object.deleted
                                    && matches!(
                                        object.kind,
                                        Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                                    )
                            })
                            .map(|object| object.id)
                            .collect::<Vec<_>>();
                        if functions.len() < 2 {
                            self.status = "Define two functions before plotting a parametric curve".into();
                        } else {
                            let mut chosen = self
                                .selection_order
                                .iter()
                                .copied()
                                .filter(|id| functions.contains(id))
                                .collect::<Vec<_>>();
                            for id in functions {
                                if !chosen.contains(&id) {
                                    chosen.push(id);
                                }
                            }
                            self.parametric_x = chosen[0];
                            self.parametric_y = chosen[1];
                            self.parametric_mode = PlotPointsMode::Cartesian;
                            self.parametric_min_draft = "0".into();
                            self.parametric_max_draft = "1".into();
                            self.parametric_dialog = true;
                        }
                        ui.close();
                    }
                    if ui.button(self.define_coordinate_system_label()).clicked() {
                        self.define_coordinate_system();
                        ui.close();
                    }
                    if ui.button("Mark Coordinate System").clicked() {
                        self.mark_coordinate_system();
                        ui.close();
                    }
                    ui.add_enabled_ui(self.doc.active_coordinate_system().is_some(), |ui| {
                        ui.menu_button("Grid Form", |ui| {
                            for (form, label) in [
                                (GridForm::Polar, "Polar"),
                                (GridForm::Square, "Square"),
                                (GridForm::Rectangular, "Rectangular"),
                            ] {
                                if ui
                                    .selectable_label(
                                        self.doc.grid_form[self.doc.active_page] == form,
                                        label,
                                    )
                                    .clicked()
                                {
                                    if let Err(error) = self.doc.set_grid_form(form) {
                                        self.status = error;
                                    }
                                    ui.close();
                                }
                            }
                            ui.separator();
                            let trigonometric = self.doc.axis_trigonometric();
                            if ui
                                .selectable_label(trigonometric, "Trigonometric X Axis")
                                .clicked()
                            {
                                if let Err(error) = self.doc.set_axis_trigonometric(!trigonometric)
                                {
                                    self.status = error;
                                }
                                ui.close();
                            }
                        });
                    });
                    let mut visible = self.doc.grid_enabled();
                    if ui
                        .add_enabled(
                            self.doc.active_coordinate_system().is_some(),
                            egui::Checkbox::new(&mut visible, "Show Grid"),
                        )
                        .changed()
                    {
                        self.doc.set_grid_visible(visible);
                        ui.close();
                    }
                    let mut dotted = self.doc.grid_dotted[self.doc.active_page];
                    if ui
                        .add_enabled(
                            self.doc.active_coordinate_system().is_some(),
                            egui::Checkbox::new(&mut dotted, "Dotted Grid"),
                        )
                        .changed()
                    {
                        self.doc.set_grid_dotted(dotted);
                        ui.close();
                    }
                    let mut snap = self.doc.snap_points[self.doc.active_page];
                    if ui
                        .add_enabled(
                            self.doc.active_coordinate_system().is_some(),
                            egui::Checkbox::new(&mut snap, "Snap Points"),
                        )
                        .changed()
                    {
                        self.doc.set_snap_points(snap);
                        ui.close();
                    }
                });
                ui.menu_button("Transform", |ui| {
                    if ui.button("Mark Center").clicked() {
                        self.mark_center_selected();
                        ui.close();
                    }
                    if ui.button("Mark Mirror").clicked() {
                        self.mark_mirror_selected();
                        ui.close();
                    }
                    if ui.button("Mark Angle").clicked() {
                        self.mark_angle_selected();
                        ui.close();
                    }
                    if ui.button("Mark Ratio").clicked() {
                        self.mark_ratio_selected();
                        ui.close();
                    }
                    if ui.button("Mark Vector").clicked() {
                        self.mark_vector_selected();
                        ui.close();
                    }
                    if ui.button("Mark Distance").clicked() {
                        self.mark_distance_selected();
                        ui.close();
                    }
                    if ui.button("Translate…").clicked() {
                        self.translate_dialog = true;
                        ui.close();
                    }
                    if ui.button("Reflect across Line").clicked() {
                        self.reflect_selected();
                        ui.close();
                    }
                    if ui.button("Dilate by Ratio…").clicked() {
                        self.dilate_dialog = true;
                        ui.close();
                    }
                    if ui.button("Rotate…").clicked() {
                        self.rotate_dialog = true;
                        ui.close();
                    }
                    if ui.button("Iterate Points…").clicked() {
                        self.iterate_dialog = true;
                        ui.close();
                    }
                });
                ui.menu_button("Display", |ui| {
                    ui.menu_button("Text", |ui| {
                        if ui.button("Font Family…").clicked() {
                            self.text_font_input = self
                                .doc
                                .selected
                                .iter()
                                .find_map(|id| self.doc.text_font_state(*id))
                                .map(|style| style.family)
                                .unwrap_or_else(|| "Tahoma".into());
                            self.text_font_dialog = true;
                            ui.close();
                        }
                        for (label, command) in [
                            ("Bold", TextStyleCommand::Bold),
                            ("Italic", TextStyleCommand::Italic),
                            ("Underline", TextStyleCommand::Underline),
                            ("Increase Size", TextStyleCommand::IncreaseSize),
                            ("Decrease Size", TextStyleCommand::DecreaseSize),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.toggle_selected_text_style(command) {
                                    Ok(count) => format!("Formatted {count} text object(s)"),
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Point Style", |ui| {
                        for (label, style) in [
                            ("Dot", PointStyle::Dot),
                            ("Small", PointStyle::Small),
                            ("Medium", PointStyle::Medium),
                            ("Thick", PointStyle::Large),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_point_style(style) {
                                    Ok(0) => "Select at least one point".into(),
                                    Ok(count) => {
                                        format!("Changed point style on {count} object(s)")
                                    }
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Line Width", |ui| {
                        for (label, width) in [
                            ("Hairline", LineWidth::Hairline),
                            ("Thin", LineWidth::Thin),
                            ("Medium", LineWidth::Medium),
                            ("Thick", LineWidth::Thick),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_line_width(width) {
                                    Ok(0) => "Select at least one line or circle".into(),
                                    Ok(count) => format!("Changed line width on {count} object(s)"),
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Line Pattern", |ui| {
                        for (label, pattern) in [
                            ("Solid", LinePattern::Solid),
                            ("Dashed", LinePattern::Dashed),
                            ("Dotted", LinePattern::Dotted),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_line_pattern(pattern) {
                                    Ok(0) => "Select at least one line or circle".into(),
                                    Ok(count) => {
                                        format!("Changed line pattern on {count} object(s)")
                                    }
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Color", |ui| {
                        for (label, color) in [
                            (
                                "Red",
                                RgbColor {
                                    red: 255,
                                    green: 0,
                                    blue: 0,
                                },
                            ),
                            (
                                "Orange",
                                RgbColor {
                                    red: 255,
                                    green: 128,
                                    blue: 0,
                                },
                            ),
                            (
                                "Green",
                                RgbColor {
                                    red: 0,
                                    green: 128,
                                    blue: 0,
                                },
                            ),
                            (
                                "Blue",
                                RgbColor {
                                    red: 0,
                                    green: 0,
                                    blue: 255,
                                },
                            ),
                            (
                                "Navy",
                                RgbColor {
                                    red: 0,
                                    green: 0,
                                    blue: 128,
                                },
                            ),
                            (
                                "Purple",
                                RgbColor {
                                    red: 128,
                                    green: 0,
                                    blue: 128,
                                },
                            ),
                            (
                                "Black",
                                RgbColor {
                                    red: 0,
                                    green: 0,
                                    blue: 0,
                                },
                            ),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_color(color) {
                                    Ok(0) => "Select a styled geometry object".into(),
                                    Ok(count) => format!("Changed color on {count} object(s)"),
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Hide Objects").clicked() {
                        self.doc.set_selected_visibility(false);
                        self.selection_order
                            .retain(|id| self.doc.selected.contains(id));
                        ui.close();
                    }
                    if ui.button("Show All Hidden").clicked() {
                        self.doc.show_all_hidden();
                        ui.close();
                    }
                    let tracing = !self.doc.selected.is_empty()
                        && self
                            .doc
                            .selected
                            .iter()
                            .all(|&id| self.doc.objects.get(id - 1).is_some_and(|o| o.tracing));
                    if ui
                        .button(if tracing {
                            "Stop Tracing"
                        } else {
                            "Trace Objects"
                        })
                        .clicked()
                    {
                        self.status = match self.doc.set_selected_tracing(!tracing) {
                            Ok(0) => "Select a geometric object".into(),
                            Ok(count) => format!("Updated tracing on {count} object(s)"),
                            Err(error) => error,
                        };
                        ui.close();
                    }
                    if ui.button("Erase Traces").clicked() {
                        self.trace_marks.clear();
                        self.status = "Erased traces".into();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Animate Objects").clicked() {
                        self.animate_selected();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.animations.is_empty(), egui::Button::new("Increase Speed"))
                        .clicked()
                    {
                        self.animation_speed = (self.animation_speed * 2.).min(8.);
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.animations.is_empty(), egui::Button::new("Decrease Speed"))
                        .clicked()
                    {
                        self.animation_speed = (self.animation_speed / 2.).max(0.125);
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.animations.is_empty() || !self.movements.is_empty(), egui::Button::new("Stop Animation"))
                        .clicked()
                    {
                        self.stop_animation();
                        ui.close();
                    }
                    if ui
                        .button(if self.show_motion_controller {
                            "Hide Motion Controller"
                        } else {
                            "Show Motion Controller"
                        })
                        .clicked()
                    {
                        self.show_motion_controller = !self.show_motion_controller;
                        ui.close();
                    }
                    if ui.button(if self.show_toolbox { "Hide Toolbox" } else { "Show Toolbox" }).clicked() {
                        self.show_toolbox = !self.show_toolbox;
                        ui.close();
                    }
                    if ui.button("Fit Sketch").clicked() {
                        self.fit();
                        ui.close();
                    }
                });
            });
        });
    }
}
