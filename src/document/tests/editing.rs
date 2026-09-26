use super::*;

#[test]
fn invalid_ref() {
    let mut d = Document::new();
    d.add(Kind::Segment(1, 2));
    assert!(d.encode().is_err());
}
#[test]
fn hidden_point_matches_original_sketchpad_bytes() {
    let mut d = Document::open(Path::new("tests/fixtures/point.gsp")).unwrap();
    d.selected.insert(1);
    d.set_selected_visibility(false);
    assert_eq!(
        d.encode().unwrap(),
        fs::read("tests/fixtures/point_hidden.gsp").unwrap()
    );
    d.undo();
    assert!(d.objects[0].visible);
}
#[test]
fn pages_and_selection_metadata() {
    let tools = Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
    assert_eq!(tools.page_count, 10);
    assert!(tools.objects.iter().any(|o| o.page == 9));
    assert!(tools.partial_view);
    let mut segment = Document::open(Path::new("tests/fixtures/segment.gsp")).unwrap();
    segment.add(Kind::Free(Point { x: 1., y: 2. }));
    let bytes = segment.encode().unwrap();
    let rs = records(&bytes).unwrap();
    let last_group = rs.iter().rposition(|r| r.kind == 0x7d7).unwrap();
    let selected = rs.iter().position(|r| r.kind == 0x2328).unwrap();
    assert!(last_group < selected);
    assert_eq!(Document::from_bytes(&bytes, None).unwrap().objects.len(), 4);
}
#[test]
fn derived_lines_update() {
    let mut d = Document::new();
    let a = d.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = d.add(Kind::Free(Point { x: 10., y: 0. }));
    let c = d.add(Kind::Free(Point { x: 1., y: 5. }));
    let line = d.add(Kind::Line(a, b));
    let parallel = d.add(Kind::Parallel(c, line));
    let perpendicular = d.add(Kind::Perpendicular(c, line));
    assert_eq!(
        d.line_points(parallel),
        Some((
            Point { x: 1., y: 5. },
            Point { x: 11., y: 5. },
            LineMode::Line
        ))
    );
    assert_eq!(
        d.line_points(perpendicular),
        Some((
            Point { x: 1., y: 5. },
            Point { x: 1., y: 15. },
            LineMode::Line
        ))
    );
    d.move_point(b, Point { x: 0., y: 10. });
    assert_eq!(d.line_points(parallel).unwrap().1, Point { x: 1., y: 15. });
    let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.line_points(parallel), d.line_points(parallel));
}
#[test]
fn save_reopen_edit_and_save_again() {
    let p = std::env::temp_dir().join(format!("sketchpad_rs_test_{}.gsp", std::process::id()));
    let mut d = Document::new();
    let id = d.add(Kind::Free(Point { x: 12., y: 34. }));
    d.save_to(&p).unwrap();
    assert!(d.move_point(id, Point { x: 56., y: 78. }));
    d.save_to(&p).unwrap();
    let r = Document::open(&p).unwrap();
    assert_eq!(r.position(id), Some(Point { x: 56., y: 78. }));
    fs::remove_file(p).unwrap();
}
#[test]
fn constrained_point_can_move_after_first_save() {
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_constraint_save_{}.gsp",
        std::process::id()
    ));
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let segment = doc.add(Kind::Segment(a, b));
    let constrained = doc.add(Kind::PointOnSegment(segment, 0.25));
    doc.save_to(&path).unwrap();
    assert!(doc.move_point(constrained, Point { x: 75., y: 10. }));
    doc.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    assert_eq!(
        reopened.position(constrained),
        Some(Point { x: 75., y: 0. })
    );
    fs::remove_file(path).unwrap();
}
#[test]
fn invalid_coordinates_do_not_replace_a_file() {
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_invalid_save_{}.gsp",
        std::process::id()
    ));
    fs::write(&path, b"existing content").unwrap();
    let mut document = Document::new();
    document.add(Kind::Free(Point { x: f64::NAN, y: 1. }));
    assert!(document.save_to(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"existing content");
    fs::remove_file(path).unwrap();
}
#[test]
fn sketchpad_saved_edit_updates_dependents() {
    let document = Document::open(Path::new("tests/fixtures/wine_moved_demo.gsp")).unwrap();
    assert_eq!(document.objects.len(), 9);
    assert_eq!(document.position(1), Some(Point { x: 261., y: 181. }));
    assert_eq!(document.position(5), Some(Point { x: 370.5, y: 270.5 }));
    assert_eq!(
        document.line_points(8).unwrap().0,
        document.position(1).unwrap()
    );
}
#[test]
fn unknown_page_record_blocks_structural_save() {
    let mut bytes = fs::read("tests/fixtures/point.gsp").unwrap();
    let trailer = records(&bytes)
        .unwrap()
        .iter()
        .find(|record| record.kind == 0x3e9)
        .unwrap()
        .start;
    let mut unknown = Vec::new();
    put_record(&mut unknown, 0x4444, &[1, 2, 3, 4]);
    bytes.splice(trailer..trailer, unknown);
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    document.add(Kind::Free(Point { x: 4., y: 5. }));
    assert!(document.encode().is_err());
}
#[test]
fn forward_reference_is_preserved_without_evaluation() {
    let mut source = Document::new();
    let a = source.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = source.add(Kind::Free(Point { x: 10., y: 0. }));
    source.add(Kind::Segment(a, b));
    let mut bytes = source.encode().unwrap();
    let references = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x7d2)
        .unwrap();
    bytes[references.data.start + 8..references.data.start + 12]
        .copy_from_slice(&3_u32.to_le_bytes());
    let document = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(document.objects[2].kind, Kind::Unsupported(2));
    assert!(document.partial_view);
    assert!(document.line_points(3).is_none());
    assert_eq!(document.encode().unwrap(), bytes);
}
#[test]
fn straight_intersections_update_and_reopen() {
    let source = Document::open(Path::new("tests/fixtures/line_intersection.gsp")).unwrap();
    assert_eq!(source.objects[6].kind, Kind::Intersection(3, 6));
    assert!(source.position(7).is_some());
    assert!(!source.partial_view);

    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = document.add(Kind::Free(Point { x: 10., y: 0. }));
    let c = document.add(Kind::Free(Point { x: 5., y: -5. }));
    let d = document.add(Kind::Free(Point { x: 5., y: 5. }));
    let horizontal = document.add(Kind::Segment(a, b));
    let vertical = document.add(Kind::Segment(c, d));
    let crossing = document.add(Kind::Intersection(horizontal, vertical));
    assert_eq!(document.position(crossing), Some(Point { x: 5., y: 0. }));
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(crossing), document.position(crossing));
    document.move_point(d, Point { x: 5., y: -1. });
    assert_eq!(document.position(crossing), None);
}
