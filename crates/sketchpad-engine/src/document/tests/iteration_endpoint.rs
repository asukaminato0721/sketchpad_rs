use super::*;

fn native(state: &str) -> Vec<u8> {
    fs::read(format!(
        "../../tests/fixtures/iteration_endpoint55_{state}_native_saved.gsp"
    ))
    .unwrap()
}
fn authored(state: &str) -> Vec<u8> {
    fs::read(format!(
        "../../tests/fixtures/iteration_endpoint55_api_{state}_input.gsp"
    ))
    .unwrap()
}
fn close(actual: Option<Point>, x: f64, y: f64) {
    let p = actual.unwrap();
    assert!(
        (p.x - x).abs() < 1e-9 && (p.y - y).abs() < 1e-9,
        "{p:?} != ({x},{y})"
    );
}
fn group(bytes: &[u8], d: &Document, id: usize) -> Vec<u8> {
    let rs = records(bytes).unwrap();
    let i = rs
        .iter()
        .position(|r| Some(r.data.start) == d.objects[id - 1].header_offset)
        .unwrap();
    let j = (i..rs.len()).find(|j| rs[*j].kind == 0x07d7).unwrap();
    bytes[rs[i].start..rs[j].data.end].to_vec()
}
#[test]
fn native_iteration_endpoint_matches_terminal_scope_zero_and_undefined_final() {
    for (state, x) in [
        ("depth0", 200.),
        ("depth1", 300.),
        ("depth3", 500.),
        ("final_only", 500.),
        ("edited", 640.),
    ] {
        let bytes = native(state);
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(
            d.partial_view,
            "original graft retains an out-of-table font selector: {state}"
        );
        assert_eq!(d.objects[10].kind, Kind::IterationPointAlias(10));
        close(d.position(11), x, 180.);
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        if state == "depth0" {
            assert!(d.point_iteration_images(10).is_none());
        }
    }
    for state in ["undefined_final", "undefined_final_only"] {
        let bytes = native(state);
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(d.partial_view);
        assert_eq!(d.objects[9].kind, Kind::IterationPointAlias(9));
        assert!(d.position(10).is_none());
        assert!(d.line_points(11).is_none());
        let prefix = d.point_iteration_images(9).unwrap();
        assert_eq!(prefix.len(), if state.ends_with("only") { 0 } else { 2 });
        if prefix.len() == 2 {
            close(Some(prefix[0]), 225., 300.);
            close(Some(prefix[1]), 300., 300.);
        }
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
    }
}
#[test]
fn iteration_endpoint_public_writer_live_edits_undo_and_deletion() {
    let bytes = authored("depth3");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let retained = group(&bytes, &d, 11);
    d.checkpoint();
    assert!(d.move_point(1, Point { x: 100., y: 180. }));
    assert!(d.move_point(2, Point { x: 140., y: 180. }));
    assert!(d.move_point(6, Point { x: 130., y: 180. }));
    close(d.position(11), 640., 180.);
    let edited = d.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(group(&edited, &reopened, 11), retained);
    d.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(11), 500., 180.);
    d.redo();
    close(d.position(11), 640., 180.);
    d.set_parameter_value(8, 0.).unwrap();
    close(d.position(11), 250., 180.);
    assert!(d.point_iteration_images(10).is_none());
    d.set_parameter_value(8, -0.5).unwrap();
    assert!(d.position(11).is_none());
    d.set_parameter_value(8, 1.).unwrap();
    close(d.position(11), 380., 180.);
    let id = d.add_iteration_point_alias(10).unwrap();
    assert_eq!(d.objects[id - 1].kind, Kind::IterationPointAlias(10));
    close(d.position(id), 380., 180.);
    d.verify_roundtrip().unwrap();
    let encoded = d.encode().unwrap();
    let rd = Document::from_bytes(&encoded, None).unwrap();
    close(rd.position(id), 380., 180.);
    assert!(d.add_iteration_point_alias(7).is_err());
    d.selected.insert(8);
    let deleted = d.clear_selected().unwrap();
    for n in [8, 9, 10, 11, 12, id] {
        assert!(deleted.contains(&n));
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    close(d.position(id), 380., 180.);
}
#[test]
fn iteration_endpoint_retains_callers_depth_seed_and_external_overrides() {
    let d = Document::from_bytes(&authored("depth3"), None).unwrap();
    let mut scoped = d.clone();
    scoped.freeze_scoped_evaluation_inputs(&d, &[]).unwrap();
    scoped
        .set_evaluation_point(4, Point { x: 200., y: 180. })
        .unwrap();
    scoped
        .set_evaluation_point(6, Point { x: 140., y: 180. })
        .unwrap();
    scoped.set_evaluation_number(8, 1.).unwrap();
    close(scoped.position(11), 480., 180.);
    // The intrinsic point also participates as a point seed of an existing
    // stored single-application mapping without enabling nested root decoding.
    let mut d = d;
    let replacement = d.add(Kind::Free(Point { x: 250., y: 220. }));
    let mapped = d.add_mapped_element(11, 11, replacement, &[11]).unwrap();
    close(d.position(mapped), 250., 220.);
    d.verify_roundtrip().unwrap();
}
#[test]
fn iteration_endpoint_rejects_wrong_wire_roles_payload_and_branches() {
    let bytes = authored("depth3");
    let d = Document::from_bytes(&bytes, None).unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs
        .iter()
        .position(|r| Some(r.data.start) == d.objects[10].header_offset)
        .unwrap();
    let h = rs[i].data.start;
    let r = rs[i..]
        .iter()
        .find(|r| r.kind == 0x07d2)
        .unwrap()
        .data
        .start;
    for (offset, value, width) in [
        (h + 6, 2u32, 2usize),
        (h + 8, 89, 2),
        (r, 2, 4),
        (r + 4, 7, 4),
        (r + 4, 11, 4),
    ] {
        let mut bad = bytes.clone();
        bad[offset..offset + width].copy_from_slice(&value.to_le_bytes()[..width]);
        let parsed = Document::from_bytes(&bad, None).unwrap();
        assert_eq!(parsed.objects[10].kind, Kind::Unsupported(88));
        assert_eq!(parsed.encode().unwrap(), bad);
    }
    let end = (i..rs.len()).find(|j| rs[*j].kind == 0x07d7).unwrap();
    for payload in [vec![], vec![1], vec![88, 0, 0, 0]] {
        let mut bad = bytes[..rs[end].start].to_vec();
        put_record(&mut bad, 0x07d3, &payload);
        bad.extend_from_slice(&bytes[rs[end].start..]);
        let parsed = Document::from_bytes(&bad, None).unwrap();
        assert_eq!(parsed.objects[10].kind, Kind::Unsupported(88));
        assert_eq!(parsed.encode().unwrap(), bad);
    }
    let mut bad = d;
    if let Kind::MappedPointIteration { branches, .. } = &mut bad.objects[8].kind {
        branches.push(vec![7]);
    }
    assert!(bad.iteration_endpoint(11).is_none());
    assert!(bad.verify_roundtrip().is_err());
}
#[test]
fn iteration_endpoint_existing_multi_page_edits_preserve_native_group() {
    let bytes = authored("depth3");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let original = group(&bytes, &d, 11);
    d.add_blank_page().unwrap();
    let bytes = d.encode().unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    assert!(d.move_point(6, Point { x: 130., y: 180. }));
    close(d.position(11), 620., 180.);
    d.verify_roundtrip().unwrap();
    let encoded = d.encode().unwrap();
    let rd = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(group(&encoded, &rd, 11), original);
}
