use super::*;
fn close(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-8, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}
#[test]
fn native_round_intersections_keep_ordered_roles_arc_domains_and_linear_limits() {
    for state in [
        "circle_circle",
        "arc_circle_both",
        "circle_arc_both",
        "arc_circle_upper",
        "circle_arc_upper",
        "arc_arc_both",
        "arc_arc_upper",
        "arc_arc_disjoint",
        "arc_reflected",
        "arc_reversed",
        "tangent",
        "endpoints",
        "nointersection",
        "three_linear",
        "linear_both",
        "linear_reverse",
        "circle_linear_both",
        "both_linear",
    ] {
        let bytes = fs::read(format!(
            "tests/fixtures/circle_arc57_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
        let upper = Point {
            x: 230.,
            y: 200. - 30. * 3f64.sqrt(),
        };
        let lower = Point {
            x: 230.,
            y: 200. + 30. * 3f64.sqrt(),
        };
        let p = |x| Some(Point { x, y: 200. });
        let expected = match state {
            "arc_arc_disjoint" | "nointersection" => [None, None],
            "arc_circle_upper" | "circle_arc_upper" | "arc_arc_upper" | "endpoints" => {
                [Some(upper); 2]
            }
            "tangent" => [p(260.); 2],
            "three_linear" => [p(200.); 2],
            "linear_both" => [p(200.), p(320.)],
            "linear_reverse" | "circle_linear_both" => [p(320.), p(200.)],
            "both_linear" => [p(250.); 2],
            _ => [Some(upper), Some(lower)],
        };
        let mut roots = Vec::new();
        for object in &d.objects {
            if let Kind::CircleCircleIntersection(_, _, variant) = object.kind {
                close(
                    d.position(object.id),
                    expected[if variant == 1 { 0 } else { 1 }],
                );
                roots.push(object.id);
            }
        }
        assert_eq!(roots.len(), 2);
        let mut observed = 0;
        for object in &d.objects {
            if let Kind::Segment(a, b) = object.kind {
                if roots.contains(&a) {
                    match d.position(a) {
                        Some(point) => {
                            let (start, end, mode) = d.line_points(object.id).unwrap();
                            close(Some(start), Some(point));
                            close(Some(end), d.position(b));
                            assert_eq!(mode, LineMode::Segment);
                        }
                        None => assert!(d.line_points(object.id).is_none()),
                    }
                    observed += 1;
                }
            }
        }
        assert_eq!(observed, 2);
    }
}
#[test]
fn round_intersections_update_across_linear_circular_and_undefined_arc_states() {
    let bytes = fs::read("tests/fixtures/circle_arc57_linear_both_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let (arc, circle, root) = d
        .objects
        .iter()
        .find_map(|o| match o.kind {
            Kind::CircleCircleIntersection(a, b, 1) => Some((a, b, o.id)),
            _ => None,
        })
        .unwrap();
    let middle = match d.objects[arc - 1].kind {
        Kind::ThreePointArc(_, b, _) => b,
        _ => panic!(),
    };
    d.checkpoint();
    assert!(d.move_point(middle, Point { x: 200., y: 180. }));
    assert!(d.position(root).is_some());
    d.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(root), Some(Point { x: 200., y: 200. }));
    d.checkpoint();
    let end = match d.objects[arc - 1].kind {
        Kind::ThreePointArc(_, _, c) => c,
        _ => panic!(),
    };
    assert!(d.move_point(end, Point { x: 160., y: 200. }));
    assert!(d.position(root).is_none());
    let observer = d.add(Kind::Segment(root, middle));
    assert!(d.line_points(observer).is_none());
    d.verify_roundtrip().unwrap();
    d.undo(); // Remove the newly authored observer.
    d.undo(); // Restore the preceding endpoint edit.
    close(d.position(root), Some(Point { x: 200., y: 200. }));
    d.selected.insert(circle);
    let removed = d.clear_selected().unwrap();
    assert!(removed.contains(&root));
    d.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(root), Some(Point { x: 200., y: 200. }));
}
