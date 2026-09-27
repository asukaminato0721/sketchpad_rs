use super::*;

fn drivers61_close(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-6, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}

// Independent marker coordinates from the pre-production Native control manifests.
fn drivers61_cases() -> Vec<(&'static str, usize, usize, usize, usize, [Point; 5])> {
    vec![
        (
            "curve_segment_coarse",
            15,
            19,
            22,
            23,
            [
                Point {
                    x: 252.204724409449,
                    y: 262.204724409449,
                },
                Point {
                    x: 256.203464566929,
                    y: 269.779138015748,
                },
                Point {
                    x: 271.102362204724,
                    y: 290.551181102362,
                },
                Point {
                    x: 327.795275590551,
                    y: 262.204724409449,
                },
                Point {
                    x: 370.314960629921,
                    y: 129.330708661417,
                },
            ],
        ),
        (
            "curve_segment_dense",
            15,
            19,
            22,
            23,
            [
                Point {
                    x: 252.204724409449,
                    y: 262.204724409449,
                },
                Point {
                    x: 256.203464566929,
                    y: 269.779138015748,
                },
                Point {
                    x: 271.102362204724,
                    y: 290.551181102362,
                },
                Point {
                    x: 327.795275590551,
                    y: 262.204724409449,
                },
                Point {
                    x: 370.314960629921,
                    y: 129.330708661417,
                },
            ],
        ),
        (
            "curve_segment_edited",
            15,
            19,
            22,
            23,
            [
                Point {
                    x: 272.204724409449,
                    y: 282.204724409449,
                },
                Point {
                    x: 276.203464566929,
                    y: 289.779138015748,
                },
                Point {
                    x: 291.102362204724,
                    y: 310.551181102362,
                },
                Point {
                    x: 347.795275590551,
                    y: 282.204724409449,
                },
                Point {
                    x: 390.314960629921,
                    y: 149.330708661417,
                },
            ],
        ),
        (
            "curve_circle",
            14,
            18,
            21,
            22,
            [
                Point {
                    x: 360.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 290.026658685144,
                    y: 200.630823894842,
                },
                Point {
                    x: 200.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 360.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 280.000000000000,
                    y: 200.000000000000,
                },
            ],
        ),
        (
            "curve_circle_reversed",
            18,
            22,
            25,
            26,
            [
                Point {
                    x: 200.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 269.973341314856,
                    y: 200.630823894842,
                },
                Point {
                    x: 360.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 200.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 280.000000000000,
                    y: 200.000000000000,
                },
            ],
        ),
        (
            "curve_parameter",
            10,
            14,
            17,
            18,
            [
                Point {
                    x: 252.204724409449,
                    y: 262.204724409449,
                },
                Point {
                    x: 269.590551181102,
                    y: 288.978897637795,
                },
                Point {
                    x: 290.000000000000,
                    y: 300.000000000000,
                },
                Point {
                    x: 327.795275590551,
                    y: 262.204724409449,
                },
                Point {
                    x: 346.692913385827,
                    y: 214.960629921260,
                },
            ],
        ),
        (
            "curve_parameter_range",
            10,
            14,
            17,
            18,
            [
                Point {
                    x: 214.409448818898,
                    y: 148.818897637795,
                },
                Point {
                    x: 240.488188976378,
                    y: 235.139527559055,
                },
                Point {
                    x: 271.102362204724,
                    y: 290.551181102362,
                },
                Point {
                    x: 327.795275590551,
                    y: 262.204724409449,
                },
                Point {
                    x: 356.141732283465,
                    y: 184.251968503937,
                },
            ],
        ),
        (
            "polygon_unequal",
            11,
            15,
            18,
            19,
            [
                Point {
                    x: 220.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 330.400000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 400.000000000000,
                    y: 220.000000000000,
                },
                Point {
                    x: 220.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 340.000000000000,
                    y: 280.000000000000,
                },
            ],
        ),
        (
            "polygon_colored",
            11,
            15,
            20,
            21,
            [
                Point {
                    x: 220.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 330.400000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 400.000000000000,
                    y: 220.000000000000,
                },
                Point {
                    x: 220.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 340.000000000000,
                    y: 280.000000000000,
                },
            ],
        ),
        (
            "polygon_edited",
            11,
            15,
            18,
            19,
            [
                Point {
                    x: 240.000000000000,
                    y: 300.000000000000,
                },
                Point {
                    x: 378.000000000000,
                    y: 300.000000000000,
                },
                Point {
                    x: 480.000000000000,
                    y: 240.000000000000,
                },
                Point {
                    x: 240.000000000000,
                    y: 300.000000000000,
                },
                Point {
                    x: 390.000000000000,
                    y: 300.000000000000,
                },
            ],
        ),
        (
            "sector_quarter",
            11,
            15,
            18,
            19,
            [
                Point {
                    x: 280.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 345.702652413026,
                    y: 280.000000000000,
                },
                Point {
                    x: 336.568542494924,
                    y: 223.431457505076,
                },
                Point {
                    x: 280.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 351.415926535898,
                    y: 280.000000000000,
                },
            ],
        ),
        (
            "sector_major",
            11,
            15,
            18,
            19,
            [
                Point {
                    x: 280.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 348.457854605988,
                    y: 238.605288468870,
                },
                Point {
                    x: 223.431457505076,
                    y: 223.431457505076,
                },
                Point {
                    x: 280.000000000000,
                    y: 280.000000000000,
                },
                Point {
                    x: 342.301419955806,
                    y: 229.815011492576,
                },
            ],
        ),
    ]
}

#[test]
fn native_nested_curve_polygon_and_sector_locus_drivers_are_lossless_and_live() {
    for (state, _host, _driver, _target, outer, markers) in drivers61_cases() {
        let bytes = fs::read(format!(
            "tests/fixtures/locus_drivers60_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert!(
            matches!(d.objects[outer - 1].kind, Kind::Locus(..)),
            "{state}"
        );
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
        for (index, marker) in markers.iter().copied().enumerate() {
            let literal = outer + 1 + 11 * index;
            let numeric = literal + 6;
            let literal_expected = if index != 4 {
                Some(marker)
            } else if state.starts_with("polygon") || state.starts_with("sector") {
                None
            } else if state.starts_with("curve_segment") {
                Some(markers[3])
            } else {
                Some(marker)
            };
            let numeric_expected = if index == 4
                && (state.starts_with("curve_segment") || state.starts_with("curve_parameter"))
            {
                None
            } else {
                Some(marker)
            };
            drivers61_close(d.position(literal), literal_expected);
            drivers61_close(d.position(numeric), numeric_expected);
        }
        assert!(!d.locus_segments(outer, 9).unwrap().is_empty(), "{state}");
    }
}

#[test]
fn authored_loci_reuse_complete_captured_driver_dependencies_and_preserve_save() {
    for (state, _host, driver, target, outer, _) in drivers61_cases() {
        let bytes = fs::read(format!(
            "tests/fixtures/locus_drivers60_{state}_native_saved.gsp"
        ))
        .unwrap();
        let mut d = Document::from_bytes(&bytes, None).unwrap();
        let Kind::Locus(original_refs, _) = d.objects[outer - 1].kind.clone() else {
            panic!("{state}")
        };
        let created = d.construct_locus(driver, target).unwrap();
        let Kind::Locus(ref refs, _) = d.objects[created - 1].kind else {
            panic!("{state}")
        };
        assert_eq!(refs, &original_refs);
        let generated = d.locus_segments(created, 9).unwrap();
        assert!(!generated.is_empty());
        let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view, "{state}");
        assert_eq!(reopened.locus_segments(created, 9), Some(generated));
    }
}

#[test]
fn locus_driver_roles_require_earlier_live_same_page_hosts_and_point_traces() {
    let bytes =
        fs::read("tests/fixtures/locus_drivers60_curve_segment_coarse_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    for bad in [0, d.objects.len() + 1] {
        let mut invalid = d.clone();
        invalid.objects[18].kind = Kind::PointOnCurve(bad, 0.23);
        assert!(super::super::kind::expected_locus_refs(&invalid.objects, 19, 22).is_none());
    }
    for mode in 0..3 {
        let mut invalid = d.clone();
        match mode {
            0 => invalid.objects[14].deleted = true,
            1 => invalid.objects[14].page += 1,
            _ => {
                let Kind::Locus(ref mut refs, _) = invalid.objects[14].kind else {
                    unreachable!()
                };
                refs[0] = refs[1];
            }
        }
        assert!(super::super::kind::expected_locus_refs(&invalid.objects, 19, 22).is_none());
    }
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn nested_locus_sampling_clears_local_frozen_dependents_and_retains_external_inputs() {
    let bytes =
        fs::read("tests/fixtures/locus_drivers60_curve_segment_coarse_native_saved.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    let frozen = original.add(Kind::Free(Point { x: 510., y: 410. }));
    let seed = original.add(Kind::Free(Point { x: 520., y: 420. }));
    let parameter = original
        .add_named_parameter("external", 2., (10, 510))
        .unwrap();
    let number = original
        .add_parameter_calculation(parameter, "external*3", (10, 540))
        .unwrap();
    let baseline = original.locus_segments(23, 9).unwrap();
    let saved = original.encode().unwrap();
    let mut scoped = original.clone();
    scoped
        .freeze_scoped_evaluation_inputs(&original, &[12, 13, 14, 22, frozen, number])
        .unwrap();
    scoped
        .set_evaluation_point(19, Point { x: 900., y: 900. })
        .unwrap();
    scoped
        .set_evaluation_point(seed, Point { x: 610., y: 510. })
        .unwrap();
    scoped.set_evaluation_scalar(parameter, 7.).unwrap();
    assert_eq!(scoped.locus_segments(23, 9), Some(baseline));
    drivers61_close(scoped.position(frozen), original.position(frozen));
    drivers61_close(scoped.position(seed), Some(Point { x: 610., y: 510. }));
    assert_eq!(scoped.numeric_value(number), Some(6.));
    assert_eq!(scoped.numeric_value(parameter), Some(7.));
    assert_eq!(original.numeric_value(parameter), Some(2.));
    assert_eq!(original.encode().unwrap(), saved);
    assert_eq!(scoped.encode().unwrap(), saved);
}

#[test]
fn polygon_locus_source_edit_undo_delete_and_reopen_follow_the_live_driver() {
    let bytes =
        fs::read("tests/fixtures/locus_drivers60_polygon_unequal_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let baseline = d.position(31);
    d.checkpoint();
    let Kind::Polygon(ref vertices) = d.objects[10].kind else {
        panic!()
    };
    let vertex = vertices[1];
    assert!(d.move_point(vertex, Point { x: 420., y: 280. }));
    assert_ne!(d.position(31), baseline);
    let edited = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    drivers61_close(edited.position(31), d.position(31));
    d.undo();
    drivers61_close(d.position(31), baseline);
    d.checkpoint();
    d.selected.clear();
    d.selected.insert(11);
    d.clear_selected().unwrap();
    assert!(d.objects[18].deleted);
    assert!(d.objects[30].deleted);
    let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
}
