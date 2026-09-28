use super::*;

fn preserved_tools(bytes: &[u8]) -> Vec<u8> {
    let entries = records(bytes).unwrap();
    let drawing: std::collections::BTreeSet<_> =
        super::super::embedded_tools::document_record_indices(&entries)
            .into_iter()
            .collect();
    entries
        .iter()
        .enumerate()
        .filter(|(index, _)| !drawing.contains(index))
        .flat_map(|(index, record)| {
            let end = entries
                .get(index + 1)
                .map_or(bytes.len(), |next| next.start);
            bytes[record.start..end].iter().copied()
        })
        .collect()
}

#[test]
fn registered_tools_survive_drawing_additions_deletions_and_parent_edits() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_square_registered_native_verified.gsp"
    );
    let mut document = Document::from_bytes(original, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.source_count, 1);
    document.can_save().unwrap();
    let tools = preserved_tools(original);
    let second = document.add(Kind::Free(Point { x: 260., y: 180. }));
    document.add(Kind::Segment(1, second));
    assert!(document.move_point(1, Point { x: 200., y: 180. }));
    let added = document.encode().unwrap();
    assert_eq!(
        added,
        include_bytes!("../../../../../tests/fixtures/registered_tool_drawing_added_native_verified.gsp")
    );
    assert_eq!(preserved_tools(&added), tools);
    let reopened = Document::from_bytes(&added, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 3);
    assert_eq!(reopened.position(1), Some(Point { x: 200., y: 180. }));
    assert_eq!(reopened.position(2), Some(Point { x: 260., y: 180. }));
    document.verify_roundtrip().unwrap();
    // Removing the original first object forces drawing ID compaction.
    document.selected.insert(1);
    document.clear_selected().unwrap();
    let deleted = document.encode().unwrap();
    assert_eq!(
        deleted,
        include_bytes!(
            "../../../../../tests/fixtures/registered_tool_drawing_deleted_native_verified.gsp"
        )
    );
    assert_eq!(preserved_tools(&deleted), tools);
    let reopened = Document::from_bytes(&deleted, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 1);
    assert_eq!(reopened.position(1), Some(Point { x: 260., y: 180. }));
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_STRUCTURE_FIXTURES").is_some() {
        fs::write("/tmp/registered_tool_drawing_added_port.gsp", added).unwrap();
        fs::write("/tmp/registered_tool_drawing_deleted_port.gsp", deleted).unwrap();
    }
}

#[test]
fn malformed_or_unregistered_tool_scopes_stay_structurally_read_only() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_square_registered_native_verified.gsp"
    );
    let entries = records(original).unwrap();
    let header = entries.iter().find(|r| r.kind == 0x0384).unwrap();
    let start = entries.iter().find(|r| r.kind == 0x03ea).unwrap();
    let end = entries.iter().find(|r| r.kind == 0x03eb).unwrap();
    let mut cases = Vec::new();
    for (offset, value) in [
        (header.data.start + 4, 0u32),
        (header.data.start + 4, 2),
        (header.data.start + 24, 2),
        (start.data.start + 24, 12),
        (start.data.start + 24, 14),
    ] {
        let mut bytes = original.to_vec();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        cases.push(bytes);
    }
    let references = entries
        .iter()
        .find(|r| r.kind == 0x07d2 && r.start > start.start)
        .unwrap();
    let mut bad_references = original.to_vec();
    bad_references[references.data.start..references.data.start + 4]
        .copy_from_slice(&99u32.to_le_bytes());
    cases.push(bad_references);
    let inputs = entries
        .iter()
        .find(|r| r.kind == 0x04b4 && r.start > start.start)
        .unwrap();
    let mut bad_inputs = original.to_vec();
    bad_inputs[inputs.data.start + 4..inputs.data.start + 8].copy_from_slice(&14u32.to_le_bytes());
    cases.push(bad_inputs);
    let section_end = entries
        .iter()
        .find(|r| r.kind == 0x0387 && r.start > end.start)
        .unwrap();
    let mut missing_section_end = original.to_vec();
    missing_section_end.drain(section_end.start..section_end.data.end);
    cases.push(missing_section_end);
    let mut missing_end = original.to_vec();
    missing_end.drain(end.start..end.data.end);
    cases.push(missing_end);
    let mut nested = original.to_vec();
    nested.splice(
        start.data.end..start.data.end,
        original[start.start..start.data.end].iter().copied(),
    );
    cases.push(nested);
    for bytes in cases {
        let Ok(mut document) = Document::from_bytes(&bytes, None) else {
            continue;
        };
        assert!(document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        document.add(Kind::Free(Point { x: 320., y: 180. }));
        assert!(document.encode().is_err());
    }
    let orphan = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_point_scene_original_native_verified.gsp"
    );
    let mut document = Document::from_bytes(orphan, None).unwrap();
    assert!(document.partial_view);
    assert!(document.move_point(1, Point { x: 130., y: 150. }));
    assert_eq!(
        document.encode().unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_point_scene_edited_native_verified.gsp"
        )
    );
}

#[test]
fn native_intersection_point_captions_keep_reference_before_label_order() {
    let square =
        include_bytes!("../../../../../tests/fixtures/embedded_tool_square_invoked_native_verified.gsp");
    let imported = Document::from_bytes(square, None).unwrap();
    assert_eq!(
        imported.objects[7]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "A"
    );
    assert_eq!(
        imported.objects[9]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "B"
    );
    assert_eq!(imported.encode().unwrap(), square);
    let mut document = Document::new();
    let center = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let radius = document.add(Kind::Free(Point { x: 260., y: 180. }));
    let circle = document.add(Kind::Circle(center, radius));
    let start = document.add(Kind::Free(Point { x: 100., y: 180. }));
    let end = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let line = document.add(Kind::Line(start, end));
    let crossing = document.add(Kind::LineCircleIntersection(line, circle, 0));
    document.objects[crossing - 1].label =
        Some(crate::text_records::LabelAux::new_point("Q", true, (6, 8)).unwrap());
    for id in [center, radius, start, end] {
        document.objects[id - 1].visible = false;
    }
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!("../../../../../tests/fixtures/labeled_line_circle_branch0_native_verified.gsp")
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.position(crossing),
        Some(Point { x: 260., y: 180. })
    );
    assert_eq!(
        reopened.objects[crossing - 1]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "Q"
    );
    document.verify_roundtrip().unwrap();
}
