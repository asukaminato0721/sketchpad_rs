use super::*;
use crate::document::{EmbeddedToolInput, EmbeddedToolInputKind};
use std::collections::BTreeMap;

#[test]
fn native_triangle_tool_outputs_decode_boundary_edges_and_keep_cached_bytes() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/embedded_tool_path_circle_on_polygon_native_verified.gsp"
    );
    let document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    for (id, edge, fraction) in [(10, 1, 0.6682658062080076), (11, 2, 0.028411740636517777)] {
        assert_eq!(
            document.objects[id - 1].kind,
            Kind::PointOnPolygon(8, edge, fraction)
        );
        let vertices = document.polygon_points(8).unwrap();
        let start = vertices[edge];
        let end = vertices[(edge + 1) % vertices.len()];
        let point = document.position(id).unwrap();
        assert!((point.x - (start.x + (end.x - start.x) * fraction)).abs() < 1e-9);
        assert!((point.y - (start.y + (end.y - start.y) * fraction)).abs() < 1e-9);
    }
    assert_eq!(document.encode().unwrap(), bytes);
    let constraint = records(bytes)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x07d3)
        .nth(1)
        .unwrap();
    for (offset, replacement) in [
        (2, vec![53, 0]),
        (12, 3u32.to_le_bytes().to_vec()),
        (16, vec![0, 0, 0, 0]),
    ] {
        let mut malformed = bytes.to_vec();
        malformed
            [constraint.data.start + offset..constraint.data.start + offset + replacement.len()]
            .copy_from_slice(&replacement);
        assert_eq!(
            Document::from_bytes(&malformed, None).unwrap().objects[9].kind,
            Kind::Unsupported(15)
        );
    }
}

#[test]
fn contextual_unplaced_points_accept_constructed_path_modes_only_in_tool_context() {
    for mode in 0..6 {
        let mut document = Document::new();
        document.add(Kind::Free(Point { x: 0., y: 0. }));
        document.add(Kind::Free(Point { x: 60., y: 80. }));
        document.add(Kind::Free(Point { x: 120., y: 0. }));
        let host = document.add(match mode {
            0 => Kind::Segment(1, 3),
            1 => Kind::Ray(1, 3),
            2 => Kind::Line(1, 3),
            3 => Kind::Circle(1, 3),
            4 => Kind::ThreePointArc(1, 2, 3),
            _ => Kind::Polygon(vec![1, 2, 3]),
        });
        let point = document.add(match mode {
            0 => Kind::PointOnSegment(host, 0.5),
            1 => Kind::PointOnRay(host, 1.5),
            2 => Kind::PointOnLine(host, -0.5),
            3 => Kind::PointOnCircle(host, 0., 1.),
            4 => Kind::PointOnArc(host, 0.5),
            _ => Kind::PointOnPolygon(host, 1, 0.5),
        });
        let authored = document.encode().unwrap();
        let mut unplaced = b"GSP4".to_vec();
        let mut id = 0;
        for record in records(&authored).unwrap() {
            if record.kind == 0x07d0 {
                id += 1;
            }
            if record.kind == 0x07d3 && id == point {
                continue;
            }
            put_record(&mut unplaced, record.kind, &authored[record.data]);
        }
        let givens: BTreeMap<_, _> = (1..=3)
            .map(|local_id| {
                (
                    local_id,
                    EmbeddedToolInput {
                        local_id,
                        native_class: 1,
                        native_subtype: 4,
                    },
                )
            })
            .collect();
        let prototype = Document::from_bytes_with_tool_inputs(&unplaced, None, &givens).unwrap();
        assert!(!prototype.partial_view, "mode {mode}");
        assert_eq!(
            prototype.objects[point - 1].kind,
            Kind::ToolPointOnPath(host)
        );
        assert_eq!(
            Document::from_bytes(&unplaced, None).unwrap().objects[point - 1].kind,
            Kind::Unsupported(15)
        );
        assert!(prototype.encode().is_err());
    }
    let native = Document::from_bytes(include_bytes!("../../../tests/fixtures/embedded_tool_constrained_point_from_endpoints_created_native_verified.gsp"), None).unwrap();
    assert!(native
        .embedded_tool_inputs(0)
        .unwrap()
        .iter()
        .all(|input| input.kind() == Some(EmbeddedToolInputKind::Point)));
    assert_eq!(
        native.embedded_tool_document(0).unwrap().objects[3].kind,
        Kind::ToolPointOnPath(3)
    );
}
