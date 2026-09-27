use super::*;

#[test]
fn function_point_command_projects_measures_and_tracks_drag_undo_and_deletion() {
    let bytes = std::fs::read("tests/fixtures/function_points60_quarter_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let query = d.add(Kind::Free(Point { x: 250. - 0.75 * 37.79527559055118, y: 300. - 0.75_f64.powi(2) * 37.79527559055118 }));
    let mut e = Editor::new(d);
    e.selection_order = vec![query, 8];
    e.doc.selected = [query, 8].into_iter().collect();
    e.point_on_object();
    let p = *e.doc.selected.iter().next().unwrap();
    assert!(matches!(e.doc.objects[p - 1].kind, Kind::PointOnCurve(8, t) if (t - 0.125).abs() < 1e-6), "{}", e.status);
    e.measure_value_equation(false);
    let value = *e.doc.selected.iter().next().unwrap();
    assert!((e.doc.numeric_value(value).unwrap() + 0.75).abs() < 1e-6);
    e.doc.checkpoint();
    assert!(e.doc.move_point(p, Point { x: 450.1023622047, y: 269.5511811024 }));
    assert_eq!(e.doc.numeric_value(value), Some(1.));
    e.doc.verify_roundtrip().unwrap();
    e.doc.undo();
    assert!((e.doc.numeric_value(value).unwrap() + 0.75).abs() < 1e-6);
    e.select_only(8);
    let deleted = e.doc.clear_selected().unwrap();
    assert!(deleted.contains(&p) && deleted.contains(&value));
    e.doc.undo();
    assert!(e.doc.position(p).is_some());
}

#[test]
fn circle_interior_command_creates_pickable_independent_fill_and_tracks_source() {
    let mut d = Document::new();
    let center = d.add(Kind::Free(Point { x: 250., y: 250. }));
    let radius = d.add(Kind::Free(Point { x: 350., y: 250. }));
    let circle = d.add(Kind::Circle(center, radius));
    d.objects[circle - 1].visible = false;
    let mut e = Editor::new(d);
    e.select_only(circle);
    e.circle_interior();
    let fill = *e.doc.selected.iter().next().unwrap();
    assert_eq!(e.doc.objects[fill - 1].kind, Kind::CircleInterior(circle), "{}", e.status);
    let view = Viewport { rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.)), scale: 1., center: Point { x: 400., y: 300. } };
    assert_eq!(e.hit(&view, view.screen(Point { x: 280., y: 280. })), Some(fill));
    let vertices: Vec<_> = e.doc.filled_region_points(fill).unwrap().into_iter().map(|p| egui::pos2(p.x as f32, p.y as f32)).collect();
    let triangles = sketchpad_rs::render::polygon_triangles(&vertices);
    assert_eq!(triangles.len(), vertices.len() - 2);
    e.doc.verify_roundtrip().unwrap();
    e.doc.checkpoint();
    assert!(e.doc.move_point(radius, Point { x: 250., y: 250. }));
    assert!(e.doc.filled_region_points(fill).is_none());
    e.doc.verify_roundtrip().unwrap();
    e.doc.undo();
    assert!(e.doc.filled_region_points(fill).is_some());
    e.select_only(circle);
    assert!(e.doc.clear_selected().unwrap().contains(&fill));
    e.doc.undo();
    assert!(e.doc.filled_region_points(fill).is_some());
}
