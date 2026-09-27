use super::*;

fn close(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(
            (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
            "{a:?} != {b:?}"
        ),
        (None, None) => {}
        pair => panic!("Point definedness differs: {pair:?}"),
    }
}

#[test]
fn native_value_and_literal_points_use_affine_transformed_straights_and_undefined_negative_rays() {
    for (bytes, ray, scaled) in [
        (
            include_bytes!("../../../tests/fixtures/rotated_ray_path_values_native_verified.gsp")
                .as_slice(),
            true,
            false,
        ),
        (
            include_bytes!("../../../tests/fixtures/rotated_line_path_values_native_verified.gsp")
                .as_slice(),
            false,
            false,
        ),
        (
            include_bytes!("../../../tests/fixtures/scaled_ray_path_values_native_verified.gsp")
                .as_slice(),
            true,
            true,
        ),
        (
            include_bytes!("../../../tests/fixtures/scaled_line_path_values_native_verified.gsp")
                .as_slice(),
            false,
            true,
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects.len(), 23);
        let (a, b, _) = doc.line_points(5).unwrap();
        close(
            Some(a),
            Some(if scaled {
                Point { x: 210., y: 230. }
            } else {
                Point { x: 170., y: 110. }
            }),
        );
        for (i, value) in [0.25, -0.25, 1.25, 0.25, -0.25, 1.25]
            .into_iter()
            .enumerate()
        {
            let id = 7 + 3 * i;
            assert_eq!(
                doc.objects[id - 1].kind,
                if i < 3 {
                    Kind::PathValuePoint(id - 1, 5)
                } else {
                    Kind::PathLiteralPoint(5, value)
                }
            );
            let expected = if ray && value < 0. {
                None
            } else {
                Some(Point {
                    x: a.x + (b.x - a.x) * value,
                    y: a.y + (b.y - a.y) * value,
                })
            };
            close(doc.position(id), expected);
        }
        doc.verify_roundtrip().unwrap();
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.set_parameter_value(9, 0.25).unwrap();
        close(doc.position(10), doc.position(7));
        doc.undo();
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.redo();
        close(doc.position(10), doc.position(7));
        doc.undo();
        assert!(doc.move_point(3, Point { x: 260., y: 200. }));
        let edited = doc.encode().unwrap();
        let reopened = Document::from_bytes(&edited, None).unwrap();
        for id in [7, 10, 13, 16, 19, 22] {
            close(reopened.position(id), doc.position(id));
        }
        doc.selected = (1..=23).collect();
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        assert_eq!(pasted.len(), 23);
        for id in [7, 10, 13, 16, 19, 22] {
            close(
                doc.position(id + 23),
                doc.position(id).map(|p| Point {
                    x: p.x + 20.,
                    y: p.y + 20.,
                }),
            );
        }
        doc.verify_roundtrip().unwrap();
        doc.selected = [28].into_iter().collect();
        doc.clear_selected().unwrap();
        for id in [7, 10, 13, 16, 19, 22] {
            assert!(doc.objects[id + 22].deleted);
        }
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn transformed_straight_plot_creation_preserves_negative_values_and_rejects_invalid_inputs_atomically()
 {
    for bytes in [
        include_bytes!("../../../tests/fixtures/fixed_rotation_ray_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/fixed_rotation_line_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/fixed_dilation_ray_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/fixed_dilation_line_original.gsp").as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        let source = doc.add_named_parameter("a", -0.25, (10, 10)).unwrap();
        let before = doc.encode().unwrap();
        let count = doc.objects.len();
        let value = doc.plot_parameter_on_path(source, Some(5)).unwrap();
        let literal = doc.plot_literal_on_path(-0.25, Some(5)).unwrap();
        close(doc.position(value), doc.position(literal));
        doc.verify_roundtrip().unwrap();
        doc.undo();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), before);
        doc.redo();
        doc.redo();
        let edited = doc.encode().unwrap();
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(doc.plot_literal_on_path(bad, Some(5)).is_err());
            assert_eq!(doc.encode().unwrap(), edited);
        }
        assert!(doc.plot_literal_on_path(0.25, Some(0)).is_err());
        assert!(doc.plot_literal_on_path(0.25, Some(count + 99)).is_err());
        assert_eq!(doc.encode().unwrap(), edited);
    }
}

#[test]
fn authored_transformed_plot_graph_matches_native_created_edited_and_copied_files() {
    let mut doc = Document::new();
    for (x, y) in [(130., 150.), (170., 190.), (210., 190.)] {
        doc.add(Kind::Free(Point { x, y }));
    }
    for kind in [
        Kind::Ray(2, 3),
        Kind::Line(2, 3),
        Kind::RotatedRay(4, 1, 90.),
        Kind::RotatedLine(5, 1, 90.),
        Kind::ScaledRay(4, 1, 2.),
        Kind::ScaledLine(5, 1, 2.),
    ] {
        doc.add(kind);
    }
    for (name, value, y) in [("a", 0.25, 10), ("b", -0.25, 35), ("c", 1.25, 60)] {
        doc.add_named_parameter(name, value, (10, y)).unwrap();
    }
    for host in 6..=9 {
        for source in 10..=12 {
            let point = doc.plot_parameter_on_path(source, Some(host)).unwrap();
            doc.add(Kind::Segment(3, point));
        }
        for value in [0.25, -0.25, 1.25] {
            let point = doc.plot_literal_on_path(value, Some(host)).unwrap();
            doc.add(Kind::Segment(3, point));
        }
    }
    assert_eq!(doc.objects.len(), 60);
    let generated = doc.encode().unwrap();
    assert_eq!(
        generated,
        include_bytes!(
            "../../../tests/fixtures/transformed_straight_plot_generated_native_verified.gsp"
        )
    );
    doc.checkpoint();
    assert!(doc.move_point(3, Point { x: 260., y: 200. }));
    let edited = doc.encode().unwrap();
    assert_eq!(
        edited,
        include_bytes!(
            "../../../tests/fixtures/transformed_straight_plot_edited_native_verified.gsp"
        )
    );
    doc.selected = (1..=60).collect();
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 60);
    let copied = doc.encode().unwrap();
    assert_eq!(
        copied,
        include_bytes!(
            "../../../tests/fixtures/transformed_straight_plot_copied_native_verified.gsp"
        )
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), edited);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), generated);
    doc.redo();
    doc.redo();
    assert_eq!(doc.encode().unwrap(), copied);
}

#[test]
fn independently_visible_native_value_and_literal_markers_use_distinct_affine_values() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/transformed_straight_plot_distinct_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects.len(), 60);
    for (host, first) in [(6, 13), (7, 25), (8, 37), (9, 49)] {
        let (start, end, mode) = doc.line_points(host).unwrap();
        for (offset, value) in [0.25, -0.25, 1.25, 0.5, -0.5, 2.5].into_iter().enumerate() {
            let point = first + 2 * offset;
            if offset < 3 {
                assert_eq!(
                    doc.objects[point - 1].kind,
                    Kind::PathValuePoint(10 + offset, host)
                );
            } else {
                assert_eq!(
                    doc.objects[point - 1].kind,
                    Kind::PathLiteralPoint(host, value)
                );
            }
            let expected = if mode == LineMode::Ray && value < 0. {
                None
            } else {
                Some(Point {
                    x: start.x + (end.x - start.x) * value,
                    y: start.y + (end.y - start.y) * value,
                })
            };
            close(doc.position(point), expected);
        }
    }
    doc.verify_roundtrip().unwrap();
    assert_eq!(doc.encode().unwrap(), bytes);
}
