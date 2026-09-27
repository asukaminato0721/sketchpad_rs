use super::*;
const PATH_CURVE_UNIT: f64 = 37.79527559055118;
fn curve_point(x: f64) -> Point {
    Point {
        x: 250. + PATH_CURVE_UNIT * x,
        y: 300. - PATH_CURVE_UNIT * x * x,
    }
}
#[test]
fn point_on_curve_command_projects_and_measures_then_tracks_drag_undo_and_delete() {
    let bytes =
        std::fs::read("tests/fixtures/path_curve_points58_geo_15_half_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let projection = d.add(Kind::Free(curve_point(-0.875)));
    let mut e = Editor::new(d);
    e.selection_order = vec![projection, 15];
    e.doc.selected = [projection, 15].into_iter().collect();
    e.point_on_object();
    let point = *e.doc.selected.iter().next().unwrap();
    assert!(
        matches!(e.doc.objects[point-1].kind,Kind::PointOnCurve(15,t) if (t-0.25).abs()<1e-5),
        "{}",
        e.status
    );
    assert!(e.doc.position(point).unwrap().distance(curve_point(-0.875)) < 1e-5);
    e.select_only(point);
    e.measure_value_equation(false);
    let measurement = *e.doc.selected.iter().next().unwrap();
    assert_eq!(
        e.doc.objects[measurement - 1].kind,
        Kind::ValueEquation {
            kind: sketchpad_rs::value_equation_records::ValueEquationKind::PointValue,
            first: point,
            second: 15
        }
    );
    assert!((e.doc.numeric_value(measurement).unwrap() - 0.25).abs() < 1e-5);
    e.doc.checkpoint();
    assert!(e.doc.move_point(point, curve_point(-0.5)));
    assert!((e.doc.numeric_value(measurement).unwrap() - 0.5).abs() < 1e-5);
    e.doc.verify_roundtrip().unwrap();
    e.doc.undo();
    assert!((e.doc.numeric_value(measurement).unwrap() - 0.25).abs() < 1e-5);
    e.doc.redo();
    assert!((e.doc.numeric_value(measurement).unwrap() - 0.5).abs() < 1e-5);
    e.select_only(15);
    let deleted = e.doc.clear_selected().unwrap();
    assert!(deleted.contains(&point));
    assert!(deleted.contains(&measurement));
    e.doc.undo();
    assert!(e.doc.position(point).is_some());
}
