use super::*;

fn quarter(mode: LineMode) -> Editor {
    let mut d = Document::new();
    for (x, y) in [
        (200., 200.),
        (260., 200.),
        (200., 140.),
        (100., 170.),
        (300., 170.),
    ] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::CenterArc(1, 2, 3));
    d.add(match mode {
        LineMode::Line => Kind::Line(4, 5),
        LineMode::Ray => Kind::Ray(4, 5),
        LineMode::Segment => Kind::Segment(4, 5),
    });
    Editor::new(d)
}

#[test]
fn straight_arc_command_accepts_both_selection_orders_and_live_domain_edits() {
    for order in [[6, 7], [7, 6]] {
        let mut e = quarter(LineMode::Line);
        e.selection_order = order.to_vec();
        e.doc.selected = order.into_iter().collect();
        e.intersection();
        assert_eq!(e.selection_order, vec![8, 9]);
        assert_eq!(e.doc.objects[7].kind, Kind::LineCircleIntersection(7, 6, 0));
        assert_eq!(e.doc.objects[8].kind, Kind::LineCircleIntersection(7, 6, 1));
        let right = Point {
            x: 200. + 30. * 3f64.sqrt(),
            y: 170.,
        };
        for id in [8, 9] {
            assert!(e.doc.position(id).unwrap().distance(right) < 1e-8);
        }
        e.doc.checkpoint();
        assert!(e.doc.move_point(3, Point { x: 140., y: 200. }));
        assert!(e.doc.position(8).unwrap().x < 200.);
        assert!(e.doc.position(9).unwrap().x > 200.);
        e.doc.undo();
        assert_eq!(e.doc.position(8), e.doc.position(9));
        e.doc.redo();
        e.doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(&e.doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(8), e.doc.position(8));
        e.select_only(6);
        let removed = e.doc.clear_selected().unwrap();
        assert!(removed.contains(&8) && removed.contains(&9));
        e.doc.undo();
        assert!(e.doc.position(8).is_some());
    }
}

#[test]
fn straight_arc_command_retains_tangent_and_runtime_undefined_branches() {
    let mut e = quarter(LineMode::Segment);
    assert!(e.doc.move_point(4, Point { x: 100., y: 140. }));
    assert!(e.doc.move_point(5, Point { x: 300., y: 140. }));
    e.selection_order = vec![7, 6];
    e.doc.selected = [6, 7].into_iter().collect();
    e.intersection();
    for id in [8, 9] {
        assert_eq!(e.doc.position(id), Some(Point { x: 200., y: 140. }));
    }
    e.doc.checkpoint();
    assert!(e.doc.move_point(4, Point { x: 100., y: 120. }));
    assert!(e.doc.move_point(5, Point { x: 300., y: 120. }));
    for id in [8, 9] {
        assert_eq!(e.doc.position(id), None);
    }
    e.doc.verify_roundtrip().unwrap();
    e.doc.undo();
    for id in [8, 9] {
        assert!(e.doc.position(id).is_some());
    }
}
