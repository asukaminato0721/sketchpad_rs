use super::*;

fn tool_records(bytes: &[u8]) -> Vec<u8> {
    let entries = records(bytes).unwrap();
    let start = entries
        .iter()
        .position(|record| record.kind == 0x03ea)
        .unwrap();
    let end = entries[start..]
        .iter()
        .position(|record| record.kind == 0x03eb)
        .unwrap()
        + start;
    bytes[entries[start].start..entries[end].data.end].to_vec()
}

#[test]
fn embedded_tool_namespace_is_separate_from_native_drawing_objects() {
    let original = include_bytes!("../../../../../tests/fixtures/embedded_tool_12t_original.gsp");
    let doc = Document::from_bytes(original, None).unwrap();
    assert_eq!(doc.page_count, 1);
    assert_eq!(doc.objects.len(), 145);
    assert_eq!(doc.source_count, 145);
    assert!(doc.partial_view);
    let tools = doc.embedded_tools().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name.as_deref(), Some("工具#1"));
    assert_eq!(tools[0].object_codes.len(), 44);
    assert!(tools[0].complete);
    assert_eq!(tools[0].input_ids.as_deref(), Some(&[5, 2, 3, 4, 1][..]));
    let tool_start = records(original)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x03ea)
        .unwrap()
        .start;
    assert!(
        doc.objects
            .iter()
            .all(|o| o.header_offset.unwrap() < tool_start)
    );
    assert_eq!(doc.position(1), Some(Point { x: 191., y: 491. }));
    assert_eq!(doc.encode().unwrap(), original);
    doc.verify_roundtrip().unwrap();

    let square = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_square_point_scene_original_native_verified.gsp"
    );
    let square_doc = Document::from_bytes(square, None).unwrap();
    assert_eq!(square_doc.objects.len(), 1);
    let square_tools = square_doc.embedded_tools().unwrap();
    assert_eq!(
        square_tools[0].name.as_deref(),
        Some("Square Given Two Vertices")
    );
    assert_eq!(square_tools[0].object_codes.len(), 13);
    assert_eq!(square_tools[0].input_ids.as_deref(), Some(&[2, 1][..]));
    assert_eq!(square_doc.encode().unwrap(), square);
}

#[test]
fn supported_scene_edits_preserve_embedded_tool_records_and_undo() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_point_scene_original_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(original, None).unwrap();
    let tools = tool_records(original);
    let summaries = doc.embedded_tools().unwrap();
    doc.checkpoint();
    assert!(doc.move_point(1, Point { x: 130., y: 150. }));
    for object in &mut doc.objects {
        object.visible = object.id == 1;
    }
    let edited = doc.encode().unwrap();
    assert_eq!(
        &edited,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_point_scene_edited_native_verified.gsp"
        )
    );
    assert_eq!(tool_records(&edited), tools);
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.objects.len(), 1);
    assert_eq!(reopened.position(1), Some(Point { x: 130., y: 150. }));
    assert_eq!(reopened.embedded_tools().unwrap(), summaries);
    doc.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_SCOPE_FIXTURE").is_some() {
        fs::write("/tmp/embedded_tool_scene_edited_port.gsp", &edited).unwrap();
    }
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), edited);
}

#[test]
fn drawing_font_lookup_excludes_tool_font_tables() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_point_scene_original_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(original, None).unwrap();
    assert_eq!(doc.text_font_families(), Some(vec!["Tahoma".to_owned()]));
    doc.objects[0].label =
        Some(crate::text_records::LabelAux::new_point("A", true, (0, 0)).unwrap());
    assert_eq!(doc.text_font_state(1).unwrap().family, "Tahoma");

    // An invalid drawing style must not resolve to the tool's first local style.
    let entries = records(original).unwrap();
    let terminator = entries.iter().find(|r| r.kind == 0x07d7).unwrap().start;
    let mut extra = Vec::new();
    put_record(&mut extra, 0x07d8, &1u16.to_le_bytes());
    let mut malformed = original.to_vec();
    malformed.splice(terminator..terminator, extra);
    let mut doc = Document::from_bytes(&malformed, None).unwrap();
    doc.objects[0].label =
        Some(crate::text_records::LabelAux::new_point("A", true, (0, 0)).unwrap());
    assert!(doc.text_font_state(1).is_none());
}

#[test]
fn malformed_tool_input_bindings_do_not_become_drawing_objects() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_point_scene_original_native_verified.gsp"
    );
    let record = records(original)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x04b4)
        .unwrap();
    let mut malformed = original.to_vec();
    malformed[record.data.start + 4..record.data.start + 8].copy_from_slice(&45u32.to_le_bytes());
    let doc = Document::from_bytes(&malformed, None).unwrap();
    assert_eq!(doc.objects.len(), 1);
    assert!(doc.embedded_tools().unwrap()[0].input_ids.is_none());
    assert_eq!(doc.encode().unwrap(), malformed);
    malformed[record.data.start + 4..record.data.start + 8].copy_from_slice(&2u32.to_le_bytes());
    let doc = Document::from_bytes(&malformed, None).unwrap();
    assert!(doc.embedded_tools().unwrap()[0].input_ids.is_none());
}

#[test]
fn native_tool_invocation_materializes_a_separate_drawing_graph() {
    let registered = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_square_registered_native_verified.gsp"
    );
    let control = Document::from_bytes(registered, None).unwrap();
    assert_eq!(control.objects.len(), 1);
    assert_eq!(
        u32::from_le_bytes(registered[16..20].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(registered[36..40].try_into().unwrap()),
        3
    );
    assert_eq!(
        control.embedded_tools().unwrap()[0].input_ids.as_deref(),
        Some(&[2, 1][..])
    );
    assert_eq!(control.encode().unwrap(), registered);

    let native =
        include_bytes!("../../../../../tests/fixtures/embedded_tool_square_invoked_native_verified.gsp");
    let doc = Document::from_bytes(native, None).unwrap();
    assert_eq!(doc.objects.len(), 14);
    assert_eq!(doc.position(2), Some(Point { x: 200., y: 180. }));
    assert_eq!(doc.position(3), Some(Point { x: 260., y: 180. }));
    assert_eq!(doc.position(8), Some(Point { x: 200., y: 120. }));
    assert_eq!(doc.position(10), Some(Point { x: 260., y: 120. }));
    assert_eq!(doc.objects[13].kind, Kind::Polygon(vec![2, 3, 10, 8]));
    for id in [5, 6, 7, 9] {
        assert!(!doc.objects[id - 1].visible);
    }
    // Native replaces the local Arial/Times fonts with Tahoma when invoking
    // this tool. Prototype objects, bindings and comments remain identical.
    let prototype_records = |bytes: &[u8]| {
        let entries = records(bytes).unwrap();
        let start = entries.iter().position(|r| r.kind == 0x03ea).unwrap();
        entries[start..]
            .iter()
            .take_while(|r| r.kind != 0x03eb)
            .filter(|r| !matches!(r.kind, 0x04b1 | 0x04b3))
            .map(|r| (r.kind, bytes[r.data.clone()].to_vec()))
            .collect::<Vec<_>>()
    };
    assert_eq!(prototype_records(native), prototype_records(registered));
    assert_eq!(
        doc.embedded_tools().unwrap(),
        control.embedded_tools().unwrap()
    );
    assert_eq!(doc.encode().unwrap(), native);
    doc.verify_roundtrip().unwrap();
}
