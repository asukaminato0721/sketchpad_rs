use super::*;
fn editor() -> Editor {
    let bytes =
        std::fs::read("tests/fixtures/sector_boundary59_center_quarter_native_saved.gsp").unwrap();
    Editor::new(Document::from_bytes(&bytes, None).unwrap())
}
#[test]
fn sector_interior_is_pickable_and_point_command_projects_to_radial_and_curved_parts() {
    let mut e = editor();
    let view = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.)),
        scale: 1.,
        center: Point { x: 400., y: 300. },
    };
    assert_eq!(
        e.hit(&view, view.screen(Point { x: 260., y: 260. })),
        Some(10)
    );
    for (query, expected) in [
        (Point { x: 300., y: 290. }, Point { x: 300., y: 280. }),
        (
            Point { x: 380., y: 140. },
            Point {
                x: 324.8528137423857,
                y: 195.1471862576143,
            },
        ),
    ] {
        let free = e.doc.add(Kind::Free(query));
        e.selection_order = vec![free, 10];
        e.doc.selected = [free, 10].into_iter().collect();
        e.point_on_object();
        let point = *e.doc.selected.iter().next().unwrap();
        assert!(
            matches!(e.doc.objects[point - 1].kind, Kind::PointOnSector(10, _)),
            "{}",
            e.status
        );
        assert!(e.doc.position(point).unwrap().distance(expected) < 1e-9);
        e.doc.verify_roundtrip().unwrap();
        e.doc.checkpoint();
        assert!(e.doc.move_point(point, Point { x: 230., y: 220. }));
        assert!(
            e.doc
                .position(point)
                .unwrap()
                .distance(Point { x: 240., y: 220. })
                < 1e-9
        );
        e.doc.undo();
        assert!(e.doc.position(point).unwrap().distance(expected) < 1e-9);
        e.doc.redo();
        e.doc.verify_roundtrip().unwrap();
    }
}
#[test]
fn sector_measurement_commands_keep_perimeter_area_and_arc_length_distinct() {
    let mut e = editor();
    for (kind, expected) in [
        (
            Kind::PolygonPerimeter(10),
            120. * (2. + std::f64::consts::FRAC_PI_2) / 37.79527559055118,
        ),
        (
            Kind::PolygonArea(10),
            3600. * std::f64::consts::PI / 37.79527559055118_f64.powi(2),
        ),
        (
            Kind::ArcLength(10),
            120. * std::f64::consts::FRAC_PI_2 / 37.79527559055118,
        ),
    ] {
        e.select_only(10);
        match kind {
            Kind::PolygonPerimeter(_) => e.measure_perimeter(),
            Kind::PolygonArea(_) => e.measure_circle(CircleMeasure::Area),
            Kind::ArcLength(_) => e.measure_arc_length(),
            _ => unreachable!(),
        }
        let id = *e.doc.selected.iter().next().unwrap();
        assert_eq!(e.doc.objects[id - 1].kind, kind, "{}", e.status);
        assert!((e.doc.numeric_value(id).unwrap() - expected).abs() < 1e-10);
        e.doc.verify_roundtrip().unwrap();
    }
    e.select_only(10);
    let deleted = e.doc.clear_selected().unwrap();
    assert!(deleted.contains(&14));
    assert!(deleted.contains(&25));
    e.doc.undo();
    assert!(e.doc.position(14).is_some());
}

#[test]
fn sector_mesh_fills_the_radial_triangle_for_minor_and_major_regions() {
    for (state, sweep) in [
        ("center_quarter", std::f64::consts::FRAC_PI_2),
        ("center_reverse", 3. * std::f64::consts::FRAC_PI_2),
    ] {
        let bytes = std::fs::read(format!(
            "tests/fixtures/sector_boundary59_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        let points: Vec<_> = d
            .filled_region_points(10)
            .unwrap()
            .into_iter()
            .map(|p| egui::pos2(p.x as f32, p.y as f32))
            .collect();
        let triangles = sketchpad_rs::render::polygon_triangles(&points);
        let area: f64 = triangles
            .iter()
            .map(|[a, b, c]| {
                let (a, b, c) = (points[*a], points[*b], points[*c]);
                f64::from(((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs()) / 2.
            })
            .sum();
        let expected = 0.5 * 120. * 120. * 192. * (sweep / 192.).sin();
        assert!(
            (area - expected).abs() < 0.05,
            "{state}: actual{area} vs{expected}; triangles{}",
            triangles.len()
        );
    }
}
