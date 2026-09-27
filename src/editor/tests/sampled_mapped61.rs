use super::*;
#[test]
fn sampled_path_point_command_reuses_live_projection_drag_and_dependent_deletion() {
    let bytes = std::fs::read("tests/fixtures/sampled_mapped60_segment_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let mut e = Editor::new(d);
    e.select_only(16);
    e.point_on_object();
    let point = *e.doc.selected.iter().next().unwrap();
    assert!(matches!(e.doc.objects[point - 1].kind, Kind::PointOnCurve(16,t) if (t - 0.5).abs() < 1e-6), "{}", e.status);
    assert!(e.doc.position(point).unwrap().distance(Point { x: 385., y: 270. }) < 1e-5);
    e.doc.verify_roundtrip().unwrap();
    let before = e.doc.encode().unwrap();
    e.doc.checkpoint();
    assert!(e.doc.move_point(point, Point { x: 466.25, y: 295. }));
    assert!(matches!(e.doc.objects[point - 1].kind, Kind::PointOnCurve(16,t) if (t - 0.75).abs() < 1e-6));
    e.doc.verify_roundtrip().unwrap();
    e.doc.undo();
    assert_eq!(e.doc.encode().unwrap(), before);
    e.select_only(16);
    assert!(e.doc.clear_selected().unwrap().contains(&point));
    e.doc.undo();
    assert!(e.doc.position(point).is_some());
}
