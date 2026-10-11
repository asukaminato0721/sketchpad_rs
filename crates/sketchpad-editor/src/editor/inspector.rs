//! The traversal is a tagless-final program over `DependencyView`. It does not
//! build a UI tree. Each interpreter decides whether to evaluate a branch body.
use super::inspector_description::{self, Definition};
use super::{Editor, Tool, i18n::tr};
use sketchpad_engine::document::{
    Document, EvaluatedDocument, Kind, Point, ToolMatch, ToolMatches,
};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet, HashSet},
    ops::Range,
    sync::Arc,
};

#[derive(Default)]
pub(super) struct InspectorState {
    query: String,
    dependents: bool,
    ungrouped: bool,
    selection: Vec<usize>,
    graph: Option<(Arc<EvaluatedDocument>, Arc<DependencyGraph>)>,
    expanded: HashSet<egui::Id>,
}

impl InspectorState {
    fn graph(&mut self, document: Arc<EvaluatedDocument>) -> Arc<DependencyGraph> {
        if let Some((previous, graph)) = &self.graph
            && Arc::ptr_eq(previous, &document)
        {
            return graph.clone();
        }
        // Visibility can evaluate dynamic colors. Use the shared evaluation
        // snapshot so these checks reuse the canvas's cached values.
        let graph = Arc::new(DependencyGraph::new(&document));
        self.graph = Some((document, graph.clone()));
        graph
    }
}

struct ObjectNode {
    id: usize,
    page: usize,
    kind: &'static str,
    name: String,
    definition: Definition,
    hidden: bool,
    parents: Vec<usize>,
    children: Vec<usize>,
}

impl ObjectNode {
    fn title(&self) -> String {
        let mut title = if self.name.is_empty() {
            format!("#{} · {}", self.id, tr(self.kind))
        } else {
            format!("#{} · {} · {}", self.id, tr(self.kind), self.name)
        };
        let definition = self.definition.text();
        if !definition.is_empty() {
            title.push_str(" · ");
            title.push_str(&definition);
        }
        title
    }
    fn row_title(&self) -> String {
        let title = self.title();
        let mut chars = title.chars();
        let mut short: String = chars.by_ref().take(160).collect();
        if chars.next().is_some() {
            short.push('…');
        }
        short
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
    tools: ToolMatches,
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
                    definition: inspector_description::definition(doc, object),
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
        Self {
            nodes,
            tools: doc.matching_tool_constructions(),
        }
    }

    fn tool_title(&self, group: &ToolMatch) -> String {
        let inputs = group
            .inputs
            .iter()
            .map(|id| match self.nodes.get(id) {
                Some(node) if !node.name.is_empty() => format!("{} · #{id}", node.name),
                _ => format!("#{id}"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} ({inputs})", group.name)
    }
}

/// The display algebra. A backend can render, record, or skip the branch body.
/// The generic traversal works without egui or a tagged UI representation.
trait DependencyView: Sized {
    fn tool_group(
        &mut self,
        index: usize,
        group: &ToolMatch,
        title: &str,
        body: impl FnOnce(&mut Self),
    );
    fn object(
        &mut self,
        node: &ObjectNode,
        path: &[usize],
        expandable: bool,
        body: impl FnOnce(&mut Self),
    );
    fn notice(&mut self, message: &str);
}

#[derive(Clone, Copy)]
enum InspectorRoot {
    Object(usize),
    Tool(usize),
}

fn inspector_roots(
    graph: &DependencyGraph,
    page: usize,
    grouped: bool,
    query: &str,
) -> Vec<InspectorRoot> {
    let matches = |id: &usize| {
        graph.nodes.get(id).is_some_and(|node| {
            query.is_empty()
                || node.title().to_lowercase().contains(query)
                || node.kind.to_lowercase().contains(query)
        })
    };
    let owners: BTreeMap<_, _> = if grouped {
        graph
            .tools
            .groups
            .iter()
            .enumerate()
            .flat_map(|(index, group)| group.objects.iter().map(move |id| (*id, index)))
            .collect()
    } else {
        BTreeMap::new()
    };
    graph
        .nodes
        .values()
        .filter(|node| node.page == page)
        .filter_map(|node| {
            if let Some(&index) = owners.get(&node.id) {
                let group = &graph.tools.groups[index];
                (node.id == group.objects[0]
                    && (graph.tool_title(group).to_lowercase().contains(query)
                        || group.objects.iter().chain(&group.inputs).any(matches)))
                .then_some(InspectorRoot::Tool(index))
            } else {
                matches(&node.id).then_some(InspectorRoot::Object(node.id))
            }
        })
        .collect()
}

fn inspector_outline<V: DependencyView>(
    graph: &DependencyGraph,
    roots: &[InspectorRoot],
    dependents: bool,
    view: &mut V,
) {
    for root in roots {
        match *root {
            InspectorRoot::Object(id) => {
                dependency_branch(graph, id, dependents, &mut vec![], &mut 256, view)
            }
            InspectorRoot::Tool(index) => {
                let group = &graph.tools.groups[index];
                view.tool_group(index, group, &graph.tool_title(group), |view| {
                    // The synthetic path component separates grouped rows from
                    // ordinary roots without changing any document object ID.
                    let mut seen = BTreeSet::new();
                    for &id in group.inputs.iter().chain(&group.objects) {
                        if !seen.insert(id) {
                            continue;
                        }
                        dependency_branch(
                            graph,
                            id,
                            dependents,
                            &mut vec![usize::MAX - index],
                            &mut 256,
                            view,
                        );
                    }
                });
            }
        }
    }
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
    active_page: usize,
    expanded: &'a HashSet<egui::Id>,
    namespace: egui::Id,
    toggle: &'a Cell<Option<egui::Id>>,
    visible: Range<usize>,
    row_index: usize,
    row_height: f32,
    depth: usize,
}

/// Count expanded rows without creating widgets or a UI tree. This interpreter
/// and the renderer use the same traversal, including its cycle and size limits.
struct RowCountView<'a> {
    expanded: &'a HashSet<egui::Id>,
    namespace: egui::Id,
    rows: usize,
    reveal: Option<usize>,
    reveal_row: Option<usize>,
}

impl DependencyView for RowCountView<'_> {
    fn tool_group(
        &mut self,
        index: usize,
        group: &ToolMatch,
        _title: &str,
        body: impl FnOnce(&mut Self),
    ) {
        if self.reveal.is_some_and(|id| group.objects.contains(&id)) {
            self.reveal_row = Some(self.rows);
        }
        self.rows += 1;
        if self
            .expanded
            .contains(&self.namespace.with(("tool", index)))
        {
            body(self);
        }
    }

    fn object(
        &mut self,
        node: &ObjectNode,
        path: &[usize],
        expandable: bool,
        body: impl FnOnce(&mut Self),
    ) {
        if self.reveal == Some(node.id)
            && (path.len() == 1 || (path.len() == 2 && path[0] > usize::MAX / 2))
        {
            self.reveal_row = Some(self.rows);
        }
        self.rows += 1;
        if expandable && self.expanded.contains(&self.namespace.with(path)) {
            body(self);
        }
    }

    fn notice(&mut self, _message: &str) {
        self.rows += 1;
    }
}

impl EguiDependencyView<'_> {
    fn row(
        ui: &mut egui::Ui,
        node: &ObjectNode,
        selected: &BTreeSet<usize>,
        pick: &Cell<Option<Pick>>,
        active_page: usize,
    ) {
        let response = ui
            .selectable_label(selected.contains(&node.id), node.row_title())
            .on_hover_text(format!(
                "{}\n\n{}",
                node.title(),
                tr("Click to select. Ctrl/Shift-click to add or remove. Double-click to center.",)
            ));
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
    fn tool_group(
        &mut self,
        index: usize,
        group: &ToolMatch,
        title: &str,
        body: impl FnOnce(&mut Self),
    ) {
        let id = self.namespace.with(("tool", index));
        let open = self.expanded.contains(&id);
        if self.visible.contains(&self.row_index) {
            self.ui.push_id(id, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(self.row_height);
                    let (_, arrow) = ui.allocate_exact_size(
                        egui::vec2(ui.spacing().indent, self.row_height), egui::Sense::click(),
                    );
                    egui::collapsing_header::paint_default_icon(ui, f32::from(open), &arrow);
                    let label = ui.selectable_label(
                        group.objects.iter().any(|id| self.selected.contains(id)), title,
                    ).on_hover_text(tr("This construction matches a custom tool. Expand to inspect its inputs and internal objects."));
                    if arrow.clicked() || label.clicked() {
                        self.toggle.set(Some(id));
                        ui.ctx().request_repaint();
                    }
                    ui.weak(tr(format!("{} internal objects", group.objects.len())));
                });
            });
        }
        self.row_index += 1;
        if open {
            body(self);
        }
    }

    fn object(
        &mut self,
        node: &ObjectNode,
        path: &[usize],
        expandable: bool,
        body: impl FnOnce(&mut Self),
    ) {
        let id = self.namespace.with(path);
        let open = expandable && self.expanded.contains(&id);
        if self.visible.contains(&self.row_index) {
            self.ui.push_id(id, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(self.row_height);
                    ui.add_space((path.len() - 1) as f32 * ui.spacing().indent);
                    let (_, response) = ui.allocate_exact_size(
                        egui::vec2(ui.spacing().indent, self.row_height),
                        egui::Sense::click(),
                    );
                    if expandable {
                        egui::collapsing_header::paint_default_icon(ui, f32::from(open), &response);
                        if response.clicked() {
                            self.toggle.set(Some(id));
                            ui.ctx().request_repaint();
                        }
                    }
                    Self::row(ui, node, self.selected, self.pick, self.active_page);
                });
            });
        }
        self.row_index += 1;
        if open {
            let previous_depth = self.depth;
            self.depth = path.len();
            body(self);
            self.depth = previous_depth;
        }
    }
    fn notice(&mut self, message: &str) {
        if self.visible.contains(&self.row_index) {
            self.ui.horizontal(|ui| {
                ui.set_min_height(self.row_height);
                ui.add_space(self.depth as f32 * ui.spacing().indent);
                ui.weak(tr(message));
            });
        }
        self.row_index += 1;
    }
}

impl Editor {
    pub(super) fn dependency_inspector_ui(&mut self, ui: &mut egui::Ui) {
        let evaluated = self.evaluated_document();
        let graph = self.inspector.graph(evaluated);
        let selection: Vec<_> = self.doc.selected.iter().copied().collect();
        let mut reveal = (selection != self.inspector.selection)
            .then(|| selection.first().copied())
            .flatten();
        self.inspector.selection = selection;
        ui.heading(tr("Dependency tree"));
        let mut grouped = !self.inspector.ungrouped;
        ui.checkbox(&mut grouped, tr("Group by custom tools"));
        self.inspector.ungrouped = !grouped;
        if grouped && !graph.tools.groups.is_empty() {
            ui.small(tr(format!(
                "{} matching tool constructions",
                graph.tools.groups.len()
            )));
        }
        if grouped && graph.tools.limited {
            ui.small(tr(
                "Search limit reached. Some constructions remain ungrouped.",
            ));
        }
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
        let roots = inspector_roots(&graph, self.doc.active_page, grouped, &query);
        let namespace = ui.make_persistent_id((
            "dependency_tree",
            self.doc.active_page,
            self.inspector.dependents,
            grouped,
        ));
        if grouped {
            for (index, group) in graph.tools.groups.iter().enumerate() {
                if reveal.is_some_and(|id| group.objects.contains(&id)) {
                    self.inspector
                        .expanded
                        .insert(namespace.with(("tool", index)));
                }
            }
        }
        // Filtering exposes matching members inside their group.
        let mut expanded = self.inspector.expanded.clone();
        if !query.is_empty() {
            for root in &roots {
                if let InspectorRoot::Tool(index) = root {
                    expanded.insert(namespace.with(("tool", *index)));
                }
            }
        }
        let mut count = RowCountView {
            expanded: &expanded,
            namespace,
            rows: 0,
            reveal,
            reveal_row: None,
        };
        inspector_outline(&graph, &roots, self.inspector.dependents, &mut count);
        let pick = Cell::new(None);
        let toggle = Cell::new(None);
        let row_height = (ui.text_style_height(&egui::TextStyle::Body)
            + 2. * ui.spacing().button_padding.y)
            .max(ui.spacing().interact_size.y);
        let mut scroll = egui::ScrollArea::both()
            .id_salt("dependency_tree_scroll")
            .max_height(270.)
            .auto_shrink([false, true]);
        if let Some(row) = count.reveal_row {
            scroll = scroll.vertical_scroll_offset(
                (row as f32 * (row_height + ui.spacing().item_spacing.y) - 120.).max(0.),
            );
        }
        scroll.show_rows(ui, row_height, count.rows.max(1), |ui, visible| {
            // Fixed-height rows make it possible to omit all offscreen widgets.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            let mut view = EguiDependencyView {
                ui: ui.new_child(egui::UiBuilder::new()),
                selected: &self.doc.selected,
                pick: &pick,
                active_page: self.doc.active_page,
                expanded: &expanded,
                namespace,
                toggle: &toggle,
                visible,
                row_index: 0,
                row_height,
                depth: 0,
            };
            inspector_outline(&graph, &roots, self.inspector.dependents, &mut view);
            if roots.is_empty() {
                view.notice("No matching objects on this page.");
            }
            ui.advance_cursor_after_rect(view.ui.min_rect());
        });
        if let Some(id) = toggle.get()
            && !self.inspector.expanded.remove(&id)
        {
            self.inspector.expanded.insert(id);
        }
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
        fn tool_group(
            &mut self,
            _index: usize,
            _group: &ToolMatch,
            _title: &str,
            body: impl FnOnce(&mut Self),
        ) {
            body(self);
        }

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
    fn formula_rows_support_selection_search_and_cached_language_changes() {
        let mut editor = editor();
        editor.doc.set_point_label(1, "A", true).unwrap();
        editor.doc.set_point_label(2, "B", true).unwrap();
        let line = editor.doc.add(Kind::Perpendicular(1, 3));
        let calculation = editor
            .doc
            .add_calculation_expression("sqrt(2)+3", (10, 10))
            .unwrap();
        let before = editor.doc.encode().unwrap();
        let ctx = egui::Context::default();
        let mut output = frame(&mut editor, &ctx, vec![]);
        let graph = editor.inspector.graph.as_ref().unwrap().1.clone();
        let title = graph.nodes[&calculation].title();
        assert!(title.contains("sqrt(2)"));
        let pos = text_center(&output, &title);
        output = click(&mut editor, &ctx, pos);
        assert_eq!(editor.doc.selected, [calculation].into());
        text_center(&output, &title);

        editor.inspector.query = "sqrt".into();
        output = frame(&mut editor, &ctx, vec![]);
        text_center(&output, &title);
        assert_eq!(inspector_roots(&graph, 0, false, "sqrt").len(), 1);
        editor.inspector.query.clear();
        editor.language = super::super::i18n::Language::Chinese;
        output = frame(&mut editor, &ctx, vec![]);
        let title = graph.nodes[&line].title();
        assert!(title.contains("过 A [#1]，垂直于 #3"));
        text_center(&output, &title);
        assert_eq!(editor.doc.encode().unwrap(), before);
        super::super::i18n::Language::English.activate();
    }

    #[test]
    fn long_definitions_keep_full_search_text_with_short_single_line_rows() {
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let id = doc.add(Kind::Polygon((1..=80).collect()));
        let graph = DependencyGraph::new(&doc);
        let node = &graph.nodes[&id];
        assert_eq!(node.row_title().chars().count(), 161);
        assert!(node.row_title().ends_with('…'));
        assert!(node.title().ends_with("#80"));
        assert!(!node.row_title().contains("#80"));
        assert_eq!(inspector_roots(&graph, 0, false, "#80").len(), 1);
    }

    fn tool_editor() -> Editor {
        let mut editor = editor();
        editor.doc.add(Kind::Midpoint(1, 2));
        editor.doc.selected = [1, 2, 3, 4].into();
        editor
            .doc
            .create_embedded_tool("Segment and midpoint")
            .unwrap();
        editor.doc.selected.clear();
        editor
    }

    #[test]
    fn grouped_roots_keep_inputs_and_all_internal_objects_accessible() {
        let editor = tool_editor();
        let graph = DependencyGraph::new(&editor.doc);
        assert_eq!(graph.tools.groups.len(), 1);
        let roots = inspector_roots(&graph, 0, true, "");
        assert_eq!(roots.len(), 3);
        let mut outline = Outline::default();
        inspector_outline(&graph, &roots, false, &mut outline);
        for id in 1..=4 {
            assert!(outline.paths.contains(&vec![usize::MAX, id]));
        }
        assert!(!outline.paths.contains(&vec![3]));
        assert!(!outline.paths.contains(&vec![4]));
        assert_eq!(inspector_roots(&graph, 0, false, "").len(), 4);
        assert!(matches!(
            inspector_roots(&graph, 0, true, "#4").as_slice(),
            [InspectorRoot::Tool(0)]
        ));
        assert!(matches!(
            inspector_roots(&graph, 0, true, "segment and midpoint").as_slice(),
            [InspectorRoot::Tool(0)]
        ));
    }

    #[test]
    fn egui_groups_expand_select_filter_and_reveal_without_document_edits() {
        let mut editor = tool_editor();
        let before = editor.doc.encode().unwrap();
        let history = editor.doc.undo.len();
        let ctx = egui::Context::default();
        let graph = DependencyGraph::new(&editor.doc);
        let title = graph.tool_title(&graph.tools.groups[0]);
        let object_title = graph.nodes[&4].title();
        let mut output = frame(&mut editor, &ctx, vec![]);
        let group_row = text_center(&output, &title);
        assert!(!has_text(&output, &object_title));
        output = click(&mut editor, &ctx, group_row);
        let member = text_center(&output, &object_title);
        click(&mut editor, &ctx, member);
        assert_eq!(editor.doc.selected, [4].into());
        assert_eq!(editor.selection_order, [4]);

        // A canvas selection opens a collapsed group and reveals the member.
        editor.doc.selected.clear();
        frame(&mut editor, &ctx, vec![]);
        editor.inspector.expanded.clear();
        editor.select_only(4);
        output = frame(&mut editor, &ctx, vec![]);
        text_center(&output, &object_title);
        assert!(!editor.inspector.expanded.is_empty());

        editor.doc.selected.clear();
        editor.inspector.expanded.clear();
        editor.inspector.query = "#4".into();
        output = frame(&mut editor, &ctx, vec![]);
        text_center(&output, &title);
        text_center(&output, &object_title);

        editor.inspector.query.clear();
        editor.inspector.ungrouped = true;
        output = frame(&mut editor, &ctx, vec![]);
        assert!(!has_text(&output, &title));
        text_center(&output, &object_title);
        assert_eq!(editor.doc.encode().unwrap(), before);
        assert_eq!(editor.doc.undo.len(), history);

        editor.language = super::super::i18n::Language::Chinese;
        output = frame(&mut editor, &ctx, vec![]);
        text_center(&output, "按自定义工具分组");
        super::super::i18n::Language::English.activate();
    }

    fn has_text(output: &FullOutput, label: &str) -> bool {
        output.shapes.iter().any(|shape| {
            matches!(&shape.shape,
            egui::epaint::Shape::Text(text) if text.galley.text() == label)
        })
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
    fn cached_graph_tracks_edits_undo_pages_and_document_replacement() {
        let mut editor = editor();
        let evaluated = editor.evaluated_document();
        let original = editor.inspector.graph(evaluated);
        let evaluated = editor.evaluated_document();
        assert!(Arc::ptr_eq(&original, &editor.inspector.graph(evaluated)));

        editor.doc.checkpoint();
        editor.doc.objects.edit(0).visible = false;
        editor.doc.objects.edit(2).kind = Kind::Midpoint(1, 1);
        let evaluated = editor.evaluated_document();
        let changed = editor.inspector.graph(evaluated);
        assert!(changed.nodes[&1].hidden);
        assert_eq!(changed.nodes[&3].parents, vec![1]);
        assert_eq!(changed.nodes[&3].kind, "Midpoint");
        assert!(!original.nodes[&1].hidden);

        editor.doc.undo();
        let evaluated = editor.evaluated_document();
        let restored = editor.inspector.graph(evaluated);
        assert!(!restored.nodes[&1].hidden);
        assert_eq!(restored.nodes[&3].parents, vec![1, 2]);

        editor.doc.set_point_label(1, "Renamed", true).unwrap();
        let evaluated = editor.evaluated_document();
        assert_eq!(editor.inspector.graph(evaluated).nodes[&1].name, "Renamed");
        editor.doc.objects.edit(0).page = 1;
        let evaluated = editor.evaluated_document();
        assert_eq!(editor.inspector.graph(evaluated).nodes[&1].page, 1);

        editor.doc = Document::new();
        editor.doc.add(Kind::Free(Point { x: 4., y: 5. }));
        let evaluated = editor.evaluated_document();
        let replaced = editor.inspector.graph(evaluated);
        assert_eq!(replaced.nodes.len(), 1);
        assert!(replaced.nodes[&1].name.is_empty());
    }

    #[test]
    fn row_counts_include_expanded_branches_and_cycle_notices() {
        let mut graph = DependencyGraph::new(&editor().doc);
        graph.nodes.get_mut(&1).unwrap().parents = vec![3];
        let namespace = egui::Id::new("count-test");
        let mut outline = Outline::default();
        dependency_branch(&graph, 3, false, &mut vec![], &mut 256, &mut outline);
        let expanded = outline
            .paths
            .iter()
            .map(|path| namespace.with(path))
            .collect();
        let mut count = RowCountView {
            expanded: &expanded,
            namespace,
            rows: 0,
            reveal: Some(3),
            reveal_row: None,
        };
        dependency_branch(&graph, 3, false, &mut vec![], &mut 256, &mut count);
        assert_eq!(count.rows, outline.paths.len() + outline.notices.len());
        assert_eq!(count.reveal_row, Some(0));
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

    #[test]
    fn large_lists_only_layout_visible_rows_and_reveal_distant_selections() {
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let prototype = doc.objects[0].clone();
        doc.objects = (1..=4071)
            .map(|id| {
                let mut object = prototype.clone();
                object.id = id;
                object.kind = Kind::Free(Point {
                    x: id as f64,
                    y: 0.,
                });
                object
            })
            .collect::<Vec<_>>()
            .into();
        let mut editor = Editor::new(doc);
        let ctx = egui::Context::default();
        super::super::chrome::apply_style(&ctx);
        let first = frame(&mut editor, &ctx, vec![]);
        let object_rows = |output: &FullOutput| {
            output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text) if text.galley.text().starts_with('#') => {
                        Some(text)
                    }
                    _ => None,
                })
                .count()
        };
        assert!(object_rows(&first) < 30);
        let graph = editor.inspector.graph.as_ref().unwrap().1.clone();
        frame(&mut editor, &ctx, vec![]);
        assert!(Arc::ptr_eq(
            &graph,
            &editor.inspector.graph.as_ref().unwrap().1
        ));

        editor.select_only(4000);
        frame(&mut editor, &ctx, vec![]);
        let output = frame(&mut editor, &ctx, vec![]);
        assert!(object_rows(&output) < 30);
        // Choose a neighboring row, rather than the selected object's detail heading.
        let neighbor = graph.nodes[&3999].title();
        let position = text_center(&output, &neighbor);
        assert!(
            position.y < 400.,
            "The selected region must be in the scroll viewport"
        );
        click(&mut editor, &ctx, position);
        assert_eq!(editor.doc.selected, [3999].into());

        editor.inspector.query = "#4071".into();
        frame(&mut editor, &ctx, vec![]);
        let output = frame(&mut editor, &ctx, vec![]);
        let last = text_center(&output, &graph.nodes[&4071].title());
        click(&mut editor, &ctx, last);
        assert_eq!(editor.doc.selected, [4071].into());
        editor.inspector.query = "no matching object".into();
        let output = frame(&mut editor, &ctx, vec![]);
        text_center(&output, "No matching objects on this page.");
    }
}
