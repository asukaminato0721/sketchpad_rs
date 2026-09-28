use super::*;
fn close(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
        "{a:?} != {b:?}"
    );
}
fn point_samples(d: &Document, id: usize, dx: f64) {
    let s = d.locus_segments(id, 8).unwrap();
    assert_eq!(s.len(), 8);
    for (i, (a, b)) in s.iter().enumerate() {
        let p = |k: usize| Point {
            x: 200. + dx * (0.2 + 2. * k as f64 / 8.),
            y: 200.,
        };
        close(*a, p(i));
        close(*b, p(i + 1));
    }
    assert!((d.numeric_value(6).unwrap() - 0.7).abs() < 1e-12);
}
#[test]
fn native_numeric_dag_loci_match_live_point_and_sweep_geometry() {
    for (bytes, id, dx) in [
        (
            include_bytes!("../../../../../tests/fixtures/locus_numeric53_point_native_saved.gsp")
                .as_slice(),
            10,
            100.,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/locus_numeric53_point_edited_native_saved.gsp")
                .as_slice(),
            10,
            150.,
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        assert_eq!(
            d.objects[id - 1].kind,
            Kind::Locus(vec![9, 3, 4, 5, 6, 9], 500)
        );
        point_samples(&d, id, dx);
    }
    let bytes = include_bytes!("../../../../../tests/fixtures/locus_numeric53_sweep_native_saved.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    assert_eq!(d.encode().unwrap(), bytes);
    let s = d.locus_strokes(13, 4).unwrap();
    assert_eq!(s.len(), 5);
    for (i, (a, b, m)) in s.iter().enumerate() {
        let r = 0.2 + 2. * i as f64 / 4.;
        close(
            *a,
            Point {
                x: 200. + 100. * r,
                y: 200.,
            },
        );
        close(
            *b,
            Point {
                x: 200. + 100. * r,
                y: 200. + 50. * r,
            },
        );
        assert_eq!(*m, LineMode::Segment);
    }
}
#[test]
fn numeric_locus_source_edits_undo_and_deletion_follow_shared_dependencies() {
    let bytes = include_bytes!("../../../../../tests/fixtures/locus_numeric53_point_native_saved.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    d.checkpoint();
    assert!(d.move_point(8, Point { x: 350., y: 200. }));
    point_samples(&d, 10, 150.);
    d.undo();
    point_samples(&d, 10, 100.);
    d.redo();
    point_samples(&d, 10, 150.);
    let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    point_samples(&reopened, 10, 150.);
    d.selected.insert(5);
    let removed = d.clear_selected().unwrap();
    for id in [5, 6, 9, 10] {
        assert!(removed.contains(&id));
        assert!(d.objects[id - 1].deleted);
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    point_samples(&d, 10, 150.);
}
fn authored(sweep: bool) -> Document {
    let mut d = Document::new();
    for p in [(100., 300.), (300., 300.)] {
        d.add(Kind::Free(Point { x: p.0, y: p.1 }));
    }
    d.add(Kind::Segment(1, 2));
    d.add(Kind::PointOnSegment(3, 0.25));
    d.add(Kind::ValueEquation {
        kind: ValueEquationKind::PointValue,
        first: 4,
        second: 3,
    });
    d.add_parameter_calculation(5, "2*v+0.2", (10, 40)).unwrap();
    d.add(Kind::Free(Point { x: 200., y: 200. }));
    d.add(Kind::Free(Point { x: 300., y: 200. }));
    d.add_calculated_ratio_dilation(8, 7, 6).unwrap();
    if sweep {
        d.add(Kind::Free(Point { x: 300., y: 250. }));
        d.add_calculated_ratio_dilation(10, 7, 6).unwrap();
        d.add(Kind::Segment(9, 11));
    }
    d
}
#[test]
fn public_numeric_dag_locus_authoring_retains_full_refs_and_strict_reopens() {
    for sweep in [false, true] {
        let mut d = authored(sweep);
        let target = if sweep { 12 } else { 9 };
        let id = d.construct_locus(4, target).unwrap();
        let refs = if sweep {
            vec![12, 3, 4, 5, 6, 9, 11, 12]
        } else {
            vec![9, 3, 4, 5, 6, 9]
        };
        assert_eq!(
            d.objects[id - 1].kind,
            Kind::Locus(refs, if sweep { 250 } else { 500 })
        );
        assert!(d.construct_locus(4, 8).is_err());
        let b = d.encode().unwrap();
        let r = Document::from_bytes(&b, None).unwrap();
        assert!(!r.partial_view);
        assert_eq!(r.objects[id - 1].kind, d.objects[id - 1].kind);
        assert_eq!(r.encode().unwrap(), b);
        if !sweep {
            point_samples(&r, id, 100.);
        } else {
            assert_eq!(r.locus_strokes(id, 4).unwrap().len(), 5);
        }
    }
}
#[test]
fn malformed_numeric_locus_refs_remain_opaque_exact() {
    let original = include_bytes!("../../../../../tests/fixtures/locus_numeric53_point_input.gsp");
    let d = Document::from_bytes(original, None).unwrap();
    let rs = records(original).unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == d.objects[9].header_offset.unwrap())
        .unwrap();
    let r = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    for (position, value) in [(4, 8u32), (16, 8), (20, 10)] {
        let mut b = original.to_vec();
        b[r.data.start + position..r.data.start + position + 4]
            .copy_from_slice(&value.to_le_bytes());
        let invalid = Document::from_bytes(&b, None).unwrap();
        assert_eq!(invalid.objects[9].kind, Kind::Unsupported(35));
        assert_eq!(invalid.encode().unwrap(), b);
    }
}
