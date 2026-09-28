use super::*;
use crate::document::EmbeddedToolInputKind;

const MIDPOINT_TOOL: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/embedded_tool_given_selection_midpoint_native_verified.gsp"
);

#[test]
fn native_segment_given_binds_primitive_and_transformed_segments_with_live_midpoints() {
    let mut document = Document::from_bytes(MIDPOINT_TOOL, None).unwrap();
    assert_eq!(
        document.embedded_tool_inputs(0).unwrap()[0].kind(),
        Some(EmbeddedToolInputKind::Segment)
    );
    let prototype = document.embedded_tool_document(0).unwrap();
    assert!(!prototype.partial_view);
    assert_eq!(
        prototype.objects[0].kind,
        Kind::ToolInput(EmbeddedToolInputKind::Segment)
    );
    assert_eq!(prototype.objects[1].kind, Kind::SegmentMidpoint(1));
    assert_eq!(prototype.line_points(1), None);
    assert!(prototype.encode().is_err());
    let a = document.add(Kind::Free(Point { x: 300., y: 280. }));
    let b = document.add(Kind::Free(Point { x: 360., y: 300. }));
    let primitive = document.add(Kind::Segment(a, b));
    let transformed = document.add(Kind::VectorTranslatedSegment(primitive, a, b));
    for host in [primitive, transformed] {
        let before = document.encode().unwrap();
        let history = document.undo.len();
        let output = document.invoke_embedded_tool(0, &[host]).unwrap()[0];
        assert_eq!(document.undo.len(), history + 1);
        let (start, end, _) = document.line_points(host).unwrap();
        assert_eq!(
            document.position(output),
            Some(Point {
                x: (start.x + end.x) / 2.,
                y: (start.y + end.y) / 2.
            })
        );
        let saved = document.encode().unwrap();
        document.verify_roundtrip().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), before);
        document.redo();
        assert_eq!(document.encode().unwrap(), saved);
    }
    let last = document.objects.len();
    assert_eq!(
        document.objects[last - 1].kind,
        Kind::SegmentMidpoint(transformed)
    );
    assert!(document.move_point(b, Point { x: 400., y: 320. }));
    let (start, end, _) = document.line_points(transformed).unwrap();
    assert_eq!(
        document.position(last),
        Some(Point {
            x: (start.x + end.x) / 2.,
            y: (start.y + end.y) / 2.
        })
    );
    for kind in [Kind::Ray(a, b), Kind::Line(a, b), Kind::Circle(a, b)] {
        let host = document.add(kind);
        let before = document.encode().unwrap();
        let history = document.undo.len();
        assert!(
            document
                .invoke_embedded_tool(0, &[host])
                .unwrap_err()
                .contains("finite segment")
        );
        assert_eq!(document.encode().unwrap(), before);
        assert_eq!(document.undo.len(), history);
    }
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_SEGMENT_FIXTURE").is_some() {
        fs::write(
            "/tmp/embedded_tool_segment_midpoint_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn native_transformed_midpoint_and_hidden_helper_decode_without_visibility_normalization() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_segment_given_midpoint_invoked_native_verified.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.objects[18].kind, Kind::SegmentMidpoint(11));
    let (a, b, _) = document.line_points(11).unwrap();
    assert_eq!(
        document.position(19),
        Some(Point {
            x: (a.x + b.x) / 2.,
            y: (a.y + b.y) / 2.
        })
    );
    assert_eq!(document.encode().unwrap(), bytes);
    let clipboard = {
        document.selected.insert(11);
        document.selected.insert(19);
        document.copy_selected().unwrap()
    };
    let copies = document.paste(&clipboard).unwrap();
    assert!(matches!(
        document.objects[*copies.last().unwrap() - 1].kind,
        Kind::SegmentMidpoint(_)
    ));
    document.verify_roundtrip().unwrap();

    let bytes = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_midpoint_hidden_helper_invoked_native_verified.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(document.objects[7].presentation_suppressed);
    assert!(!document.is_displayed(8));
    assert!(document.objects[8].visible);
    let header = document.objects[7].header_offset.unwrap();
    assert_eq!(bytes[header + 2], 2);
    let prototype = document.embedded_tool_document(0).unwrap();
    assert!(prototype.objects[2].presentation_suppressed);
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(document.move_point(6, Point { x: 300., y: 280. }));
    let saved = document.encode().unwrap();
    assert_eq!(saved[header + 2], 2);
    document.verify_roundtrip().unwrap();
    document.selected.clear();
    document.selected.insert(3);
    document.set_selected_visibility(false);
    assert!(!document.is_displayed(3));
    document.show_all_hidden();
    assert!(document.is_displayed(3));
    assert!(!document.is_displayed(8));
    assert!(document.objects[7].presentation_suppressed);
    assert_eq!(document.encode().unwrap()[header + 2], 2);
}
