use super::*;
use crate::document::MovementValue;

#[test]
fn movement_controls_project_paths_and_preserve_dependencies() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 200., y: 100. }));
    let c = doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let segment = doc.add(Kind::Segment(a, b));
    let ray = doc.add(Kind::Ray(a, b));
    let line = doc.add(Kind::Line(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let arc = doc.add(Kind::CenterArc(a, b, c));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let controls = [
        doc.add(Kind::PointOnSegment(segment, 0.5)),
        doc.add(Kind::PointOnRay(ray, 0.5)),
        doc.add(Kind::PointOnLine(line, 0.5)),
        doc.add(Kind::PointOnCircle(circle, 1., 0.)),
        doc.add(Kind::PointOnArc(arc, 0.5)),
        doc.add(Kind::PointOnPolygon(polygon, 0, 0.5)),
    ];
    let destination = doc.add(Kind::Free(Point { x: 50., y: 150. }));
    let value = doc.movement_target_value(destination).unwrap();
    let before: Vec<_> = controls
        .iter()
        .map(|&id| doc.objects[id - 1].kind.clone())
        .collect();
    doc.checkpoint();
    for &id in &controls {
        assert!(doc.movement_pair_supported(id, destination));
        assert!(doc.set_movement_value(id, value));
    }
    assert_eq!(doc.position(controls[0]), Some(Point { x: 100., y: 100. }));
    assert_eq!(doc.position(controls[1]), Some(Point { x: 100., y: 100. }));
    assert_eq!(doc.position(controls[2]), Some(Point { x: 50., y: 100. }));
    assert!(
        (doc.position(controls[3])
            .unwrap()
            .distance(Point { x: 100., y: 100. })
            - 100.)
            .abs()
            < 1e-10
    );
    for (id, kind) in controls.iter().zip(&before) {
        assert_eq!(doc.objects[id - 1].kind.dependencies(), kind.dependencies());
    }
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    for &id in &controls {
        assert_eq!(reopened.position(id), doc.position(id));
    }
    doc.undo();
    for (id, kind) in controls.iter().zip(&before) {
        assert_eq!(&doc.objects[id - 1].kind, kind);
    }
    doc.redo();
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn movement_numeric_values_use_live_targets_and_one_action_checkpoint() {
    let mut doc = Document::new();
    let parameter = doc.add_named_parameter("a", 1., (20, 20)).unwrap();
    let other = doc.add_named_parameter("b", 7.12345, (20, 50)).unwrap();
    let target = doc
        .add_parameter_calculation(other, "b*2", (20, 80))
        .unwrap();
    assert!(doc.movement_pair_supported(parameter, target));
    let destination = doc.movement_target_value(target).unwrap();
    assert_eq!(
        destination,
        MovementValue::Numeric {
            value: 14.2469,
            unit: NumericUnit::Scalar
        }
    );
    assert!(!doc.set_movement_value(parameter, MovementValue::Point(Point { x: 0., y: 0. })));
    doc.checkpoint();
    let start = doc.movement_source_value(parameter).unwrap();
    assert!(doc.set_movement_value(parameter, start.interpolate(destination, 0.5).unwrap()));
    assert!(doc.set_movement_value(parameter, destination));
    assert_eq!(doc.numeric_value(parameter), Some(14.2469));
    doc.undo();
    assert_eq!(doc.numeric_value(parameter), Some(1.));
    doc.redo();
    assert_eq!(doc.numeric_value(parameter), Some(14.2469));
    let before = doc.encode().unwrap();
    assert!(!doc.set_movement_value(
        parameter,
        MovementValue::Numeric {
            value: f64::NAN,
            unit: NumericUnit::Scalar
        }
    ));
    assert_eq!(doc.encode().unwrap(), before);
    assert!(!doc.can_move_movement_source(target));
    let length = doc
        .add_calculation_expression("7.125 cm", (20, 110))
        .unwrap();
    assert!(doc.movement_pair_supported(parameter, length));
    let start = doc.movement_source_value(parameter).unwrap();
    let destination = doc.movement_target_value(length).unwrap();
    let endpoint = start.interpolate(destination, 1.).unwrap();
    assert_eq!(
        endpoint,
        MovementValue::Numeric {
            value: 7.125,
            unit: NumericUnit::Scalar
        }
    );
    assert!(doc.set_movement_value(parameter, destination));
    assert_eq!(doc.numeric_value(parameter), Some(7.125));
    assert_eq!(
        doc.objects[parameter - 1].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    let point = doc.add(Kind::Free(Point { x: 10., y: 10. }));
    assert!(!doc.movement_pair_supported(parameter, point));
    assert!(!doc.movement_pair_supported(point, parameter));
    assert!(!doc.set_movement_value(point, destination));
}

#[test]
fn unavailable_parameter_movement_preserves_readable_targets() {
    let mut doc = Document::new();
    let source = doc.add_named_parameter("a", 1., (20, 20)).unwrap();
    let target = doc.add_named_parameter("b", 3., (20, 50)).unwrap();
    let value = doc.movement_target_value(target).unwrap();
    doc.partial_view = true;
    assert!(!doc.can_move_movement_source(source));
    assert_eq!(doc.movement_source_value(source), None);
    assert_eq!(
        doc.movement_target_value(source),
        Some(MovementValue::Numeric {
            value: 1.,
            unit: NumericUnit::Scalar,
        })
    );
    assert!(!doc.movement_pair_supported(source, target));
    assert!(!doc.set_movement_value(source, value));
    assert_eq!(doc.numeric_value(source), Some(1.));
    doc.partial_view = false;
    doc.safe_structure = false;
    assert!(!doc.can_move_movement_source(source));
    assert!(!doc.set_movement_value(source, value));
    doc.safe_structure = true;
    assert!(doc.can_move_movement_source(source));
    assert!(doc.set_movement_value(source, value));
    assert_eq!(doc.numeric_value(source), Some(3.));
}

#[test]
fn horizontal_unit_movement_matches_native_square_grid_projection() {
    let mut doc = Document::new();
    doc.add_default_coordinate_system(Point { x: 200., y: 300. })
        .unwrap();
    let destination = doc.add(Kind::Free(Point { x: 275., y: 200. }));
    let vertical = doc.objects[3].kind.clone();
    assert!(doc.movement_pair_supported(2, destination));
    assert!(!doc.can_move_movement_source(4));
    assert!(doc.set_movement_value(2, doc.movement_target_value(destination).unwrap()));
    assert_eq!(doc.objects[1].kind, Kind::GraphHorizontalUnit(1, 75.));
    assert_eq!(doc.position(2), Some(Point { x: 275., y: 300. }));
    assert_eq!(doc.objects[3].kind, vertical);
    assert!(!doc.set_movement_value(2, MovementValue::Point(Point { x: 100., y: 200. })));
    assert_eq!(doc.objects[1].kind, Kind::GraphHorizontalUnit(1, 75.));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[1].kind, doc.objects[1].kind);
    assert_eq!(reopened.objects[3].kind, vertical);
    assert_eq!(reopened.grid_form[0], GridForm::Square);
}

#[test]
fn projected_movement_controls_match_native_clicked_documents() {
    for (name, source, destination) in [
        ("segment", 4, 5),
        ("circle", 4, 5),
        ("arc", 5, 6),
        ("graph_x", 2, 7),
        ("api_graph_rect", 2, 7),
        ("api_graph_rect", 4, 8),
    ] {
        let input = fs::read(format!("tests/fixtures/movement46_{name}_input.gsp")).unwrap();
        let clicked = fs::read(format!(
            "tests/fixtures/movement46_{name}_clicked_native_verified.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&input, None).unwrap();
        let native = Document::from_bytes(&clicked, None).unwrap();
        let value = doc.movement_target_value(destination).unwrap();
        assert!(doc.set_movement_value(source, value), "{name}");
        assert!(
            doc.position(source)
                .unwrap()
                .distance(native.position(source).unwrap())
                < 1e-8,
            "{name}"
        );
        let actual = &doc.objects[source - 1].kind;
        let expected = &native.objects[source - 1].kind;
        assert_eq!(
            std::mem::discriminant(actual),
            std::mem::discriminant(expected),
            "{name}"
        );
        assert_eq!(actual.dependencies(), expected.dependencies(), "{name}");
        match (actual, expected) {
            (Kind::PointOnSegment(_, a), Kind::PointOnSegment(_, b))
            | (Kind::PointOnArc(_, a), Kind::PointOnArc(_, b))
            | (Kind::GraphHorizontalUnit(_, a), Kind::GraphHorizontalUnit(_, b))
            | (Kind::GraphVerticalUnit(_, a), Kind::GraphVerticalUnit(_, b)) => {
                assert!((a - b).abs() < 1e-12, "{name}");
            }
            (Kind::PointOnCircle(_, ax, ay), Kind::PointOnCircle(_, bx, by)) => {
                assert!((ax - bx).abs() < 1e-12 && (ay - by).abs() < 1e-12, "{name}");
            }
            _ => panic!("Unexpected movement control {name}: {actual:?}"),
        }
        let bytes = doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(reopened.objects[source - 1].kind, *actual, "{name}");
        assert_eq!(reopened.encode().unwrap(), bytes, "{name}");
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn stored_parameter_movement_on_inactive_page_remains_valid() {
    let mut doc = Document::new();
    let source = doc.add_named_parameter("a", 1., (20, 20)).unwrap();
    let target = doc.add_named_parameter("b", 3., (20, 50)).unwrap();
    let bytes = doc.encode().unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    doc.add_blank_page().unwrap();
    assert_eq!(doc.active_page, 1);
    assert!(doc.movement_pair_valid(source, target));
    assert!(!doc.movement_pair_supported(source, target));
    assert_eq!(doc.movement_source_value(source), None);
    assert_eq!(
        doc.movement_target_value(source),
        Some(MovementValue::Numeric {
            value: 1.,
            unit: NumericUnit::Scalar
        })
    );
}

#[test]
fn numeric_movement_uses_native_magnitudes_across_units() {
    for name in ["parameter", "parameter_mismatch"] {
        let input = fs::read(format!("tests/fixtures/movement46_{name}_input.gsp")).unwrap();
        let clicked = fs::read(format!(
            "tests/fixtures/movement46_{name}_clicked_native_verified.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&input, None).unwrap();
        let native = Document::from_bytes(&clicked, None).unwrap();
        // Native numeric controls capture destination quantities before setting
        // parameters, as established by the a→b,b→constant control.
        let first = doc.movement_target_value(2).unwrap();
        let second = doc.movement_target_value(3).unwrap();
        assert!(doc.set_movement_value(1, first));
        assert!(doc.set_movement_value(2, second));
        for id in [1, 2] {
            assert_eq!(
                doc.numeric_value(id),
                native.numeric_value(id),
                "{name} source{id}"
            );
            assert_eq!(
                doc.objects[id - 1].kind.numeric_unit(),
                Some(NumericUnit::Scalar)
            );
        }
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_square_vertical_unit_follows_live_horizontal_spacing() {
    let original = include_bytes!(
        "../../../tests/fixtures/movement46_api_graph_square_y_probe_edited_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(original, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects[1].kind, Kind::GraphHorizontalUnit(1, 75.));
    let cached_vertical = doc.objects[3].kind.clone();
    assert_eq!(
        cached_vertical,
        Kind::GraphVerticalUnit(2, 37.79527559055118)
    );
    // Native's visible translated point establishes the evaluated dependent
    // Y coordinate; the stored vertical spacing remains its original cache.
    assert_eq!(doc.position(4), Some(Point { x: 200., y: 225. }));
    assert_eq!(doc.position(11), Some(Point { x: 300., y: 225. }));
    doc.checkpoint();
    assert!(doc.set_movement_value(2, MovementValue::Point(Point { x: 300., y: 400. })));
    assert_eq!(doc.objects[1].kind, Kind::GraphHorizontalUnit(1, 100.));
    assert_eq!(doc.objects[3].kind, cached_vertical);
    assert_eq!(doc.position(4), Some(Point { x: 200., y: 200. }));
    assert_eq!(doc.position(11), Some(Point { x: 300., y: 200. }));
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(reopened.objects[3].kind, cached_vertical);
    assert_eq!(reopened.position(11), doc.position(11));
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.position(11), Some(Point { x: 300., y: 225. }));
    assert_eq!(doc.encode().unwrap(), original);
    doc.redo();
    assert_eq!(doc.position(11), Some(Point { x: 300., y: 200. }));
    assert_eq!(doc.encode().unwrap(), bytes);
}
