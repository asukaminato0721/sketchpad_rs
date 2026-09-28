use super::*;
fn native(state: &str) -> Vec<u8> {
    fs::read(format!(
        "../../tests/fixtures/sampled_mapped60_{state}_native_saved.gsp"
    ))
    .unwrap()
}
fn close(p: Option<Point>, q: Point) {
    assert!(p.unwrap().distance(q) < 1e-8, "{p:?} vs {q:?}");
}
fn output(p: Point) -> Point {
    Point {
        x: 160. + (p.x - 100.).powi(2) / 100.,
        y: 120. + (p.y - 100.) * (p.x - 100.) / 100.,
    }
}
fn expected(state: &str, t: f64) -> Point {
    if state == "frozen" {
        return Point { x: 260., y: 200. };
    }
    let p = match state {
        "circle" => {
            let a = std::f64::consts::TAU * t;
            Point {
                x: 250. + 50. * a.cos(),
                y: 200. - 50. * a.sin(),
            }
        }
        "arc" => {
            let a = std::f64::consts::PI * t;
            Point {
                x: 250. + 50. * a.cos(),
                y: 200. - 50. * a.sin(),
            }
        }
        "segment_edited" => Point {
            x: 200. + 120. * t,
            y: 200. + 20. * t,
        },
        _ => Point {
            x: 200. + 100. * t,
            y: 200.,
        },
    };
    output(p)
}
#[test]
fn native_sampled_mapped_paths_and_normalized_children_preserve_all_eight_profiles() {
    for state in [
        "segment",
        "circle",
        "arc",
        "segment_edited",
        "frozen",
        "seed_edited",
        "constrained_quarter",
        "constrained_threequarters",
    ] {
        let bytes = native(state);
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        let Kind::SampledMapped(def) = &d.objects[15].kind else {
            panic!("{state}")
        };
        assert_eq!(def.output(), 15);
        assert_eq!(
            def.seed(),
            if state.starts_with("constrained_") {
                11
            } else {
                1
            }
        );
        assert_eq!(
            def.carried(),
            if state == "frozen" {
                &[15][..]
            } else {
                &[12, 15][..]
            }
        );
        for t in [0., 0.1, 0.25, 0.5, 0.75, 0.9, 1., 0.3141592653589793] {
            close(d.sampled_mapped_point(16, t), expected(state, t));
        }
        close(d.position(22), expected(state, 0.25));
        close(d.position(23), expected(state, 0.75));
        assert!(d.line_points(24).is_some());
        assert!(d.numeric_value(25).is_some());
        if state == "arc" {
            assert_eq!(d.sampled_mapped_point(16, -0.1), None);
            assert_eq!(d.sampled_mapped_point(16, 1.1), None);
        } else {
            close(d.sampled_mapped_point(16, -0.1), expected(state, -0.1));
            close(d.sampled_mapped_point(16, 1.1), expected(state, 1.1));
        }
        let mut evaluator = d.sampled_mapped_evaluator(16).unwrap();
        for t in [0.75, 0.25, 1., 0., 0.5] {
            close(evaluator.at(t), expected(state, t));
        }
        assert!(d.sampled_mapped_strokes(16, 16).unwrap().len() >= 15);
    }
}
#[test]
fn sampled_mapped_live_path_edit_seed_independence_and_scoped_freezing_are_distinct() {
    let bytes = native("segment");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let initial = d.sampled_mapped_point(16, 0.5).unwrap();
    d.checkpoint();
    assert!(d.move_point(1, Point { x: 240., y: 150. }));
    close(d.sampled_mapped_point(16, 0.5), initial);
    d.verify_roundtrip().unwrap();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    d.checkpoint();
    assert!(d.move_point(4, Point { x: 320., y: 220. }));
    close(d.position(22), expected("segment_edited", 0.25));
    d.verify_roundtrip().unwrap();
    let saved = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    close(
        saved.sampled_mapped_point(16, 0.75),
        expected("segment_edited", 0.75),
    );
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    let base = Document::from_bytes(&bytes, None).unwrap();
    let mut caller = base.clone();
    caller.freeze_scoped_evaluation_inputs(&base, &[]).unwrap();
    caller
        .set_evaluation_point(4, Point { x: 320., y: 220. })
        .unwrap();
    close(
        caller.sampled_mapped_point(16, 0.5),
        expected("segment_edited", 0.5),
    );
    let mut frozen = caller.clone();
    frozen
        .freeze_scoped_evaluation_inputs(&caller, &[16])
        .unwrap();
    frozen
        .set_evaluation_point(4, Point { x: 500., y: 400. })
        .unwrap();
    close(
        frozen.sampled_mapped_point(16, 0.5),
        expected("segment_edited", 0.5),
    );
    // A frozen intermediate stays frozen across nonmonotonic evaluator calls.
    let frozen = Document::from_bytes(&native("frozen"), None).unwrap();
    let mut eval = frozen.sampled_mapped_evaluator(16).unwrap();
    for t in [1., 0., 0.25, 0.75, 0.5] {
        close(eval.at(t), Point { x: 260., y: 200. });
    }
}
#[test]
fn sampled_mapped_public_authoring_metadata_and_large_sample_count_roundtrip() {
    let mut d = Document::new();
    let seed = d.add(Kind::Free(Point { x: 200., y: 180. }));
    let other = d.add(Kind::Free(Point { x: 100., y: 100. }));
    let a = d.add(Kind::Free(Point { x: 200., y: 200. }));
    let b = d.add(Kind::Free(Point { x: 300., y: 200. }));
    let path = d.add(Kind::Segment(a, b));
    let midpoint_host = d.add(Kind::Segment(seed, other));
    let target = d.add(Kind::Midpoint(seed, other));
    let trace = d
        .add_sampled_mapped(target, path, seed, &[midpoint_host, target], 50000)
        .unwrap();
    let child = d.add(Kind::PointOnCurve(trace, 0.3141592653589793));
    close(
        d.position(child),
        Point {
            x: 165.70796326794897,
            y: 150.,
        },
    );
    assert!(d.sampled_mapped_strokes(trace, 8).unwrap().len() <= 8);
    d.verify_roundtrip().unwrap();
    let saved = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    let Kind::SampledMapped(def) = &saved.objects[trace - 1].kind else {
        panic!()
    };
    assert_eq!(def.samples(), 50000);
    close(saved.position(child), d.position(child).unwrap());
    let mut range = *def.range_payload();
    range[24..32].copy_from_slice(&[0x17, 0x53, 0xff, 0x7f, 0x12, 0, 0, 0]);
    let another = d
        .add_sampled_mapped_with_range(target, path, seed, &[target], range)
        .unwrap();
    d.verify_roundtrip().unwrap();
    let saved = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    let Kind::SampledMapped(def) = &saved.objects[another - 1].kind else {
        panic!()
    };
    assert_eq!(def.range_payload(), &range);
    close(
        saved.sampled_mapped_point(another, 0.75),
        Point { x: 150., y: 140. },
    );
    assert!(d
        .add_sampled_mapped(target, path, seed, &[target, target], 500)
        .is_err());
    assert!(d
        .add_sampled_mapped(target, seed, seed, &[target], 500)
        .is_err());
    assert!(d
        .add_sampled_mapped(target, path, other, &[target], 1)
        .is_err());
    let refs = vec![target, path, seed, target];
    let definition =
        super::super::sampled_mapped::SampledMappedDefinition::from_native(&refs, &range).unwrap();
    let mut ids = (0..d.objects.len() + 1).collect::<Vec<_>>();
    ids.swap(seed, other);
    let remapped = definition.remap(&ids);
    assert_eq!(remapped.seed(), other);
    assert_eq!(
        remapped.native_references(),
        vec![target, path, other, target]
    );
    d.selected = [trace].into_iter().collect();
    assert!(d.clear_selected().unwrap().contains(&child));
    d.verify_roundtrip().unwrap();
    d.undo();
    assert!(matches!(d.objects[trace - 1].kind, Kind::SampledMapped(_)));
}

#[test]
fn sampled_mapping_keeps_unrelated_caller_replacement_when_it_is_carried() {
    let mut base = Document::new();
    let seed = base.add(Kind::Free(Point { x: 0., y: 0. }));
    let other = base.add(Kind::Free(Point { x: 100., y: 100. }));
    let a = base.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = base.add(Kind::Free(Point { x: 300., y: 180. }));
    let path = base.add(Kind::Segment(a, b));
    let host = base.add(Kind::Segment(seed, other));
    let output = base.add(Kind::Midpoint(seed, other));
    let trace = base.add_sampled_mapped(output, path, seed, &[other, host, output], 100).unwrap();
    close(base.sampled_mapped_point(trace, 0.), Point { x: 150., y: 140. });

    let mut caller = base.clone();
    caller.freeze_scoped_evaluation_inputs(&base, &[]).unwrap();
    caller.set_evaluation_point(seed, Point { x: 500., y: 500. }).unwrap();
    caller.set_evaluation_point(other, Point { x: 160., y: 140. }).unwrap();
    close(caller.sampled_mapped_point(trace, 0.), Point { x: 180., y: 160. });
    close(caller.position(other), Point { x: 160., y: 140. });
    close(caller.position(seed), Point { x: 500., y: 500. });
}
#[test]
fn malformed_sampled_mapped_range_source_and_carried_roles_remain_opaque() {
    let bytes = native("segment");
    let d = Document::from_bytes(&bytes, None).unwrap();
    let h = d.objects[15].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs.iter().position(|r| r.data.start == h).unwrap();
    let end = (i..rs.len()).find(|&j| rs[j].kind == 0x7d7).unwrap();
    let range = rs[i..=end].iter().find(|r| r.kind == 0x902).unwrap();
    let refs = rs[i..=end].iter().find(|r| r.kind == 0x7d2).unwrap();
    let mut cases = vec![];
    for (offset, value) in [
        (range.data.start + 16, 1u32),
        (range.data.start + 20, 4),
        (range.data.start + 32, 1),
        (range.data.start + 36, 1),
        (refs.data.start + 8, 1),
        (refs.data.start + 12, 5),
        (refs.data.start + 16, 15),
    ] {
        let mut b = bytes.clone();
        b[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        cases.push(b);
    }
    let mut nan = bytes.clone();
    nan[range.data.start..range.data.start + 8].copy_from_slice(&f64::NAN.to_le_bytes());
    cases.push(nan);
    let mut header = bytes.clone();
    header[h + 8..h + 10].copy_from_slice(&42u16.to_le_bytes());
    cases.push(header);
    for r in [range, refs] {
        let mut b = bytes.clone();
        b.splice(r.start..r.start, bytes[r.start..r.data.end].iter().copied());
        cases.push(b);
    }
    for b in cases {
        let p = Document::from_bytes(&b, None).unwrap();
        assert_eq!(p.objects[15].kind, Kind::Unsupported(102));
        assert_eq!(p.encode().unwrap(), b);
    }
}

#[test]
fn native_sampled_mapped_literal_extension_wrap_and_linear_arc_readout_are_distinct() {
    for state in ["segment", "circle", "arc", "linear"] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/sampled_mapped61_numeric_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.encode().unwrap(), bytes);
        let mut evaluator = d.sampled_mapped_evaluator(16).unwrap();
        for t in [0., 0.25, 0.5, 0.75, 1.] {
            let p = if state == "linear" {
                output(Point {
                    x: 300. - 100. * t,
                    y: 200.,
                })
            } else {
                expected(state, t)
            };
            close(evaluator.at(t), p);
        }
        assert_eq!(evaluator.logical_point_value(0.25), Some(0.25));
        match state {
            "segment" => {
                close(evaluator.at(1.25), expected(state, 1.25));
                close(d.position(32), expected(state, 1.25));
                assert_eq!(evaluator.logical_point_value(1.25), Some(1.));
                assert_eq!(evaluator.logical_point_value(-0.25), Some(0.));
            }
            "circle" => {
                close(evaluator.at(1.25), expected(state, 0.25));
                close(d.position(32), expected(state, 0.25));
                assert_eq!(evaluator.logical_point_value(1.25), Some(0.25));
            }
            _ => {
                assert_eq!(evaluator.at(1.25), None);
                assert_eq!(d.position(32), None);
                assert_eq!(evaluator.logical_point_value(1.25), None);
            }
        }
        assert_eq!(evaluator.at(f64::NAN), None);
        assert_eq!(evaluator.logical_point_value(f64::INFINITY), None);
    }
}

#[test]
fn native_sampled_mapped_numeric_children_are_normalized_and_strictly_bounded() {
    for state in ["segment", "circle", "arc", "linear"] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/sampled_mapped61_numeric_only_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.encode().unwrap(), bytes);
        for (id, t) in [(29, 0.25), (30, 0.75)] {
            let p = if state == "linear" {
                output(Point {
                    x: 300. - 100. * t,
                    y: 200.,
                })
            } else {
                expected(state, t)
            };
            close(d.position(id), p);
        }
        assert_eq!(d.position(31), None, "{state}: numeric outside");
        assert_eq!(d.numeric_value(36), Some(0.25));
        assert_eq!(
            d.numeric_value(37),
            match state {
                "segment" => Some(1.),
                "circle" => Some(0.25),
                _ => None,
            }
        );
    }

    let mut d = Document::new();
    let seed = d.add(Kind::Free(Point { x: 200., y: 180. }));
    let other = d.add(Kind::Free(Point { x: 100., y: 100. }));
    let a = d.add(Kind::Free(Point { x: 200., y: 200. }));
    let b = d.add(Kind::Free(Point { x: 300., y: 200. }));
    let path = d.add(Kind::Segment(a, b));
    let midpoint_host = d.add(Kind::Segment(seed, other));
    let target = d.add(Kind::Midpoint(seed, other));
    let trace = d
        .add_sampled_mapped(target, path, seed, &[midpoint_host, target], 500)
        .unwrap();
    let literal = d.add(Kind::PathLiteralPoint(trace, 0.75));
    let outside_literal = d.add(Kind::PathLiteralPoint(trace, 1.25));
    close(d.position(literal), Point { x: 187.5, y: 150. });
    assert_eq!(d.position(outside_literal), None);
    let q = d.add_named_parameter("q", 0.25, (30, 30)).unwrap();
    let child = d.add(Kind::PathValuePoint(q, trace));
    let readout = d.add(Kind::PointDistance(other, child));
    close(d.position(child), Point { x: 162.5, y: 150. });
    assert_eq!(
        d.numeric_value(readout),
        Some(Point { x: 100., y: 100. }.distance(Point { x: 162.5, y: 150. }) / 37.79527559055118)
    );
    d.verify_roundtrip().unwrap();
    d.set_parameter_value(q, 1.25).unwrap();
    assert_eq!(d.position(child), None);
    assert_eq!(d.numeric_value(readout), None);
    let saved = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    assert_eq!(saved.position(child), None);
    saved.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(child), Point { x: 162.5, y: 150. });
    d.set_parameter_value(q, 0.75).unwrap();
    close(d.position(child), Point { x: 187.5, y: 150. });
    assert_eq!(
        d.numeric_value(readout),
        Some(Point { x: 100., y: 100. }.distance(Point { x: 187.5, y: 150. }) / 37.79527559055118)
    );
    d.verify_roundtrip().unwrap();
}

#[test]
fn native_sampled_mapped_literal123_children_are_normalized_and_strictly_bounded() {
    for state in ["segment", "circle", "arc", "linear"] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/sampled_mapped61_literal_only_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.encode().unwrap(), bytes);
        for (id, t) in [(33, 0.25), (34, 0.75)] {
            let p = if state == "linear" {
                output(Point {
                    x: 300. - 100. * t,
                    y: 200.,
                })
            } else {
                expected(state, t)
            };
            close(d.position(id), p);
        }
        assert_eq!(d.position(35), None, "{state}: literal123 outside");
    }
}
