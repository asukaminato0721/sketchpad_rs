use super::*;
fn close(a: Option<Point>, b: Option<Point>) {
    match (a, b) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-8, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}
fn root(right: bool, cx: f64, y: f64) -> Point {
    Point {
        x: cx + (if right { 1. } else { -1. }) * (3600. - (y - 200.).powi(2)).max(0.).sqrt(),
        y,
    }
}
#[test]
fn native_twenty_arc_intersection_controls_preserve_branch_orientation_domains_and_bytes() {
    let primary = [
        "center_both",
        "center_quarter",
        "line_reverse",
        "ray_domain",
        "segment_domain",
        "endpoints",
        "tangent",
        "nointersection",
        "three_circular",
        "three_linear",
        "arc_translated",
        "arc_reflected",
        "arc_major",
    ];
    let extra = [
        "quarter_segments",
        "both_reverse",
        "arc_reversed",
        "start_rotated_major",
        "segment_single_left",
        "segment_single_right",
        "reflected_both",
    ];
    for (prefix, states) in [("", primary.as_slice()), ("extra_", extra.as_slice())] {
        for state in states {
            let bytes = fs::read(format!(
                "../../tests/fixtures/arc_intersection55_{prefix}{state}_native_saved.gsp"
            ))
            .unwrap();
            let d = Document::from_bytes(&bytes, None).unwrap();
            assert!(!d.partial_view, "{state}");
            assert_eq!(d.encode().unwrap(), bytes);
            let mut branch = [None, None];
            for o in &d.objects {
                if let Kind::LineCircleIntersection(_, _, v) = o.kind {
                    branch[if v == 1 { 0 } else { 1 }] = Some(o.id);
                }
            }
            let cx = if *state == "arc_translated" {
                237.79527559055118
            } else {
                200.
            };
            let y = if *state == "endpoints" {
                200.
            } else if *state == "tangent" {
                140.
            } else {
                170.
            };
            let right = root(true, cx, y);
            let left = root(false, cx, y);
            let expected = match *state {
                "segment_domain" | "nointersection" => [None, None],
                "three_linear" => [Some(Point { x: 220., y: 200. }); 2],
                "center_quarter"
                | "line_reverse"
                | "ray_domain"
                | "quarter_segments"
                | "segment_single_right" => [Some(right); 2],
                "arc_reflected" | "arc_major" | "segment_single_left" => [Some(left); 2],
                "both_reverse" | "arc_reversed" | "reflected_both" => [Some(left), Some(right)],
                _ => [Some(right), Some(left)],
            };
            for i in 0..2 {
                close(d.position(branch[i].unwrap()), expected[i]);
            }
            if prefix == "extra_" {
                let mut observed = 0;
                for o in &d.objects {
                    if let Kind::Segment(a, b) = o.kind {
                        if branch.contains(&Some(a)) {
                            let (a_position, b_position, m) = d.line_points(o.id).unwrap();
                            close(Some(a_position), d.position(a));
                            close(Some(b_position), d.position(b));
                            assert_eq!(m, LineMode::Segment);
                            observed += 1;
                        }
                    }
                }
                assert_eq!(observed, 2);
            }
        }
    }
}
fn authored(mode: LineMode, quarter: bool) -> Document {
    let mut d = Document::new();
    for p in [(200., 200.), (260., 200.), (140., 200.), (200., 140.)] {
        d.add(Kind::Free(Point { x: p.0, y: p.1 }));
    }
    d.add(Kind::CenterArc(1, 2, if quarter { 4 } else { 3 }));
    d.add(Kind::Free(Point {
        x: if mode == LineMode::Line { 100. } else { 200. },
        y: 170.,
    }));
    d.add(Kind::Free(Point { x: 300., y: 170. }));
    d.add(match mode {
        LineMode::Line => Kind::Line(6, 7),
        LineMode::Ray => Kind::Ray(6, 7),
        LineMode::Segment => Kind::Segment(6, 7),
    });
    d.add(Kind::LineCircleIntersection(8, 5, 1));
    d.add(Kind::LineCircleIntersection(8, 5, 0));
    d
}
#[test]
fn arc_branches_compact_filtered_hits_while_circle_branches_keep_native_fixed_roots() {
    let mut d = authored(LineMode::Ray, false);
    let r = root(true, 200., 170.);
    close(d.position(9), Some(r));
    close(d.position(10), Some(r));
    let circle = d.add(Kind::Circle(1, 2));
    let low = d.add(Kind::LineCircleIntersection(8, circle, 1));
    let high = d.add(Kind::LineCircleIntersection(8, circle, 0));
    close(d.position(low), None);
    close(d.position(high), Some(r));
    d.verify_roundtrip().unwrap();
}
#[test]
fn public_arc_intersections_follow_source_edits_undo_delete_and_strict_reopens() {
    let mut d = authored(LineMode::Line, true);
    let r = root(true, 200., 170.);
    let l = root(false, 200., 170.);
    close(d.position(9), Some(r));
    close(d.position(10), Some(r));
    d.verify_roundtrip().unwrap();
    d.checkpoint();
    assert!(d.move_point(4, Point { x: 140., y: 200. }));
    close(d.position(9), Some(r));
    close(d.position(10), Some(l));
    d.undo();
    close(d.position(10), Some(r));
    d.redo();
    close(d.position(10), Some(l));
    d.checkpoint();
    assert!(d.move_point(4, Point { x: 260., y: 200. }));
    close(d.position(9), None);
    close(d.position(10), None);
    d.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(10), Some(l));
    let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    close(reopened.position(9), Some(r));
    close(reopened.position(10), Some(l));
    d.selected.insert(5);
    let removed = d.clear_selected().unwrap();
    for id in [5, 9, 10] {
        assert!(removed.contains(&id));
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(9), Some(r));
}
#[test]
fn three_point_arc_linear_limit_is_bounded_and_recovers_live_circular_geometry() {
    let bytes =
        fs::read("../../tests/fixtures/arc_intersection55_three_circular_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    d.checkpoint();
    assert!(d.move_point(4, Point { x: 200., y: 200. }));
    close(d.position(10), None);
    close(d.position(13), None);
    assert!(d.move_point(7, Point { x: 220., y: 100. }));
    assert!(d.move_point(8, Point { x: 220., y: 300. }));
    for id in [10, 13] {
        close(d.position(id), Some(Point { x: 220., y: 200. }));
    }
    d.verify_roundtrip().unwrap();
    assert!(d.move_point(7, Point { x: 280., y: 100. }));
    assert!(d.move_point(8, Point { x: 280., y: 300. }));
    close(d.position(10), None);
    close(d.position(13), None);
    d.undo();
    close(d.position(10), Some(root(true, 200., 170.)));
}
#[test]
fn malformed_arc_intersection_class_parent_order_and_constraint_records_stay_opaque() {
    let bytes = fs::read("../../tests/fixtures/arc_intersection55_center_both_input.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let rs = records(&bytes).unwrap();
    let h = d.objects[9].header_offset.unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let r = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    for (at, data) in [
        (h + 6, 2u16.to_le_bytes().to_vec()),
        (h + 8, 4u16.to_le_bytes().to_vec()),
        (r.data.start + 4, 6u32.to_le_bytes().to_vec()),
        (r.data.start + 8, 10u32.to_le_bytes().to_vec()),
    ] {
        let mut b = bytes.clone();
        b[at..at + data.len()].copy_from_slice(&data);
        let invalid = Document::from_bytes(&b, None).unwrap();
        assert_eq!(invalid.objects[9].kind, Kind::Unsupported(11));
        assert_eq!(invalid.encode().unwrap(), b);
    }
    let end = rs[i..].iter().find(|r| r.kind == 0x7d7).unwrap().start;
    let mut b = bytes.clone();
    let mut record = vec![];
    put_record(&mut record, 0x7d3, &11u32.to_le_bytes());
    b.splice(end..end, record);
    let invalid = Document::from_bytes(&b, None).unwrap();
    assert_eq!(invalid.objects[9].kind, Kind::Unsupported(11));
    assert_eq!(invalid.encode().unwrap(), b);
}
