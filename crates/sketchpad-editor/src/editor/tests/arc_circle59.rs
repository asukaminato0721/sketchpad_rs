use super::*;
#[test]
fn arc_on_circle_command_preserves_general_endpoint_identity_and_live_radius() {
    let mut d = Document::new();
    let center = d.add(Kind::Free(Point { x: 250., y: 260. }));
    let radius = d.add(Kind::Free(Point { x: 330., y: 260. }));
    let circle = d.add(Kind::Circle(center, radius));
    let start = d.add(Kind::Free(Point { x: 410., y: 260. }));
    let end = d.add(Kind::Free(Point { x: 250., y: 220. }));
    let mut e = Editor::new(d);
    e.selection_order = vec![circle, start, end];
    e.doc.selected = [circle, start, end].into_iter().collect();
    e.arc_on_circle();
    let arc = *e.doc.selected.iter().next().unwrap();
    assert_eq!(
        e.doc.objects[arc - 1].kind,
        Kind::ArcOnCircle(circle, start, end),
        "{}",
        e.status
    );
    assert_eq!(
        e.doc.objects[start - 1].kind,
        Kind::Free(Point { x: 410., y: 260. })
    );
    let point = e.doc.add(Kind::PointOnArc(arc, 0.5));
    let expected = Point {
        x: 250. + 160. / 2f64.sqrt(),
        y: 260. - 160. / 2f64.sqrt(),
    };
    assert!(e.doc.position(point).unwrap().distance(expected) < 1e-8);
    e.doc.checkpoint();
    assert!(e.doc.move_point(end, Point { x: 250., y: 260. }));
    assert!(e.doc.position(point).is_none());
    let bytes = e.doc.encode().unwrap();
    assert!(
        Document::from_bytes(&bytes, None)
            .unwrap()
            .position(point)
            .is_none()
    );
    e.doc.undo();
    assert!(e.doc.position(point).unwrap().distance(expected) < 1e-8);
    let count = e.doc.objects.len();
    e.selection_order = vec![start, circle, end];
    e.arc_on_circle();
    assert_eq!(e.doc.objects.len(), count);
    e.select_only(circle);
    e.clear_selected();
    assert!(e.doc.objects[arc - 1].deleted);
    assert!(e.doc.objects[point - 1].deleted);
    e.doc.undo();
    assert!(e.doc.position(point).is_some());
}
