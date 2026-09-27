use super::*;

fn close_boundary(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-8, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}
fn boundary_document(mode: LineMode, start: Point, end: Point) -> Document {
    let mut d = Document::new();
    for (x, y) in [(150., 150.), (250., 150.), (250., 250.), (150., 250.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::Polygon(vec![1, 2, 3, 4]));
    d.add(Kind::Free(start));
    d.add(Kind::Free(end));
    d.add(match mode {
        LineMode::Segment => Kind::Segment(6, 7),
        LineMode::Ray => Kind::Ray(6, 7),
        LineMode::Line => Kind::Line(6, 7),
    });
    d.add(Kind::BoundaryIntersection(8, 5));
    d
}
#[test]
fn native_twenty_four_boundary_controls_preserve_packets_and_directed_contacts() {
    let states = [
        "ray_outside",
        "ray_inside",
        "ray_away",
        "ray_boundary_in",
        "ray_boundary_out",
        "line_forward",
        "line_reverse",
        "line_inside",
        "vertex_start_rotated",
        "segment_both",
        "segment_one",
        "segment_none",
        "concave",
        "repeated",
        "winding_reverse",
        "overlap",
        "tangent_vertex",
        "through_vertices",
        "transformed",
        "edited",
        "zero_ray",
        "zero_polygon",
        "retraced",
        "overlap_inside",
    ];
    for state in states {
        let bytes = fs::read(format!(
            "tests/fixtures/polygon_boundary56_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
        let point = d
            .objects
            .iter()
            .find(|o| matches!(o.kind, Kind::BoundaryIntersection(..)))
            .unwrap()
            .id;
        let expected = match state {
            "ray_away" | "ray_boundary_out" | "zero_ray" | "zero_polygon" => None,
            "ray_inside" | "ray_boundary_in" | "line_reverse" | "line_inside" | "segment_none" => {
                Some(Point { x: 250., y: 200. })
            }
            "overlap" | "tangent_vertex" | "through_vertices" => Some(Point { x: 150., y: 150. }),
            "overlap_inside" => Some(Point { x: 250., y: 150. }),
            "concave" => Some(Point { x: 150., y: 210. }),
            "transformed" => Some(Point { x: 190., y: 200. }),
            "edited" => Some(Point { x: 280., y: 220. }),
            _ => Some(Point { x: 150., y: 200. }),
        };
        close_boundary(d.position(point), expected);
        let observer = d
            .objects
            .iter()
            .find(|o| matches!(o.kind,Kind::Segment(a,_) if a==point))
            .unwrap();
        match expected {
            Some(p) => close_boundary(d.line_points(observer.id).map(|(a, _, _)| a), Some(p)),
            None => assert!(d.line_points(observer.id).is_none(), "{state}"),
        }
    }
}
#[test]
fn authored_boundary_query_uses_forward_direction_for_every_straight_extent() {
    for mode in [LineMode::Segment, LineMode::Ray, LineMode::Line] {
        let mut d = boundary_document(mode, Point { x: 180., y: 200. }, Point { x: 220., y: 200. });
        close_boundary(d.position(9), Some(Point { x: 250., y: 200. }));
        d.verify_roundtrip().unwrap();
        d.checkpoint();
        assert!(d.move_point(7, Point { x: 100., y: 200. }));
        close_boundary(d.position(9), Some(Point { x: 150., y: 200. }));
        d.undo();
        close_boundary(d.position(9), Some(Point { x: 250., y: 200. }));
    }
}
#[test]
fn authored_boundary_contact_excludes_origin_and_preserves_undefined_dependents() {
    let mut d = boundary_document(
        LineMode::Ray,
        Point { x: 150., y: 200. },
        Point { x: 300., y: 200. },
    );
    close_boundary(d.position(9), Some(Point { x: 250., y: 200. }));
    let anchor = d.add(Kind::Free(Point { x: 340., y: 340. }));
    let observer = d.add(Kind::Segment(9, anchor));
    d.checkpoint();
    assert!(d.move_point(7, Point { x: 100., y: 200. }));
    assert!(d.position(9).is_none());
    assert!(d.line_points(observer).is_none());
    d.verify_roundtrip().unwrap();
    d.undo();
    close_boundary(d.position(9), Some(Point { x: 250., y: 200. }));
    d.checkpoint();
    assert!(d.move_point(7, Point { x: 150., y: 200. }));
    assert!(d.position(9).is_none());
    d.verify_roundtrip().unwrap();
    d.undo();
    d.selected.insert(5);
    let removed = d.clear_selected().unwrap();
    assert!(removed.contains(&9) && removed.contains(&observer));
    d.undo();
    assert!(d.position(9).is_some());
}
#[test]
fn authored_boundary_query_follows_polygon_edits_and_strict_reopen() {
    for vertices in [vec![1, 2, 2, 3, 4, 1], vec![1, 2, 3, 4, 1, 2, 1]] {
        let mut repeated = boundary_document(
            LineMode::Ray,
            Point { x: 100., y: 200. },
            Point { x: 300., y: 200. },
        );
        repeated.objects[4].kind = Kind::Polygon(vertices.clone());
        close_boundary(repeated.position(9), Some(Point { x: 150., y: 200. }));
        let saved = repeated.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[4].kind, Kind::Polygon(vertices));
        close_boundary(reopened.position(9), Some(Point { x: 150., y: 200. }));
        assert_eq!(reopened.encode().unwrap(), saved);
    }
    let mut invalid = boundary_document(
        LineMode::Ray,
        Point { x: 100., y: 200. },
        Point { x: 300., y: 200. },
    );
    invalid.objects[4].kind = Kind::Polygon(vec![1, 2, 1, 2]);
    assert!(invalid.encode().is_err());
    let mut d = boundary_document(
        LineMode::Ray,
        Point { x: 200., y: 200. },
        Point { x: 300., y: 200. },
    );
    d.checkpoint();
    assert!(d.move_point(2, Point { x: 280., y: 150. }));
    assert!(d.move_point(3, Point { x: 280., y: 250. }));
    close_boundary(d.position(9), Some(Point { x: 280., y: 200. }));
    let saved = d.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    close_boundary(reopened.position(9), Some(Point { x: 280., y: 200. }));
    d.undo();
    close_boundary(d.position(9), Some(Point { x: 250., y: 200. }));
    d.redo();
    close_boundary(d.position(9), Some(Point { x: 280., y: 200. }));
    let point = &reopened.objects[8];
    let rs = records(&saved).unwrap();
    let h = point.header_offset.unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    assert_eq!(&saved[h..h + 2], &93u16.to_le_bytes());
    assert_eq!(
        &saved[refs.data.start..refs.data.end],
        &[2u32, 8, 5]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>()
    );
}
#[test]
fn malformed_boundary_profiles_stay_opaque_without_parent_reinterpretation() {
    let bytes = fs::read("tests/fixtures/polygon_boundary56_ray_outside_input.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let id = d
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::BoundaryIntersection(..)))
        .unwrap()
        .id;
    let h = d.objects[id - 1].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    for (at, value) in [
        (h + 6, 2u16.to_le_bytes().to_vec()),
        (h + 8, 4u16.to_le_bytes().to_vec()),
        (refs.data.start + 4, 5u32.to_le_bytes().to_vec()),
        (refs.data.start + 8, (id as u32).to_le_bytes().to_vec()),
    ] {
        let mut b = bytes.clone();
        b[at..at + value.len()].copy_from_slice(&value);
        let invalid = Document::from_bytes(&b, None).unwrap();
        assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(93));
        assert_eq!(invalid.encode().unwrap(), b);
    }
    let end = rs[i..].iter().find(|r| r.kind == 0x7d7).unwrap().start;
    let mut b = bytes.clone();
    let mut constraint = vec![];
    put_record(&mut constraint, 0x7d3, &93u32.to_le_bytes());
    b.splice(end..end, constraint);
    let invalid = Document::from_bytes(&b, None).unwrap();
    assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(93));
    assert_eq!(invalid.encode().unwrap(), b);
}
