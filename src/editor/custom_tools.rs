use super::{Editor, Tool};
use sketchpad_rs::document::{EmbeddedToolInputKind, EmbeddedToolSummary, Kind, LineMode, Point};
use sketchpad_rs::render::Viewport;

#[derive(Clone, Copy)]
pub(super) enum PathStart {
    Existing(usize),
    New(Point),
}

fn tool_name(index: usize, tool: &EmbeddedToolSummary) -> String {
    tool.name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("Tool {}", index + 1))
}

impl Editor {
    pub(super) fn reset_custom_tool_after_history(&mut self) {
        if matches!(self.tool, Tool::Custom(_)) {
            self.tool = Tool::Select;
            self.pending = None;
            self.custom_tool_inputs.clear();
            self.custom_tool_path_start = None;
        }
        self.selection_order = self.doc.selected.iter().copied().collect();
    }

    pub(super) fn cancel_custom_tool(&mut self) {
        self.tool = Tool::Select;
        self.pending = None;
        self.custom_tool_inputs.clear();
        self.custom_tool_path_start = None;
        self.status = "Custom tool input selection cancelled".into();
    }

    pub(super) fn custom_tools_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Create New Tool…").clicked() {
            self.create_tool_dialog = true;
            self.tool_name_draft.clear();
            ui.close();
        }
        ui.separator();
        match self.doc.embedded_tools() {
            Ok(tools) if tools.is_empty() => {
                ui.label("No custom tools in this document");
            }
            Ok(tools) => {
                for (index, tool) in tools.iter().enumerate() {
                    ui.menu_button(tool_name(index, tool), |ui| {
                        if let Some(inputs) = &tool.input_ids {
                            ui.label(format!("{} input(s), in order", inputs.len()));
                        }
                        if ui.button("Apply to Selected Inputs").clicked() {
                            self.invoke_selected_tool(index);
                            ui.close();
                        }
                        if ui.button("Choose Inputs by Clicking").clicked() {
                            self.begin_custom_tool(index);
                            ui.close();
                        }
                    });
                }
            }
            Err(error) => {
                ui.label(error);
            }
        }
    }

    pub(super) fn create_selected_tool(&mut self) -> bool {
        match self.doc.create_embedded_tool(&self.tool_name_draft) {
            Ok(index) => {
                self.status = format!("Created custom tool {}", index + 1);
                self.create_tool_dialog = false;
                true
            }
            Err(error) => {
                self.status = error;
                false
            }
        }
    }

    pub(super) fn custom_tool_dialog_ui(&mut self, root: &mut egui::Ui) {
        if !self.create_tool_dialog {
            return;
        }
        let mut create = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("create_custom_tool")).show(root.ctx(), |ui| {
            ui.heading("Create New Tool");
            ui.label("Use the selected construction to define a custom tool.");
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.tool_name_draft);
            });
            ui.horizontal(|ui| {
                create = ui.button("Create").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        if cancel {
            self.create_tool_dialog = false;
        }
        if create {
            self.create_selected_tool();
        }
    }

    pub(super) fn invoke_selected_tool(&mut self, index: usize) -> bool {
        let inputs: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.selected.contains(id))
            .collect();
        match self.doc.invoke_embedded_tool(index, &inputs) {
            Ok(outputs) => {
                self.selection_order = outputs
                    .into_iter()
                    .filter(|id| self.doc.selected.contains(id))
                    .collect();
                self.pending = None;
                self.custom_tool_path_start = None;
                self.custom_tool_inputs.clear();
                self.status = format!(
                    "Applied custom tool: {} output(s)",
                    self.selection_order.len()
                );
                true
            }
            Err(error) => {
                self.status = error;
                false
            }
        }
    }

    fn custom_tool_binding(&self, index: usize) -> Result<(String, usize), String> {
        let tools = self.doc.embedded_tools()?;
        let tool = tools.get(index).ok_or("Custom tool does not exist")?;
        let inputs = tool
            .input_ids
            .as_ref()
            .ok_or("Custom tool input declaration is unsupported")?;
        let prototype = self.doc.embedded_tool_document(index)?;
        if prototype.partial_view {
            return Err("Tool prototype contains unsupported records or constructions".into());
        }
        if inputs.is_empty()
            || self.doc.embedded_tool_inputs(index)?.iter().any(|input| {
                !prototype
                    .objects
                    .get(input.local_id - 1)
                    .is_some_and(|object| match input.kind() {
                        Some(EmbeddedToolInputKind::Point) => matches!(object.kind, Kind::Free(_)),
                        Some(
                            kind @ (EmbeddedToolInputKind::Path
                            | EmbeddedToolInputKind::Segment
                            | EmbeddedToolInputKind::Numeric),
                        ) => {
                            matches!(object.kind, Kind::ToolInput(actual) if actual == kind)
                        }
                        _ => false,
                    })
            })
        {
            return Err("This custom tool requires unsupported input types".into());
        }
        Ok((tool_name(index, tool), inputs.len()))
    }

    pub(super) fn begin_custom_tool(&mut self, index: usize) {
        let result = self.custom_tool_binding(index);
        match result {
            Ok((name, count)) => {
                self.tool = Tool::Custom(index);
                self.pending = None;
                self.custom_tool_inputs.clear();
                self.custom_tool_path_start = None;
                self.doc.selected.clear();
                self.selection_order.clear();
                self.status = format!("{name}: choose {count} input(s) in order");
            }
            Err(error) => self.status = error,
        }
    }

    pub(super) fn custom_tool_click(&mut self, index: usize, viewport: &Viewport, at: egui::Pos2) {
        let result = (|| {
            let (name, count) = self.custom_tool_binding(index)?;
            if self.custom_tool_inputs.len() >= count {
                return Err("Choose the custom tool again to restart its inputs".into());
            }
            let input = self.doc.embedded_tool_inputs(index)?[self.custom_tool_inputs.len()];
            let id = if matches!(
                input.kind(),
                Some(EmbeddedToolInputKind::Path | EmbeddedToolInputKind::Segment)
            ) {
                let Some(id) = self.custom_tool_path_click(
                    viewport,
                    at,
                    input.kind() == Some(EmbeddedToolInputKind::Segment),
                )?
                else {
                    return Ok(None);
                };
                id
            } else if input.kind() == Some(EmbeddedToolInputKind::Numeric) {
                let overlays = self.overlay_rows();
                self.doc
                    .objects
                    .iter()
                    .rev()
                    .find(|object| {
                        self.doc.is_displayed(object.id)
                            && object.kind.numeric_unit().is_some()
                            && self
                                .doc
                                .numeric_value(object.id)
                                .is_some_and(f64::is_finite)
                            && (sketchpad_rs::render::number_screen_rect_for_object(
                                viewport, &self.doc, object.id,
                            )
                            .is_some_and(|rect| rect.contains(at))
                                || overlays.iter().enumerate().any(|(row, (id, text))| {
                                    *id == object.id
                                        && self
                                            .overlay_row_rect(viewport, row, *id, text)
                                            .contains(at)
                                }))
                    })
                    .map(|object| object.id)
                    .ok_or("Choose an available numeric caption")?
            } else if let Some(id) = self.hit_point(viewport, at) {
                id
            } else {
                let world = viewport.world(at);
                let kind = self
                    .hit(viewport, at)
                    .and_then(|host| self.doc.constrained_point(host, world))
                    .unwrap_or_else(|| Kind::Free(self.doc.snap_point(world)));
                let mut candidate = self.doc.clone();
                let id = candidate.add(kind);
                candidate.encode()?;
                self.doc = candidate;
                id
            };
            Ok(Some((id, count, name)))
        })();
        match result {
            Ok(Some((id, count, name))) => {
                if self.custom_tool_inputs.is_empty() {
                    self.doc.selected.clear();
                    self.selection_order.clear();
                }
                self.custom_tool_inputs.push(id);
                self.selection_order.push(id);
                self.doc.selected.insert(id);
                if self.selection_order.len() == count {
                    self.invoke_selected_tool(index);
                    self.custom_tool_inputs.clear();
                    self.custom_tool_path_start = None;
                } else {
                    self.status = format!(
                        "{name}: choose input {} of {count}",
                        self.selection_order.len() + 1
                    );
                }
            }
            Ok(None) => {}
            Err(error) => self.status = error,
        }
    }

    fn custom_tool_path_click(
        &mut self,
        viewport: &Viewport,
        at: egui::Pos2,
        finite_segment: bool,
    ) -> Result<Option<usize>, String> {
        if self.custom_tool_path_start.is_none() {
            if let Some(path) = self.hit_custom_tool_path(viewport, at, finite_segment) {
                return Ok(Some(path));
            }
            if finite_segment {
                return Err("Choose an existing finite segment for this tool input".into());
            }
            self.custom_tool_path_start = Some(match self.hit_point(viewport, at) {
                Some(id) => PathStart::Existing(id),
                None => PathStart::New(self.doc.snap_point(viewport.world(at))),
            });
            self.status = "Choose the second endpoint for the tool input segment".into();
            return Ok(None);
        }
        let mut candidate = self.doc.clone();
        let history = candidate.undo.len();
        let first = match self.custom_tool_path_start.unwrap() {
            PathStart::Existing(id) => id,
            PathStart::New(point) => candidate.add(Kind::Free(point)),
        };
        let second = if let Some(id) = self.hit_point(viewport, at) {
            id
        } else {
            let world = viewport.world(at);
            let kind = self
                .hit(viewport, at)
                .and_then(|host| candidate.constrained_point(host, world))
                .unwrap_or_else(|| Kind::Free(candidate.snap_point(world)));
            candidate.add(kind)
        };
        if candidate.position(first) == candidate.position(second) {
            return Err("Choose a different second endpoint".into());
        }
        let segment = candidate.add(Kind::Segment(first, second));
        candidate.encode()?;
        // Endpoint creation and its segment form one completed input construction.
        candidate.undo.truncate(history + 1);
        self.doc = candidate;
        self.custom_tool_path_start = None;
        Ok(Some(segment))
    }

    fn hit_custom_tool_path(
        &self,
        viewport: &Viewport,
        at: egui::Pos2,
        finite_segment: bool,
    ) -> Option<usize> {
        self.doc
            .objects
            .iter()
            .rev()
            .filter(|object| self.doc.is_displayed(object.id))
            .filter(|object| {
                !finite_segment
                    || self
                        .doc
                        .line_points(object.id)
                        .is_some_and(|(_, _, mode)| mode == LineMode::Segment)
            })
            .filter_map(|object| {
                let distance = if let Some((start, end, mode)) = self.doc.line_points(object.id) {
                    let a = viewport.screen(start);
                    let b = viewport.screen(end);
                    let direction = b - a;
                    let parameter = if direction.length_sq() < 1e-6 {
                        0.
                    } else {
                        (at - a).dot(direction) / direction.length_sq()
                    };
                    let parameter = match mode {
                        LineMode::Segment => parameter.clamp(0., 1.),
                        LineMode::Ray => parameter.max(0.),
                        LineMode::Line => parameter,
                    };
                    at.distance(a + direction * parameter)
                } else if let Some((center, radius)) = self.doc.circle_points(object.id) {
                    (at.distance(viewport.screen(center)) - radius as f32 * viewport.scale.abs())
                        .abs()
                } else if let Some(points) = self.doc.arc_points(object.id, 128) {
                    points
                        .windows(2)
                        .map(|pair| {
                            segment_distance(at, viewport.screen(pair[0]), viewport.screen(pair[1]))
                        })
                        .min_by(f32::total_cmp)?
                } else if let Some(points) = self.doc.polygon_points(object.id) {
                    let points: Vec<_> = points
                        .into_iter()
                        .map(|point| viewport.screen(point))
                        .collect();
                    let edges = || {
                        points
                            .iter()
                            .zip(points.iter().cycle().skip(1))
                            .take(points.len())
                    };
                    let inside = edges().fold(false, |inside, (a, b)| {
                        inside
                            ^ ((a.y > at.y) != (b.y > at.y)
                                && at.x < (b.x - a.x) * (at.y - a.y) / (b.y - a.y) + a.x)
                    });
                    if inside {
                        0.
                    } else {
                        edges()
                            .map(|(a, b)| segment_distance(at, *a, *b))
                            .min_by(f32::total_cmp)?
                    }
                } else {
                    return None;
                };
                (distance < 7.).then_some((object.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }
}

fn segment_distance(at: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
    let direction = b - a;
    let fraction = if direction.length_sq() < 1e-6 {
        0.
    } else {
        ((at - a).dot(direction) / direction.length_sq()).clamp(0., 1.)
    };
    at.distance(a + direction * fraction)
}
