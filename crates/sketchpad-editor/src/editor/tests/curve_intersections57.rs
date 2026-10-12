use super::*;
const CURVE_UNIT: f64 = 37.79527559055118;
fn editor_curve(expression: &str, y: f64) -> Editor {
    let mut d = Document::new();
    let plot = d
        .plot_new_function(
            SimpleFunction::parse(expression).unwrap(),
            Point { x: 250., y: 300. },
        )
        .unwrap();
    d.set_plot_range(plot, -3., 3., 301).unwrap();
    d.add(Kind::Free(Point {
        x: 250. - 3. * CURVE_UNIT,
        y: 300. - y * CURVE_UNIT,
    }));
    d.add(Kind::Free(Point {
        x: 250. + 3. * CURVE_UNIT,
        y: 300. - y * CURVE_UNIT,
    }));
    d.add(Kind::Line(9, 10));
    Editor::new(d)
}
#[test]
fn curve_intersection_command_normalizes_roles_and_creates_every_ranked_crossing() {
    for order in [[8, 11], [11, 8]] {
        let mut e = editor_curve("x^3-x", 0.);
        e.selection_order = order.to_vec();
        e.doc.selected = order.into_iter().collect();
        e.intersection();
        assert_eq!(e.selection_order, vec![12, 13, 14]);
        for (rank, (id, x)) in [12, 13, 14].into_iter().zip([-1., 0., 1.]).enumerate() {
            assert_eq!(
                e.doc.objects()[id - 1].kind,
                Kind::CurveIntersection(11, 8, rank as u32)
            );
            assert!(
                e.doc.position(id).unwrap().distance(Point {
                    x: 250. + x * CURVE_UNIT,
                    y: 300.
                }) < 1e-6
            );
        }
        e.doc.verify_roundtrip().unwrap();
        e.select_only(8);
        let deleted = e.doc.clear_selected().unwrap();
        for id in [12, 13, 14] {
            assert!(deleted.contains(&id));
        }
        e.doc.undo();
        assert!(e.doc.position(13).is_some());
    }
}
#[test]
fn curve_intersection_command_live_edits_suppress_touches_and_restore_on_undo() {
    let mut e = editor_curve("x^2", 1.);
    e.selection_order = vec![11, 8];
    e.doc.selected = [11, 8].into_iter().collect();
    e.intersection();
    assert_eq!(e.selection_order, vec![12, 13]);
    e.doc.checkpoint();
    for id in [9, 10] {
        let p = e.doc.position(id).unwrap();
        assert!(e.doc.move_point(id, Point { x: p.x, y: 300. }));
    }
    for id in [12, 13] {
        assert!(e.doc.position(id).is_none());
    }
    e.doc.verify_roundtrip().unwrap();
    let saved = e.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.position(12).is_none());
    e.doc.undo();
    assert!(e.doc.position(12).is_some());
    e.doc.redo();
    assert!(e.doc.position(12).is_none());
}
