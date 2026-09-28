use super::*;
const POINT_CURVE_UNIT: f64 = 37.79527559055118;
fn path_curve_world(x: f64) -> Point {
    Point {
        x: 250. + x * POINT_CURVE_UNIT,
        y: 300. - x * x * POINT_CURVE_UNIT,
    }
}
fn path_curve_close(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-6, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}
fn path_curve_source(geometric: bool) -> (Document, usize, usize) {
    let mut d = Document::new();
    let frame = d
        .add_default_coordinate_system(Point { x: 250., y: 300. })
        .unwrap();
    if geometric {
        let a = d.add(Kind::Free(Point { x: 150., y: 350. }));
        let b = d.add(Kind::Free(Point { x: 350., y: 350. }));
        let path = d.add(Kind::Segment(a, b));
        let driver = d.add(Kind::PointOnSegment(path, 0.5));
        let value = d.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: driver,
            second: path,
        });
        let x = d
            .add_parameter_calculation(value, "2*v*v-1", (10, 100))
            .unwrap();
        let y = d.add_parameter_calculation(x, "v*v", (10, 130)).unwrap();
        let target = d.add(Kind::NumericCoordinatePoint(x, y, frame));
        let curve = d.construct_locus(driver, target).unwrap();
        (d, curve, driver)
    } else {
        let driver = d.add_named_parameter("u", 0., (10, 100)).unwrap();
        let square = d
            .add_parameter_calculation(driver, "u*u", (10, 130))
            .unwrap();
        let target = d.add(Kind::NumericCoordinatePoint(driver, square, frame));
        let curve = d
            .construct_parameter_locus(driver, target, -1., 1., 9)
            .unwrap();
        (d, curve, driver)
    }
}
#[test]
fn malformed_point_curve_schema_and_roles_remain_opaque_exactly() {
    for (state, code, id, curve, scalar) in [
        ("geo_15_quarter", 15u16, 20, 15, 16),
        ("geo_95_quarter", 95u16, 20, 15, 16),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/path_curve_points58_{state}_input.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        let rs = records(&bytes).unwrap();
        let h = d.objects[id - 1].header_offset.unwrap();
        let i = rs
            .iter()
            .position(|r| r.kind == 0x7d0 && r.data.start == h)
            .unwrap();
        let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
        let mut mutations = vec![
            (h + 6, 2u16.to_le_bytes().to_vec()),
            (h + 8, 99u16.to_le_bytes().to_vec()),
        ];
        if code == 15 {
            let constraint = rs[i..].iter().find(|r| r.kind == 0x7d3).unwrap();
            mutations.extend([
                (constraint.data.start + 2, 43u16.to_le_bytes().to_vec()),
                (constraint.data.start + 4, f64::NAN.to_le_bytes().to_vec()),
                (refs.data.start + 4, (scalar as u32).to_le_bytes().to_vec()),
                (refs.data.start + 4, (id as u32).to_le_bytes().to_vec()),
            ]);
        } else {
            mutations.extend([
                (refs.data.start + 4, (curve as u32).to_le_bytes().to_vec()),
                (refs.data.start + 8, (scalar as u32).to_le_bytes().to_vec()),
                (refs.data.start + 8, (id as u32).to_le_bytes().to_vec()),
            ]);
        }
        for (at, payload) in mutations {
            let mut altered = bytes.clone();
            altered[at..at + payload.len()].copy_from_slice(&payload);
            let invalid = Document::from_bytes(&altered, None).unwrap();
            assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(code));
            assert_eq!(invalid.encode().unwrap(), altered);
        }
    }
}

#[test]
fn numeric_point_curve_authoring_rejects_non_scalar_sources() {
    let typed = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/numeric_typed_parameters_native_verified.gsp"),
        None,
    )
    .unwrap();
    let (mut d, host, _) = path_curve_source(true);
    let length = d.add(typed.objects[1].kind.clone());
    assert_eq!(
        d.objects[length - 1].kind.numeric_unit(),
        Some(crate::table_records::NumericUnit::Length)
    );
    assert!(d.plot_parameter_on_path(length, Some(host)).is_err());
    let point = d.add(Kind::PathValuePoint(length, host));
    assert!(d.position(point).is_none());
    assert!(d.encode().is_err());
}

#[test]
fn point_curve_definitions_keep_interactive_and_numeric_roles_live_and_scoped() {
    for geometric in [false, true] {
        let (mut d, curve, driver) = path_curve_source(geometric);
        let q = d.add_named_parameter("q", 0.5, (10, 160)).unwrap();
        let interactive = d.add(Kind::PointOnCurve(curve, 0.5));
        let numeric = d.plot_parameter_on_path(q, Some(curve)).unwrap();
        let readout = d.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: interactive,
            second: curve,
        });
        let expected_interactive = path_curve_world(if geometric { -0.5 } else { 0. });
        let expected_numeric = path_curve_world(if geometric { -0.5 } else { 0.5 });
        path_curve_close(d.position(interactive), Some(expected_interactive));
        path_curve_close(d.position(numeric), Some(expected_numeric));
        assert_eq!(
            d.numeric_value(readout),
            Some(if geometric { 0.5 } else { 0. })
        );
        assert!(!d.move_point(numeric, path_curve_world(0.)));
        d.verify_roundtrip().unwrap();
        let target = match &d.objects[curve - 1].kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs[0],
            _ => unreachable!(),
        };
        let external = d.add(Kind::Free(Point { x: 510., y: 410. }));
        let original_bytes = d.encode().unwrap();
        let mut scoped = d.clone();
        scoped
            .freeze_scoped_evaluation_inputs(&d, &[target, external])
            .unwrap();
        scoped
            .set_evaluation_point(external, Point { x: 610., y: 510. })
            .unwrap();
        if geometric {
            scoped
                .set_evaluation_point(driver, Point { x: 330., y: 350. })
                .unwrap();
        } else {
            scoped.set_evaluation_number(driver, 0.9).unwrap();
        }
        // A trace samples its own local driver even when the caller froze its
        // dependent target; external substitutions remain caller-owned.
        path_curve_close(scoped.position(interactive), Some(expected_interactive));
        path_curve_close(scoped.position(numeric), Some(expected_numeric));
        path_curve_close(scoped.position(external), Some(Point { x: 610., y: 510. }));
        assert_eq!(scoped.encode().unwrap(), d.encode().unwrap());
        d.checkpoint();
        assert!(d.move_point(interactive, path_curve_world(-0.875)));
        path_curve_close(d.position(interactive), Some(path_curve_world(-0.875)));
        assert!(
            (d.numeric_value(readout).unwrap() - if geometric { 0.25 } else { -0.875 }).abs()
                < 1e-5
        );
        d.verify_roundtrip().unwrap();
        d.undo();
        path_curve_close(d.position(interactive), Some(expected_interactive));
        d.redo();
        path_curve_close(d.position(interactive), Some(path_curve_world(-0.875)));
        d.selected.insert(curve);
        let deleted = d.clear_selected().unwrap();
        for id in [interactive, numeric, readout] {
            assert!(deleted.contains(&id));
        }
        d.undo();
        assert!(d.position(interactive).is_some());
        assert_eq!(scoped.encode().unwrap(), original_bytes);
    }
}

#[test]
fn retained_second_page_curve_points_drag_and_save_with_local_refs() {
    let bytes =
        fs::read("../../tests/fixtures/path_curve_points58_geo_15_half_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    d.add_blank_page().unwrap();
    d.move_page(0, 1).unwrap();
    d.active_page = 0;
    let a = d.add(Kind::Free(Point { x: 80., y: 100. }));
    let b = d.add(Kind::Free(Point { x: 180., y: 100. }));
    d.add(Kind::Segment(a, b));
    d.active_page = 1;
    let mut d = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    let base = d.objects.iter().filter(|o| o.page == 0).count();
    assert_eq!(base, 3);
    let (point, host) = d
        .objects
        .iter()
        .find_map(|o| match o.kind {
            Kind::PointOnCurve(host, _) => Some((o.id, host)),
            _ => None,
        })
        .unwrap();
    let created = d.add(Kind::PointOnCurve(host, 0.5));
    d.checkpoint();
    assert!(d.move_point(point, path_curve_world(-0.875)));
    path_curve_close(d.position(point), Some(path_curve_world(-0.875)));
    let bytes = d.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    for id in [point, created] {
        assert_eq!(reopened.objects[id - 1].page, 1);
        let rs = records(&bytes).unwrap();
        let h = reopened.objects[id - 1].header_offset.unwrap();
        let i = rs
            .iter()
            .position(|r| r.kind == 0x7d0 && r.data.start == h)
            .unwrap();
        let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
        assert_eq!(
            &bytes[refs.data.start + 4..refs.data.start + 8],
            &((host - base) as u32).to_le_bytes()
        );
        let constraint = rs[i..].iter().find(|r| r.kind == 0x7d3).unwrap();
        assert_eq!(
            &bytes[constraint.data.start..constraint.data.start + 4],
            &[15, 0, 42, 0]
        );
    }
    path_curve_close(reopened.position(point), Some(path_curve_world(-0.875)));
    path_curve_close(reopened.position(created), Some(path_curve_world(-0.5)));
    assert_eq!(reopened.encode().unwrap(), bytes);
    d.undo();
    path_curve_close(d.position(point), Some(path_curve_world(-0.5)));
}

#[test]
fn native_point_curve_samples_preserve_distinct_parameter_recipes_and_packets() {
    let mut checked = 0;
    for family in ["geo", "parameter"] {
        for code in [15u16, 95] {
            let mut states = if family == "geo" {
                vec![
                    ("low", -0.25),
                    ("start", 0.),
                    ("quarter", 0.25),
                    ("half", 0.5),
                    ("end", 1.),
                    ("high", 1.25),
                    ("quarter_dense", 0.25),
                    ("half_dense", 0.5),
                ]
            } else {
                vec![
                    ("quarter", 0.25),
                    ("half", 0.5),
                    ("quarter_dense", 0.25),
                    ("half_dense", 0.5),
                ]
            };
            states.extend([("offgrid", 0.23), ("offgrid_dense", 0.23)]);
            for (tag, t) in states {
                let state = format!("{family}_{code}_{tag}");
                let bytes = fs::read(format!(
                    "../../tests/fixtures/path_curve_points58_{state}_native_saved.gsp"
                ))
                .unwrap();
                let d = Document::from_bytes(&bytes, None).unwrap();
                let (id, host, source, observer) = if family == "geo" {
                    (20, 15, 16, 22)
                } else {
                    (15, 10, 11, 17)
                };
                assert_eq!(
                    d.objects[id - 1].kind,
                    if code == 15 {
                        Kind::PointOnCurve(host, t)
                    } else {
                        Kind::PathValuePoint(source, host)
                    },
                    "{state}"
                );
                let expected = if family == "geo" {
                    if code == 95 && !(0. ..=1.).contains(&t) {
                        None
                    } else {
                        let v = t.clamp(0., 1.);
                        Some(path_curve_world(2. * v * v - 1.))
                    }
                } else {
                    Some(path_curve_world(if code == 15 { 2. * t - 1. } else { t }))
                };
                path_curve_close(d.position(id), expected);
                match (d.line_points(observer), expected) {
                    (Some((a, _, _)), Some(p)) => path_curve_close(Some(a), Some(p)),
                    (None, None) => {}
                    other => panic!("{state}: {other:?}"),
                }
                if code == 15 && tag.starts_with("offgrid") {
                    let measurement = if family == "geo" { 27 } else { 22 };
                    let value = if family == "geo" { t } else { 2. * t - 1. };
                    assert!(
                        (d.numeric_value(measurement).unwrap() - value).abs() < 1e-9,
                        "{state}"
                    );
                }
                let rs = records(&bytes).unwrap();
                let h = d.objects[id - 1].header_offset.unwrap();
                let i = rs
                    .iter()
                    .position(|r| r.kind == 0x7d0 && r.data.start == h)
                    .unwrap();
                let end = (i..rs.len()).find(|&j| rs[j].kind == 0x7d7).unwrap();
                assert_eq!(&bytes[h..h + 2], &code.to_le_bytes());
                if code == 15 {
                    let constraint = rs[i..=end].iter().find(|r| r.kind == 0x7d3).unwrap();
                    assert_eq!(constraint.data.len(), 12);
                    assert_eq!(
                        &bytes[constraint.data.start..constraint.data.start + 4],
                        &[15, 0, 42, 0]
                    );
                    assert_eq!(
                        &bytes[constraint.data.start + 4..constraint.data.end],
                        &t.to_le_bytes()
                    );
                } else {
                    assert!(!rs[i..=end].iter().any(|r| r.kind == 0x7d3));
                }
                assert_eq!(d.encode().unwrap(), bytes, "{state}");
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 32);
    for (tag, t) in [
        ("low", -1.25f64),
        ("start", -1.),
        ("zero", 0.),
        ("end", 1.),
        ("high", 1.25),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/path_curve_points58_parameter_95_bounds_{tag}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.objects[14].kind, Kind::PathValuePoint(11, 10));
        let expected = (-1. ..=1.).contains(&t).then(|| path_curve_world(t));
        path_curve_close(d.position(15), expected);
        assert_eq!(d.line_points(17).is_some(), expected.is_some());
        assert_eq!(d.encode().unwrap(), bytes);
    }
    for (tag, t) in [
        ("low", -0.25f64),
        ("start", 0.),
        ("end", 1.),
        ("high", 1.25),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/path_curve_points58_parameter_15_bounds_{tag}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.objects[14].kind, Kind::PointOnCurve(10, t));
        path_curve_close(d.position(15), Some(path_curve_world(2. * t - 1.)));
        assert!(d.line_points(17).is_some());
        assert_eq!(d.encode().unwrap(), bytes);
    }
    for (state, code, t, angle) in [
        ("degree_15", 15, 0.25f64, std::f64::consts::FRAC_PI_2),
        ("degree_95", 95, 0.5, 0.5),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/path_curve_points58_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(
            d.objects[11].kind,
            if code == 15 {
                Kind::PointOnCurve(7, t)
            } else {
                Kind::PathValuePoint(8, 7)
            }
        );
        path_curve_close(
            d.position(12),
            Some(Point {
                x: 250. + POINT_CURVE_UNIT * angle.cos(),
                y: 300. - POINT_CURVE_UNIT * angle.sin(),
            }),
        );
        assert_eq!(d.encode().unwrap(), bytes);
    }
    for family in ["circle", "arc"] {
        for code in [15u16, 95] {
            for (tag, t) in [("low", -0.25f64), ("quarter", 0.25), ("high", 1.25)] {
                let bytes = fs::read(format!(
                    "../../tests/fixtures/path_curve_points58_{family}_{code}_{tag}_native_saved.gsp"
                ))
                .unwrap();
                let d = Document::from_bytes(&bytes, None).unwrap();
                let (id, host, source, observer) = if family == "circle" {
                    (20, 15, 16, 22)
                } else {
                    (21, 16, 17, 23)
                };
                assert_eq!(
                    d.objects[id - 1].kind,
                    if code == 15 {
                        Kind::PointOnCurve(host, t)
                    } else {
                        Kind::PathValuePoint(source, host)
                    }
                );
                let expected = if family == "arc" && !(0. ..=1.).contains(&t) {
                    None
                } else {
                    let v = if family == "circle" {
                        t.rem_euclid(1.)
                    } else {
                        t
                    };
                    Some(path_curve_world(2. * v * v - 1.))
                };
                path_curve_close(d.position(id), expected);
                assert_eq!(d.line_points(observer).is_some(), expected.is_some());
                assert_eq!(d.encode().unwrap(), bytes);
            }
        }
    }
    for code in [15u16, 95] {
        for (tag, t) in [("start", 0f64), ("end", 1.)] {
            let bytes = fs::read(format!(
                "../../tests/fixtures/path_curve_points58_circle_{code}_bounds_{tag}_native_saved.gsp"
            ))
            .unwrap();
            let d = Document::from_bytes(&bytes, None).unwrap();
            assert_eq!(
                d.objects[19].kind,
                if code == 15 {
                    Kind::PointOnCurve(15, t)
                } else {
                    Kind::PathValuePoint(16, 15)
                }
            );
            // The exact maximum wraps before evaluating the nonperiodic target;
            // simply using the target's last sampled point would produce x=+1.
            path_curve_close(d.position(20), Some(path_curve_world(-1.)));
            assert!(d.line_points(22).is_some());
            assert_eq!(d.encode().unwrap(), bytes);
        }
    }
    let bytes =
        fs::read("../../tests/fixtures/path_curve_points58_degree_15_value_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    path_curve_close(
        d.position(12),
        Some(Point {
            x: 250.,
            y: 300. - POINT_CURVE_UNIT,
        }),
    );
    assert_eq!(
        d.objects[20].kind,
        Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: 12,
            second: 7
        }
    );
    assert!((d.numeric_value(21).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn unproved_swept_curve_point_packets_remain_opaque_exactly() {
    let bytes = fs::read("../../tests/fixtures/curve_intersection57_sweep_h0_input.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let rs = records(&bytes).unwrap();
    let h = d.objects[26].header_offset.unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let end = (i..rs.len()).find(|&j| rs[j].kind == 0x7d7).unwrap();
    for code in [15u16, 95] {
        let mut group = vec![];
        for record in &rs[i..=end] {
            let mut payload = bytes[record.data.clone()].to_vec();
            match record.kind {
                0x7d0 => {
                    payload[..2].copy_from_slice(&code.to_le_bytes());
                    payload[8..10]
                        .copy_from_slice(&(if code == 15 { 10u16 } else { 56u16 }).to_le_bytes());
                }
                0x7d2 => {
                    let parents = if code == 15 {
                        vec![17u32]
                    } else {
                        vec![11u32, 17]
                    };
                    payload = (parents.len() as u32).to_le_bytes().to_vec();
                    for parent in parents {
                        payload.extend(parent.to_le_bytes())
                    }
                }
                0x7d3 => {
                    if code == 95 {
                        continue;
                    }
                    payload = vec![15, 0, 42, 0];
                    payload.extend(0.5f64.to_le_bytes())
                }
                _ => {}
            }
            put_record(&mut group, record.kind, &payload);
        }
        let mut altered = bytes.clone();
        altered.splice(rs[i].start..rs[end].data.end, group);
        let imported = Document::from_bytes(&altered, None).unwrap();
        assert_eq!(imported.objects[26].kind, Kind::Unsupported(code));
        assert!(imported.position(27).is_none());
        assert_eq!(imported.encode().unwrap(), altered);
    }
}

#[test]
fn undefined_point_curve_targets_suppress_dependents_and_recover_after_edit() {
    let mut d = Document::new();
    let frame = d
        .add_default_coordinate_system(Point { x: 250., y: 300. })
        .unwrap();
    let u = d.add_named_parameter("u", 1., (10, 100)).unwrap();
    let y = d.add_parameter_calculation(u, "1/u", (10, 130)).unwrap();
    let target = d.add(Kind::NumericCoordinatePoint(u, y, frame));
    let curve = d.construct_parameter_locus(u, target, -1., 1., 9).unwrap();
    let q = d.add_named_parameter("q", 0., (10, 160)).unwrap();
    let point = d.add(Kind::PointOnCurve(curve, 0.5));
    let numeric = d.add(Kind::PathValuePoint(q, curve));
    let anchor = d.add(Kind::Free(Point { x: 440., y: 440. }));
    let observer = d.add(Kind::Segment(point, anchor));
    for id in [point, numeric] {
        assert!(d.position(id).is_none());
    }
    assert!(d.line_points(observer).is_none());
    d.verify_roundtrip().unwrap();
    let bytes = d.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(reopened.position(point).is_none());
    assert!(reopened.position(numeric).is_none());
    d.checkpoint();
    d.objects[point - 1].kind = Kind::PointOnCurve(curve, 0.75);
    d.set_parameter_value(q, 0.5).unwrap();
    let expected = Point {
        x: 250. + 0.5 * POINT_CURVE_UNIT,
        y: 300. - 2. * POINT_CURVE_UNIT,
    };
    path_curve_close(d.position(point), Some(expected));
    path_curve_close(d.position(numeric), Some(expected));
    assert!(d.line_points(observer).is_some());
    d.verify_roundtrip().unwrap();
    d.undo();
    assert!(d.position(numeric).is_none());
}
