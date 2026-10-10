//! Opt-in MCP access to the document owned by the window thread.
mod construction;
mod protocol;
use construction::{Construct, CreatePoint};

use super::{Editor, Tool};
use crate::render::{Viewport, svg_page_with_button_offset};
use serde::Deserialize;
use serde_json::{Value, json};
use sketchpad_engine::document::{Object, Point};
use std::fmt::Write as _;
use std::sync::mpsc;
use winit::event_loop::EventLoopProxy;

pub(super) enum Event {
    Call {
        call: ToolCall,
        reply: mpsc::Sender<Result<Value, String>>,
    },
    Disconnected,
}

pub(super) fn start(proxy: EventLoopProxy<Event>) {
    std::thread::spawn(move || {
        let result = protocol::serve(std::io::stdin().lock(), std::io::stdout().lock(), |call| {
            let (reply, response) = mpsc::channel();
            proxy
                .send_event(Event::Call { call, reply })
                .map_err(|_| "The editor window is closed".to_owned())?;
            // Do not time out a queued edit: it could still execute afterwards.
            response
                .recv()
                .map_err(|_| "The editor window is closed".to_owned())?
        });
        if let Err(error) = result {
            eprintln!("MCP connection closed: {error}");
        }
        let _ = proxy.send_event(Event::Disconnected);
    });
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name", content = "arguments", rename_all = "snake_case")]
pub(super) enum ToolCall {
    GetState(Empty),
    ListObjects(ListObjects),
    InspectObject(ObjectId),
    SelectObjects(Selection),
    CreatePoint(CreatePoint),
    Construct(Construct),
    MovePoint(MovePoint),
    SetParameter(SetParameter),
    Undo(Empty),
    Redo(Empty),
    ExportSvg(Empty),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Empty {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ListObjects {
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_limit")]
    limit: usize,
    page: Option<usize>,
}

fn default_limit() -> usize {
    100
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObjectId {
    id: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Selection {
    ids: Vec<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MovePoint {
    id: usize,
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SetParameter {
    id: usize,
    value: f64,
}

// Image and table definitions can contain large byte arrays. Stop formatting
// once the preview is full, without first allocating the entire debug string.
#[derive(Default)]
struct DefinitionPreview(String);

impl std::fmt::Write for DefinitionPreview {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let mut size = text.len().min(16 * 1024 - self.0.len());
        while !text.is_char_boundary(size) {
            size -= 1;
        }
        self.0.push_str(&text[..size]);
        if size < text.len() {
            Err(std::fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub(super) fn tool_list() -> Value {
    let id =
        json!({"type":"integer", "minimum":1, "description":"Document object ID. IDs start at 1."});
    let definitions = [
        (
            "get_state",
            "Read the live window state, selection, animation status, and undo history sizes.",
            json!({}),
            vec![],
            true,
        ),
        (
            "list_objects",
            "List objects on a page. Page numbers start at 1. The default is the active page. Deleted objects are omitted.",
            json!({"offset":{"type":"integer","minimum":0,"default":0}, "limit":{"type":"integer","minimum":1,"maximum":500,"default":100}, "page":{"type":"integer","minimum":1}}),
            vec![],
            true,
        ),
        (
            "inspect_object",
            "Read an object's definition, dependencies, evaluated point or numeric value, and source offsets. Null values are unavailable or inapplicable.",
            json!({"id":id}),
            vec!["id"],
            true,
        ),
        (
            "select_objects",
            "Replace the active page selection in the supplied order. An empty list clears selection. Hidden objects are allowed.",
            json!({"ids":{"type":"array","items":id,"maxItems":256,"uniqueItems":true}}),
            vec!["ids"],
            false,
        ),
        (
            "move_point",
            "Move a point on the active page in one undo step. Coordinates use document space, with y increasing downwards. Constraints and snapping apply.",
            json!({"id":id,"x":{"type":"number"},"y":{"type":"number"}}),
            vec!["id", "x", "y"],
            false,
        ),
        (
            "set_parameter",
            "Set a numeric parameter on the active page in one undo step. Use the parameter's native unit.",
            json!({"id":id,"value":{"type":"number"}}),
            vec!["id", "value"],
            false,
        ),
        (
            "undo",
            "Undo one document edit. Stop animations, movement, presentations, and sound, as the Edit menu does.",
            json!({}),
            vec![],
            false,
        ),
        (
            "redo",
            "Redo one document edit. Stop animations, movement, presentations, and sound, as the Edit menu does.",
            json!({}),
            vec![],
            false,
        ),
        (
            "export_svg",
            "Return SVG for the current canvas viewport. This is a document export, not a GPU or window screenshot. No file is written.",
            json!({}),
            vec![],
            true,
        ),
    ];
    let mut result = json!({"tools": definitions.into_iter().map(|(name, description, properties, required, read_only)| json!({
        "name":name, "description":description,
        "inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false},
        "annotations":{"readOnlyHint":read_only,"destructiveHint":!read_only,"openWorldHint":false}
    })).collect::<Vec<_>>()});
    result["tools"]
        .as_array_mut()
        .unwrap()
        .extend(construction::tool_definitions());
    result
}

impl Editor {
    fn mcp_object(&self, id: usize) -> Result<&Object, String> {
        id.checked_sub(1)
            .and_then(|index| self.doc.objects.get(index))
            .filter(|object| !object.deleted)
            .ok_or_else(|| format!("Object {id} does not exist"))
    }

    fn mcp_active_object(&self, id: usize) -> Result<&Object, String> {
        let object = self.mcp_object(id)?;
        if object.page != self.doc.active_page {
            return Err(format!("Object {id} is not on the active page"));
        }
        Ok(object)
    }

    fn mcp_object_summary(&self, object: &Object) -> Value {
        json!({
            "id":object.id, "page":object.page + 1,
            "class": self.doc.object_class(object.id).map(|class| format!("{class:?}")),
            "label":object.label.as_ref().and_then(|label| label.name_utf8().ok()),
            "visible":object.visible, "displayed":self.doc.is_displayed(object.id),
            "selected":self.doc.selected.contains(&object.id)
        })
    }

    fn mcp_inspect(&self, id: usize) -> Result<Value, String> {
        let object = self.mcp_object(id)?;
        let evaluated = self.evaluated_document();
        let mut summary = self.mcp_object_summary(object);
        let fields = summary.as_object_mut().expect("object summary");
        let mut definition = DefinitionPreview::default();
        let truncated = write!(&mut definition, "{:?}", object.kind).is_err();
        fields.insert("definition".into(), json!(definition.0));
        fields.insert("definition_truncated".into(), json!(truncated));
        fields.insert(
            "parents".into(),
            json!(self.doc.dependencies_of(id).unwrap_or_default()),
        );
        fields.insert(
            "children".into(),
            json!(
                self.doc
                    .objects
                    .iter()
                    .filter(|child| !child.deleted
                        && self
                            .doc
                            .dependencies_of(child.id)
                            .is_some_and(|parents| parents.contains(&id)))
                    .map(|child| child.id)
                    .collect::<Vec<_>>()
            ),
        );
        fields.insert(
            "position".into(),
            json!(
                evaluated
                    .position(id)
                    .map(|point| json!({"x":point.x,"y":point.y}))
            ),
        );
        fields.insert("numeric_value".into(), json!(evaluated.numeric_value(id)));
        fields.insert(
            "numeric_unit".into(),
            json!(object.kind.numeric_unit().map(|unit| format!("{unit:?}"))),
        );
        fields.insert("source_offsets".into(), json!({"header":object.header_offset,"point":object.point_offset,"constraint":object.constraint_offset}));
        Ok(summary)
    }

    fn mcp_state(&self) -> Value {
        let (tool, custom_tool) = match self.tool {
            Tool::Select => ("select", None),
            Tool::Point => ("point", None),
            Tool::Segment => ("segment", None),
            Tool::Circle => ("circle", None),
            Tool::Text => ("text", None),
            Tool::Custom(source) => ("custom", Some(format!("{source:?}"))),
        };
        json!({
            "ui_language":self.language.code(),
            "path":self.doc.path.as_ref().map(|path| path.to_string_lossy()), "dirty":self.doc.dirty,
            "active_page":self.doc.active_page + 1, "page_count":self.doc.page_count,
            "page_names":self.doc.page_names,
            "object_count":self.doc.objects.iter().filter(|object| !object.deleted).count(),
            "selected":self.doc.selected, "selection_order":self.selection_order,
            "tool":tool, "custom_tool":custom_tool, "dragging":self.dragging,
            "undo_steps":self.doc.undo.len(), "redo_steps":self.doc.redo.len(),
            "partial_view":self.doc.partial_view, "edit_restriction":format!("{:?}",self.doc.edit_restriction()),
            "animation":{"paused":self.animation_paused,"tracks":self.animations.len(),"movements":self.movements.len(),"presentations":self.presentations.len()},
            "viewport":{"scale":self.scale,"center":{"x":self.center.x,"y":self.center.y},
                "rect":self.viewport_rect.map(|rect| json!({"x":rect.min.x,"y":rect.min.y,"width":rect.width(),"height":rect.height()}))},
            "status":self.status
        })
    }

    pub(super) fn mcp_call(&mut self, call: ToolCall) -> Result<Value, String> {
        let mutating = matches!(
            call,
            ToolCall::SelectObjects(_)
                | ToolCall::CreatePoint(_)
                | ToolCall::Construct(_)
                | ToolCall::MovePoint(_)
                | ToolCall::SetParameter(_)
                | ToolCall::Undo(_)
                | ToolCall::Redo(_)
        );
        if mutating && (self.dragging.is_some() || self.pending_action.is_some()) {
            return Err("Finish the drag or document confirmation before an MCP edit".into());
        }
        match call {
            ToolCall::GetState(_) => Ok(self.mcp_state()),
            ToolCall::ListObjects(args) => {
                if !(1..=500).contains(&args.limit) {
                    return Err("limit must be from 1 to 500".into());
                }
                let page = args.page.unwrap_or(self.doc.active_page + 1);
                if !(1..=self.doc.page_count).contains(&page) {
                    return Err("Page does not exist".into());
                }
                let objects: Vec<_> = self
                    .doc
                    .objects
                    .iter()
                    .filter(|object| !object.deleted && object.page + 1 == page)
                    .collect();
                let next = args.offset.saturating_add(args.limit);
                Ok(
                    json!({"page":page,"total":objects.len(),"objects":objects.iter().skip(args.offset).take(args.limit).map(|object|self.mcp_object_summary(object)).collect::<Vec<_>>(),
                    "next_offset":(next < objects.len()).then_some(next)}),
                )
            }
            ToolCall::InspectObject(args) => self.mcp_inspect(args.id),
            ToolCall::CreatePoint(args) => self.mcp_create_point(args),
            ToolCall::Construct(args) => self.mcp_construct(args),
            ToolCall::SelectObjects(args) => {
                let selected: std::collections::BTreeSet<_> = args.ids.iter().copied().collect();
                if args.ids.len() > 256 || selected.len() != args.ids.len() {
                    return Err("Supply at most 256 unique object IDs".into());
                }
                for &id in &args.ids {
                    self.mcp_active_object(id)?;
                }
                self.doc.selected = selected;
                self.selection_order = args.ids;
                Ok(self.mcp_state())
            }
            ToolCall::MovePoint(args) => {
                self.mcp_active_object(args.id)?;
                if !self.doc.move_point_with_history(
                    args.id,
                    Point {
                        x: args.x,
                        y: args.y,
                    },
                ) {
                    return Err("The point cannot move to these coordinates".into());
                }
                self.mcp_inspect(args.id)
            }
            ToolCall::SetParameter(args) => {
                self.mcp_active_object(args.id)?;
                if !args.value.is_finite() {
                    return Err("value must be finite".into());
                }
                self.doc.set_parameter_value(args.id, args.value)?;
                self.mcp_inspect(args.id)
            }
            ToolCall::Undo(_) | ToolCall::Redo(_) => {
                self.apply_history(matches!(call, ToolCall::Redo(_)));
                Ok(self.mcp_state())
            }
            ToolCall::ExportSvg(_) => {
                let rect = self
                    .viewport_rect
                    .ok_or("The canvas has not rendered yet. Retry after the window appears")?;
                let viewport = Viewport {
                    rect,
                    scale: self.scale,
                    center: self.center,
                };
                let svg = svg_page_with_button_offset(
                    &self.evaluated_document(),
                    &viewport,
                    self.button_offset(&viewport),
                )?;
                Ok(json!({"mime_type":"image/svg+xml", "svg":svg}))
            }
        }
    }
}

#[cfg(test)]
mod tests;
