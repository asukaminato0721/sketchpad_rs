use super::*;

#[test]
fn native_polygon_boundary_retains_local_edge_fraction_and_linked_value() {
    let bytes = include_bytes!("../../../tests/fixtures/polygon_boundary_original.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    let t = 0.4450450665338869;
    assert_eq!(doc.objects[5].kind, Kind::PointOnPolygon(5, 2, t));
    let point = doc.position(6).unwrap();
    assert!((point.x - (601. + 14. * t)).abs() < 1e-9);
    assert!((point.y - (398. - 96. * t)).abs() < 1e-9);
    assert!((doc.polygon_boundary_parameter(5, 2, t).unwrap() - 0.58).abs() < 0.005);
    assert_eq!(
        doc.numeric_value(7),
        doc.polygon_boundary_parameter(5, 2, t)
    );
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(doc.move_point(6, Point { x: 443.5, y: 365. }));
    assert_eq!(doc.objects[5].kind, Kind::PointOnPolygon(5, 0, 0.5));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(6), Some(Point { x: 443.5, y: 365. }));
    assert_eq!(reopened.objects[5].kind, doc.objects[5].kind);
    assert_eq!(
        reopened.numeric_value(7),
        reopened.polygon_boundary_parameter(5, 0, 0.5)
    );
}

#[test]
fn polygon_boundary_creation_drag_clipboard_and_vertex_changes_round_trip() {
    let mut doc = Document::new();
    let vertices = [(0., 0.), (120., 0.), (120., 60.), (0., 60.)]
        .into_iter()
        .map(|(x, y)| doc.add(Kind::Free(Point { x, y })))
        .collect::<Vec<_>>();
    let polygon = doc.add(Kind::Polygon(vertices.clone()));
    let point = doc.add(
        doc.constrained_point(polygon, Point { x: 130., y: 30. })
            .unwrap(),
    );
    assert_eq!(
        doc.objects[point - 1].kind,
        Kind::PointOnPolygon(polygon, 1, 0.5)
    );
    assert_eq!(doc.position(point), Some(Point { x: 120., y: 30. }));
    assert!((doc.polygon_boundary_parameter(polygon, 1, 0.5).unwrap() - 150. / 360.).abs() < 1e-12);
    assert!(doc.move_point(vertices[2], Point { x: 180., y: 60. }));
    assert_eq!(doc.position(point), Some(Point { x: 150., y: 30. }));
    assert!(doc.move_point(point, Point { x: 10., y: 70. }));
    assert!(
        matches!(doc.objects[point-1].kind,Kind::PointOnPolygon(_,2,t) if (t-170./180.).abs()<1e-12)
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(point), doc.position(point));
    doc.selected.insert(point);
    let clipboard = doc.copy_selected().unwrap();
    let mut pasted = Document::new();
    let ids = pasted.paste(&clipboard).unwrap();
    assert_eq!(ids.len(), 6);
    assert!(matches!(
        pasted.objects.last().unwrap().kind,
        Kind::PointOnPolygon(5, 2, _)
    ));
    assert!(
        !Document::from_bytes(&pasted.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    doc.selected.clear();
    doc.selected.insert(polygon);
    assert!(doc.clear_selected().unwrap().contains(&point));
}

#[test]
fn polygon_boundary_perimeter_wraps_and_skips_collapsed_edges() {
    let mut doc = Document::new();
    let vertices = [(0., 0.), (0., 0.), (100., 0.), (100., 100.)]
        .into_iter()
        .map(|(x, y)| doc.add(Kind::Free(Point { x, y })))
        .collect::<Vec<_>>();
    let polygon = doc.add(Kind::Polygon(vertices));
    assert_eq!(
        doc.point_at_polygon_parameter(polygon, 0.),
        Some(Point { x: 0., y: 0. })
    );
    assert_eq!(
        doc.point_at_polygon_parameter(polygon, 1.),
        doc.point_at_polygon_parameter(polygon, 0.)
    );
    assert_eq!(
        doc.point_at_polygon_parameter(polygon, -0.2),
        doc.point_at_polygon_parameter(polygon, 0.8)
    );
    assert!(
        doc.constrained_point(polygon, Point { x: 50., y: -10. })
            .is_some()
    );
}

#[test]
fn native_polygon_drag_changes_edge_and_updates_perimeter_value() {
    let bytes = include_bytes!("../../../tests/fixtures/polygon_boundary_dragged_native.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.objects[5].kind,
        Kind::PointOnPolygon(5, 0, 0.5116564522216039)
    );
    let p = doc.position(6).unwrap();
    assert!((p.x - 443.5116564522216).abs() < 1e-9);
    assert!((p.y - (310. + 110. * 0.5116564522216039)).abs() < 1e-9);
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn polygon_boundary_creation_and_drag_work_on_supported_multi_page_document() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_polygon_port.gsp"),
        None,
    )
    .unwrap();
    let polygon = doc
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::Polygon(_)))
        .unwrap()
        .id;
    doc.active_page = doc.objects[polygon - 1].page;
    let vertex = doc.polygon_points(polygon).unwrap()[0];
    let point = doc.add(doc.constrained_point(polygon, vertex).unwrap());
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(
        matches!(reopened.objects[point-1].kind,Kind::PointOnPolygon(host,_,_) if host==polygon)
    );
    assert!(reopened.move_point(
        point,
        Point {
            x: vertex.x + 20.,
            y: vertex.y + 40.
        }
    ));
    let moved = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(moved.position(point), reopened.position(point));
}

#[test]
fn polygon_boundary_animation_button_uses_closed_perimeter_direction() {
    let mut doc = Document::new();
    let vertices = [(0., 0.), (120., 0.), (120., 60.), (0., 60.)]
        .into_iter()
        .map(|(x, y)| doc.add(Kind::Free(Point { x, y })))
        .collect::<Vec<_>>();
    let polygon = doc.add(Kind::Polygon(vertices));
    let point = doc.add(Kind::PointOnPolygon(polygon, 1, 0.5));
    assert_eq!(
        doc.animation_button_default_settings(point)
            .unwrap()
            .direction,
        crate::action_records::AnimationDirection::Forward
    );
    let button = doc.add_animation_button(point).unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(reopened.objects[button-1].kind,Kind::AnimationButton{ref targets,..} if targets==&[point])
    );
}

#[test]
fn native_verified_generated_polygon_point_and_linked_value_round_trip() {
    let bytes = include_bytes!("../../../tests/fixtures/polygon_boundary_generated_verified.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.position(6), Some(Point { x: 150., y: 220. }));
    assert!((doc.numeric_value(7).unwrap() - 430. / 600.).abs() < 1e-12);
}

#[test]
fn native_calculated_polygon_path_uses_live_wrapped_perimeter_values() {
    let bytes = include_bytes!("../../../tests/fixtures/numeric_polygon_path_quarter_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects[8].kind, Kind::PathValuePoint(8, 5));
    let label = doc.objects[8].label.clone();
    assert!(label.is_some());
    assert_eq!(doc.position(9), Some(Point { x: 250., y: 100. }));
    assert_eq!(doc.encode().unwrap(), bytes);
    // Native marker segments confirm normalized perimeter traversal and wrapping
    // for values outside the first traversal.
    for (value, point) in [
        (-0.25, Point { x: 150., y: 200. }),
        (0., Point { x: 100., y: 100. }),
        (0.5, Point { x: 300., y: 200. }),
        (1., Point { x: 100., y: 100. }),
        (1.25, Point { x: 250., y: 100. }),
    ] {
        doc.set_parameter_value(8, value).unwrap();
        assert_eq!(doc.position(9), Some(point));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(9), Some(point));
        assert_eq!(reopened.objects[8].label, label);
    }
    doc.set_parameter_value(8, 0.25).unwrap();
    let before = doc.position(9).unwrap();
    assert!(doc.move_point(2, Point { x: 400., y: 100. }));
    assert_ne!(doc.position(9), Some(before));
    assert_eq!(doc.position(9), doc.point_at_polygon_parameter(5, 0.25));
    let calculation = doc.add_parameter_calculation(8, "p/2", (30, 20)).unwrap();
    let created = doc.plot_parameter_on_path(calculation, Some(5)).unwrap();
    assert_eq!(doc.position(created), doc.point_at_polygon_parameter(5, 0.125));
    assert!(doc.plot_literal_on_path(0.25, Some(5)).is_err());
    doc.selected.insert(9);
    doc.selected.insert(created);
    let clipboard = doc.copy_selected().unwrap();
    let mut pasted = Document::new();
    let ids = pasted.paste(&clipboard).unwrap();
    let copied = *ids.last().unwrap();
    assert!(matches!(pasted.objects[copied - 1].kind, Kind::PathValuePoint(..)));
    let original = doc.position(created).unwrap();
    assert_eq!(pasted.position(copied), Some(Point { x: original.x + 20., y: original.y + 20. }));
    let reopened = Document::from_bytes(&pasted.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(copied), pasted.position(copied));
    let original_state = doc.objects.clone();
    assert!(doc.set_parameter_value(8, f64::NAN).is_err());
    assert_eq!(doc.objects, original_state);
    doc.selected.clear();
    doc.selected.insert(8);
    doc.clear_selected().unwrap();
    assert!(doc.objects[8].deleted);
    assert!(doc.objects[created - 1].deleted);
}

#[test]
fn original_ningde_calculated_point_retains_polygon_and_scalar_dependencies() {
    let bytes = include_bytes!("../../../tests/fixtures/numeric_polygon_path_ningde_original.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(doc.objects[16].kind, Kind::PathValuePoint(16, 15));
    assert!(matches!(&doc.objects[14].kind, Kind::Polygon(vertices) if vertices == &[11,10,9,10]));
    assert_eq!(doc.objects[15].kind.numeric_unit(), Some(crate::table_records::NumericUnit::Scalar));
    let value = doc.numeric_value(16).unwrap();
    assert_eq!(doc.position(17), doc.point_at_polygon_parameter(15, value));
    assert!(doc.position(17).is_some());
    assert!(doc.objects[16].label.is_some());
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn native_verified_authored_polygon_calculated_paths_round_trip() {
    for (bytes, parameter, point) in [
        (include_bytes!("../../../tests/fixtures/numeric_polygon_path_generated_verified.gsp").as_slice(),
            0.5, Point { x: 250., y: 100. }),
        (include_bytes!("../../../tests/fixtures/numeric_polygon_path_negative_verified.gsp").as_slice(),
            -0.5, Point { x: 150., y: 200. }),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[7].kind, Kind::PathValuePoint(7, 5));
        assert_eq!(doc.numeric_value(6), Some(parameter));
        assert_eq!(doc.numeric_value(7), Some(parameter / 2.));
        assert_eq!(doc.position(8), Some(point));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
