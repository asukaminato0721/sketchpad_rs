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
fn arc_intersection_commands_match_native_branch_order_and_live_domains() {
    for order in [[6, 7], [7, 6]] {
        let mut e = quarter(LineMode::Line);
        assert!(e.doc.move_point(3, Point { x: 140., y: 200. }));
        e.selection_order = order.to_vec();
        e.doc.selected = order.into_iter().collect();
        e.intersection();
        assert_eq!(e.selection_order, vec![8, 9]);
        assert_eq!(e.doc.objects[7].kind, Kind::LineCircleIntersection(7, 6, 1));
        assert_eq!(e.doc.objects[8].kind, Kind::LineCircleIntersection(7, 6, 0));
        let right = Point {
            x: 200. + 30. * 3f64.sqrt(),
            y: 170.,
        };
        assert!(e.doc.position(8).unwrap().distance(right) < 1e-8);
        assert!(e.doc.position(9).unwrap().x < 200.);
        e.doc.checkpoint();
        assert!(e.doc.move_point(3, Point { x: 200., y: 140. }));
        assert_eq!(e.doc.position(8), e.doc.position(9));
        assert!(e.doc.position(8).unwrap().distance(right) < 1e-8);
        e.doc.undo();
        assert_ne!(e.doc.position(8), e.doc.position(9));
        e.doc.redo();
        assert_eq!(e.doc.position(8), e.doc.position(9));
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

    let circle_arc = || {
        let mut d = Document::new();
        for (x, y) in [
            (200., 200.),
            (260., 200.),
            (240., 260.),
            (180., 200.),
            (240., 140.),
        ] {
            d.add(Kind::Free(Point { x, y }));
        }
        d.add(Kind::Circle(1, 2));
        d.add(Kind::ThreePointArc(3, 4, 5));
        Editor::new(d)
    };
    for (order, first_is_lower) in [([6, 7], true), ([7, 6], false)] {
        let mut e = circle_arc();
        e.selection_order = order.to_vec();
        e.doc.selected = order.into_iter().collect();
        e.intersection();
        assert_eq!(
            e.doc.objects[7].kind,
            Kind::CircleCircleIntersection(order[0], order[1], 1)
        );
        assert_eq!(
            e.doc.objects[8].kind,
            Kind::CircleCircleIntersection(order[0], order[1], 0)
        );
        let first = e.doc.position(8).unwrap();
        let second = e.doc.position(9).unwrap();
        assert_eq!(first.y > 200., first_is_lower);
        assert_eq!(second.y < 200., first_is_lower);
        e.doc.verify_roundtrip().unwrap();
    }

    let mut single = circle_arc();
    assert!(single.doc.move_point(3, Point { x: 180., y: 200. }));
    assert!(single.doc.move_point(
        4,
        Point {
            x: 197.573_593_128_807_15,
            y: 157.573_593_128_807_15,
        }
    ));
    single.selection_order = vec![6, 7];
    single.doc.selected = [6, 7].into_iter().collect();
    single.intersection();
    assert_eq!(single.selection_order, vec![8]);
    assert_eq!(
        single.doc.objects[7].kind,
        Kind::CircleCircleIntersection(6, 7, 1)
    );

    let mut d = Document::new();
    for (x, y) in [
        (200., 140.),
        (260., 200.),
        (200., 260.),
        (240., 260.),
        (180., 200.),
        (240., 140.),
    ] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::ThreePointArc(1, 2, 3));
    d.add(Kind::ThreePointArc(4, 5, 6));
    let mut e = Editor::new(d);
    e.selection_order = vec![7, 8];
    e.doc.selected = [7, 8].into_iter().collect();
    e.intersection();
    assert!(e.doc.position(9).unwrap().y > 200.);
    assert!(e.doc.position(10).unwrap().y < 200.);
    e.doc.verify_roundtrip().unwrap();
}

#[test]
fn arc_intersection_command_deduplicates_tangency_and_tracks_undefined_results() {
    let mut e = quarter(LineMode::Segment);
    assert!(e.doc.move_point(4, Point { x: 100., y: 140. }));
    assert!(e.doc.move_point(5, Point { x: 300., y: 140. }));
    e.selection_order = vec![7, 6];
    e.doc.selected = [6, 7].into_iter().collect();
    e.intersection();
    assert_eq!(e.selection_order, vec![8]);
    assert_eq!(e.doc.position(8), Some(Point { x: 200., y: 140. }));
    e.doc.checkpoint();
    assert!(e.doc.move_point(4, Point { x: 100., y: 120. }));
    assert!(e.doc.move_point(5, Point { x: 300., y: 120. }));
    assert_eq!(e.doc.position(8), None);
    e.doc.verify_roundtrip().unwrap();
    e.doc.undo();
    assert!(e.doc.position(8).is_some());
}
