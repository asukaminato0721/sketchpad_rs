use super::*;

const STATES: [&str; 10] = [
    "function_coarse",
    "function_dense",
    "function_range_edited",
    "function_gap",
    "mapped_segment",
    "mapped_circle",
    "mapped_circle_closed",
    "mapped_arc",
    "mapped_segment_edited",
    "mapped_frozen",
];

fn source62(state: &str) -> (Vec<u8>, Document, usize) {
    let bytes = fs::read(format!(
        "../../tests/fixtures/locus_point_sources62_{state}_native_saved.gsp"
    ))
    .unwrap();
    let document = Document::from_bytes(&bytes, None).unwrap();
    let outer = if state.starts_with("function") {
        16
    } else {
        24
    };
    (bytes, document, outer)
}

// Independent analytic coordinates used by the Native observer controls.
fn analytic62(state: &str, fraction: f64) -> Option<Point> {
    if state.starts_with("function") {
        let (min, max) = if state == "function_range_edited" {
            (-2., 1.)
        } else {
            (-1., 1.)
        };
        let x = min + (max - min) * fraction;
        let y = if state == "function_gap" {
            if x == 0. {
                return None;
            }
            1. / x
        } else {
            x * x
        };
        let units = 37.79527559055118;
        return Some(Point {
            x: 290. + units * x,
            y: 300. - units * y,
        });
    }
    if state == "mapped_frozen" {
        return Some(Point { x: 300., y: 200. });
    }
    if state == "mapped_arc" && !(0.0..=1.0).contains(&fraction) {
        return None;
    }
    let (x, y) = match state {
        "mapped_circle" | "mapped_circle_closed" | "mapped_arc" => {
            let angle = fraction
                * if state.starts_with("mapped_circle") {
                    std::f64::consts::TAU
                } else {
                    std::f64::consts::PI
                };
            (250. + 50. * angle.cos(), 200. - 50. * angle.sin())
        }
        "mapped_segment_edited" => (200. + 120. * fraction, 200. + 20. * fraction),
        _ => (200. + 100. * fraction, 200.),
    };
    let dx = x - 100.;
    let dy = y - 100.;
    Some(Point {
        x: 200. + dx * dx / 100.,
        y: 120. + dx * dy / 100.,
    })
}

fn close62(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-6, "{a:?} != {b:?}"),
        (None, None) => {}
        pair => panic!("{pair:?}"),
    }
}

#[test]
fn native_function_and_mapped_locus_drivers_preserve_bytes_and_analytic_queries() {
    for state in STATES {
        let (bytes, document, outer) = source62(state);
        assert!(!document.partial_view, "{state}");
        assert_eq!(document.encode().unwrap(), bytes, "{state}");
        let mut literal_count = 0;
        let mut numeric_count = 0;
        for object in &document.objects {
            let (q, numeric) = match object.kind {
                Kind::PointOnCurve(host, q) if host == outer => {
                    literal_count += 1;
                    (q, false)
                }
                Kind::PathValuePoint(parameter, host) if host == outer => {
                    numeric_count += 1;
                    (document.numeric_value(parameter).unwrap(), true)
                }
                _ => continue,
            };
            let expected =
                if numeric && !state.starts_with("mapped_circle") && !(0.0..=1.0).contains(&q) {
                    None
                } else {
                    analytic62(state, q)
                };
            close62(document.position(object.id), expected);
            // Every query has an observer segment, including undefined samples.
            assert_eq!(
                document.line_points(object.id + 2).is_some(),
                expected.is_some(),
                "{state} q{q} numeric{numeric}"
            );
        }
        assert_eq!((literal_count, numeric_count), (6, 6), "{state}");
        let segments = document.locus_segments(outer, 8).unwrap();
        assert!(!segments.is_empty(), "{state}");
        if state == "function_gap" {
            // The pole at .5 splits the locus; its valid neighbors cannot join.
            assert_eq!(segments.len(), 6);
            assert!(
                segments
                    .iter()
                    .all(|(a, b)| (a.y - 300.).signum() == (b.y - 300.).signum())
            );
        }
    }
}

#[test]
fn authored_function_and_mapped_loci_reuse_native_driver_roles_and_roundtrip() {
    for state in STATES {
        let (_, mut document, outer) = source62(state);
        let Kind::Locus(refs, _) = document.objects[outer - 1].kind.clone() else {
            panic!("{state}");
        };
        let created = document.construct_locus(refs[2], refs[0]).unwrap();
        let Kind::Locus(ref generated_refs, _) = document.objects[created - 1].kind else {
            panic!("{state}");
        };
        assert_eq!(generated_refs, &refs);
        let closure = u32::from(!state.starts_with("mapped_circle"));
        assert_eq!(
            super::super::locus::locus_driver_range_info(&document.objects, refs[2]),
            Some((1., closure))
        );
        let saved = document.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view, "{state}");
        assert_eq!(
            reopened.locus_segments(created, 8),
            document.locus_segments(created, 8)
        );
        close62(
            reopened.point_on_curve(created, 0.23),
            analytic62(state, 0.23),
        );
    }
}

#[test]
fn function_locus_edits_undo_and_reopen_propagate_to_outer_queries() {
    let (bytes, mut document, outer) = source62("function_coarse");
    document
        .set_function_expression(7, SimpleFunction::parse("x*x*x").unwrap())
        .unwrap();
    let x = -1. + 2. * 0.23;
    close62(
        document.point_on_curve(outer, 0.23),
        Some(Point {
            x: 290. + 37.79527559055118 * x,
            y: 300. - 37.79527559055118 * x * x * x,
        }),
    );
    let saved = document.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(
        reopened.point_on_curve(outer, 0.23),
        document.point_on_curve(outer, 0.23)
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), bytes);
    close62(
        document.point_on_curve(outer, 0.23),
        analytic62("function_coarse", 0.23),
    );
}

#[test]
fn function_locus_clears_driver_caches_and_retains_unrelated_scoped_inputs() {
    let (_, mut document, outer) = source62("function_coarse");
    let unrelated = document.add(Kind::Free(Point { x: 500., y: 400. }));
    let baseline = document.locus_segments(outer, 8).unwrap();
    let mut scoped = document.clone();
    scoped
        .freeze_scoped_evaluation_inputs(&document, &[12, 15, unrelated])
        .unwrap();
    scoped
        .set_evaluation_point(12, Point { x: 900., y: 900. })
        .unwrap();
    assert_eq!(scoped.locus_segments(outer, 8), Some(baseline));
    assert_eq!(scoped.position(unrelated), document.position(unrelated));
    assert_eq!(scoped.position(12), Some(Point { x: 900., y: 900. }));
}

#[test]
fn new_driver_admission_keeps_page_order_and_direct_numeric_function_guards() {
    for state in ["function_coarse", "mapped_segment"] {
        let (_, document, outer) = source62(state);
        let Kind::Locus(ref refs, _) = document.objects[outer - 1].kind else {
            panic!("{state}");
        };
        let (host, driver, target) = (refs[1], refs[2], refs[0]);
        for mode in 0..4 {
            let mut invalid = document.clone();
            match mode {
                0 => invalid.objects[host - 1].deleted = true,
                1 => invalid.objects[host - 1].page += 1,
                2 => invalid.objects[driver - 1].kind = Kind::PointOnCurve(outer, 0.23),
                _ => invalid.objects[driver - 1].kind = Kind::PointOnCurve(0, 0.23),
            }
            assert!(
                super::super::kind::expected_locus_refs(&invalid.objects, driver, target).is_none()
            );
        }
        if state.starts_with("function") {
            assert!(!super::super::curve_points::supported(
                &document.objects,
                host,
                1,
                driver
            ));
            assert_eq!(document.plotted_curve_position(host, 0.23), None);
        }
    }
}

#[test]
fn locus_range_profiles_reject_non_finite_and_unproved_metadata() {
    for state in ["function_coarse", "mapped_segment", "mapped_circle"] {
        let (bytes, document, outer) = source62(state);
        let header = document.objects[outer - 1].header_offset.unwrap();
        let entries = records(&bytes).unwrap();
        let first = entries
            .iter()
            .position(|entry| entry.kind == 0x7d0 && entry.data.start == header)
            .unwrap();
        let range = entries[first..]
            .iter()
            .find(|entry| entry.kind == 0x902)
            .unwrap()
            .data
            .start;
        for maximum in [f64::NAN, f64::INFINITY, 2.] {
            let mut invalid = bytes.clone();
            invalid[range + 8..range + 16].copy_from_slice(&maximum.to_le_bytes());
            let parsed = Document::from_bytes(&invalid, None).unwrap();
            assert!(
                matches!(parsed.objects[outer - 1].kind, Kind::Unsupported(35)),
                "{state}"
            );
            assert_eq!(parsed.encode().unwrap(), invalid);
        }
        let mut invalid = bytes.clone();
        invalid[range + 20..range + 24].copy_from_slice(&0x20004_u32.to_le_bytes());
        let parsed = Document::from_bytes(&invalid, None).unwrap();
        assert!(matches!(
            parsed.objects[outer - 1].kind,
            Kind::Unsupported(35)
        ));
    }
}

#[test]
fn curve_driver_drag_undo_and_deletion_keep_locus_dependents_linked() {
    for state in ["function_coarse", "mapped_segment"] {
        let (bytes, mut document, outer) = source62(state);
        let Kind::Locus(ref refs, _) = document.objects[outer - 1].kind else {
            panic!("{state}");
        };
        let (host, driver, target) = (refs[1], refs[2], refs[0]);
        let baseline = document.locus_segments(outer, 8).unwrap();
        let expected = analytic62(state, 0.71).unwrap();
        document.checkpoint();
        assert!(document.move_point(
            driver,
            Point {
                x: expected.x - 40.,
                y: expected.y
            }
        ));
        close62(document.position(target), Some(expected));
        assert_eq!(document.locus_segments(outer, 8), Some(baseline));
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        close62(reopened.position(target), document.position(target));
        document.undo();
        assert_eq!(document.encode().unwrap(), bytes);
        document.selected.insert(host);
        document.clear_selected().unwrap();
        assert!(document.objects[driver - 1].deleted);
        assert!(document.objects[outer - 1].deleted);
        assert!(document.objects[outer].deleted);
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
    }
}
