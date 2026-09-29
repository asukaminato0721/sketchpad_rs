use super::{
    Editor, LinkButtonDraft, MovementButtonDraft, OutputAction, PendingAction,
    PresentationButtonDraft, ScrollButtonDraft, ShapeMeasure,
};
use sketchpad_engine::action_records::{MovementSpeed, PresentationMode, ScrollPlacement};
use sketchpad_engine::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_engine::document::{Document, ImagePlacement, Kind, TextStyleCommand};
use sketchpad_engine::graph::GridForm;
use sketchpad_engine::plot_points_records::PlotPointsMode;
use sketchpad_engine::style_records::{LinePattern, LineWidth, PointStyle, RgbColor};

impl Editor {
    pub(super) fn menu_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("menu").show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Custom Tools", |ui| self.custom_tools_menu(ui));
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
                    if ui.button("Page Setup…").clicked() {
                        self.page_setup_dialog = true;
                        ui.close();
                    }
                    if ui.button("Print Preview").clicked() {
                        self.print_preview_texture = None;
                        self.print_preview_error = None;
                        self.print_preview_dialog = true;
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
                        let movement_pairs = self.selection_order.as_chunks::<2>().0.iter()
                            .map(|pair| (pair[0], pair[1])).collect::<Vec<_>>();
                        let movement_enabled = self.selection_order.len().is_multiple_of(2)
                            && self.doc.can_add_movement_button_for_pairs(&movement_pairs).is_ok();
                        if ui.add_enabled(movement_enabled, egui::Button::new("Movement")).clicked() {
                            self.movement_button_dialog = Some(MovementButtonDraft {
                                pairs: movement_pairs, speed: MovementSpeed::Medium, follow: true,
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
                                options: sketchpad_engine::action_records::PresentationOptions::for_mode(
                                    PresentationMode::Simultaneous,
                                ),
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
                    let crop_pair = if self.selection_order.len() == 2 {
                        let picture = self.selection_order.iter().copied().find(|id| {
                            self.doc.objects.get(id - 1).is_some_and(|object| {
                                matches!(
                                    object.kind,
                                    Kind::Image {
                                        placement: ImagePlacement::Independent(_),
                                        ..
                                    }
                                )
                            })
                        });
                        let polygon = self.selection_order.iter().copied().find(|id| {
                            self.doc
                                .objects
                                .get(id - 1)
                                .is_some_and(|object| object.kind.is_polygon())
                        });
                        picture.zip(polygon).filter(|(picture, polygon)| {
                            self.doc
                                .can_crop_picture_to_polygon(*picture, *polygon)
                                .is_ok()
                        })
                    } else {
                        None
                    };
                    if ui
                        .add_enabled(
                            crop_pair.is_some(),
                            egui::Button::new("Crop Picture to Polygon"),
                        )
                        .clicked()
                    {
                        let (picture, polygon) = crop_pair.unwrap();
                        match self.doc.crop_picture_to_polygon(picture, polygon) {
                            Ok(id) => {
                                self.select_only(id);
                                self.status = "Created a live cropped picture".into();
                            }
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    let split_point = (self.doc.selected.len() == 1)
                        .then(|| *self.doc.selected.iter().next().unwrap())
                        .filter(|&id| self.doc.can_split_point(id));
                    if ui
                        .add_enabled(split_point.is_some(), egui::Button::new("Split Point"))
                        .clicked()
                    {
                        let id = split_point.unwrap();
                        match self.doc.split_point(id) {
                            Ok(()) => self.status = format!("Split point {id} from its parents"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    let merge_points = {
                        let ordered = self
                            .selection_order
                            .iter()
                            .copied()
                            .filter(|id| self.doc.selected.contains(id))
                            .collect::<Vec<_>>();
                        (ordered.len() == 2
                            && self.doc.can_merge_points(ordered[0], ordered[1]))
                        .then_some((ordered[0], ordered[1]))
                    };
                    if ui
                        .add_enabled(merge_points.is_some(), egui::Button::new("Merge Points"))
                        .clicked()
                    {
                        let (source, target) = merge_points.unwrap();
                        match self.doc.merge_points(source, target) {
                            Ok(()) => {
                                self.selection_order = vec![target];
                                self.status =
                                    format!("Merged independent point {source} into point {target}");
                            }
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    let edit_definition = if self.doc.selected.len() == 1 {
                        let id = *self.doc.selected.iter().next().unwrap();
                        self.doc.objects.get(id - 1).and_then(|object| {
                            (object.page == self.doc.active_page && !object.deleted)
                                .then_some(&object.kind)
                        }).and_then(|kind| match kind {
                            Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) => {
                                Some((id, "Edit Function"))
                            }
                            Kind::Parameter(_) => Some((id, "Edit Parameter")),
                            Kind::Calculation(_) => Some((id, "Edit Calculation")),
                            _ => None,
                        })
                    } else {
                        None
                    };
                    if ui
                        .add_enabled(
                            edit_definition.is_some(),
                            egui::Button::new(
                                edit_definition.map_or("Edit Definition", |(_, label)| label),
                            ),
                        )
                        .clicked()
                    {
                        let (id, _) = edit_definition.unwrap();
                        if let Err(error) = self.begin_edit_definition(id) {
                            self.status = error;
                        }
                        ui.close();
                    }
                    let properties_id = (self.doc.selected.len() == 1)
                        .then(|| *self.doc.selected.iter().next().unwrap())
                        .filter(|id| {
                            self.doc.objects.get(id - 1).is_some_and(|object| {
                                object.page == self.doc.active_page && !object.deleted
                            })
                        });
                    if ui
                        .add_enabled(properties_id.is_some(), egui::Button::new("Properties…"))
                        .clicked()
                    {
                        self.properties_dialog = properties_id;
                        self.properties_draft_for = None;
                        ui.close();
                    }
                    if ui.button("Preferences…").clicked() {
                        self.preferences_draft = self.preferences;
                        self.preferences_dialog = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.doc.undo.is_empty(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.clear_animation_tracks();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.undo();
                        self.reset_custom_tool_after_history();
                        self.image_textures.clear();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.doc.redo.is_empty(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.clear_animation_tracks();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.redo();
                        self.reset_custom_tool_after_history();
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
                    if ui.button("Arc by Center").clicked() {
                        self.center_arc();
                        ui.close();
                    }
                    if ui.button("Arc on Circle").clicked() {
                        self.arc_on_circle();
                        ui.close();
                    }
                    if ui.button("Circle Interior").clicked() {
                        self.circle_interior();
                        ui.close();
                    }
                    if ui.button("Sector Interior").clicked() {
                        self.sector_boundary();
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
                        self.measure_shape(ShapeMeasure::Circumference);
                        ui.close();
                    }
                    if ui.button("Angle").clicked() {
                        self.measure_angle();
                        ui.close();
                    }
                    if ui.button("Radius").clicked() {
                        self.measure_shape(ShapeMeasure::Radius);
                        ui.close();
                    }
                    if ui.button("Area").clicked() {
                        self.measure_shape(ShapeMeasure::Area);
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
                    if ui.button("Live Caption from Selection").clicked() {
                        self.add_selected_referenced_caption();
                        ui.close();
                    }
                    if ui.button("Attach Selected Value to Point").clicked() {
                        self.attach_numeric_label_selected();
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
                        self.calculation_edit_id = None;
                        self.calculation_draft = if self.doc.selected.len() >= 2 {
                            let ids = self.selection_order.iter().copied().filter(|id| self.doc.selected.contains(id)).collect::<Vec<_>>();
                            let argument = ids.iter().position(|id| self.doc.objects[*id - 1].kind.numeric_unit().is_some())
                                .map(|index| format!("v{}", index+1)).unwrap_or_else(|| "2".into());
                            let calls = ids.iter().enumerate().filter(|(_, id)| matches!(self.doc.objects[**id - 1].kind, Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                                .map(|(index, _)| format!("f{}({argument})", index+1)).collect::<Vec<_>>();
                            if calls.is_empty() { "a + b".into() } else { calls.join(" + ") }
                        } else if self.doc.selected.len() == 1 {
                            match self.doc.selected.iter().next().and_then(|&id| self.doc.objects.get(id - 1)).map(|object| &object.kind) {
                                Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)) => "f1(2)".into(),
                                Some(Kind::Parameter(group)) => {
                                    let source = *self.doc.selected.iter().next().unwrap();
                                    let name = self.doc.parameter_display_name(source).unwrap_or_else(|| "v1".into());
                                    if group.display_mode() == 0 { format!("({name})^2") } else { format!("{name}^2") }
                                },
                                Some(kind) if kind.numeric_unit() == Some(sketchpad_engine::table_records::NumericUnit::Degrees) => "v / 2".into(),
                                Some(Kind::AngleMeasurement(_,_,_) | Kind::ArcAngle(_)) => "v / 2".into(),
                                Some(Kind::ParameterCalculation(_, group)) if matches!(group.unit(),sketchpad_engine::table_records::NumericUnit::Length | sketchpad_engine::table_records::NumericUnit::Area) => "v * 2".into(),
                                Some(Kind::ParameterCalculation(_, _)) => "v + 1".into(),
                                Some(kind) if kind.is_distance_value() => "× 2".into(),
                                Some(kind) if kind.numeric_unit()==Some(sketchpad_engine::table_records::NumericUnit::Area) => "v * 2".into(),
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
                        self.begin_function_dialog(false, None);
                        ui.close();
                    }
                    let selected_function = if self.doc.selected.len() == 1 {
                        self.doc.selected.iter().copied().next().filter(|&id| {
                            self.doc.objects.get(id - 1).is_some_and(|object| {
                                !object.deleted
                                    && matches!(
                                        object.kind,
                                        Kind::FunctionDefinition(_)
                                            | Kind::ReferencedFunction(_, _)
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
                            | Kind::ReferencedFunction(_, expression)
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
                    let selected_picture = match self.selection_order.as_slice() {
                        [id]
                            if self.doc.objects.get(id - 1).is_some_and(|object| {
                                !object.deleted
                                    && object.page == self.doc.active_page
                                    && matches!(
                                        object.kind,
                                        Kind::Image {
                                            placement: ImagePlacement::Independent(_),
                                            ..
                                        }
                                    )
                            }) => Some(*id),
                        _ => None,
                    };
                    if ui
                        .add_enabled(
                            selected_picture.is_some(),
                            egui::Button::new("Define Function from Picture"),
                        )
                        .clicked()
                    {
                        match self
                            .doc
                            .add_picture_function(selected_picture.unwrap(), self.center)
                        {
                            Ok(id) => {
                                self.select_only(id);
                                self.status = format!("Created picture function {id}");
                            }
                            Err(error) => self.status = error,
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
                                        Some(Kind::Segment(_, _) | Kind::ThreePointArc(..) | Kind::ArcOnCircle(..) | Kind::CenterArc(..)) => "0.5",
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
                                    Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)
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
                            self.begin_function_dialog(true, None);
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
                                        Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)
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
                    if ui.button("Iterate…").clicked() {
                        self.iterate_dialog = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.can_define_custom_transform(),
                            egui::Button::new("Define Custom Transform…"),
                        )
                        .clicked()
                    {
                        self.begin_define_custom_transform();
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            !self.custom_transforms.is_empty(),
                            egui::Button::new("Edit Custom Transforms…"),
                        )
                        .clicked()
                    {
                        self.custom_transform_edit_selected = 0;
                        self.edit_custom_transforms_dialog = true;
                        ui.close();
                    }
                    if !self.custom_transforms.is_empty() {
                        ui.separator();
                    }
                    for (index, definition) in self.custom_transforms.clone().into_iter().enumerate()
                    {
                        let enabled = self.selection_order.len() == 1
                            && self
                                .doc
                                .custom_transform_carried(definition.seed, definition.output)
                                .is_ok();
                        if ui
                            .add_enabled(enabled, egui::Button::new(definition.name))
                            .clicked()
                        {
                            self.apply_custom_transform(index);
                            ui.close();
                        }
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
                                self.apply_selected_text_style(command);
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
                            let response = ui.button(label);
                            if response.clicked() {
                                let update_defaults = !ui.input(|input| input.modifiers.shift);
                                self.apply_selected_point_style(style, update_defaults);
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
                            let response = ui.button(label);
                            if response.clicked() {
                                let update_defaults = !ui.input(|input| input.modifiers.shift);
                                self.apply_selected_line_width(width, update_defaults);
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
                            let response = ui.button(label);
                            if response.clicked() {
                                let update_defaults = !ui.input(|input| input.modifiers.shift);
                                self.apply_selected_line_pattern(pattern, update_defaults);
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
                            let response = ui.button(label);
                            if response.clicked() {
                                let update_defaults = !ui.input(|input| input.modifiers.shift);
                                self.apply_selected_color(color, update_defaults);
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Color from Values", |ui| {
                        for (label, model) in [
                            ("Spectrum", super::colorized::ColorModel::Spectrum),
                            ("Grayscale", super::colorized::ColorModel::Grayscale),
                            ("RGB (red, green, blue)", super::colorized::ColorModel::Rgb),
                            ("HSV (hue, saturation, brightness)", super::colorized::ColorModel::Hsv),
                        ] {
                            if ui.button(label).clicked() {
                                self.colorize_selected(model);
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
                    let selected_labels = self
                        .doc
                        .selected
                        .iter()
                        .copied()
                        .filter(|&id| {
                            self.doc.can_edit_geometry_label(id)
                                && self.doc.objects[id - 1].label.is_some()
                        })
                        .collect::<Vec<_>>();
                    let labels_visible = !selected_labels.is_empty()
                        && selected_labels.iter().all(|&id| {
                            self.doc.objects[id - 1]
                                .label
                                .as_ref()
                                .is_some_and(|label| label.visible())
                        });
                    if ui
                        .add_enabled(
                            !selected_labels.is_empty(),
                            egui::Button::new(if labels_visible {
                                "Hide Labels"
                            } else {
                                "Show Labels"
                            }),
                        )
                        .clicked()
                    {
                        self.status = match self
                            .doc
                            .set_selected_geometry_label_visibility(!labels_visible)
                        {
                            Ok(count) => format!("Updated {count} object label(s)"),
                            Err(error) => error,
                        };
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
                    if ui
                        .button(if self.show_text_palette {
                            "Hide Text Palette"
                        } else {
                            "Show Text Palette"
                        })
                        .clicked()
                    {
                        self.show_text_palette = !self.show_text_palette;
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
