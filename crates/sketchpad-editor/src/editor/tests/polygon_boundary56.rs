use super::*;
fn boundary_editor(mode: LineMode) -> Editor {
    let mut d = Document::new();
    for (x, y) in [
        (150., 150.),
        (250., 150.),
        (250., 250.),
        (150., 250.),
        (180., 200.),
        (220., 200.),
    ] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::Polygon(vec![1, 2, 3, 4]));
    d.add(match mode {
        LineMode::Segment => Kind::Segment(5, 6),
        LineMode::Ray => Kind::Ray(5, 6),
        LineMode::Line => Kind::Line(5, 6),
    });
    Editor::new(d)
}
#[test]
fn polygon_boundary_command_preserves_source_identity_both_selection_orders_and_modes() {
    for mode in [LineMode::Segment, LineMode::Ray, LineMode::Line] {
        for order in [[7, 8], [8, 7]] {
            let mut e = boundary_editor(mode);
            e.selection_order = order.to_vec();
            e.doc.selected = order.into_iter().collect();
            e.intersection();
            assert_eq!(e.selection_order, vec![9]);
            assert_eq!(e.doc.objects[8].kind, Kind::BoundaryIntersection(8, 7));
            assert_eq!(e.doc.position(9), Some(Point { x: 250., y: 200. }));
            e.doc.verify_roundtrip().unwrap();
            e.doc.checkpoint();
            assert!(e.doc.move_point(6, Point { x: 100., y: 200. }));
            assert_eq!(e.doc.position(9), Some(Point { x: 150., y: 200. }));
            e.doc.undo();
            assert_eq!(e.doc.position(9), Some(Point { x: 250., y: 200. }));
            e.doc.redo();
            e.doc.verify_roundtrip().unwrap();
            e.select_only(7);
            let deleted = e.doc.clear_selected().unwrap();
            assert!(deleted.contains(&9));
            e.doc.undo();
            assert_eq!(e.doc.position(9), Some(Point { x: 150., y: 200. }));
        }
    }
}
#[test]
fn polygon_boundary_command_live_origin_contact_can_become_undefined_and_recover() {
    let mut e = boundary_editor(LineMode::Ray);
    assert!(e.doc.move_point(5, Point { x: 150., y: 200. }));
    assert!(e.doc.move_point(6, Point { x: 300., y: 200. }));
    e.selection_order = vec![7, 8];
    e.doc.selected = [7, 8].into_iter().collect();
    e.intersection();
    assert_eq!(e.doc.position(9), Some(Point { x: 250., y: 200. }));
    e.doc.checkpoint();
    assert!(e.doc.move_point(6, Point { x: 100., y: 200. }));
    assert_eq!(e.doc.position(9), None);
    e.doc.verify_roundtrip().unwrap();
    let saved = e.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(9), None);
    e.doc.undo();
    assert_eq!(e.doc.position(9), Some(Point { x: 250., y: 200. }));
}
