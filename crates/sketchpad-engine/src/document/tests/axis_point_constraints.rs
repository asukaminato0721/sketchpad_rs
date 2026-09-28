use super::*;
fn near(actual: Point, expected: Point) {
    assert!(
        actual.distance(expected) < 1e-9,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn native_axis_point_constraints_use_fixed_span_independently_of_frame_units() {
    for mode in ["default", "circle", "distance"] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/graph_axis_point49_{mode}_native_verified.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        let points: Vec<_> = doc
            .objects
            .iter()
            .filter_map(|o| matches!(o.kind, Kind::PointOnLine(_, _)).then_some(o.id))
            .collect();
        assert_eq!(points.len(), 2);
        near(doc.position(points[0]).unwrap(), Point { x: 490., y: 260. });
        near(doc.position(points[1]).unwrap(), Point { x: 240., y: 10. });
        assert_eq!(doc.encode().unwrap(), bytes);
        for id in &points {
            let Kind::PointOnLine(host, t) = doc.objects[*id - 1].kind else {
                unreachable!()
            };
            let measurement = doc.add(Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                first: *id,
                second: host,
            });
            assert!(
                matches!(doc.value_equation_value(measurement),Some(EquationValue::PointValue(value)) if (value-t).abs()<1e-12)
            );
        }
        doc.checkpoint();
        assert!(doc.move_point(points[0], Point { x: 280., y: 360. }));
        near(doc.position(points[0]).unwrap(), Point { x: 280., y: 260. });
        assert!(
            matches!(doc.objects[points[0]-1].kind,Kind::PointOnLine(_,t) if (t-0.7).abs()<1e-12)
        );
        doc.verify_roundtrip().unwrap();
        doc.undo();
        near(doc.position(points[0]).unwrap(), Point { x: 490., y: 260. });
        doc.redo();
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_axis_parameter_samples_and_trigonometric_ticks_preserve_the_fixed_span() {
    for (name, t) in [
        ("t0", 0.),
        ("t05", 0.5),
        ("t1", 1.),
        ("trigonometric", 1.75),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/graph_axis_point49_{name}_native_verified.gsp"
        ))
        .unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        near(
            doc.position(7).unwrap(),
            Point {
                x: 240. + 200. * (t - 0.5),
                y: 260.,
            },
        );
        let y_t = if name == "trigonometric" { -0.75 } else { t };
        near(
            doc.position(8).unwrap(),
            Point {
                x: 240.,
                y: 260. + 200. * (y_t - 0.5),
            },
        );
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn axis_point_authoring_projection_origin_edits_and_constraint_writer_agree() {
    let mut doc = Document::new();
    let frame = doc
        .add_default_coordinate_system(Point { x: 240., y: 260. })
        .unwrap();
    let Kind::CoordinateSystem(x, y) = doc.objects[frame - 1].kind else {
        panic!()
    };
    let Kind::GraphAxis(origin, unit, _) = doc.objects[x - 1].kind else {
        panic!()
    };
    let point = doc.add(
        doc.constrained_point(x, Point { x: 315., y: 200. })
            .unwrap(),
    );
    let vertical = doc.add(
        doc.constrained_point(y, Point { x: 315., y: 200. })
            .unwrap(),
    );
    near(doc.position(point).unwrap(), Point { x: 315., y: 260. });
    near(doc.position(vertical).unwrap(), Point { x: 240., y: 200. });
    doc.checkpoint();
    assert!(doc.set_movement_value(
        unit,
        super::super::movement::MovementValue::Point(Point { x: 340., y: 260. })
    ));
    near(doc.position(point).unwrap(), Point { x: 315., y: 260. });
    doc.checkpoint();
    assert!(doc.move_point(origin, Point { x: 300., y: 300. }));
    near(doc.position(point).unwrap(), Point { x: 375., y: 300. });
    near(doc.position(vertical).unwrap(), Point { x: 300., y: 240. });
    doc.verify_roundtrip().unwrap();
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    for id in [point, vertical] {
        let offset = reopened.objects[id - 1].constraint_offset.unwrap();
        assert_eq!(&bytes[offset..offset + 4], &[15, 0, 38, 0]);
    }
}

#[test]
fn axis_constraint_subtype_rejects_ordinary_lines_and_nonfinite_parameters() {
    let bytes = fs::read("../../tests/fixtures/graph_axis_point49_default_native_verified.gsp").unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    let point = 7;
    let offset = doc.objects[point - 1].constraint_offset.unwrap();
    let mut malformed = bytes.clone();
    malformed[offset + 4..offset + 12].copy_from_slice(&f64::NAN.to_le_bytes());
    let guarded = Document::from_bytes(&malformed, None).unwrap();
    assert!(matches!(
        guarded.objects[point - 1].kind,
        Kind::Unsupported(15)
    ));
    assert_eq!(guarded.encode().unwrap(), malformed);
    let mut ordinary = Document::new();
    let a = ordinary.add(Kind::Free(Point { x: 10., y: 20. }));
    let b = ordinary.add(Kind::Free(Point { x: 30., y: 20. }));
    let line = ordinary.add(Kind::Line(a, b));
    let point = ordinary.add(Kind::PointOnLine(line, 0.75));
    let mut bytes = ordinary.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    bytes[reopened.objects[point - 1].constraint_offset.unwrap() + 2] = 38;
    let guarded = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        guarded.objects[point - 1].kind,
        Kind::Unsupported(15)
    ));
    assert_eq!(guarded.encode().unwrap(), bytes);
}
