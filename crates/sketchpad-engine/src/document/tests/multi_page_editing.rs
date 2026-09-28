use super::*;

#[test]
fn multi_page_paste_remaps_dependencies_on_same_and_earlier_pages() {
    fn pasted(base: &[u8], selected: usize, destination: usize, expected: &[u8], count: usize) {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        document.selected.insert(selected);
        let clipboard = document.copy_selected().unwrap();
        document.active_page = destination;
        assert_eq!(document.paste(&clipboard).unwrap().len(), count);
        document.verify_roundtrip().unwrap();
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_paste_{}_{}_{}.gsp",
            std::process::id(),
            selected,
            destination
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            expected_page(expected, document.active_page)
        );
        assert_eq!(Document::from_bytes(expected, None).unwrap().page_count, 2);
        std::fs::remove_file(path).unwrap();
    }
    let segment_base = include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp");
    pasted(
        segment_base,
        4,
        1,
        include_bytes!("../../../../../tests/fixtures/two_page_pasted_segment_same_page_port.gsp"),
        3,
    );
    pasted(
        segment_base,
        4,
        0,
        include_bytes!("../../../../../tests/fixtures/two_page_pasted_segment_first_page_port.gsp"),
        3,
    );
    pasted(
        include_bytes!("../../../../../tests/fixtures/two_page_line_point_port.gsp"),
        6,
        0,
        include_bytes!("../../../../../tests/fixtures/two_page_pasted_line_point_first_page_port.gsp"),
        4,
    );
    pasted(
        include_bytes!("../../../../../tests/fixtures/two_page_one_host_measurements_port.gsp"),
        10,
        0,
        include_bytes!("../../../../../tests/fixtures/two_page_pasted_perimeter_first_page_port.gsp"),
        5,
    );

    let mut source = Document::new();
    let parameter = source.add_default_parameter((10, 5)).unwrap();
    source.selected.insert(parameter);
    let clipboard = source.copy_selected().unwrap();
    let mut destination = Document::from_bytes(segment_base, None).unwrap();
    destination.active_page = 1;
    assert!(destination.paste(&clipboard).is_err());
    assert_eq!(destination.objects.len(), 4);
    assert!(!destination.dirty);
}
#[test]
fn multi_page_clear_remaps_survivors_and_removes_dependents() {
    fn cleared(base: &[u8], page: usize, selected: usize, removed: &[usize], expected: &[u8]) {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = page;
        document.selected.insert(selected);
        assert_eq!(document.clear_selected().unwrap(), removed);
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_clear_{}_{}.gsp",
            std::process::id(),
            selected
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            expected_page(expected, document.active_page)
        );
        std::fs::remove_file(path).unwrap();
    }
    let segment = include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp");
    cleared(
        segment,
        1,
        4,
        &[4],
        include_bytes!("../../../../../tests/fixtures/two_page_segment_cleared_port.gsp"),
    );
    cleared(
        segment,
        1,
        2,
        &[2, 4],
        include_bytes!("../../../../../tests/fixtures/two_page_point_and_segment_cleared_port.gsp"),
    );
    cleared(
        include_bytes!("../../../../../tests/fixtures/two_page_pasted_segment_first_page_port.gsp"),
        0,
        1,
        &[1],
        include_bytes!("../../../../../tests/fixtures/two_page_first_point_cleared_port.gsp"),
    );
    cleared(
        include_bytes!("../../../../../tests/fixtures/two_page_one_host_measurements_port.gsp"),
        1,
        5,
        &[5, 10, 14],
        include_bytes!("../../../../../tests/fixtures/two_page_polygon_measurements_cleared_port.gsp"),
    );
    cleared(
        include_bytes!("../../../../../tests/fixtures/two_page_midpoint_circle_segment_mixed_port.gsp"),
        1,
        4,
        &[4, 5, 6],
        include_bytes!("../../../../../tests/fixtures/two_page_midpoint_circle_cleared_port.gsp"),
    );

    let mut document = Document::from_bytes(segment, None).unwrap();
    document.active_page = 1;
    document.selected.insert(4);
    document.clear_selected().unwrap();
    document.undo();
    assert_eq!(
        document.encode().unwrap(),
        expected_page(segment, document.active_page)
    );
    document.redo();
    assert_eq!(
        document.encode().unwrap(),
        expected_page(
            include_bytes!("../../../../../tests/fixtures/two_page_segment_cleared_port.gsp"),
            document.active_page
        )
    );

    let mut document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    document.active_page = 1;
    let first = document.add(Kind::Free(Point { x: 120., y: 180. }));
    let second = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let segment = document.add(Kind::Segment(first, second));
    document.selected.insert(first);
    assert_eq!(document.clear_selected().unwrap(), vec![first, segment]);
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_clear_new_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../../../tests/fixtures/two_page_unsaved_chain_cleared_port.gsp"),
            document.active_page
        )
    );
    std::fs::remove_file(path).unwrap();

    let mut document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp"),
        None,
    )
    .unwrap();
    document.active_page = 1;
    document.selected.insert(4);
    let clipboard = document.copy_selected().unwrap();
    assert_eq!(document.clear_selected().unwrap(), vec![4]);
    assert_eq!(document.paste(&clipboard).unwrap(), vec![5, 6, 7]);
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_cut_paste_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../../../tests/fixtures/two_page_cut_pasted_segment_port.gsp"),
            document.active_page
        )
    );
    std::fs::remove_file(path).unwrap();

    let source = include_bytes!("../../../../../tests/fixtures/action_link_page_port_generated.gsp");
    let mut document = Document::from_bytes(source, None).unwrap();
    document.selected.insert(1);
    assert!(document.clear_selected().is_err());
    assert_eq!(document.encode().unwrap(), source);
    assert!(!document.dirty);
}
#[test]
fn multi_page_state_writer_refuses_geometry_changes() {
    let base = include_bytes!(
        "../../../../../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
    );
    let mut document = Document::from_bytes(base, None).unwrap();
    document.checkpoint();
    document.objects[0].kind = Kind::Segment(1, 2);
    assert!(document.can_save().is_err());
    assert!(document.encode().is_err());
}

#[test]
fn multi_page_parameter_tables_keep_local_refs_rows_and_other_page_bytes() {
    let bytes = include_bytes!("../../../../../tests/fixtures/two_page_parameter_tables_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(doc.objects.len(), 8);
    assert!(
        doc.objects
            .iter()
            .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
    );
    doc.active_page = 0;
    let other_page = |bytes: &[u8]| {
        let rs = records(bytes).unwrap();
        let start = rs
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
            .nth(1)
            .unwrap()
            .start;
        let end = rs
            .iter()
            .find(|record| record.kind == 0x0385)
            .unwrap()
            .start;
        bytes[start..end].to_vec()
    };
    let untouched = other_page(bytes);
    let mut changed_caption = doc.clone();
    if let Kind::Parameter(group) = &mut changed_caption.objects[0].kind {
        group.set_anchor((200, 200)).unwrap();
    }
    let invalid_state = changed_caption.objects.clone();
    assert!(changed_caption.set_parameter_value(1, 2.).is_err());
    assert_eq!(changed_caption.objects, invalid_state);
    assert!(changed_caption.undo.is_empty());
    assert!(!changed_caption.dirty);
    let mut unknown = doc.clone();
    unknown.objects[7].kind = Kind::Unsupported(0xffff);
    assert!(unknown.set_parameter_value(1, 2.).is_err());
    assert!(!unknown.dirty);
    let old_pair = match &doc.objects[2].kind {
        Kind::ValueTablePair(2, 1, group) => group.pair_values().unwrap(),
        kind => panic!("Unexpected paired table: {kind:?}"),
    };
    let old_single = match &doc.objects[3].kind {
        Kind::ValueTable(1, group) => group.default_values().unwrap(),
        kind => panic!("Unexpected single table: {kind:?}"),
    };
    assert!(doc.set_parameter_value(5, 2.).is_err());
    assert!(!doc.dirty);
    doc.set_parameter_value(1, -12.34).unwrap();
    doc.set_parameter_value(2, 7.5).unwrap();
    if let Kind::ValueTablePair(first, second, group) = &doc.objects[2].kind {
        assert_eq!((*first, *second), (2, 1));
        assert_eq!(group.pair_source_ids(), Some([2, 1]));
        let rows = group.pair_values().unwrap();
        assert_eq!(&rows[..rows.len() - 1], &old_pair[..old_pair.len() - 1]);
        assert_eq!(*rows.last().unwrap(), [7.5, -12.34]);
    } else {
        panic!("Lost paired table");
    }
    if let Kind::ValueTable(source, group) = &doc.objects[3].kind {
        assert_eq!(*source, 1);
        assert_eq!(group.source_id(), 1);
        let rows = group.default_values().unwrap();
        assert_eq!(&rows[..rows.len() - 1], &old_single[..old_single.len() - 1]);
        assert_eq!(*rows.last().unwrap(), -12.34);
    } else {
        panic!("Lost single table");
    }
    let edited = doc.encode().unwrap();
    assert_eq!(other_page(&edited), untouched);
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.numeric_value(1), Some(-12.34));
    assert_eq!(reopened.numeric_value(2), Some(7.5));
    assert_eq!(reopened.numeric_value(5), Some(1.));
    assert_eq!(reopened.numeric_value(6), Some(1.));
    assert!(
        matches!(&reopened.objects[6].kind, Kind::ValueTablePair(6,5,group) if group.pair_source_ids()==Some([2,1]))
    );
    let state = doc.objects.clone();
    let history = doc.undo.len();
    assert!(doc.set_parameter_value(1, 100.).is_err());
    assert_eq!(doc.objects, state);
    assert_eq!(doc.undo.len(), history);
    doc.set_parameter_value(1, 8.).unwrap();
    assert!(
        matches!(&doc.objects[0].kind, Kind::Parameter(group) if group.precision()==2 && group.value_text()=="8.00")
    );
    assert_eq!(other_page(&doc.encode().unwrap()), untouched);
    doc.undo();
    assert_eq!(doc.numeric_value(1), Some(-12.34));
    doc.redo();
    assert_eq!(doc.numeric_value(1), Some(8.));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.numeric_value(1), Some(8.));
    assert_eq!(reopened.numeric_value(5), Some(1.));
}
