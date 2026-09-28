use super::*;
use crate::document::EmbeddedToolInputKind;

const SQUARE: &[u8] =
    include_bytes!("../../../../../tests/fixtures/embedded_tool_square_registered_native_verified.gsp");
const SIMPLE: &[u8] =
    include_bytes!("../../../../../tests/fixtures/embedded_tool_simple_created_native_verified.gsp");
const PATH_SEGMENT: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/embedded_tool_path_segment_created_native_verified.gsp"
);
const PATH_CIRCLE: &[u8] =
    include_bytes!("../../../../../tests/fixtures/embedded_tool_path_circle_created_native_verified.gsp");

#[test]
fn native_path_tool_results_extrapolate_on_rays_and_lines() {
    for (bytes, ray) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_path_circle_on_ray_native_verified.gsp"
            )
            .as_slice(),
            true,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_path_circle_on_line_native_verified.gsp"
            )
            .as_slice(),
            false,
        ),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let id = document.objects.len();
        let (host, parameter) = match document.objects[id - 1].kind {
            Kind::PointOnRay(host, parameter) if ray => {
                assert!(parameter > 1.);
                (host, parameter)
            }
            Kind::PointOnLine(host, parameter) if !ray => {
                assert!(parameter < 0.);
                (host, parameter)
            }
            ref kind => panic!("Unexpected native result: {kind:?}"),
        };
        let (a, b, _) = document.line_points(host).unwrap();
        assert_eq!(
            document.position(id),
            Some(Point {
                x: a.x + (b.x - a.x) * parameter,
                y: a.y + (b.y - a.y) * parameter
            })
        );
        assert_eq!(document.encode().unwrap(), bytes);
        let endpoint = match document.objects[host - 1].kind {
            Kind::Ray(_, endpoint) | Kind::Line(_, endpoint) => endpoint,
            ref kind => panic!("Unexpected native host: {kind:?}"),
        };
        assert!(document.move_point(
            endpoint,
            Point {
                x: b.x + 40.,
                y: b.y + 20.
            }
        ));
        assert_eq!(
            document.position(id),
            Some(Point {
                x: a.x + (b.x + 40. - a.x) * parameter,
                y: a.y + (b.y + 20. - a.y) * parameter
            })
        );
        document.verify_roundtrip().unwrap();
    }
}

#[test]
fn path_tool_binds_straights_arcs_and_polygon_boundaries_with_live_saved_hosts() {
    let mut document = Document::from_bytes(PATH_CIRCLE, None).unwrap();
    let a = document.add(Kind::Free(Point { x: 300., y: 280. }));
    let b = document.add(Kind::Free(Point { x: 360., y: 300. }));
    let c = document.add(Kind::Free(Point { x: 330., y: 220. }));
    let ray = document.add(Kind::Ray(a, b));
    let line = document.add(Kind::Line(a, b));
    let arc = document.add(Kind::ThreePointArc(a, c, b));
    let polygon = document.add(Kind::Polygon(vec![a, b, c]));
    for host in [ray, line, arc, polygon] {
        let before = document.encode().unwrap();
        let history = document.undo.len();
        let point = document.invoke_embedded_tool(0, &[host]).unwrap()[0];
        let kind = document.objects[point - 1].kind.clone();
        match kind {
            Kind::PointOnRay(h, t) => assert!(h == ray && t >= 0.),
            Kind::PointOnLine(h, t) => assert!(h == line && t.is_finite()),
            Kind::PointOnArc(h, t) => assert!(h == arc && (0.0..1.0).contains(&t)),
            Kind::PointOnPolygon(h, edge, t) => {
                assert_eq!(h, polygon);
                assert!(edge < 3 && (0.0..1.0).contains(&t));
                assert_eq!(
                    document.position(point),
                    document.point_on_polygon(h, edge, t)
                );
            }
            _ => panic!("Unexpected path result: {kind:?}"),
        }
        assert!(document.position(point).is_some());
        assert_eq!(document.undo.len(), history + 1);
        let saved = document.encode().unwrap();
        let reparsed = Document::from_bytes(&saved, None).unwrap();
        assert_eq!(reparsed.objects[point - 1].kind, kind);
        document.verify_roundtrip().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), before);
        document.redo();
        assert_eq!(document.objects[point - 1].kind, kind);
    }
    let kinds: Vec<_> = document
        .objects
        .iter()
        .map(|object| object.kind.clone())
        .collect();
    assert!(document.move_point(b, Point { x: 400., y: 320. }));
    for (index, kind) in kinds.iter().enumerate() {
        if matches!(
            kind,
            Kind::PointOnRay(..)
                | Kind::PointOnLine(..)
                | Kind::PointOnArc(..)
                | Kind::PointOnPolygon(..)
        ) {
            assert_eq!(&document.objects[index].kind, kind);
            assert!(document.position(index + 1).is_some());
        }
    }
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_PATH_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_path_extended_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn native_path_givens_are_abstract_and_bind_segment_or_circle_with_persistent_placement() {
    for bytes in [PATH_SEGMENT, PATH_CIRCLE] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        let inputs = document.embedded_tool_inputs(0).unwrap();
        assert_eq!(inputs.len(), 1);
        assert!(inputs[0].is_path());
        assert!(!inputs[0].is_point());
        let prototype = document.embedded_tool_document(0).unwrap();
        assert!(!prototype.partial_view);
        assert_eq!(
            prototype.objects[0].kind,
            Kind::ToolInput(EmbeddedToolInputKind::Path)
        );
        assert_eq!(prototype.objects[1].kind, Kind::ToolPointOnPath(1));
        assert_eq!(prototype.position(1), None);
        assert_eq!(prototype.line_points(1), None);
        assert_eq!(prototype.circle_points(1), None);
        assert!(prototype.encode().unwrap_err().contains("Tool givens"));
        let start = document.add(Kind::Free(Point { x: 300., y: 280. }));
        let end = document.add(Kind::Free(Point { x: 360., y: 300. }));
        let segment = document.add(Kind::Segment(start, end));
        let before = document.encode().unwrap();
        let history = document.undo.len();
        let point = document.invoke_embedded_tool(0, &[segment]).unwrap()[0];
        let Kind::PointOnSegment(host, fraction) = document.objects[point - 1].kind else {
            panic!("Expected a constrained point")
        };
        assert_eq!(host, segment);
        assert!((0.0..1.0).contains(&fraction));
        assert_eq!(
            document.position(point),
            Some(Point {
                x: 300. + 60. * fraction,
                y: 280. + 20. * fraction
            })
        );
        assert_eq!(document.undo.len(), history + 1);
        let encoded = document.encode().unwrap();
        assert_eq!(prototype_bytes(&encoded), prototype_bytes(bytes));
        document.verify_roundtrip().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), before);
        document.redo();
        assert_eq!(document.encode().unwrap(), encoded);
        assert_eq!(
            document.objects[point - 1].kind,
            Kind::PointOnSegment(segment, fraction)
        );
        assert!(document.move_point(end, Point { x: 400., y: 320. }));
        assert_eq!(
            document.position(point),
            Some(Point {
                x: 300. + 100. * fraction,
                y: 280. + 40. * fraction
            })
        );
        let circle = document.add(Kind::Circle(start, end));
        let circle_point = document.invoke_embedded_tool(0, &[circle]).unwrap()[0];
        let Kind::PointOnCircle(host, x, y) = document.objects[circle_point - 1].kind else {
            panic!("Expected a circle point")
        };
        assert_eq!(host, circle);
        assert!((x * x + y * y - 1.).abs() < 1e-12);
        let (center, radius) = document.circle_points(circle).unwrap();
        assert!((document.position(circle_point).unwrap().distance(center) - radius).abs() < 1e-10);
        document.verify_roundtrip().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_TOOL_PATH_FIXTURES").is_some() {
            let suffix = if bytes == PATH_SEGMENT {
                "segment"
            } else {
                "circle"
            };
            fs::write(
                format!("/tmp/embedded_tool_path_{suffix}_invoked_port.gsp"),
                document.encode().unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn tool_given_categories_and_construction_roles_reject_invalid_bindings_atomically() {
    let mut document = Document::from_bytes(PATH_SEGMENT, None).unwrap();
    let before = document.encode().unwrap();
    assert!(
        document
            .invoke_embedded_tool(0, &[1])
            .unwrap_err()
            .contains("available path")
    );
    assert_eq!(document.encode().unwrap(), before);
    assert!(document.undo.is_empty());
    let entries = records(SIMPLE).unwrap();
    let start = entries
        .iter()
        .position(|record| record.kind == 0x03ea)
        .unwrap();
    let input = entries[start..]
        .iter()
        .find(|record| record.kind == 0x07d0)
        .unwrap();
    for (class, subtype) in [(1u16, 39u16), (11, 57)] {
        let mut bytes = SIMPLE.to_vec();
        bytes[input.data.start + 6..input.data.start + 8].copy_from_slice(&class.to_le_bytes());
        bytes[input.data.start + 8..input.data.start + 10].copy_from_slice(&subtype.to_le_bytes());
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        let prototype = document.embedded_tool_document(0).unwrap();
        assert!(matches!(prototype.objects[0].kind, Kind::Unsupported(0)));
        assert!(document.invoke_embedded_tool(0, &[1, 2]).is_err());
        assert_eq!(document.encode().unwrap(), bytes);
        assert!(document.undo.is_empty());
    }
    // A path given used as a segment endpoint is malformed even when its
    // input header and every construction code are individually recognized.
    let mut bytes = SIMPLE.to_vec();
    bytes[input.data.start + 8..input.data.start + 10].copy_from_slice(&5u16.to_le_bytes());
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(
        document
            .invoke_embedded_tool(0, &[1, 3])
            .unwrap_err()
            .contains("input types")
    );
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(document.undo.is_empty());
}

fn prototype_bytes(bytes: &[u8]) -> Vec<u8> {
    let entries = records(bytes).unwrap();
    let start = entries
        .iter()
        .position(|record| record.kind == 0x03ea)
        .unwrap();
    let end = entries[start..]
        .iter()
        .position(|record| record.kind == 0x03eb)
        .unwrap()
        + start;
    bytes[entries[start].start..entries[end].data.end].to_vec()
}

#[test]
fn isolated_native_tool_uses_local_objects_fonts_and_point_placeholders() {
    let document = Document::from_bytes(SQUARE, None).unwrap();
    let tool = document.embedded_tool_document(0).unwrap();
    assert_eq!(tool.objects.len(), 13);
    assert!(!tool.partial_view);
    assert_eq!(tool.position(1), Some(Point { x: 0., y: 0. }));
    assert_eq!(tool.position(2), Some(Point { x: 0., y: 0. }));
    assert_eq!(tool.objects[2].kind, Kind::Segment(2, 1));
    assert_eq!(tool.objects[12].kind, Kind::Polygon(vec![2, 1, 9, 7]));
    assert_eq!(
        tool.text_font_families(),
        Some(vec!["Arial".into(), "Times".into()])
    );
    assert_eq!(tool.text_font_state(1).unwrap().family, "Arial");
    assert_eq!(document.objects.len(), 1);
    assert_eq!(document.encode().unwrap(), SQUARE);
    assert!(document.embedded_tool_document(1).is_err());
}

#[test]
fn native_square_tool_binds_ordered_points_tracks_inputs_and_undoes_atomically() {
    let mut document = Document::from_bytes(SQUARE, None).unwrap();
    let first = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let second = document.add(Kind::Free(Point { x: 260., y: 180. }));
    let before = document.encode().unwrap();
    let undo_count = document.undo.len();
    let created = document.invoke_embedded_tool(0, &[first, second]).unwrap();
    assert_eq!(created, (4..=14).collect::<Vec<_>>());
    assert_eq!(document.undo.len(), undo_count + 1);
    assert_eq!(document.objects[3].kind, Kind::Segment(first, second));
    assert_eq!(
        document.objects[13].kind,
        Kind::Polygon(vec![first, second, 10, 8])
    );
    assert_eq!(document.position(8), Some(Point { x: 200., y: 120. }));
    assert_eq!(document.position(10), Some(Point { x: 260., y: 120. }));
    for id in [5, 6, 7, 9] {
        assert!(!document.objects[id - 1].visible);
        assert!(!document.selected.contains(&id));
    }
    for id in &created {
        let prototype = document.embedded_tool_document(0).unwrap();
        let source = &prototype.objects[*id - 2];
        assert_eq!(document.objects[*id - 1].visual_style, source.visual_style);
        assert_eq!(document.objects[*id - 1].label, source.label);
        assert_eq!(document.objects[*id - 1].visible, source.visible);
        assert_eq!(document.objects[*id - 1].tracing, source.tracing);
    }
    assert_eq!(
        document.objects[7]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "C"
    );
    assert_eq!(
        document.objects[9]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "D"
    );
    let encoded = document.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_RUNTIME_FIXTURES").is_some() {
        fs::write("/tmp/embedded_tool_square_invoked_port.gsp", &encoded).unwrap();
    }
    assert_eq!(
        encoded,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_square_invoked_suppression_writer_native_verified.gsp"
        )
    );
    assert_eq!(prototype_bytes(&encoded), prototype_bytes(SQUARE));
    document.verify_roundtrip().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.polygon_points(14), document.polygon_points(14));
    assert_eq!(
        reopened.embedded_tools().unwrap(),
        document.embedded_tools().unwrap()
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    assert_eq!(document.encode().unwrap(), encoded);
    assert!(document.move_point(second, Point { x: 280., y: 180. }));
    assert_eq!(document.position(8), Some(Point { x: 200., y: 100. }));
    assert_eq!(document.position(10), Some(Point { x: 280., y: 100. }));
    document.verify_roundtrip().unwrap();
}

#[test]
fn native_simple_tool_preserves_midpoint_host_and_input_styles() {
    let mut document = Document::from_bytes(SIMPLE, None).unwrap();
    let first = document.add(Kind::Free(Point { x: 300., y: 200. }));
    let second = document.add(Kind::Free(Point { x: 360., y: 220. }));
    let inputs_before = document.objects[4..6].to_vec();
    let created = document.invoke_embedded_tool(0, &[first, second]).unwrap();
    assert_eq!(created, vec![7, 8]);
    assert_eq!(document.objects[6].kind, Kind::Segment(first, second));
    assert_eq!(document.objects[7].kind, Kind::Midpoint(first, second));
    assert_eq!(document.objects[7].midpoint_host, Some(7));
    assert_eq!(document.position(8), Some(Point { x: 330., y: 210. }));
    assert_eq!(document.objects[4..6], inputs_before);
    document.verify_roundtrip().unwrap();
    assert_eq!(
        document.encode().unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_simple_invoked_writer_native_verified.gsp"
        )
    );
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_RUNTIME_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_simple_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn tool_invocation_rejects_bad_inputs_and_unsupported_prototypes_without_mutation() {
    let mut document = Document::from_bytes(SIMPLE, None).unwrap();
    document.selected.insert(1);
    let original = document.encode().unwrap();
    let objects = document.objects.clone();
    let selected = document.selected.clone();
    let history = document.undo.len();
    for inputs in [vec![], vec![1], vec![1, 99], vec![1, 3]] {
        assert!(document.invoke_embedded_tool(0, &inputs).is_err());
        assert_eq!(document.objects, objects);
        assert_eq!(document.selected, selected);
        assert_eq!(document.undo.len(), history);
        assert_eq!(document.encode().unwrap(), original);
    }
    let mut unsupported = original.clone();
    let entries = records(&unsupported).unwrap();
    let tool_start = entries
        .iter()
        .position(|record| record.kind == 0x03ea)
        .unwrap();
    let output = entries[tool_start..]
        .iter()
        .filter(|record| record.kind == 0x07d0)
        .nth(2)
        .unwrap();
    unsupported[output.data.start..output.data.start + 2].copy_from_slice(&999u16.to_le_bytes());
    let mut document = Document::from_bytes(&unsupported, None).unwrap();
    assert!(document.invoke_embedded_tool(0, &[1, 2]).is_err());
    assert_eq!(document.encode().unwrap(), unsupported);
    assert!(document.undo.is_empty());

    // A syntactically valid declaration that omits a free input must not
    // materialize the remaining native placeholder as a drawing point.
    let declaration = records(SIMPLE)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x04b4)
        .unwrap();
    let mut replacement = Vec::new();
    put_record(&mut replacement, 0x04b4, &[1, 0, 0, 0, 2, 0, 0, 0]);
    let mut missing_input = SIMPLE.to_vec();
    missing_input.splice(declaration.start..declaration.data.end, replacement);
    let mut document = Document::from_bytes(&missing_input, None).unwrap();
    assert_eq!(
        document.embedded_tools().unwrap()[0].input_ids.as_deref(),
        Some(&[2][..])
    );
    assert!(
        document
            .invoke_embedded_tool(0, &[1])
            .unwrap_err()
            .contains("undeclared free-point")
    );
    assert_eq!(document.encode().unwrap(), missing_input);
    assert!(document.undo.is_empty());
}
