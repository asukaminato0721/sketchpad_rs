use super::*;
#[test]
fn native_mapped_prefix61_preserves_flag_word_and_defined_and_undefined_outputs() {
    for state in ["baseline", "undefined_final"] {
        let mut baseline = None;
        for flag in [0, 1] {
            let bytes = fs::read(format!("../../tests/fixtures/mapped_prefix60_{state}_{flag}_native_saved.gsp")).unwrap();
            let d = Document::from_bytes(&bytes, None).unwrap();
            assert!(!d.partial_view);
            assert!(matches!(d.objects[13].kind, Kind::MappedPointIteration { .. }));
            let points = d.point_iteration_images(15).unwrap();
            let expected = if state == "baseline" {
                vec![Point { x: 108.88888888888889, y: 182.22222222222223 }, Point { x: 135., y: 190. }, Point { x: 300., y: 240. }]
            } else { vec![Point { x: 120., y: 185. }, Point { x: 240., y: 220. }] };
            assert_eq!(points.len(), expected.len());
            for (actual, expected) in points.iter().zip(expected) { assert!(actual.distance(expected) < 1e-9); }
            assert_eq!(d.position(17).is_some(), state == "baseline");
            assert_eq!(d.line_points(18).is_some(), state == "baseline");
            assert_eq!(d.encode().unwrap(), bytes);
            let values = (points, d.position(17), d.line_points(18));
            if let Some(base) = &baseline { assert_eq!(&values, base); } else { baseline = Some(values); }
            let mut edited = d.clone();
            edited.checkpoint();
            assert!(edited.move_point(1, Point { x: 130., y: 180. }));
            let encoded = edited.encode().unwrap();
            let reopened = Document::from_bytes(&encoded, None).unwrap();
            assert!(!reopened.partial_view);
            let rs = records(&encoded).unwrap();
            let at = reopened.objects[13].header_offset.unwrap();
            let i = rs.iter().position(|r| r.data.start == at).unwrap();
            let r = rs[i..].iter().find(|r| r.kind == 0x090a).unwrap();
            assert_eq!(&encoded[r.data.start..r.data.start+4], &[0, 0, flag, 0]);
            edited.undo();
            assert_eq!(edited.encode().unwrap(), bytes);
        }
    }
}
#[test]
fn mapped_prefix61_rejects_unproved_reserved_bits_byte_exactly() {
    let bytes = fs::read("../../tests/fixtures/mapped_prefix60_baseline_1_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let rs = records(&bytes).unwrap();
    let at = d.objects[13].header_offset.unwrap();
    let i = rs.iter().position(|r| r.data.start == at).unwrap();
    let r = rs[i..].iter().find(|r| r.kind == 0x090a).unwrap();
    for word in [[1, 0, 1, 0], [0, 0, 2, 0], [0, 0, 0, 1]] {
        let mut b = bytes.clone();
        b[r.data.start..r.data.start+4].copy_from_slice(&word);
        let parsed = Document::from_bytes(&b, None).unwrap();
        assert_eq!(parsed.objects[13].kind, Kind::Unsupported(89));
        assert_eq!(parsed.encode().unwrap(), b);
    }
}
