use super::*;

#[test]
fn native_function_points60_use_unbounded_normalized_domain_and_retain_undefined_dag() {
    for (state, t, min, max, expression) in [
        ("low", -0.25, -1., 1., "x*x"),
        ("start", 0., -1., 1., "x*x"),
        ("quarter", 0.25, -1., 1., "x*x"),
        ("half", 0.5, -1., 1., "x*x"),
        ("end", 1., -1., 1., "x*x"),
        ("high", 1.25, -1., 1., "x*x"),
        ("asymmetric", 0.25, -2., 3., "x*x"),
        ("reciprocal_zero", 0., -1., 1., "1/x"),
        ("reciprocal_half", 0.5, -1., 1., "1/x"),
        ("degrees", 0.5, -1., 1., "sin(x)"),
        ("degrees_quarter", 0.75, -1., 1., "sin(x)"),
        ("degrees_mode_quarter", 0.75, -1., 1., "sin(x)"),
        ("point_value", 0.25, -2., 3., "x*x"),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/function_points60_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert_eq!(d.objects[8].kind, Kind::PointOnCurve(8, t));
        let x = min + (max - min) * t;
        let y = if expression == "x*x" {
            Some(x * x)
        } else if expression == "1/x" {
            if x == 0. { None } else { Some(1. / x) }
        } else {
            Some(if d.angle_unit_radians {
                x.sin()
            } else {
                x.to_radians().sin()
            })
        };
        let expected = y.map(|y| Point {
            x: 250. + 37.79527559055118 * x,
            y: 300. - 37.79527559055118 * y,
        });
        match (d.position(9), expected) {
            (Some(a), Some(b)) => assert!(a.distance(b) < 1e-9, "{state}: {a:?}/{b:?}"),
            (None, None) => {}
            other => panic!("{state}: {other:?}"),
        }
        assert_eq!(d.line_points(11).is_some(), expected.is_some(), "{state}");
        assert_eq!(
            d.point_distance_cm(12).is_some(),
            expected.is_some(),
            "{state}"
        );
        if state == "point_value" {
            assert_eq!(d.numeric_value(14), Some(-0.75));
        }
        assert_eq!(d.encode().unwrap(), bytes);
    }
}

#[test]
fn authored_function_points60_encode_native58_and_follow_function_range_frame_and_undo() {
    let mut d = Document::new();
    let frame = d
        .add_default_coordinate_system(Point { x: 250., y: 300. })
        .unwrap();
    let f = d.add_function_definition(SimpleFunction::Square).unwrap();
    let curve = d.add(Kind::FunctionPlot(f, frame, -1., 1., 301));
    let p = d.add(Kind::PointOnCurve(curve, -0.25));
    let expected = Point {
        x: 250. - 1.5 * 37.79527559055118,
        y: 300. - 2.25 * 37.79527559055118,
    };
    assert!(d.position(p).unwrap().distance(expected) < 1e-9);
    let bytes = d.encode().unwrap();
    let mut reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    let at = reopened.objects[p - 1].constraint_offset.unwrap();
    assert_eq!(&bytes[at..at + 4], &[15, 0, 58, 0]);
    reopened.checkpoint();
    reopened.set_plot_range(curve, -2., 3., 301).unwrap();
    assert!(
        reopened.position(p).unwrap().distance(Point {
            x: 250. - 3.25 * 37.79527559055118,
            y: 300. - 3.25_f64.powi(2) * 37.79527559055118
        }) < 1e-9
    );
    reopened.verify_roundtrip().unwrap();
    reopened.undo();
    assert_eq!(reopened.encode().unwrap(), bytes);
    reopened
        .set_function_expression(f, SimpleFunction::parse("1/x").unwrap())
        .unwrap();
    reopened.objects[p - 1].kind = Kind::PointOnCurve(curve, 0.5);
    assert!(reopened.position(p).is_none());
    reopened.verify_roundtrip().unwrap();
    reopened.objects[p - 1].kind = Kind::PointOnCurve(curve, 0.75);
    assert!(reopened.position(p).is_some());
    reopened.verify_roundtrip().unwrap();
}

#[test]
fn malformed_function_points60_wrong_subtype_host_and_duplicate_constraints_stay_opaque() {
    let bytes = fs::read("../../tests/fixtures/function_points60_quarter_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let constraint = d.objects[8].constraint_offset.unwrap();
    let header = d.objects[8].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs.iter().position(|r| r.data.start == header).unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    let mut bad = Vec::new();
    let mut wrong = bytes.clone();
    wrong[constraint + 2..constraint + 4].copy_from_slice(&42u16.to_le_bytes());
    bad.push(wrong);
    let mut wrong = bytes.clone();
    wrong[refs.data.start + 4..refs.data.start + 8].copy_from_slice(&7u32.to_le_bytes());
    bad.push(wrong);
    let r = rs.iter().find(|r| r.data.start == constraint).unwrap();
    let mut wrong = bytes.clone();
    wrong.splice(r.start..r.start, bytes[r.start..r.data.end].iter().copied());
    bad.push(wrong);
    let mut wrong = bytes.clone();
    wrong[constraint + 4..constraint + 12].copy_from_slice(&f64::NAN.to_le_bytes());
    bad.push(wrong);
    for b in bad {
        let parsed = Document::from_bytes(&b, None).unwrap();
        assert_eq!(parsed.objects[8].kind, Kind::Unsupported(15));
        assert_eq!(parsed.encode().unwrap(), b);
    }
}

#[test]
fn function_points60_projection_matches_native_drag_and_clamps_to_displayed_range() {
    let bytes = fs::read("../../tests/fixtures/function_points60_quarter_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    for (state, query) in [
        ("drag", Point { x: 300.1023622047, y: 349.5511811024 }),
        ("drag_outside", Point { x: 450.1023622047, y: 269.5511811024 }),
    ] {
        let saved = fs::read(format!("../../tests/fixtures/function_points60_{state}_native_saved.gsp")).unwrap();
        let native = Document::from_bytes(&saved, None).unwrap();
        let Kind::PointOnCurve(8, expected) = native.objects[8].kind else { panic!("{state}"); };
        let actual = d.curve_parameter(8, query).unwrap();
        assert!((actual - expected).abs() < 1e-7, "{state}: {actual}/{expected}");
        let Kind::PointOnCurve(8, constrained) = d.constrained_point(8, query).unwrap() else { panic!(); };
        assert_eq!(actual, constrained);
        assert_eq!(native.encode().unwrap(), saved);
    }
    assert!(d.position(9).is_some());
}
