use super::i18n::tr;
use super::{Editor, Tool};
use crate::render::Viewport;
use sketchpad_engine::document::{
    Document, EmbeddedToolInput, EmbeddedToolInputKind, EmbeddedToolSummary, Kind, LineMode, Point,
};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ToolSource {
    Embedded(usize),
    Folder { document: usize, tool: usize },
}

pub(super) struct ToolFolder {
    path: PathBuf,
    documents: Vec<ToolDocument>,
    skipped: usize,
}

struct ToolDocument {
    path: PathBuf,
    document: Document,
    tools: Vec<EmbeddedToolSummary>,
}

impl ToolFolder {
    fn load(path: &Path) -> Result<Self, String> {
        let path = path
            .canonicalize()
            .map_err(|error| format!("Cannot open tool folder: {error}"))?;
        let mut files = std::fs::read_dir(&path)
            .map_err(|error| format!("Cannot read tool folder: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|entry| {
                entry.is_file()
                    && entry
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("gsp"))
            })
            .collect::<Vec<_>>();
        files.sort();
        let mut documents = Vec::new();
        let mut skipped = 0;
        for file in files {
            let Ok(document) = Document::open(&file) else {
                skipped += 1;
                continue;
            };
            let Ok(tools) = document.embedded_tools() else {
                skipped += 1;
                continue;
            };
            if !tools.is_empty() {
                documents.push(ToolDocument {
                    path: file,
                    document,
                    tools,
                });
            }
        }
        Ok(Self {
            path,
            documents,
            skipped,
        })
    }

    fn tool_count(&self) -> usize {
        self.documents
            .iter()
            .map(|document| document.tools.len())
            .sum()
    }
}

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
        self.properties_draft_for = None;
        self.selection_order = self.doc.selected.iter().copied().collect();
    }

    pub(super) fn cancel_custom_tool(&mut self) {
        self.cancel_custom_tool_state();
        self.status = "Custom tool input selection cancelled".into();
    }

    fn cancel_custom_tool_state(&mut self) {
        self.tool = Tool::Select;
        self.pending = None;
        self.custom_tool_inputs.clear();
        self.custom_tool_path_start = None;
    }

    pub(super) fn custom_tools_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button(tr("Create New Tool…")).clicked() {
            self.create_tool_dialog = true;
            self.tool_name_draft.clear();
            ui.close();
        }
        ui.separator();
        ui.label(tr("Current Document"));
        match self.doc.embedded_tools() {
            Ok(tools) if tools.is_empty() => {
                ui.label(tr("No custom tools in this document"));
            }
            Ok(tools) => {
                for (index, tool) in tools.iter().enumerate() {
                    ui.menu_button(tool_name(index, tool), |ui| {
                        if let Some(inputs) = &tool.input_ids {
                            ui.label(tr(format!("{} input(s), in order", inputs.len())));
                        }
                        if ui.button(tr("Apply to Selected Inputs")).clicked() {
                            self.invoke_selected_tool(ToolSource::Embedded(index));
                            ui.close();
                        }
                        if ui.button(tr("Choose Inputs by Clicking")).clicked() {
                            self.begin_custom_tool(ToolSource::Embedded(index));
                            ui.close();
                        }
                    });
                }
            }
            Err(error) => {
                ui.label(error);
            }
        }
        if let Some(folder) = &self.tool_folder {
            let entries = folder
                .documents
                .iter()
                .enumerate()
                .flat_map(|(document_index, document)| {
                    document
                        .tools
                        .iter()
                        .enumerate()
                        .map(move |(tool_index, tool)| {
                            (
                                ToolSource::Folder {
                                    document: document_index,
                                    tool: tool_index,
                                },
                                tool_name(tool_index, tool),
                                tool.input_ids.as_ref().map(Vec::len),
                                document
                                    .path
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("tool document")
                                    .to_owned(),
                            )
                        })
                })
                .collect::<Vec<_>>();
            ui.separator();
            ui.label(tr(format!("Tool Folder: {}", folder.path.display())));
            if entries.is_empty() {
                ui.label(tr("No custom tools in this folder"));
            }
            for (source, name, input_count, file_name) in entries {
                ui.menu_button(tr(format!("{name} — {file_name}")), |ui| {
                    if let Some(input_count) = input_count {
                        ui.label(tr(format!("{input_count} input(s), in order")));
                    }
                    if ui.button(tr("Apply to Selected Inputs")).clicked() {
                        self.invoke_selected_tool(source);
                        ui.close();
                    }
                    if ui.button(tr("Choose Inputs by Clicking")).clicked() {
                        self.begin_custom_tool(source);
                        ui.close();
                    }
                });
            }
        }
        ui.separator();
        if ui.button(tr("Choose Tool Folder…")).clicked() {
            self.choose_tool_folder();
            ui.close();
        }
        if ui
            .add_enabled(
                self.tool_folder.is_some(),
                egui::Button::new(tr("Forget Tool Folder")),
            )
            .clicked()
        {
            self.forget_tool_folder();
            ui.close();
        }
    }

    fn choose_tool_folder(&mut self) {
        let mut dialog = rfd::FileDialog::new();
        if let Some(folder) = &self.tool_folder {
            dialog = dialog.set_directory(&folder.path);
        }
        let Some(path) = dialog.pick_folder() else {
            return;
        };
        match ToolFolder::load(&path) {
            Ok(folder) => {
                self.cancel_custom_tool_state();
                let tool_count = folder.tool_count();
                let document_count = folder.documents.len();
                let skipped = folder.skipped;
                self.tool_folder = Some(folder);
                self.status = format!(
                    "Loaded {tool_count} tool(s) from {document_count} document(s){}",
                    if skipped == 0 {
                        String::new()
                    } else {
                        format!("; skipped {skipped} unreadable file(s)")
                    }
                );
            }
            Err(error) => self.status = error,
        }
    }

    fn forget_tool_folder(&mut self) {
        self.cancel_custom_tool_state();
        self.tool_folder = None;
        self.status = "Forgot the tool folder".into();
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
            ui.heading(tr("Create New Tool"));
            ui.label(tr("Use the selected construction to define a custom tool."));
            ui.horizontal(|ui| {
                ui.label(tr("Name"));
                ui.text_edit_singleline(&mut self.tool_name_draft);
            });
            ui.horizontal(|ui| {
                create = ui.button(tr("Create")).clicked();
                cancel = ui.button(tr("Cancel")).clicked();
            });
        });
        if cancel {
            self.create_tool_dialog = false;
        }
        if create {
            self.create_selected_tool();
        }
    }

    pub(super) fn invoke_selected_tool(&mut self, source: ToolSource) -> bool {
        let inputs: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.selected.contains(id))
            .collect();
        let result = match source {
            ToolSource::Embedded(index) => self.doc.invoke_embedded_tool(index, &inputs),
            ToolSource::Folder { document, tool } => {
                let Some(source) = self
                    .tool_folder
                    .as_ref()
                    .and_then(|folder| folder.documents.get(document))
                else {
                    self.status = "Tool folder entry no longer exists".into();
                    return false;
                };
                self.doc
                    .invoke_tool_from_document(&source.document, tool, &inputs)
            }
        };
        match result {
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

    fn custom_tool_document(&self, source: ToolSource) -> Result<(&Document, usize), String> {
        match source {
            ToolSource::Embedded(index) => Ok((&self.doc, index)),
            ToolSource::Folder { document, tool } => self
                .tool_folder
                .as_ref()
                .and_then(|folder| folder.documents.get(document))
                .map(|source| (&source.document, tool))
                .ok_or_else(|| "Tool folder entry no longer exists".into()),
        }
    }

    fn custom_tool_inputs(&self, source: ToolSource) -> Result<Vec<EmbeddedToolInput>, String> {
        let (document, index) = self.custom_tool_document(source)?;
        document.embedded_tool_inputs(index)
    }

    fn custom_tool_binding(&self, source: ToolSource) -> Result<(String, usize), String> {
        let (document, index) = self.custom_tool_document(source)?;
        let tools = document.embedded_tools()?;
        let tool = tools.get(index).ok_or("Custom tool does not exist")?;
        let inputs = tool
            .input_ids
            .as_ref()
            .ok_or("Custom tool input declaration is unsupported")?;
        let prototype = document.embedded_tool_document(index)?;
        if prototype.partial_view {
            return Err("Tool prototype contains unsupported records or constructions".into());
        }
        if inputs.is_empty()
            || document.embedded_tool_inputs(index)?.iter().any(|input| {
                !prototype
                    .objects
                    .get(input.local_id - 1)
                    .is_some_and(|object| match input.kind() {
                        Some(EmbeddedToolInputKind::Point) => matches!(object.kind, Kind::Free(_)),
                        Some(
                            kind @ (EmbeddedToolInputKind::Path
                            | EmbeddedToolInputKind::Segment
                            | EmbeddedToolInputKind::SegmentDistance
                            | EmbeddedToolInputKind::Polygon
                            | EmbeddedToolInputKind::CoordinateSystem
                            | EmbeddedToolInputKind::Numeric
                            | EmbeddedToolInputKind::Distance
                            | EmbeddedToolInputKind::Function
                            | EmbeddedToolInputKind::ScalarParameter
                            | EmbeddedToolInputKind::DistanceParameter
                            | EmbeddedToolInputKind::AngleParameter
                            | EmbeddedToolInputKind::Preimage),
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

    pub(super) fn begin_custom_tool(&mut self, source: ToolSource) {
        let result = self.custom_tool_binding(source);
        match result {
            Ok((name, count)) => {
                self.tool = Tool::Custom(source);
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

    pub(super) fn custom_tool_click(
        &mut self,
        source: ToolSource,
        viewport: &Viewport,
        at: egui::Pos2,
    ) {
        let result = (|| {
            let (name, count) = self.custom_tool_binding(source)?;
            if self.custom_tool_inputs.len() >= count {
                return Err("Choose the custom tool again to restart its inputs".into());
            }
            let input = self.custom_tool_inputs(source)?[self.custom_tool_inputs.len()];
            let id = if matches!(
                input.kind(),
                Some(
                    EmbeddedToolInputKind::Path
                        | EmbeddedToolInputKind::Segment
                        | EmbeddedToolInputKind::SegmentDistance
                )
            ) {
                let Some(id) = self.custom_tool_path_click(
                    viewport,
                    at,
                    matches!(
                        input.kind(),
                        Some(
                            EmbeddedToolInputKind::Segment | EmbeddedToolInputKind::SegmentDistance
                        )
                    ),
                )?
                else {
                    return Ok(None);
                };
                id
            } else if input.kind() == Some(EmbeddedToolInputKind::Polygon) {
                self.hit(viewport, at)
                    .filter(|id| self.doc.objects[*id - 1].kind.is_polygon())
                    .ok_or("Choose an available polygon")?
            } else if input.kind() == Some(EmbeddedToolInputKind::CoordinateSystem) {
                self.hit_grid(viewport, at)
                    .ok_or("Choose an available coordinate system")?
            } else if input.kind() == Some(EmbeddedToolInputKind::Function) {
                let overlays = self.overlay_rows();
                overlays
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(row, (id, text))| {
                        self.doc.objects[*id - 1].kind.is_function()
                            && self
                                .overlay_row_rect(viewport, *row, *id, text)
                                .contains(at)
                    })
                    .map(|(_, (id, _))| *id)
                    .ok_or("Choose an available function caption")?
            } else if input.kind() == Some(EmbeddedToolInputKind::Preimage) {
                self.hit(viewport, at)
                    .filter(|id| self.doc.objects[*id - 1].kind.is_preimage_geometry())
                    .ok_or("Choose an available preimage")?
            } else if matches!(
                input.kind(),
                Some(
                    EmbeddedToolInputKind::Numeric
                        | EmbeddedToolInputKind::Distance
                        | EmbeddedToolInputKind::ScalarParameter
                        | EmbeddedToolInputKind::DistanceParameter
                        | EmbeddedToolInputKind::AngleParameter
                )
            ) {
                let scalar_parameter = input.kind() == Some(EmbeddedToolInputKind::ScalarParameter);
                let distance_parameter =
                    input.kind() == Some(EmbeddedToolInputKind::DistanceParameter);
                let angle_parameter = input.kind() == Some(EmbeddedToolInputKind::AngleParameter);
                let distance = input.kind() == Some(EmbeddedToolInputKind::Distance);
                let overlays = self.overlay_rows();
                self.doc
                    .objects
                    .iter()
                    .rev()
                    .find(|object| {
                        self.doc.is_displayed(object.id)
                            && object.kind.numeric_unit().is_some()
                            && (!distance || object.kind.is_distance_value())
                            && (!scalar_parameter
                                || matches!(&object.kind, Kind::Parameter(group)
                                    if group.unit() == sketchpad_engine::table_records::NumericUnit::Scalar))
                            && (!distance_parameter
                                || matches!(&object.kind, Kind::Parameter(group)
                                    if group.unit() == sketchpad_engine::table_records::NumericUnit::Length))
                            && (!angle_parameter
                                || matches!(&object.kind, Kind::Parameter(group)
                                    if group.unit() == sketchpad_engine::table_records::NumericUnit::Degrees))
                            && self
                                .doc
                                .numeric_value(object.id)
                                .is_some_and(f64::is_finite)
                            && (crate::render::number_screen_rect_for_object(
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
                    self.invoke_selected_tool(source);
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
                } else {
                    let points = self.doc.polygon_points(object.id)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn folder_tool_catalog_invokes_survives_open_and_forgets_without_drawing_edits() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "sketchpad-tool-folder-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let source_path = directory.join("segment-and-midpoint.gsp");

        let mut source = Document::new();
        let a = source.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = source.add(Kind::Free(Point { x: 40., y: 0. }));
        let segment = source.add(Kind::Segment(a, b));
        let midpoint = source.add(Kind::Midpoint(a, b));
        source.selected = BTreeSet::from([a, b, segment, midpoint]);
        source.create_embedded_tool("Segment and midpoint").unwrap();
        source.save_to(&source_path).unwrap();

        let folder = ToolFolder::load(&directory).unwrap();
        assert_eq!(folder.tool_count(), 1);
        let mut editor = Editor::new(Document::new());
        editor.tool_folder = Some(folder);
        let left = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
        let right = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
        editor.doc.selected = BTreeSet::from([left, right]);
        editor.selection_order = vec![right, left];
        assert!(editor.invoke_selected_tool(ToolSource::Folder {
            document: 0,
            tool: 0,
        }));
        assert_eq!(editor.selection_order.len(), 2);
        assert!(
            editor
                .selection_order
                .iter()
                .any(|&id| matches!(editor.doc.objects[id - 1].kind, Kind::Segment(_, _)))
        );
        let object_count = editor.doc.objects.len();

        editor.forget_tool_folder();
        assert!(editor.tool_folder.is_none());
        assert_eq!(editor.doc.objects.len(), object_count);

        editor.tool_folder = Some(ToolFolder::load(&directory).unwrap());
        editor.replace_document(Document::new());
        assert_eq!(editor.tool_folder.as_ref().unwrap().tool_count(), 1);
        editor.forget_tool_folder();
        assert!(editor.tool_folder.is_none());
        assert!(editor.doc.objects.is_empty());

        std::fs::remove_dir_all(directory).unwrap();
    }
}
