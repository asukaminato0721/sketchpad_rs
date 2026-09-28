use super::super::referenced_text::{ReferenceRole, ReferencedText};
use super::*;

fn native64(state: &str) -> (Vec<u8>, Document) {
    let bytes = fs::read(format!(
        "../../tests/fixtures/referenced_roles52_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    (bytes, doc)
}
fn rows64(doc: &Document, id: usize) -> String {
    doc.referenced_text_runs(id)
        .unwrap()
        .into_iter()
        .map(|row| row.into_iter().map(|run| run.text).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
fn compact64(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn native_text_and_function_roles_evaluate_live_sources_and_keep_cache_bytes() {
    for (state, plain, value, formula, expression) in [
        ("text_bare_baseline", "SOURCE_TEXT", "2.0", "f(x)=x²", "x²"),
        ("text_cached_edited", "EDIT_TEXT", "4.0", "f(x)=x", "x"),
    ] {
        let (bytes, doc) = native64(state);
        assert!(!doc.partial_view, "{state}");
        assert_eq!(doc.encode().unwrap(), bytes);
        for (row, expected) in [(0, plain.to_owned()), (1, format!("VALUE{value}"))] {
            for (col, role) in ["0", "A", "B", "E"].into_iter().enumerate() {
                let prefix = if row == 0 { "T" } else { "R" };
                assert_eq!(
                    compact64(&rows64(&doc, 12 + row * 4 + col)),
                    format!("{prefix}{role}:{expected}")
                );
            }
        }
        for (col, role, expected) in [
            (0, "0", formula),
            (1, "A", "f"),
            (2, "B", formula),
            (3, "E", expression),
        ] {
            assert_eq!(
                compact64(&rows64(&doc, 20 + col)),
                format!("F{role}:{expected}")
            );
        }
    }
}

#[test]
fn native_anchored_text_function_and_fraction_follow_both_parent_roles() {
    for (state, plain, value, formula) in [
        ("anchored_baseline", "SOURCE_TEXT", "2.0", "f(x)=x²"),
        ("anchored_edited", "EDIT_TEXT", "4.0", "f(x)=x"),
    ] {
        let (bytes, doc) = native64(state);
        assert!(!doc.partial_view, "{state}");
        assert_eq!(doc.encode().unwrap(), bytes);
        for (id, y, expected) in [
            (9, 100., format!("TEXT{plain}")),
            (10, 200., format!("REFVALUE{value}")),
            (11, 300., format!("FUNCTION{formula}")),
            (12, 400., format!("MATH{value}+(1)/(2)")),
        ] {
            let (point, text) = doc.point_numeric_label(id).unwrap();
            assert_eq!(point, Point { x: 250., y });
            assert_eq!(compact64(&text), expected);
            assert_eq!(
                doc.dependencies_of(id),
                Some(vec![
                    id - 8,
                    match id {
                        9 => 6,
                        10 => 7,
                        11 => 8,
                        _ => 5,
                    }
                ])
            );
        }
    }
}

#[test]
fn authored_anchored_text_tracks_sources_scopes_fonts_undo_and_dependency_deletion() {
    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let q = doc.add_named_parameter("q", 2., (10, 10)).unwrap();
    let source = doc
        .add_referenced_text(vec![q], "<H<T6xVALUE ><?x1>>", (20, 20))
        .unwrap();
    let attached = doc
        .add_point_referenced_text(point, source, "<H<T1x[><?1x2<T5xSTALE>><T1x]>>")
        .unwrap();
    let original = doc.encode().unwrap();
    assert_eq!(
        doc.point_numeric_label(attached),
        Some((Point { x: 100., y: 200. }, "[VALUE 2.0]".into()))
    );
    doc.set_parameter_value(q, 4.).unwrap();
    doc.checkpoint();
    assert!(doc.move_point(point, Point { x: 130., y: 230. }));
    let moved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&moved, None).unwrap();
    assert_eq!(
        reopened.point_numeric_label(attached),
        Some((Point { x: 130., y: 230. }, "[VALUE 4.0]".into()))
    );
    doc.undo();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);

    let mut scope = doc.clone();
    scope.freeze_evaluation_inputs(&doc, &[]).unwrap();
    scope.set_evaluation_scalar(q, 8.).unwrap();
    scope
        .set_evaluation_point(point, Point { x: 300., y: 400. })
        .unwrap();
    assert_eq!(
        scope.point_numeric_label(attached),
        Some((Point { x: 300., y: 400. }, "[VALUE 8.0]".into()))
    );
    assert_eq!(scope.encode().unwrap(), original);
    let mut frozen = doc.clone();
    frozen.freeze_evaluation_inputs(&doc, &[source]).unwrap();
    frozen.set_evaluation_scalar(q, 9.).unwrap();
    assert_eq!(
        frozen.point_numeric_label(attached).unwrap().1,
        "[VALUE 2.0]"
    );

    doc.selected.insert(attached);
    let old_bold = doc.numeric_label_font_state(attached).unwrap().bold;
    assert_eq!(
        doc.toggle_selected_text_style(TextStyleCommand::Bold)
            .unwrap(),
        1
    );
    doc.verify_roundtrip().unwrap();
    let styled = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        styled.numeric_label_font_state(attached).unwrap().bold,
        !old_bold
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.selected.clear();
    doc.selected.insert(q);
    assert_eq!(doc.clear_selected().unwrap(), vec![q, source, attached]);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn shared_references_reject_unknown_roles_broken_fraction_and_invalid_attachment_parents() {
    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 10., y: 20. }));
    let text = doc.add_text("SOURCE", Point { x: 20., y: 30. }).unwrap();
    let initial = doc.encode().unwrap();
    for markup in [
        "<?xD0002>",
        "<?x1E0002>",
        "<?xA0000>",
        "</<T1x1>>",
        "</<T1x1><T1x2><T1x3>>",
        "<R<?x2>>",
        "<H<?x2><!70>>",
    ] {
        assert!(
            doc.add_point_referenced_text(point, text, markup).is_err(),
            "{markup}"
        );
        assert_eq!(doc.encode().unwrap(), initial);
    }
    for (anchor, source, markup) in [
        (0, text, "<?x2>"),
        (point, 0, "<?x2>"),
        (text, point, "<?x2>"),
        (point, text, "<?x1>"),
    ] {
        assert!(
            doc.add_point_referenced_text(anchor, source, markup)
                .is_err()
        );
    }
    doc.objects[point - 1].page = 1;
    assert!(doc.add_point_referenced_text(point, text, "<?x2>").is_err());
    assert_eq!(
        ReferencedText::from_markup("<?xB0001>", (0, 0), 1)
            .unwrap()
            .referenced_roles(),
        [(1, ReferenceRole::Definition)]
    );
}

fn names_fixture64(state: &str) -> (Vec<u8>, Document) {
    let bytes = fs::read(format!(
        "../../tests/fixtures/referenced_names64_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    (bytes, doc)
}

#[test]
fn native_measurement_names_follow_requests_counters_and_explicit_labels() {
    for (state, sources, expected) in [
        ("baseline", [5, 6, 8, 12], ["m[1]", "m[2]", "m[3]", "m[4]"]),
        ("leading", [8, 9, 11, 15], ["m[1]", "m[2]", "m[3]", "m[4]"]),
        (
            "leading_before",
            [8, 9, 11, 15],
            ["m[3]", "m[4]", "m[5]", "m[6]"],
        ),
        (
            "leading_after",
            [8, 9, 11, 15],
            ["m[1]", "m[2]", "m[3]", "m[4]"],
        ),
        (
            "reverse_refs",
            [5, 6, 8, 12],
            ["m[4]", "m[3]", "m[2]", "m[1]"],
        ),
        (
            "counter_967",
            [5, 6, 8, 12],
            ["m[11]", "m[12]", "m[13]", "m[14]"],
        ),
        (
            "counter_first5",
            [5, 6, 8, 12],
            ["m[1]", "m[2]", "m[3]", "m[4]"],
        ),
        (
            "counter_high",
            [5, 6, 8, 12],
            ["m[1]", "m[2]", "m[3]", "m[4]"],
        ),
        ("explicit", [5, 6, 8, 12], ["X", "Y", "Z", "m[1]"]),
        ("renamed", [5, 6, 8, 12], ["W", "U", "V", "m[1]"]),
        (
            "same_slots_forward",
            [5, 6, 8, 12],
            ["m[1]", "m[3]", "m[4]", "m[2]"],
        ),
        (
            "same_slots_reverse",
            [5, 6, 8, 12],
            ["m[2]", "m[3]", "m[4]", "m[1]"],
        ),
        (
            "leading_hidden",
            [8, 9, 11, 15],
            ["m[1]", "m[2]", "m[3]", "m[4]"],
        ),
        (
            "leading_consumed",
            [8, 9, 11, 15],
            ["m[2]", "m[3]", "m[4]", "m[5]"],
        ),
    ] {
        let (bytes, doc) = names_fixture64(state);
        assert!(!doc.partial_view, "{state}");
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
        for (source, name) in sources.into_iter().zip(expected) {
            assert_eq!(
                doc.measurement_reference_name(source).as_deref(),
                Some(name),
                "{state}: {source}"
            );
        }
        for object in &doc.objects {
            if let Kind::ReferencedText { .. } = object.kind {
                let value = rows64(&doc, object.id);
                assert!(!value.contains("STALE"), "{state}: {}", object.id);
            }
        }
    }
}

#[test]
fn native_hidden_name_survives_live_edit_dirty_save_reopen_and_undo() {
    let (bytes, mut doc) = names_fixture64("leading_before");
    let original_names = doc.measurement_names.clone();
    doc.checkpoint();
    doc.objects[15].visible = false;
    assert_eq!(doc.measurement_names, original_names);
    assert_eq!(doc.measurement_reference_name(8).as_deref(), Some("m[3]"));
    let saved = doc.encode().unwrap();
    assert_ne!(saved, bytes);
    // Encoding evaluates a save projection; it does not mutate history/source.
    assert_eq!(doc.original, bytes);
    assert!(doc.objects[7].label.is_none());
    doc.verify_roundtrip().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.objects[15].visible);
    assert_eq!(
        super::super::parameter_names::measurement_name_counter(&saved, 0),
        Some(7)
    );
    for (source, number) in [(5, 1), (7, 2), (8, 3), (9, 4), (11, 5), (15, 6)] {
        let label = reopened.objects[source - 1].label.as_ref().unwrap();
        assert_eq!(label.name_utf8().unwrap(), format!("m[{number}]"));
        assert!(!label.visible());
        assert_eq!(label.offset(), (8, 5));
    }
    doc.undo();
    assert!(doc.objects[15].visible);
    assert_eq!(doc.measurement_names, original_names);
    assert_eq!(doc.original, bytes);
    doc.redo();
    assert!(!doc.objects[15].visible);
    assert_eq!(doc.encode().unwrap(), saved);
    let (native_saved, native) = names_fixture64("lifecycle_hide");
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_saved);
    assert_eq!(
        native.measurement_reference_name(8).as_deref(),
        Some("m[3]")
    );
}

#[test]
fn authored_name_requests_allocate_once_and_later_requests_undo_without_reusing_live_names() {
    let mut doc = Document::new();
    let n = doc.add_named_parameter("n", 2., (20, 30)).unwrap();
    let first = doc.add_parameter_calculation(n, "n+1", (20, 60)).unwrap();
    let second = doc.add_parameter_calculation(n, "n*2", (20, 90)).unwrap();
    let caption = doc.add_referenced_text(
        vec![first, second],
        "<H<?xA0002><T1x/>< ?xA0001>>",
        (20, 130),
    );
    assert!(caption.is_err());
    let caption = doc
        .add_referenced_text(
            vec![first, second],
            "<H<?xA0002><T1x/><?xA0001>>",
            (20, 130),
        )
        .unwrap();
    assert_eq!(rows64(&doc, caption), "m[1]/m[2]");
    assert_eq!(
        doc.measurement_reference_name(first).as_deref(),
        Some("m[2]")
    );
    doc.checkpoint();
    doc.objects[caption - 1].visible = false;
    let third = doc.add_parameter_calculation(n, "n+3", (20, 160)).unwrap();
    let last = doc
        .add_referenced_text(vec![third], "<?xA0001>", (20, 200))
        .unwrap();
    assert_eq!(rows64(&doc, last), "m[3]");
    doc.verify_roundtrip().unwrap();
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(rows64(&reopened, last), "m[3]");
    assert_eq!(
        reopened.measurement_reference_name(first).as_deref(),
        Some("m[2]")
    );
    doc.undo();
    assert_eq!(doc.objects.len(), last - 1);
    doc.redo();
    assert_eq!(rows64(&doc, last), "m[3]");
}

#[test]
fn naming_projection_preserves_page_edits_and_strict_counter_reads() {
    let (bytes, mut doc) = names_fixture64("baseline");
    doc.set_page_name(0, "Live captions").unwrap();
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.original, bytes);
    assert_eq!(doc.measurement_reference_name(5).as_deref(), Some("m[1]"));
    for number in [0u32, u32::MAX] {
        let mut broken = bytes.clone();
        let entries = super::super::records::records(&broken).unwrap();
        let counter = entries.iter().find(|r| r.kind == 0x0967).unwrap();
        broken[counter.data.start + 2..counter.data.start + 6]
            .copy_from_slice(&number.to_le_bytes());
        let parsed = Document::from_bytes(&broken, None).unwrap();
        assert_eq!(parsed.measurement_reference_name(5), None);
        assert_eq!(parsed.encode().unwrap(), broken);
    }
}

#[test]
fn native_numeric_roles_include_generated_names_and_live_definitions() {
    for state in ["numeric_bare_baseline", "numeric_cached_edited"] {
        let (bytes, doc) = native64(state);
        assert!(!doc.partial_view, "{state}");
        assert_eq!(doc.encode().unwrap(), bytes);
        for (id, expected) in [(17, "CA:m[1]"), (21, "RA:m[2]"), (25, "LA:m[3]")] {
            assert_eq!(compact64(&rows64(&doc, id)), expected);
        }
        for id in 12..=27 {
            assert!(!rows64(&doc, id).contains("STALE"));
            assert!(!rows64(&doc, id).contains("undefined"));
        }
    }
}

#[test]
fn native_public_anchored_role_writer_keeps_roles_live_and_both_dependencies() {
    let bytes =
        fs::read("../../tests/fixtures/referenced_shared64_api_anchored_roles_native_saved.gsp")
            .unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (id, source, expected) in [
        (11, 4, "PARAM2:2.0"),
        (13, 4, "PARAMA0002:n"),
        (15, 4, "PARAMB0002:n"),
        (17, 4, "PARAME0002:2.0"),
        (19, 9, "FUNCTION2:f(x)=x²"),
        (21, 9, "FUNCTIONA0002:f"),
        (23, 9, "FUNCTIONB0002:f(x)=x²"),
        (25, 9, "FUNCTIONE0002:x²"),
        (27, 5, "CALC2:3.00"),
        (29, 5, "CALCB0002:(n+1)"),
        (31, 5, "CALCE0002:(n+1)"),
        (33, 6, "RATIO2:0.50"),
        (35, 6, "RATIOB0002:AC/AB"),
        (37, 6, "RATIOE0002:AC/AB"),
        (39, 8, "LENGTH2:2.65cm"),
        (41, 8, "LENGTHB0002:A\u{305}B\u{305}"),
        (43, 8, "LENGTHE0002:A\u{305}B\u{305}"),
    ] {
        let (point, text) = doc.point_numeric_label(id).unwrap();
        assert_eq!(point, doc.position(id - 1).unwrap());
        assert_eq!(compact64(&text), expected);
        assert_eq!(doc.dependencies_of(id), Some(vec![id - 1, source]));
    }
    doc.set_parameter_value(4, 4.).unwrap();
    assert_eq!(
        compact64(&doc.point_numeric_label(11).unwrap().1),
        "PARAM2:4.0"
    );
    assert_eq!(
        compact64(&doc.point_numeric_label(13).unwrap().1),
        "PARAMA0002:n"
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn authored_measurement_names_are_written_for_all_three_source_families() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let c = doc.add(Kind::Free(Point { x: 50., y: 0. }));
    let n = doc.add_named_parameter("n", 2., (0, 0)).unwrap();
    let calc = doc.add_parameter_calculation(n, "n+1", (0, 0)).unwrap();
    let ratio = doc.add(Kind::PointRatio(a, b, c));
    let segment = doc.add(Kind::Segment(a, b));
    let length = doc.add(Kind::SegmentLength(segment));
    let text = doc
        .add_referenced_text(
            vec![calc, ratio, length],
            "<H<?xA0001><T1x/><?xA0002><T1x/><?xA0003>>",
            (20, 20),
        )
        .unwrap();
    assert_eq!(rows64(&doc, text), "m[1]/m[2]/m[3]");
    doc.verify_roundtrip().unwrap();
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(rows64(&reopened, text), "m[1]/m[2]/m[3]");
    for (id, name) in [(calc, "m[1]"), (ratio, "m[2]"), (length, "m[3]")] {
        assert_eq!(
            reopened.objects[id - 1]
                .label
                .as_ref()
                .unwrap()
                .name_utf8()
                .unwrap(),
            name
        );
    }
}

#[test]
fn native_final_names_api_goldens_preserve_physical_labels_order_and_live_values() {
    for (state, expected) in [
        ("request_forward", [1, 2, 3, 4]),
        ("request_reverse", [4, 3, 2, 1]),
        ("pair_forward", [1, 3, 4, 2]),
        ("pair_reverse", [2, 3, 4, 1]),
        ("hidden_unconsumed", [1, 2, 3, 4]),
        ("hidden_consumed", [2, 3, 4, 5]),
        ("lifecycle_hidden", [2, 3, 4, 5]),
        ("lifecycle_edited", [2, 3, 4, 5]),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/referenced_names64_api_{state}_native_saved.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
        for (source, number) in [6, 7, 9, 10].into_iter().zip(expected) {
            let label = doc.objects[source - 1].label.as_ref().unwrap();
            assert_eq!(
                label.name_utf8().unwrap(),
                format!("m[{number}]"),
                "{state}: {source}"
            );
            assert!(!label.visible());
        }
        if state == "hidden_unconsumed" {
            assert!(doc.objects[4].label.is_none());
        }
        let original_value = doc.numeric_value(4).unwrap();
        assert_eq!(
            original_value,
            if state == "lifecycle_edited" { 4. } else { 2. }
        );
        doc.set_parameter_value(4, original_value + 2.).unwrap();
        assert_eq!(
            doc.measurement_reference_name(6).unwrap(),
            format!("m[{}]", expected[0])
        );
        assert_eq!(doc.numeric_value(6), Some(original_value + 3.));
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
    }
}
