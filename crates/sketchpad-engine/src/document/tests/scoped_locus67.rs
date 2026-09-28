//! Register after scoped ordinary-Locus collection support lands.
use super::*;
fn native(extra: bool, state: &str) -> (Vec<u8>, Document) {
    let prefix = if extra { "scoped_locus67_extra" } else { "scoped_locus67" };
    let bytes = fs::read(format!("../../tests/fixtures/{prefix}_{state}_native_saved.gsp")).unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    (bytes, doc)
}
fn near(point: Point, expected: Point) {
    assert!((point.x - expected.x).abs() < 1e-8 && (point.y - expected.y).abs() < 1e-8, "{point:?} vs{expected:?}");
}
fn expected(factor: f64, t: f64, host_dx: f64, shifted: bool) -> (Point, Point) {
    let (dx, dy) = if shifted { (30., -20.) } else { (0., 0.) };
    (Point { x: 100. + (100. + host_dx*t)*factor + dx, y: 100. + 80.*factor + dy },
     Point { x: 100. + 130.*factor + dx, y: 100. + 120.*factor + dy })
}
fn check(doc: &Document, binding: usize, factors: &[f64], host_dx: f64, gap: bool, rgb: bool, shifted: bool) {
    let strokes = doc.mapped_collection_strokes(binding, 4).unwrap();
    assert_eq!(strokes.len(), factors.len() * if gap { 4 } else { 5 });
    let mut at = 0;
    for (outer, factor) in factors.iter().enumerate() {
        for (local, t) in [0_f64, 0.25, 0.5, 0.75, 1.].into_iter().enumerate() {
            if gap && local == 2 { continue; }
            let factor = if gap { factor*(0.03/((t-0.5)*(t-0.5))+0.5) } else { *factor };
            let (a,b) = expected(factor, t, host_dx, shifted);
            let stroke = &strokes[at]; at += 1;
            near(stroke.start, a); near(stroke.end, b);
            assert_eq!(stroke.mode, LineMode::Segment);
            let VisualStyle::Stroke { line, color } = stroke.style else { panic!("collection member must retain stroke style") };
            assert_eq!(line.width, LineWidth::Thin);
            assert_eq!(line.pattern, LinePattern::Solid);
            if rgb {
                let expected: [u8; 3] = [([102,153,204][outer]), [25,70,114,159,204][local], 51];
                // Native t1 green204 versus binary-float floor203 is a bounded
                // palette rounding difference, not permission to inherit black.
                for (actual,want) in [color.red,color.green,color.blue].into_iter().zip(expected) {
                    assert!((i16::from(actual)-i16::from(want)).abs() <= 1, "{color:?} vs{expected:?}");
                }
            } else {
                assert_eq!(color, RgbColor { red:0, green:0, blue:0 });
            }
        }
    }
}
#[test]
fn native_scoped_locus_families_minimal_capture_depth_and_direct_wrapper_style_are_exact() {
    for (extra,state,binding,factors,gap,rgb,shifted) in [
        (false,"live",18,vec![2.,3.,4.],false,false,false),
        (false,"frozen",18,vec![2.,3.,4.],false,false,false),
        (false,"edited",18,vec![2.75,3.75,4.75],false,false,false),
        (false,"undefined_gap",18,vec![2.,3.,4.],true,false,false),
        (true,"colorized",22,vec![2.,3.,4.],false,true,false),
        (true,"transform_colorized",25,vec![2.,3.,4.],false,false,true),
        (true,"frozen_entry_edited",18,vec![2.75,3.75,4.75],false,false,false),
        (true,"frozen_depth1",18,vec![2.],false,false,false),
    ] {
        let (bytes,doc) = native(extra,state);
        assert!(!doc.partial_view, "{state}");
        check(&doc,binding,&factors,40.,gap,rgb,shifted);
        assert_eq!(doc.position(5),Some(Point{x:210.,y:180.}), "local driver leaked: {state}");
        assert_eq!(doc.encode().unwrap(),bytes,"{state}");
        doc.verify_roundtrip().unwrap();
    }
}
#[test]
fn minimal_carried_locus_replays_live_caller_and_external_edits_without_mutating_driver() {
    let (bytes,mut doc) = native(false,"frozen");
    doc.set_parameter_value(7,1.).unwrap();
    doc.set_parameter_value(9,1.5).unwrap();
    check(&doc,18,&[3.5,4.5,5.5],40.,false,false,false);
    doc.set_parameter_value(16,1.).unwrap();
    doc.checkpoint();
    assert!(doc.move_point(3,Point{x:260.,y:180.}));
    check(&doc,18,&[3.5],60.,false,false,false);
    assert_eq!(doc.position(5),Some(Point{x:215.,y:180.}));
    let saved=doc.encode().unwrap();
    let reopened=Document::from_bytes(&saved,None).unwrap();
    check(&reopened,18,&[3.5],60.,false,false,false);
    assert_eq!(reopened.encode().unwrap(),saved);
    doc.verify_roundtrip().unwrap();
    for _ in 0..4 { doc.undo(); }
    assert_eq!(doc.encode().unwrap(),bytes);
    check(&doc,18,&[2.,3.,4.],40.,false,false,false);
}
#[test]
fn undefined_local_member_does_not_join_neighbors_or_discard_other_outer_families() {
    let (_,doc)=native(false,"undefined_gap");
    let before=doc.encode().unwrap();
    check(&doc,18,&[2.,3.,4.],40.,true,false,false);
    // Stored4 gives quarter samples; this control explicitly checks its quarter-grid domain. Each
    // undefined slot is omitted as an independent straight, never a connecting line.
    let strokes=doc.mapped_collection_strokes(18,4).unwrap();
    assert_eq!(strokes.len(),12);
    assert!(strokes.iter().all(|s| s.start.x.is_finite() && s.end.y.is_finite()));
    assert_eq!(doc.position(5),Some(Point{x:210.,y:180.}));
    assert_eq!(doc.encode().unwrap(),before);
}

#[test]
fn native_public_final_only_collection_retains_the_last_sweep_and_direct_sample_colors() {
    for (state, source, root, binding, rgb) in [
        ("live", 15, 17, 18, false),
        ("colorized", 19, 21, 22, true),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/scoped_locus67_final_final_{state}_native_saved.gsp"
        )).unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        assert!(matches!(doc.objects[root - 1].kind,
            Kind::MappedPointIteration { final_only: true, .. }));
        assert_eq!(doc.objects[binding - 1].kind, Kind::IterationBinding(source, root));
        if rgb {
            let strokes = doc.mapped_collection_strokes(binding, 4).unwrap();
            assert_eq!(strokes.len(), 5);
            for (i, t) in [0., 0.25, 0.5, 0.75, 1.].into_iter().enumerate() {
                let (a, b) = expected(4., t, 40., false);
                near(strokes[i].start, a);
                near(strokes[i].end, b);
                let VisualStyle::Stroke { color, .. } = strokes[i].style else { panic!() };
                assert_eq!(color.red, 204);
                assert_eq!(color.blue, 51);
                assert!((i16::from(color.green) - [25, 70, 114, 159, 204][i]).abs() <= 1);
            }
        } else {
            check(&doc, binding, &[4.], 40., false, false, false);
        }
        assert_eq!(doc.position(5), Some(Point { x: 210., y: 180. }));
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}
