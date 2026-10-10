//! The traversal is a tagless-final program over `DependencyView`. It does not
//! build a UI tree. Each interpreter decides whether to evaluate a branch body.
use super::{Editor, Tool, i18n::tr};
use sketchpad_engine::document::{Document, Kind, Point};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Default)]
pub(super) struct InspectorState {
    query: String,
    dependents: bool,
    selection: Vec<usize>,
}

struct ObjectNode {
    id: usize,
    page: usize,
    kind: &'static str,
    name: String,
    hidden: bool,
    parents: Vec<usize>,
    children: Vec<usize>,
}

impl ObjectNode {
    fn title(&self) -> String {
        if self.name.is_empty() {
            format!("#{} · {}", self.id, tr(self.kind))
        } else {
            format!("#{} · {} · {}", self.id, tr(self.kind), self.name)
        }
    }
    fn edges(&self, dependents: bool) -> &[usize] {
        if dependents {
            &self.children
        } else {
            &self.parents
        }
    }
}

struct DependencyGraph {
    nodes: BTreeMap<usize, ObjectNode>,
}

impl DependencyGraph {
    fn new(doc: &Document) -> Self {
        let mut nodes = BTreeMap::new();
        for object in doc.objects.iter().filter(|object| !object.deleted) {
            let kind = match &object.kind {
                Kind::Free(_) => "Free Point",
                Kind::Midpoint(..) => "Midpoint",
                Kind::Parallel(..) => "Parallel Line",
                Kind::Perpendicular(..) => "Perpendicular Line",
                Kind::CircleInterior(..) => "Circle Interior",
                Kind::SectorBoundary(..) => "Sector Interior",
                Kind::Parameter(_) => "Parameter",
                Kind::Calculation(_)
                | Kind::MeasuredCalculation(..)
                | Kind::LengthPairCalculation(..)
                | Kind::ParameterCalculation(..) => "Calculation",
                kind if kind.numeric_unit().is_some() => "Measurement",
                _ => super::properties::object_type_name(object),
            };
            let name = object
                .label
                .as_ref()
                .and_then(|label| label.name_utf8().ok())
                .filter(|label| !label.is_empty())
                .map(str::to_owned)
                .or_else(|| match &object.kind {
                    Kind::PlainText { text, .. } => Some(text.clone()),
                    Kind::Parameter(_) => doc.parameter_display_name(object.id),
                    _ => None,
                })
                .unwrap_or_default();
            let name: String = name
                .chars()
                .take(64)
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect();
            let mut parents = doc.dependencies_of(object.id).unwrap_or_default();
            let mut seen = BTreeSet::new();
            parents.retain(|id| seen.insert(*id));
            nodes.insert(
                object.id,
                ObjectNode {
                    id: object.id,
                    page: object.page,
                    kind,
                    name,
                    hidden: !object.visible
                        || (object.page == doc.active_page && !doc.is_displayed(object.id)),
                    parents,
                    children: Vec::new(),
                },
            );
        }
        let edges: Vec<_> = nodes
            .values()
            .flat_map(|node| node.parents.iter().map(move |parent| (*parent, node.id)))
            .collect();
        for (parent, child) in edges {
            if let Some(node) = nodes.get_mut(&parent) {
                node.children.push(child);
            }
        }
        Self { nodes }
    }
}

/// The display algebra. A backend can render, record, or skip the branch body.
/// The generic traversal works without egui or a tagged UI representation.
trait DependencyView: Sized {
    fn object(
        &mut self,
        node: &ObjectNode,
        path: &[usize],
        expandable: bool,
        body: impl FnOnce(&mut Self),
    );
    fn notice(&mut self, message: &str);
}

fn dependency_branch<V: DependencyView>(
    graph: &DependencyGraph,
    id: usize,
    dependents: bool,
    path: &mut Vec<usize>,
    budget: &mut usize,
    view: &mut V,
) {
    if *budget == 0 {
        view.notice("Expand fewer branches to see more objects.");
        return;
    }
    *budget -= 1;
    let Some(node) = graph.nodes.get(&id) else {
        view.notice(&format!("Object #{id} is unavailable."));
        return;
    };
    let cycle = path.contains(&id);
    path.push(id);
    // Copy the path only for the duration of the backend call. No UI AST is built.
    let key = path.clone();
    view.object(node, &key, !node.edges(dependents).is_empty(), |view| {
        if cycle {
            view.notice("Cycle reference; expansion stopped.");
        } else if path.len() >= 24 {
            view.notice("Depth limit reached. Select this object to inspect it directly.");
        } else {
            for &related in node.edges(dependents) {
                if *budget == 0 {
                    view.notice("Expand fewer branches to see more objects.");
                    break;
                }
                dependency_branch(graph, related, dependents, path, budget, view);
            }
        }
    });
    path.pop();
}

#[derive(Clone, Copy)]
struct Pick {
    id: usize,
    multi: bool,
    center: bool,
}

struct EguiDependencyView<'a> {
    ui: egui::Ui,
    selected: &'a BTreeSet<usize>,
    pick: &'a Cell<Option<Pick>>,
    reveal: Option<usize>,
    active_page: usize,
}

impl EguiDependencyView<'_> {
    fn row(
        ui: &mut egui::Ui,
        node: &ObjectNode,
        selected: &BTreeSet<usize>,
        pick: &Cell<Option<Pick>>,
        reveal: bool,
        active_page: usize,
    ) {
        let response = ui
            .selectable_label(selected.contains(&node.id), node.title())
            .on_hover_text(tr(
                "Click to select. Ctrl/Shift-click to add or remove. Double-click to center.",
            ));
        if reveal {
            response.scroll_to_me(Some(egui::Align::Center));
        }
        if response.clicked() || response.double_clicked() {
            pick.set(Some(Pick {
                id: node.id,
                multi: ui.input(|input| input.modifiers.command || input.modifiers.shift),
                center: response.double_clicked(),
            }));
        }
        if node.hidden {
            ui.weak(tr("Hidden"));
        }
        if node.page != active_page {
            ui.weak(tr(format!("Page {}", node.page + 1)));
        }
    }
}

impl DependencyView for EguiDependencyView<'_> {
    fn object(
        &mut self,
        node: &ObjectNode,
        path: &[usize],
        expandable: bool,
        body: impl FnOnce(&mut Self),
    ) {
        let selected = self.selected;
        let reveal = path.len() == 1 && self.reveal == Some(node.id);
        let active_page = self.active_page;
        let pick = self.pick;
        if expandable {
            let id = self.ui.make_persistent_id(path);
            egui::collapsing_header::CollapsingState::load_with_default_open(
                self.ui.ctx(),
                id,
                false,
            )
            .show_header(&mut self.ui, |ui| {
                Self::row(ui, node, selected, pick, reveal, active_page)
            })
            .body(|ui| {
                let mut nested = EguiDependencyView {
                    ui: ui.new_child(egui::UiBuilder::new()),
                    selected,
                    pick,
                    reveal: None,
                    active_page,
                };
                body(&mut nested);
                ui.advance_cursor_after_rect(nested.ui.min_rect());
            });
        } else {
            self.ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent);
                Self::row(ui, node, selected, pick, reveal, active_page);
            });
        }
    }
    fn notice(&mut self, message: &str) {
        self.ui.weak(tr(message));
    }
}

impl Editor {
    pub(super) fn dependency_inspector_ui(&mut self, ui: &mut egui::Ui) {
        let graph = DependencyGraph::new(&self.doc);
        let selection: Vec<_> = self.doc.selected.iter().copied().collect();
        let mut reveal = (selection != self.inspector.selection)
            .then(|| selection.first().copied())
            .flatten();
        self.inspector.selection = selection;
        ui.heading(tr("Dependency tree"));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.inspector.dependents, false, tr("Depends on"));
            ui.selectable_value(&mut self.inspector.dependents, true, tr("Used by"));
        });
        ui.small(tr(if self.inspector.dependents {
            "Expand an object to see what uses it."
        } else {
            "Expand an object to see its source objects."
        }));
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.inspector.query)
                    .desired_width(210.)
                    .hint_text(tr("Filter by ID, type, or label")),
            );
            if ui.button(tr("Show selected")).clicked() {
                self.inspector.query.clear();
                reveal = self.doc.selected.first().copied();
            }
        });
        let query = self.inspector.query.trim().to_lowercase();
        let page_nodes: Vec<_> = graph
            .nodes
            .values()
            .filter(|node| node.page == self.doc.active_page)
            .collect();
        ui.small(tr(format!("{} objects on this page", page_nodes.len())));
        let pick = Cell::new(None);
        egui::ScrollArea::both()
            .id_salt("dependency_tree_scroll")
            .max_height(270.)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.push_id(
                    (
                        "dependency_tree",
                        self.doc.active_page,
                        self.inspector.dependents,
                    ),
                    |ui| {
                        let mut view = EguiDependencyView {
                            ui: ui.new_child(egui::UiBuilder::new()),
                            selected: &self.doc.selected,
                            pick: &pick,
                            reveal,
                            active_page: self.doc.active_page,
                        };
                        let mut any = false;
                        for node in page_nodes.iter().filter(|node| {
                            query.is_empty()
                                || node.title().to_lowercase().contains(&query)
                                || node.kind.to_lowercase().contains(&query)
                        }) {
                            any = true;
                            // Each root remains accessible even if another branch reaches its limit.
                            dependency_branch(
                                &graph,
                                node.id,
                                self.inspector.dependents,
                                &mut Vec::new(),
                                &mut 256,
                                &mut view,
                            );
                        }
                        if !any {
                            view.notice("No matching objects on this page.");
                        }
                        ui.advance_cursor_after_rect(view.ui.min_rect());
                    },
                );
            });
        if let Some(pick) = pick.get() {
            self.inspector_pick(pick);
            // A tree click keeps the expanded branch in view. Canvas picks reveal a root.
            self.inspector.selection = self.doc.selected.iter().copied().collect();
        }
        if let Some(id) = self.doc.selected.first().copied()
            && let Some(node) = graph.nodes.get(&id)
        {
            ui.separator();
            ui.strong(node.title());
            ui.label(tr(format!(
                "{} sources, {} dependents",
                node.parents.len(),
                node.children.len()
            )));
            let evaluated = self.evaluated_document();
            if let Some(point) = evaluated.position(id) {
                ui.monospace(format!("x = {:.4}, y = {:.4}", point.x, point.y));
            }
            if let Some(value) = evaluated.numeric_value(id) {
                ui.label(format!("{}: {value:.6}", tr("Value")));
            }
            if node.hidden {
                ui.weak(tr(
                    "This object is hidden. Change its visibility in Properties.",
                ));
            }
            ui.horizontal(|ui| {
                if ui.button(tr("Center on object")).clicked() {
                    self.inspector_center(id);
                }
                if ui.button(tr("Properties…")).clicked() {
                    self.properties_dialog = Some(id);
                }
            });
        }
    }

    fn inspector_pick(&mut self, pick: Pick) {
        let Some(object) = pick
            .id
            .checked_sub(1)
            .and_then(|index| self.doc.objects.get(index))
        else {
            return;
        };
        if object.deleted || self.dragging.is_some() || self.pending_action.is_some() {
            return;
        }
        let page = object.page;
        if page != self.doc.active_page {
            self.clear_animation_tracks();
            self.movements.clear();
            self.presentations.clear();
            self.stop_all_sounds();
            self.doc.active_page = page;
            self.doc.selected.clear();
            self.selection_order.clear();
            self.fit();
        }
        self.tool = Tool::Select;
        self.pending = None;
        self.custom_tool_inputs.clear();
        self.custom_tool_path_start = None;
        if pick.multi && !pick.center {
            if self.doc.selected.remove(&pick.id) {
                self.selection_order.retain(|id| *id != pick.id);
            } else {
                self.doc.selected.insert(pick.id);
                self.selection_order.push(pick.id);
            }
        } else {
            self.select_only(pick.id);
        }
        if pick.center {
            self.inspector_center(pick.id);
        }
    }

    fn inspector_center(&mut self, id: usize) {
        let evaluated = self.evaluated_document();
        let target = evaluated
            .position(id)
            .or_else(|| {
                evaluated.line_points(id).map(|(a, b, _)| Point {
                    x: (a.x + b.x) / 2.,
                    y: (a.y + b.y) / 2.,
                })
            })
            .or_else(|| evaluated.circle_points(id).map(|(center, _)| center))
            .or_else(|| {
                evaluated
                    .polygon_points(id)
                    .filter(|points| !points.is_empty())
                    .map(|points| Point {
                        x: points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64,
                        y: points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64,
                    })
            })
            .or_else(|| {
                evaluated
                    .objects
                    .get(id.saturating_sub(1))
                    .and_then(|object| match &object.kind {
                        Kind::PlainText { rect, .. } => Some(Point {
                            x: (f64::from(rect.left) + f64::from(rect.right)) / 2.,
                            y: (f64::from(rect.top) + f64::from(rect.bottom)) / 2.,
                        }),
                        _ => None,
                    })
            });
        if let Some(target) = target.filter(|point| point.x.is_finite() && point.y.is_finite()) {
            self.center = target;
        } else {
            self.status = "This object has no center in document coordinates.".into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, FullOutput, PointerButton, Pos2};

    #[derive(Default)]
    struct Outline {
        paths: Vec<Vec<usize>>,
        notices: Vec<String>,
        collapsed: BTreeSet<usize>,
    }
    impl DependencyView for Outline {
        fn object(
            &mut self,
            node: &ObjectNode,
            path: &[usize],
            expandable: bool,
            body: impl FnOnce(&mut Self),
        ) {
            self.paths.push(path.to_vec());
            if expandable && !self.collapsed.contains(&node.id) {
                body(self);
            }
        }
        fn notice(&mut self, message: &str) {
            self.notices.push(message.to_owned());
        }
    }

    fn editor() -> Editor {
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 100., y: 100. }));
        doc.add(Kind::Free(Point { x: 200., y: 100. }));
        doc.add(Kind::Segment(1, 2));
        Editor::new(Document::from_bytes(&doc.encode().unwrap(), None).unwrap())
    }

    #[test]
    #[ignore = "Set SKETCHPAD_INSPECTOR_SAMPLE to a local GSP file"]
    fn external_sample_inspector_frames() {
        let path = std::env::var("SKETCHPAD_INSPECTOR_SAMPLE").unwrap();
        let bytes = std::fs::read(path).unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        eprintln!(
            "Sample: {} objects, {} pages",
            doc.objects.len(),
            doc.page_count
        );
        let start = std::time::Instant::now();
        let graph = DependencyGraph::new(&doc);
        eprintln!("Graph: {:?}, {} nodes", start.elapsed(), graph.nodes.len());
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
        let mut editor = Editor::new(doc);
        let ctx = egui::Context::default();
        for index in 0..3 {
            let start = std::time::Instant::now();
            frame(&mut editor, &ctx, vec![]);
            eprintln!("Inspector frame {index}: {:?}", start.elapsed());
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
        }
        for page in 0..editor.doc.page_count {
            editor.doc.active_page = page;
            let start = std::time::Instant::now();
            frame(&mut editor, &ctx, vec![]);
            assert!(
                start.elapsed() < std::time::Duration::from_secs(2),
                "page {}",
                page + 1
            );
        }
    }

    #[test]
    fn inspector_types_do_not_require_defined_geometry() {
        let mut doc = Document::new();
        // Missing source coordinates must not change the declared object type.
        for kind in [
            Kind::Segment(90, 91),
            Kind::Ray(90, 91),
            Kind::Line(90, 91),
            Kind::PolarTranslatedSegment(1, 0., 1., 10.),
            Kind::Circle(90, 91),
            Kind::GraphAxis(90, 91, 1),
        ] {
            doc.add(kind);
        }
        let graph = DependencyGraph::new(&doc);
        for (id, expected) in [
            (1, "Segment"),
            (2, "Ray"),
            (3, "Line"),
            (4, "Segment"),
            (5, "Circle"),
            (6, "Axis"),
        ] {
            assert_eq!(graph.nodes[&id].kind, expected);
        }
        assert_eq!(graph.nodes[&1].parents, vec![90, 91]);
        assert_eq!(graph.nodes[&1].children, vec![4]);
    }

    #[test]
    fn outline_interpreter_follows_native_parents_and_reverse_edges() {
        let doc = sketchpad_engine::demo::geometry_document(5).unwrap();
        let graph = DependencyGraph::new(&doc);
        let mut outline = Outline::default();
        dependency_branch(&graph, 5, false, &mut vec![], &mut 256, &mut outline);
        assert!(outline.paths.contains(&vec![5, 1]));
        assert!(outline.paths.contains(&vec![5, 3, 1]));
        assert!(outline.paths.contains(&vec![5, 3, 2]));
        let mut children = Outline::default();
        dependency_branch(&graph, 1, true, &mut vec![], &mut 256, &mut children);
        assert!(children.paths.contains(&vec![1, 3]));
        assert!(children.paths.contains(&vec![1, 3, 5]));
        let mut collapsed = Outline {
            collapsed: [5].into(),
            ..Default::default()
        };
        dependency_branch(&graph, 5, false, &mut vec![], &mut 256, &mut collapsed);
        assert_eq!(collapsed.paths, vec![vec![5]]);
    }

    #[test]
    fn graph_omits_deleted_objects_but_keeps_hidden_sources() {
        let mut editor = editor();
        editor.doc.objects.edit(0).visible = false;
        editor.doc.objects.edit(1).deleted = true;
        let graph = DependencyGraph::new(&editor.doc);
        assert!(graph.nodes[&1].hidden);
        assert!(!graph.nodes.contains_key(&2));
        let mut outline = Outline::default();
        dependency_branch(&graph, 3, false, &mut vec![], &mut 256, &mut outline);
        assert!(outline.paths.contains(&vec![3, 1]));
        assert!(outline.notices.iter().any(|message| message.contains("#2")));
    }

    #[test]
    fn cycle_depth_and_branch_limits_bound_all_interpreters() {
        let mut graph = DependencyGraph::new(&editor().doc);
        graph.nodes.get_mut(&1).unwrap().parents = vec![3];
        let mut outline = Outline::default();
        dependency_branch(&graph, 3, false, &mut vec![], &mut 256, &mut outline);
        assert!(
            outline
                .notices
                .iter()
                .any(|message| message.contains("Cycle"))
        );
        assert!(outline.paths.len() < 10);
        let mut bounded = Outline::default();
        dependency_branch(&graph, 3, false, &mut vec![], &mut 2, &mut bounded);
        assert_eq!(bounded.paths.len(), 2);
        assert!(
            bounded
                .notices
                .iter()
                .any(|message| message.contains("fewer"))
        );
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 0., y: 0. }));
        for id in 2..=40 {
            doc.add(Kind::Midpoint(id - 1, 1));
        }
        let graph = DependencyGraph::new(&doc);
        let mut deep = Outline::default();
        dependency_branch(&graph, 40, false, &mut vec![], &mut 256, &mut deep);
        assert!(deep.paths.iter().all(|path| path.len() <= 24));
        assert!(deep.notices.iter().any(|message| message.contains("Depth")));
    }

    #[test]
    fn picking_uses_original_ids_and_does_not_edit_or_unhide_objects() {
        let mut editor = editor();
        editor.doc.objects.edit(0).visible = false;
        let before = editor.doc.encode().unwrap();
        let history = editor.doc.undo.len();
        editor.inspector_pick(Pick {
            id: 3,
            multi: false,
            center: true,
        });
        assert_eq!(editor.doc.selected, [3].into());
        assert_eq!(editor.selection_order, vec![3]);
        assert_eq!(editor.center, Point { x: 150., y: 100. });
        editor.inspector_pick(Pick {
            id: 1,
            multi: true,
            center: false,
        });
        assert_eq!(editor.selection_order, vec![3, 1]);
        assert!(!editor.doc.objects[0].visible);
        editor.inspector_pick(Pick {
            id: 3,
            multi: true,
            center: false,
        });
        assert_eq!(editor.selection_order, vec![1]);
        for id in [0, 999] {
            editor.inspector_pick(Pick {
                id,
                multi: false,
                center: false,
            });
        }
        assert_eq!(editor.selection_order, vec![1]);
        assert_eq!(editor.doc.undo.len(), history);
        assert_eq!(editor.doc.encode().unwrap(), before);
        assert!(!editor.doc.dirty);
    }

    fn frame(editor: &mut Editor, ctx: &egui::Context, events: Vec<Event>) -> FullOutput {
        editor.language.activate();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(800., 700.),
                )),
                events,
                ..Default::default()
            },
            |ui| editor.dependency_inspector_ui(ui),
        );
        output.textures_delta.clear();
        output
    }
    fn text_center(output: &FullOutput, label: &str) -> Pos2 {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                    Some(text.galley.rect.translate(text.pos.to_vec2()).center())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("Missing inspector text: {label}"))
    }
    fn click(editor: &mut Editor, ctx: &egui::Context, pos: Pos2) -> FullOutput {
        for pressed in [true, false] {
            frame(
                editor,
                ctx,
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
        frame(editor, ctx, vec![])
    }

    #[test]
    fn egui_rows_select_the_canvas_object_and_canvas_selection_updates_the_inspector() {
        let mut editor = editor();
        let ctx = egui::Context::default();
        ctx.style_mut_of(egui::Theme::Dark, |style| style.animation_time = 0.);
        let mut output = frame(&mut editor, &ctx, vec![]);
        let label = DependencyGraph::new(&editor.doc).nodes[&3].title();
        let row = text_center(&output, &label);
        output = click(&mut editor, &ctx, row);
        assert_eq!(editor.doc.selected, [3].into());
        assert_eq!(editor.selection_order, vec![3]);
        text_center(&output, "2 sources, 0 dependents");
        // Arrow clicks expand. A nested row selects the same original ID.
        let left = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => Some(text.pos.x),
                _ => None,
            })
            .unwrap();
        output = click(&mut editor, &ctx, egui::pos2(left - 12., row.y));
        // Complete the opening animation before inspecting nested rows.
        for _ in 0..5 {
            output = frame(&mut editor, &ctx, vec![]);
        }
        let parent_label = DependencyGraph::new(&editor.doc).nodes[&1].title();
        let rows: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == parent_label => {
                    Some(text.galley.rect.translate(text.pos.to_vec2()).center())
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            rows.len(),
            2,
            "Parent must appear as a root and inside the expanded segment"
        );
        click(&mut editor, &ctx, rows[1]);
        assert_eq!(editor.doc.selected, [1].into());
        editor.select_only(2);
        output = frame(&mut editor, &ctx, vec![]);
        assert_eq!(editor.inspector.selection, vec![2]);
        text_center(&output, "x = 200.0000, y = 100.0000");
        editor.language = super::super::i18n::Language::Chinese;
        output = frame(&mut editor, &ctx, vec![]);
        text_center(&output, "依赖树");
        text_center(&output, "被哪些对象使用");
        assert!(!editor.doc.dirty);
        super::super::i18n::Language::English.activate();
    }
}
