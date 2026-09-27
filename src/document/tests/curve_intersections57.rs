use super::*;
const CURVE_STEP: f64 = 37.79527559055118;
fn curve_close(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-6, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}
fn curve_root(x: f64, y: f64) -> Point {
    Point {
        x: 250. + x * CURVE_STEP,
        y: 300. - y * CURVE_STEP,
    }
}
fn function_intersections(expression: &str, samples: u32) -> Document {
    let mut d = Document::new();
    let plot = d
        .plot_new_function(
            SimpleFunction::parse(expression).unwrap(),
            Point { x: 250., y: 300. },
        )
        .unwrap();
    assert_eq!(plot, 8);
    d.set_plot_range(plot, -3., 3., samples).unwrap();
    d.add(Kind::Free(curve_root(-3., 1.)));
    d.add(Kind::Free(curve_root(3., 1.)));
    d.add(Kind::Line(9, 10));
    for hint in [0, 1, 200] {
        d.add(Kind::CurveIntersection(11, 8, hint));
    }
    d
}
#[test]
fn authored_curve_intersections_refine_function_values_and_preserve_ranked_native_operands() {
    for samples in [9, 301, 1001] {
        let d = function_intersections("x^2", samples);
        let roots = d.curve_intersections(11, 8).unwrap();
        assert_eq!(roots.len(), 2);
        for (actual, x) in roots.iter().zip([-1., 1.]) {
            curve_close(Some(*actual), Some(curve_root(x, 1.)));
        }
        curve_close(d.position(12), Some(curve_root(-1., 1.)));
        for id in [13, 14] {
            curve_close(d.position(id), Some(curve_root(1., 1.)));
        }
        let bytes = d.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[13].kind,
            Kind::CurveIntersection(11, 8, 200)
        );
        let rs = records(&bytes).unwrap();
        let h = reopened.objects[13].header_offset.unwrap();
        let i = rs
            .iter()
            .position(|r| r.kind == 0x7d0 && r.data.start == h)
            .unwrap();
        let c = rs[i..].iter().find(|r| r.kind == 0x7d3).unwrap();
        assert_eq!(
            &bytes[c.data.clone()],
            &[98u32, 200]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>()
        );
    }
}
#[test]
fn curve_intersections_follow_ordered_carrier_domains_live_edits_undo_and_delete() {
    let mut d = function_intersections("x^2", 301);
    d.checkpoint();
    assert!(d.move_point(9, curve_root(3., 1.)));
    assert!(d.move_point(10, curve_root(-3., 1.)));
    curve_close(d.position(12), Some(curve_root(1., 1.)));
    curve_close(d.position(13), Some(curve_root(-1., 1.)));
    d.undo();
    curve_close(d.position(12), Some(curve_root(-1., 1.)));
    d.redo();
    curve_close(d.position(12), Some(curve_root(1., 1.)));
    d.set_plot_range(8, -3., 0., 301).unwrap();
    for id in [12, 13, 14] {
        curve_close(d.position(id), Some(curve_root(-1., 1.)));
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    curve_close(d.position(12), Some(curve_root(1., 1.)));
    d.checkpoint();
    d.objects[10].kind = Kind::Ray(9, 10);
    assert!(d.move_point(9, curve_root(0., 1.)));
    assert!(d.move_point(10, curve_root(3., 1.)));
    for id in [12, 13, 14] {
        curve_close(d.position(id), Some(curve_root(1., 1.)));
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    d.selected.insert(8);
    let removed = d.clear_selected().unwrap();
    for id in [8, 12, 13, 14] {
        assert!(removed.contains(&id));
    }
    d.undo();
    assert!(d.position(12).is_some());
}
#[test]
fn proper_curve_crossings_exclude_tangencies_and_false_pole_bridges() {
    let mut d = function_intersections("x^2", 301);
    let anchor = d.add(Kind::Free(Point { x: 440., y: 440. }));
    let observer = d.add(Kind::Segment(12, anchor));
    d.checkpoint();
    assert!(d.move_point(9, curve_root(-3., 0.)));
    assert!(d.move_point(10, curve_root(3., 0.)));
    assert!(d.curve_intersections(11, 8).unwrap().is_empty());
    for id in [12, 13, 14] {
        assert!(d.position(id).is_none());
    }
    assert!(d.line_points(observer).is_none());
    d.verify_roundtrip().unwrap();
    d.undo();
    assert!(d.position(12).is_some());
    let mut pole = function_intersections("1/x", 301);
    assert!(pole.move_point(9, curve_root(-3., 0.)));
    assert!(pole.move_point(10, curve_root(3., 0.)));
    assert!(pole.curve_intersections(11, 8).unwrap().is_empty());
    assert!(pole.position(12).is_none());
}
#[test]
fn malformed_curve_operand_and_parent_roles_remain_opaque_exactly() {
    let bytes = fs::read("tests/fixtures/curve_intersection57_base_h0_input.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let id = 18;
    let h = d.objects[id - 1].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    let constraint = rs[i..].iter().find(|r| r.kind == 0x7d3).unwrap();
    for (at, value) in [
        (h + 6, 2u16.to_le_bytes().to_vec()),
        (h + 8, 99u16.to_le_bytes().to_vec()),
        (h + 8, 4u16.to_le_bytes().to_vec()),
        (refs.data.start + 4, 8u32.to_le_bytes().to_vec()),
        (refs.data.start + 8, 18u32.to_le_bytes().to_vec()),
        (constraint.data.start, 99u32.to_le_bytes().to_vec()),
    ] {
        let mut b = bytes.clone();
        b[at..at + value.len()].copy_from_slice(&value);
        let invalid = Document::from_bytes(&b, None).unwrap();
        assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(98));
        assert_eq!(invalid.encode().unwrap(), b);
    }
    let mut long = vec![];
    for word in [98u32, 0, 0] {
        long.extend(word.to_le_bytes());
    }
    let mut replacement = vec![];
    put_record(&mut replacement, 0x7d3, &long);
    let mut b = bytes.clone();
    b.splice(constraint.start..constraint.data.end, replacement);
    let invalid = Document::from_bytes(&b, None).unwrap();
    assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(98));
    assert_eq!(invalid.encode().unwrap(), b);
    let mut extra = vec![];
    put_record(
        &mut extra,
        0x7d3,
        &[98u32, 0]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    let mut b = bytes.clone();
    b.splice(constraint.data.end..constraint.data.end, extra);
    let invalid = Document::from_bytes(&b, None).unwrap();
    assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(98));
    assert_eq!(invalid.encode().unwrap(), b);
    let mut b = bytes.clone();
    let short = 98u32.to_le_bytes();
    let mut replacement = vec![];
    put_record(&mut replacement, 0x7d3, &short);
    b.splice(constraint.start..constraint.data.end, replacement);
    let invalid = Document::from_bytes(&b, None).unwrap();
    assert_eq!(invalid.objects[id - 1].kind, Kind::Unsupported(98));
    assert_eq!(invalid.encode().unwrap(), b);
}

#[test]
fn native_curve_intersection_goldens_preserve_all_ranked_crossings_and_opaque_sweeps() {
    let r = ((5f64.sqrt() - 1.) / 2.).sqrt();
    let mut cases: Vec<(&str, Option<Point>)> = vec![];
    let mut add = |names: &[&'static str], point| {
        cases.extend(names.iter().map(|name| (*name, point)));
    };
    add(
        &[
            "base_h1",
            "base_h2",
            "base_h3",
            "base_h10",
            "base_h23",
            "base_h100",
            "base_h200",
            "reverse_h0",
            "coarse_h1",
            "dense_h1",
            "ray_one_h0",
            "ray_one_h1",
            "metadata_empty232e_h1",
            "endpoint_h0",
            "endpoint_h1",
            "parameter_h1",
            "locus_h1",
            "parameter_endpoint_h0",
            "locus_endpoint_h0",
        ],
        Some(curve_root(1., 1.)),
    );
    add(
        &[
            "base_h0",
            "reverse_h1",
            "range_left_h0",
            "range_left_h1",
            "segment_one_h0",
            "segment_one_h1",
            "parameter_h0",
            "locus_h0",
            "parameter_endpoint_left_h0",
        ],
        Some(curve_root(-1., 1.)),
    );
    add(
        &[
            "swapped_h0",
            "swapped_h1",
            "cubic_swapped_h1",
            "tangent_h0",
            "tangent_h1",
            "none_h0",
            "endpoint_left_h0",
            "locus_endpoint_left_h0",
        ],
        None,
    );
    add(&["cubic_h0", "axis_h0"], Some(curve_root(-1., 0.)));
    add(
        &[
            "cubic_h1",
            "cubic_reverse_h1",
            "cubic_range_h0",
            "cubic_coarse_h1",
        ],
        Some(curve_root(0., 0.)),
    );
    add(
        &[
            "cubic_h2",
            "cubic_h3",
            "cubic_h10",
            "cubic_range_h1",
            "cubic_range_h2",
            "axis_h1",
        ],
        Some(curve_root(1., 0.)),
    );
    add(&["edited_h1"], Some(curve_root(2., 4.)));
    add(
        &[
            "carrier_circle_h0",
            "carrier_locus_circle_h0",
            "carrier_arc_h1",
            "carrier_arc_reverse_h0",
            "carrier_locus_circle_reverse_h1",
        ],
        Some(curve_root(-r, r * r)),
    );
    add(
        &[
            "carrier_circle_h1",
            "carrier_circle_h2",
            "carrier_locus_circle_h1",
            "carrier_locus_circle_h2",
            "carrier_arc_h0",
            "carrier_arc_reverse_h1",
            "carrier_locus_circle_reverse_h0",
        ],
        Some(curve_root(r, r * r)),
    );
    let low = (1.2 - 0.6f64.sqrt()) / 2.;
    let high = (1.2 + 0.6f64.sqrt()) / 2.;
    for (name, sign, z) in [
        ("carrier_fourcircle_h0", -1., high),
        ("carrier_fourcircle_h1", -1., low),
        ("carrier_fourcircle_h2", 1., low),
        ("carrier_fourcircle_h3", 1., high),
    ] {
        cases.push((name, Some(curve_root(sign * z.sqrt(), z - 1.1))));
    }
    assert_eq!(cases.len(), 65);
    for (name, expected) in cases {
        let bytes = fs::read(format!(
            "tests/fixtures/curve_intersection57_{name}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        let (index, object) = d
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| {
                matches!(
                    object.kind,
                    Kind::CurveIntersection(..) | Kind::Unsupported(98)
                )
            })
            .unwrap();
        let id = index + 1;
        match object.kind {
            Kind::CurveIntersection(carrier, curve, hint) => {
                let roots = d.curve_intersections(carrier, curve).unwrap();
                assert_eq!(roots.is_empty(), expected.is_none(), "{name}");
                if let Some(expected) = expected {
                    curve_close(
                        Some(roots[(hint as usize).min(roots.len() - 1)]),
                        Some(expected),
                    );
                }
            }
            Kind::Unsupported(98) => {
                assert!(name.contains("swapped"), "{name}");
                assert!(expected.is_none());
            }
            _ => unreachable!(),
        }
        curve_close(d.position(id), expected);
        let observer = d
            .objects
            .iter()
            .position(|o| matches!(o.kind, Kind::Segment(a, _) if a == id))
            .unwrap()
            + 1;
        match (d.line_points(observer), expected) {
            (Some((start, _, _)), Some(expected)) => curve_close(Some(start), Some(expected)),
            (None, None) => {}
            other => panic!("{name}: {other:?}"),
        }
        assert_eq!(d.encode().unwrap(), bytes, "{name}");
    }
    // These two raw packets are retained without claiming a successful Native save:
    // the first swept-target control crashed Native before painting; the second was not run.
    for name in ["sweep_h0", "sweep_h1"] {
        let bytes = fs::read(format!(
            "tests/fixtures/curve_intersection57_{name}_input.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.objects[26].kind, Kind::Unsupported(98));
        assert!(d.position(27).is_none());
        assert_eq!(d.encode().unwrap(), bytes);
    }
}
