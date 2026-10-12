use super::*;
use sketchpad_engine::document::Kind;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::editor) struct CreatePoint {
    x: f64,
    y: f64,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::editor) struct Construct {
    kind: Construction,
    inputs: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Construction {
    Segment,
    Line,
    Ray,
    Circle,
    Midpoint,
    Parallel,
    Perpendicular,
    CircleBySegment,
    Polygon,
    Intersection,
    AngleBisector,
    ThreePointArc,
    CenterArc,
    ArcOnCircle,
    PointOnObject,
    CircleInterior,
    SectorInterior,
}

pub(super) fn tool_definitions() -> Vec<Value> {
    vec![
        json!({"name":"create_point","description":"Create a free point on the active page. Coordinates use document space. Grid snapping applies. An optional label is shown. One call creates one undo step.",
            "inputSchema":{"type":"object","properties":{"x":{"type":"number"},"y":{"type":"number"},"label":{"type":"string","maxLength":256}},"required":["x","y"],"additionalProperties":false},
            "annotations":{"readOnlyHint":false,"destructiveHint":false,"openWorldHint":false}}),
        json!({"name":"construct","description":"Construct geometry from ordered object IDs on the active page. Point pairs: segment, line, ray, circle (center first), midpoint. Point and straight path: parallel, perpendicular. Center and segment: circle_by_segment. Ordered vertices: polygon. Two paths: intersection. Three points: angle_bisector (vertex second), three_point_arc (start, through, end), center_arc (center, start, end). Circle then two points: arc_on_circle. Host and optional point: point_on_object. One circle: circle_interior. One arc: sector_interior. All outputs and helper objects form one undo step. Failure preserves the document and selection.",
            "inputSchema":{"type":"object","properties":{"kind":{"type":"string","enum":["segment","line","ray","circle","midpoint","parallel","perpendicular","circle_by_segment","polygon","intersection","angle_bisector","three_point_arc","center_arc","arc_on_circle","point_on_object","circle_interior","sector_interior"]},"inputs":{"type":"array","items":{"type":"integer","minimum":1},"minItems":1,"maxItems":256,"uniqueItems":true}},"required":["kind","inputs"],"additionalProperties":false},
            "annotations":{"readOnlyHint":false,"destructiveHint":false,"openWorldHint":false}}),
    ]
}

impl Editor {
    fn mcp_create_transaction(
        &mut self,
        create: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<Value, String> {
        let mut candidate = self.doc.clone();
        let order = self.selection_order.clone();
        let status = self.status.clone();
        let count = candidate.objects().len();
        let outcome = candidate.transaction(|working| {
            std::mem::swap(&mut self.doc, working);
            let outcome = create(self).and_then(|()| {
                let created = self.doc.objects().len().saturating_sub(count);
                if created == 0 {
                    return Err(self.status.clone());
                }
                if created > 256 {
                    return Err("A construction can create at most 256 objects".into());
                }
                // Check the actual writer before accepting additions to imported documents.
                self.doc.encode().map(|_| ())
            });
            std::mem::swap(&mut self.doc, working);
            outcome
        });
        if let Err(error) = outcome {
            self.selection_order = order;
            self.status = status;
            return Err(error);
        }
        self.doc = candidate;
        self.status = format!("Created {} object(s)", self.doc.objects().len() - count);
        let objects: Vec<_> = self
            .doc
            .objects()
            .iter()
            .skip(count)
            .map(|object| self.mcp_object_summary(object))
            .collect();
        Ok(
            json!({"created_ids":objects.iter().map(|object| object["id"].clone()).collect::<Vec<_>>(),"selected_ids":self.selection_order,"objects":objects}),
        )
    }

    pub(super) fn mcp_create_point(&mut self, args: CreatePoint) -> Result<Value, String> {
        if !args.x.is_finite() || !args.y.is_finite() {
            return Err("Point coordinates must be finite".into());
        }
        if args
            .label
            .as_ref()
            .is_some_and(|label| label.chars().count() > 256)
        {
            return Err("A label can contain at most 256 characters".into());
        }
        self.mcp_create_transaction(|editor| {
            let point = editor.doc.snap_point(Point {
                x: args.x,
                y: args.y,
            });
            let id = editor.doc.add(Kind::Free(point));
            if let Some(label) = args.label {
                editor.doc.set_object_label(id, &label, true)?;
            }
            editor.select_only(id);
            Ok(())
        })
    }

    pub(super) fn mcp_construct(&mut self, args: Construct) -> Result<Value, String> {
        use Construction::*;
        let expected = match args.kind {
            Polygon => 3..=256,
            PointOnObject => 1..=2,
            CircleInterior | SectorInterior => 1..=1,
            AngleBisector | ThreePointArc | CenterArc | ArcOnCircle => 3..=3,
            _ => 2..=2,
        };
        if !expected.contains(&args.inputs.len()) {
            return Err(format!(
                "This construction requires {} to {} inputs",
                expected.start(),
                expected.end()
            ));
        }
        let unique: std::collections::BTreeSet<_> = args.inputs.iter().copied().collect();
        if unique.len() != args.inputs.len() {
            return Err("Construction inputs must be distinct".into());
        }
        for &id in &args.inputs {
            self.mcp_active_object(id)?;
        }
        if matches!(args.kind, PointOnObject)
            && args.inputs.len() == 2
            && self.doc.position(args.inputs[1]).is_none()
        {
            return Err("The second input must be a point to project".into());
        }
        self.mcp_create_transaction(|editor| {
            editor.doc.selected = unique;
            editor.selection_order = args.inputs;
            editor.status = "The inputs do not support this construction".into();
            match args.kind {
                Segment => editor.pair(0),
                Circle => editor.pair(1),
                Midpoint => editor.pair(2),
                Line => editor.pair(3),
                Ray => editor.pair(4),
                Parallel => editor.derived_line(false),
                Perpendicular => editor.derived_line(true),
                CircleBySegment => editor.circle_by_segment(),
                Polygon => editor.polygon_interior(),
                Intersection => editor.intersection(),
                AngleBisector => editor.angle_bisector(),
                ThreePointArc => editor.three_point_arc(),
                CenterArc => editor.center_arc(),
                ArcOnCircle => editor.arc_on_circle(),
                PointOnObject => editor.point_on_object(),
                CircleInterior => editor.circle_interior(),
                SectorInterior => editor.sector_boundary(),
            }
            Ok(())
        })
    }
}
