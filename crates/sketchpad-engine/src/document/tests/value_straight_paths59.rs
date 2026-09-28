use super::*;

#[test]
fn native_straight_paths59_affine_extent_and_exact_retention() {
    let mut checked = 0;
    for family in [
        "segment",
        "parallel",
        "perpendicular",
        "vector_segment",
        "numeric_rotation_segment",
        "polar_ray",
        "vector_ray",
        "reflected_segment",
        "scaled_segment",
    ] {
        for state in ["inside", "low", "high", "ends"] {
            let bytes = fs::read(format!(
                "../../tests/fixtures/value_straight_paths59_{family}_{state}_native_saved.gsp"
            ))
            .unwrap();
            let d = Document::from_bytes(&bytes, None).unwrap();
            assert!(!d.partial_view, "{family}/{state}");
            for o in &d.objects {
                let (host, t) = match o.kind {
                    Kind::PathValuePoint(source, host) => (host, d.numeric_value(source).unwrap()),
                    Kind::PathLiteralPoint(host, t) => (host, t),
                    _ => continue,
                };
                let (a, b, mode) = d.line_points(host).unwrap();
                let defined = match mode {
                    LineMode::Segment => (0.0..=1.0).contains(&t),
                    LineMode::Ray => t >= 0.0,
                    LineMode::Line => true,
                };
                let actual = d.position(o.id);
                if defined {
                    assert!(
                        actual.unwrap().distance(Point {
                            x: a.x + (b.x - a.x) * t,
                            y: a.y + (b.y - a.y) * t
                        }) < 1e-8,
                        "{family}/{state}"
                    );
                } else {
                    assert!(actual.is_none(), "{family}/{state}");
                }
                for dependent in &d.objects {
                    if let Kind::Segment(_, point) = dependent.kind {
                        if point == o.id {
                            assert_eq!(d.line_points(dependent.id).is_some(), defined);
                        }
                    }
                }
                checked += 1;
            }
            assert_eq!(d.encode().unwrap(), bytes);
        }
    }
    assert_eq!(checked, 72);
    for code in [95, 123] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/value_straight_paths59_vector_ray_inside_empty{code}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        for o in &d.objects {
            if matches!(
                o.kind,
                Kind::PathValuePoint(..) | Kind::PathLiteralPoint(..)
            ) {
                assert!(d.position(o.id).is_some());
            }
        }
    }
}

#[test]
fn authored_straight_paths59_outside_values_save_and_recover() {
    let mut d = Document::new();
    let a = d.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = d.add(Kind::Free(Point { x: 200., y: 100. }));
    let segment = d.add(Kind::Segment(a, b));
    let value = d.add_named_parameter("q", -0.25, (10, 55)).unwrap();
    let p = d.plot_parameter_on_path(value, Some(segment)).unwrap();
    let literal = d.plot_literal_on_path(1.25, Some(segment)).unwrap();
    assert!(d.position(p).is_none());
    assert!(d.position(literal).is_none());
    let measurement = d.add(Kind::ValueEquation {
        kind: ValueEquationKind::PointValue,
        first: p,
        second: segment,
    });
    assert!(d.numeric_value(measurement).is_none());
    let bytes = d.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.position(p).is_none());
    assert!(reopened.position(literal).is_none());
    d.set_parameter_value(value, 0.25).unwrap();
    assert_eq!(d.position(p), Some(Point { x: 125., y: 100. }));
    let parallel = d.add(Kind::Parallel(a, segment));
    let extended = d.plot_literal_on_path(-2., Some(parallel)).unwrap();
    assert!(d.position(extended).is_some());
    assert!(
        !Document::from_bytes(&d.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
