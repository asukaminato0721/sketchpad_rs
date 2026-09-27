use super::*;

#[test]
fn native_lone_numeric_coordinate_point_clipboard_freezes_only_the_selected_point() {
    let source = include_bytes!("../../../tests/fixtures/numeric_calculation_trailing_decimal_cube84_native_verified.gsp");
    let native = Document::from_bytes(include_bytes!("../../../tests/fixtures/numeric_calculation_cube84_point_clipboard_native_verified.gsp"), None).unwrap();
    let mut document = Document::from_bytes(source, None).unwrap();
    document.selected.insert(14);
    let original_position = document.position(14).unwrap();
    let clipboard = document.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 1);
    assert_eq!(document.paste(&clipboard).unwrap(), vec![15]);
    assert_eq!(document.objects[14].kind, Kind::Free(Point {
        x: original_position.x + 20., y: original_position.y + 20.,
    }));
    assert_eq!(document.position(15), native.position(15));
    assert_eq!(document.objects[14].visual_style, native.objects[14].visual_style);
    assert_eq!(document.objects[14].visible, native.objects[14].visible);
    assert!(!document.objects[14].presentation_suppressed);
    let bytes = document.encode().unwrap();
    assert_eq!(bytes, include_bytes!("../../../tests/fixtures/numeric_coordinate_point_clipboard_writer_native_verified.gsp"));
    if std::env::var_os("SKETCHPAD_WRITE_CLIPBOARD_FIXTURE").is_some() {
        fs::write("/tmp/numeric_coordinate_point_clipboard_port.gsp", &bytes).unwrap();
    }
    let saved = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(saved.position(15), native.position(15));
    assert_eq!(saved.objects[14].visual_style, native.objects[14].visual_style);
    assert_eq!(saved.objects.len(), 15);
    document.undo();
    assert_eq!(document.objects.len(), 14);
    document.redo();
    assert_eq!(document.objects.len(), 15);
    let copied_position = document.position(15);
    document.set_parameter_value(2, 2.).unwrap();
    assert_ne!(document.position(14), Some(original_position));
    assert_eq!(document.position(15), copied_position);
}

#[test]
fn native_selected_numeric_graph_clipboard_keeps_live_coordinate_points() {
    let mut document = Document::from_bytes(include_bytes!("../../../tests/fixtures/numeric_calculation_trailing_decimal_cube84_native_verified.gsp"), None).unwrap();
    document.selected.extend(1..=14);
    let clipboard = document.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 14);
    assert_eq!(document.paste(&clipboard).unwrap().len(), 14);
    assert!(matches!(document.objects[27].kind, Kind::NumericCoordinatePoint(..)));
    assert_eq!(document.objects[18].kind.dependencies(), vec![16]);
    assert!(!document.objects[18].presentation_suppressed);
    assert!(!document.objects[18].visible);
    assert_eq!(document.objects.len(), 28);
    document.encode().unwrap();
}

#[test]
fn native_two_numeric_point_selection_copies_only_two_free_positions() {
    let mut document = Document::from_bytes(include_bytes!("../../../tests/fixtures/numeric_calculation_trailing_decimal_cube84_native_verified.gsp"), None).unwrap();
    let native = Document::from_bytes(include_bytes!("../../../tests/fixtures/numeric_coordinate_two_points_clipboard_native_verified.gsp"), None).unwrap();
    document.selected.extend([13, 14]);
    let clipboard = document.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 2);
    assert_eq!(document.paste(&clipboard).unwrap(), vec![15, 16]);
    let positions = |document: &Document| {
        let mut points = [document.position(15).unwrap(), document.position(16).unwrap()];
        points.sort_by(|a, b| a.x.total_cmp(&b.x));
        points
    };
    assert_eq!(positions(&document), positions(&native));
    for id in [15, 16] {
        assert!(matches!(document.objects[id - 1].kind, Kind::Free(_)));
        assert_eq!(document.objects[id - 1].visual_style, native.objects[id - 1].visual_style);
    }
    document.encode().unwrap();
}

#[test]
fn native_midpoint_alone_clipboard_freezes_the_position_and_drops_its_host() {
    let mut document = Document::from_bytes(include_bytes!("../../../tests/fixtures/embedded_tool_given_selection_source_native_verified.gsp"), None).unwrap();
    let native = Document::from_bytes(include_bytes!("../../../tests/fixtures/midpoint_alone_clipboard_native_verified.gsp"), None).unwrap();
    document.selected.insert(4);
    let clipboard = document.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 1);
    assert_eq!(document.paste(&clipboard).unwrap(), vec![6]);
    assert_eq!(document.objects[5].kind, native.objects[5].kind);
    assert_eq!(document.objects[5].midpoint_host, None);
    assert_eq!(document.objects[5].visual_style, native.objects[5].visual_style);
    let bytes = document.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_CLIPBOARD_FIXTURE").is_some() {
        fs::write("/tmp/midpoint_alone_clipboard_port.gsp", bytes).unwrap();
    }
    let frozen = document.position(6);
    assert!(document.move_point(1, Point { x: 160., y: 140. }));
    assert_eq!(document.position(6), frozen);
}

#[test]
fn native_midpoint_selection_keeps_selected_ancestors_and_hides_only_unselected_host() {
    let source = include_bytes!("../../../tests/fixtures/embedded_tool_given_selection_source_native_verified.gsp");
    for (name, selection, fixture, host_visible) in [
        ("one_endpoint", vec![1, 4], include_bytes!("../../../tests/fixtures/midpoint_one_endpoint_clipboard_native_verified.gsp").as_slice(), false),
        ("both_endpoints", vec![1, 2, 4], include_bytes!("../../../tests/fixtures/midpoint_both_endpoints_clipboard_native_verified.gsp").as_slice(), false),
        ("segment", vec![3, 4], include_bytes!("../../../tests/fixtures/midpoint_segment_clipboard_native_verified.gsp").as_slice(), true),
    ] {
        let native = Document::from_bytes(fixture, None).unwrap();
        let mut document = Document::from_bytes(source, None).unwrap();
        document.selected.extend(selection);
        let clipboard = document.copy_selected().unwrap();
        assert_eq!(clipboard.len(), 4);
        assert_eq!(document.paste(&clipboard).unwrap(), vec![6, 7, 8, 9]);
        assert!(matches!(document.objects[8].kind, Kind::Midpoint(6, 7)));
        assert_eq!(document.objects[8].midpoint_host, Some(8));
        assert_eq!(document.position(9), native.position(9));
        assert_eq!(document.objects[7].visible, host_visible);
        assert_eq!(native.objects[7].visible, host_visible);
        assert!(!document.objects[7].presentation_suppressed);
        let bytes = document.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(reopened.objects[7].visible, host_visible);
        assert_eq!(reopened.position(9), document.position(9));
        if std::env::var_os("SKETCHPAD_WRITE_CLIPBOARD_FIXTURE").is_some() {
            fs::write(format!("/tmp/midpoint_{name}_clipboard_port.gsp"), bytes).unwrap();
        }
        assert!(document.move_point(6, Point { x: 240., y: 220. }));
        assert_eq!(document.position(9), Some(Point { x: 280., y: 210. }));
        assert_eq!(document.position(4), Some(Point { x: 250., y: 180. }));
    }
}
