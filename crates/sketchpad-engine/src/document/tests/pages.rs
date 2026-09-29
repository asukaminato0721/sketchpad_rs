use super::*;

#[test]
fn selected_page_persists_without_content_edits_and_native_bytes_match() {
    let native =
        include_bytes!("../../../../../tests/fixtures/two_page_selection_native_verified.gsp");
    let mut document = Document::from_bytes(native, None).unwrap();
    assert_eq!(document.active_page, 1);
    assert!(!document.dirty);
    assert_eq!(document.encode().unwrap(), native);
    document.active_page = 0;
    let first_page = document.encode().unwrap();
    assert!(!document.dirty);
    assert_eq!(first_page, expected_page(native, 0));
    assert_eq!(
        Document::from_bytes(&first_page, None).unwrap().active_page,
        0
    );
    document.active_page = 1;
    assert_eq!(document.encode().unwrap(), native);
    let path = std::env::temp_dir().join(format!(
        "sketchpad_page_selection_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(Document::open(&path).unwrap().active_page, 1);
    assert_eq!(std::fs::read(&path).unwrap(), native);
    std::fs::remove_file(path).unwrap();
    document.active_page = 2;
    assert!(document.encode().is_err());
}

#[test]
fn selected_page_tracks_content_changes_and_page_undo_redo() {
    let mut document = Document::new();
    document.add_blank_page().unwrap();
    assert_eq!(document.active_page, 1);
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .active_page,
        1
    );
    document.undo();
    assert_eq!(document.page_count, 1);
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .active_page,
        0
    );
    document.redo();
    assert_eq!(document.page_count, 2);
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .active_page,
        1
    );
    let point = document.add(Kind::Free(Point { x: 100., y: 180. }));
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.active_page, 1);
    assert_eq!(reopened.position(point), Some(Point { x: 100., y: 180. }));
    document.active_page = 0;
    document.add(Kind::Free(Point { x: 200., y: 180. }));
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .active_page,
        0
    );
}

#[test]
fn blank_pages_roundtrip_with_geometry_and_undo() {
    let native = include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp");
    let native_records = records(native).unwrap();
    let second_page = native_records
        .iter()
        .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
        .nth(1)
        .unwrap()
        .start;
    let end = native_records.last().unwrap().start;
    assert_eq!(
        &native[second_page..end],
        include_bytes!("../../../../../assets/blank_page_records.bin")
    );
    let mut document = Document::new();
    document.add(Kind::Free(Point { x: 120., y: 180. }));
    assert_eq!(document.add_blank_page(), Ok(2));
    assert_eq!(document.active_page, 1);
    document.add(Kind::Free(Point { x: 240., y: 280. }));
    assert_eq!(document.add_blank_page(), Ok(3));
    document.undo();
    assert_eq!(document.page_count, 2);
    assert_eq!(document.active_page, 1);
    document.redo();
    assert_eq!(document.page_count, 3);
    let encoded = document.encode().unwrap();
    let mut reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.page_count, 3);
    assert_eq!(reopened.active_page, 2);
    assert_eq!(reopened.objects.len(), 2);
    assert_eq!(reopened.objects[0].page, 0);
    assert_eq!(reopened.objects[1].page, 1);
    assert_eq!(reopened.objects[0].kind, document.objects[0].kind);
    assert_eq!(reopened.objects[1].kind, document.objects[1].kind);
    assert_eq!(reopened.encode().unwrap(), encoded);
    assert_eq!(reopened.add_blank_page(), Ok(4));
    reopened.add(Kind::Free(Point { x: 200., y: 160. }));
    let four_pages = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(four_pages.page_count, 4);
    assert_eq!(four_pages.active_page, 3);
    assert_eq!(four_pages.objects[2].page, 3);
}
#[test]
fn native_two_page_document_can_gain_a_page() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    assert_eq!(document.add_blank_page(), Ok(3));
    document.add(Kind::Free(Point { x: 310., y: 220. }));
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.page_count, 3);
    assert_eq!(reopened.objects.len(), 2);
    assert_eq!(reopened.objects[0].page, 0);
    assert_eq!(reopened.objects[1].page, 2);
    assert_eq!(reopened.objects[1].kind, document.objects[1].kind);
}
#[test]
fn page_names_roundtrip_and_undo() {
    let native = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_review_native.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(native.page_names, vec![None, Some("Review".into())]);
    let native = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/three_page_alpha_native.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(native.page_names, vec![None, None, Some("Alpha".into())]);
    let mut document = Document::new();
    document.set_page_name(0, "Front").unwrap();
    assert_eq!(document.page_names, vec![Some("Front".into())]);
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    document.undo();
    assert_eq!(document.page_names, vec![None]);
    document.redo();
    assert_eq!(document.page_names, vec![Some("Front".into())]);
    document.add_blank_page().unwrap();
    document.set_page_name(1, "Review").unwrap();
    let named_bytes = document.encode().unwrap();
    let review = records(&named_bytes)
        .unwrap()
        .into_iter()
        .find(|record| {
            record.kind == 0x044c && named_bytes[record.data.clone()].starts_with(b"Review")
        })
        .unwrap();
    assert_eq!(&named_bytes[review.data], b"Review\0\0");
    let named = Document::from_bytes(&named_bytes, None).unwrap();
    assert_eq!(
        named.page_names,
        vec![Some("Front".into()), Some("Review".into())]
    );
    let mut renamed = named.clone();
    renamed.set_page_name(0, "Overview").unwrap();
    renamed.set_page_name(1, "").unwrap();
    let reopened = Document::from_bytes(&renamed.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.page_names, vec![Some("Overview".into()), None]);
    assert_eq!(reopened.page_count, 2);
}
#[test]
fn removing_pages_preserves_survivors_and_undo() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.remove_page(1).unwrap();
    assert_eq!(document.page_count, 1);
    assert_eq!(document.objects.len(), 1);
    assert_eq!(document.objects[0].page, 0);
    let removed = document.encode().unwrap();
    assert_eq!(removed.len(), 1836);
    assert_eq!(Document::from_bytes(&removed, None).unwrap().page_count, 1);
    document.undo();
    assert_eq!(document.page_count, 2);
    assert_eq!(document.encode().unwrap(), base);
    document.redo();
    assert_eq!(document.encode().unwrap(), removed);

    let mut document = Document::from_bytes(base, None).unwrap();
    document.remove_page(0).unwrap();
    assert_eq!(document.page_count, 1);
    assert!(document.objects.is_empty());
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .page_count,
        1
    );

    let mut document = Document::new();
    document.add(Kind::Free(Point { x: 120., y: 180. }));
    document.add_blank_page().unwrap();
    document.add(Kind::Free(Point { x: 240., y: 280. }));
    document.add_blank_page().unwrap();
    document.add(Kind::Free(Point { x: 320., y: 220. }));
    document.set_page_name(2, "Last").unwrap();
    document.remove_page(1).unwrap();
    assert_eq!(document.page_count, 2);
    assert_eq!(document.page_names, vec![None, Some("Last".into())]);
    assert_eq!(document.objects.len(), 2);
    assert_eq!(document.objects[0].page, 0);
    assert_eq!(document.objects[1].page, 1);
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects.len(), 2);
    document.undo();
    assert_eq!(document.page_count, 3);
    assert_eq!(document.objects.len(), 3);
    document.redo();
    assert_eq!(document.page_count, 2);
}
#[test]
fn removing_page_remaps_surviving_page_links() {
    let mut document = Document::new();
    document.add_blank_page().unwrap();
    document.add_blank_page().unwrap();
    document.active_page = 0;
    document.add_link_page_button(2).unwrap();
    document.remove_page(1).unwrap();
    assert_eq!(document.page_count, 2);
    assert!(matches!(
        document.objects[0].kind,
        Kind::LinkPageButton { target_page: 1, .. }
    ));
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(matches!(
        reopened.objects[0].kind,
        Kind::LinkPageButton { target_page: 1, .. }
    ));
    let before = document.encode().unwrap();
    document.remove_page(1).unwrap();
    assert_eq!(document.page_count, 1);
    assert!(document.partial_view);
    assert!(matches!(
        document.objects[0].kind,
        Kind::LinkPageButton {
            target_page: 0xbd66,
            ..
        }
    ));
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .objects[0]
            .kind,
        document.objects[0].kind
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
}
#[test]
fn removing_link_destination_invalidates_link_like_native_sketchpad() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/one_page_removed_targeted_link_native.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/two_page_removed_targeted_link_native.gsp")
            .as_slice(),
    ] {
        let native = Document::from_bytes(bytes, None).unwrap();
        assert!(native.partial_view);
        assert!(matches!(
            native.objects[1].kind,
            Kind::LinkPageButton {
                target_page: 0xbd66,
                ..
            }
        ));
        assert_eq!(native.encode().unwrap(), bytes);
        let action = records(bytes)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x0906 && record.data.len() == 24)
            .unwrap();
        assert_eq!(
            &bytes[action.data.start + 14..action.data.start + 16],
            &0xbd66u16.to_le_bytes()
        );
    }
    let mut document = Document::new();
    document.add(Kind::Free(Point { x: 120., y: 180. }));
    document.add_blank_page().unwrap();
    document.add(Kind::Free(Point { x: 240., y: 180. }));
    document.add_blank_page().unwrap();
    document.add(Kind::Free(Point { x: 320., y: 180. }));
    document.active_page = 0;
    document.add_link_page_button(1).unwrap();
    let before = document.encode().unwrap();
    document.remove_page(1).unwrap();
    assert_eq!(document.page_count, 2);
    assert!(matches!(
        document.objects[1].kind,
        Kind::LinkPageButton {
            target_page: 0xbd66,
            ..
        }
    ));
    assert_eq!(document.position(3), Some(Point { x: 320., y: 180. }));
    let generated = document.encode().unwrap();
    let action = records(&generated)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x0906 && record.data.len() == 24)
        .unwrap();
    assert_eq!(
        &generated[action.data.start + 14..action.data.start + 16],
        &0xbd66u16.to_le_bytes()
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    assert_eq!(document.encode().unwrap(), generated);
    document.move_page(0, 1).unwrap();
    assert!(matches!(
        document.objects[2].kind,
        Kind::LinkPageButton {
            target_page: 0xbd66,
            ..
        }
    ));
}
#[test]
fn moving_pages_preserves_geometry_names_and_links() {
    let base = include_bytes!("../../../../../tests/fixtures/three_page_from_new_port.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.move_page(2, 0).unwrap();
    assert_eq!(document.active_page, 0);
    assert_eq!(document.page_count, 3);
    assert_eq!(
        document
            .objects
            .iter()
            .map(|object| object.page)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        document.objects[0].kind,
        Kind::Free(Point { x: 320., y: 220. })
    );
    assert_eq!(
        document.objects[1].kind,
        Kind::Free(Point { x: 120., y: 180. })
    );
    assert_eq!(
        document.objects[2].kind,
        Kind::Free(Point { x: 240., y: 280. })
    );
    let reordered = document.encode().unwrap();
    document.undo();
    assert_eq!(document.encode().unwrap(), base);
    document.redo();
    assert_eq!(document.encode().unwrap(), reordered);

    let mut document = Document::new();
    document.set_page_name(0, "First").unwrap();
    document.add_blank_page().unwrap();
    document.set_page_name(1, "Second").unwrap();
    document.add_blank_page().unwrap();
    document.set_page_name(2, "Third").unwrap();
    document.active_page = 0;
    document.add_link_page_button(2).unwrap();
    document.move_page(2, 0).unwrap();
    assert_eq!(
        document.page_names,
        vec![
            Some("Third".into()),
            Some("First".into()),
            Some("Second".into())
        ]
    );
    assert_eq!(document.objects[0].page, 1);
    assert!(matches!(
        document.objects[0].kind,
        Kind::LinkPageButton { target_page: 0, .. }
    ));
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.page_names, document.page_names);
    assert!(matches!(
        reopened.objects[0].kind,
        Kind::LinkPageButton { target_page: 0, .. }
    ));
}
#[test]
fn copying_native_pages_preserves_geometry_and_undo() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp");
    for (source_page, expected_kinds) in [(0, 1), (1, 3)] {
        let mut document = Document::from_bytes(base, None).unwrap();
        assert_eq!(document.copy_page(source_page), Ok(3));
        assert_eq!(document.active_page, 2);
        assert_eq!(document.objects.len(), 4 + expected_kinds);
        let copied = document.encode().unwrap();
        let reopened = Document::from_bytes(&copied, None).unwrap();
        assert_eq!(reopened.page_count, 3);
        assert_eq!(reopened.objects.len(), 4 + expected_kinds);
        assert_eq!(reopened.encode().unwrap(), copied);
        document.undo();
        assert_eq!(document.encode().unwrap(), base);
        document.redo();
        assert_eq!(document.encode().unwrap(), copied);
    }
}
#[test]
fn copying_page_preserves_name_and_page_link_target() {
    let mut document = Document::new();
    document.set_page_name(0, "Source").unwrap();
    document.add_blank_page().unwrap();
    document.active_page = 0;
    document.add_link_page_button(1).unwrap();
    assert_eq!(document.copy_page(0), Ok(3));
    assert_eq!(
        document.page_names,
        vec![Some("Source".into()), None, Some("Source".into())]
    );
    assert_eq!(document.objects.len(), 2);
    assert_eq!(document.objects[1].page, 2);
    assert!(matches!(
        document.objects[1].kind,
        Kind::LinkPageButton { target_page: 1, .. }
    ));
    let bytes = document.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(reopened.page_names, document.page_names);
    assert_eq!(reopened.objects[1].kind, document.objects[1].kind);
}

#[test]
fn copying_and_reordering_parameter_table_pages_preserves_local_sources() {
    let bytes = include_bytes!("../../../../../tests/fixtures/two_page_parameter_tables_port.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(
        document
            .objects
            .iter()
            .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
    );
    let original = document.encode().unwrap();
    let count = document.objects.len();
    document.copy_page(1).unwrap();
    assert_eq!(document.page_count, 3);
    let copied = document.encode().unwrap();
    let reopened = Document::from_bytes(&copied, None).unwrap();
    assert_eq!(reopened.objects, document.objects);
    for object in document.objects.iter().filter(|object| object.page == 2) {
        let source = &document.objects[object.id - count / 2 - 1];
        assert_eq!(source.page, 1);
        assert_eq!(
            document.numeric_value(object.id),
            document.numeric_value(source.id)
        );
        match (&source.kind, &object.kind) {
            (Kind::ValueTable(_, before), Kind::ValueTable(_, after))
            | (Kind::ValueTablePair(_, _, before), Kind::ValueTablePair(_, _, after)) => {
                assert_eq!(before.as_bytes(), after.as_bytes())
            }
            _ => {}
        }
    }
    document.undo();
    assert_eq!(document.encode().unwrap(), original);
    document.redo();
    assert_eq!(document.encode().unwrap(), copied);
    document.move_page(2, 0).unwrap();
    let moved = document.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&moved, None).unwrap().objects,
        document.objects
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), copied);
}
