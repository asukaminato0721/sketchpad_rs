use super::*;

#[test]
fn source_point_style_undoes_and_survives_save() {
    let mut document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/style_blue_point.gsp"),
        None,
    )
    .unwrap();
    document.selected.insert(1);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Large)
            .unwrap(),
        1
    );
    assert_eq!(
        document.objects[0].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Large,
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 255
            },
        })
    );
    document.undo();
    assert!(matches!(
        document.objects[0].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            ..
        })
    ));
    document.redo();
    let check = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(
        check.objects[0].visual_style,
        document.objects[0].visual_style
    );
    assert_eq!(check.objects[0].kind, document.objects[0].kind);
    let path =
        std::env::temp_dir().join(format!("sketchpad-style-point-{}.gsp", std::process::id()));
    document.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        reopened.objects[0].visual_style,
        document.objects[0].visual_style
    );
    assert!(!reopened.partial_view);
}

#[test]
fn new_styled_segment_and_copied_points_reopen() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 100., y: 120. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 120. }));
    let segment = document.add(Kind::Segment(a, b));
    document.selected.insert(segment);
    assert_eq!(
        document.set_selected_line_width(LineWidth::Thick).unwrap(),
        1
    );
    assert_eq!(
        document
            .set_selected_line_pattern(LinePattern::Dashed)
            .unwrap(),
        1
    );
    assert_eq!(
        document
            .set_selected_color(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
            .unwrap(),
        1
    );
    document.selected.clear();
    document.selected.insert(a);
    document
        .set_selected_point_style(PointStyle::Small)
        .unwrap();
    let copied = document.copy_selected().unwrap();
    let pasted = document.paste(&copied).unwrap();
    assert_eq!(
        document.objects[pasted[0] - 1].visual_style,
        document.objects[a - 1].visual_style
    );
    let path = std::env::temp_dir().join(format!(
        "sketchpad-style-segment-{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        reopened.objects[segment - 1].visual_style,
        Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Dashed
            },
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 255
            },
        })
    );
    assert_eq!(
        reopened.objects[pasted[0] - 1].visual_style,
        document.objects[a - 1].visual_style
    );
}
#[test]
fn imported_picture_moves_undoes_and_survives_save() {
    let png = include_bytes!("../../../tests/fixtures/image_tiny.png");
    let mut document = Document::new();
    document.add(Kind::Free(Point { x: 80., y: 90. }));
    let image = document
        .import_png(png, Point { x: 120., y: 130. })
        .unwrap();
    assert_eq!(image, 2);
    document
        .move_image(image, Point { x: 160., y: 155. })
        .unwrap();
    assert!(
        matches!(&document.objects[1].kind, Kind::Image { transform, .. } if transform[2] == 160.)
    );
    document.undo();
    assert!(
        matches!(&document.objects[1].kind, Kind::Image { transform, .. } if transform[2] == 120.)
    );
    document.redo();
    let path = std::env::temp_dir().join(format!("sketchpad-image-{}.gsp", std::process::id()));
    document.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 2);
    assert_eq!(reopened.objects[1].kind, document.objects[1].kind);
}

#[test]
fn picture_import_rejects_bad_png_and_read_only_sources() {
    let mut document = Document::new();
    assert!(
        document
            .import_png(b"broken", Point { x: 0., y: 0. })
            .is_err()
    );
    assert!(document.objects.is_empty());
    let png = include_bytes!("../../../tests/fixtures/image_tiny.png");
    document.import_png(png, Point { x: 0., y: 0. }).unwrap();
    assert!(document.import_png(png, Point { x: 20., y: 20. }).is_err());
    assert_eq!(document.objects.len(), 1);
    for source in [
        include_bytes!("../../../tests/fixtures/image_original_cropped.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/image_original_pasted.gsp").as_slice(),
    ] {
        let mut read_only = Document::from_bytes(source, None).unwrap();
        assert_eq!(read_only.encode().unwrap(), source);
        read_only.checkpoint();
        assert!(read_only.can_save().unwrap_err().contains("read-only"));
    }
}

#[test]
fn original_drag_of_port_picture_reopens_and_remains_editable() {
    let source = include_bytes!("../../../tests/fixtures/image_port_imported_dragged_wine.gsp");
    let mut document = Document::from_bytes(source, None).unwrap();
    assert!(!document.partial_view);
    assert!(
        matches!(&document.objects[1].kind, Kind::Image { transform, .. }
        if transform[2] == 184. && transform[5] == 170.)
    );
    document.move_image(2, Point { x: 204., y: 180. }).unwrap();
    let written = document.encode().unwrap();
    let reopened = Document::from_bytes(&written, None).unwrap();
    assert!(
        matches!(&reopened.objects[1].kind, Kind::Image { transform, .. }
        if transform[2] == 204. && transform[5] == 180.)
    );
}
#[test]
fn plain_text_edit_move_and_label_survive_save() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/text_object_original_Hello.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/text_object_original_Hello.gsp")
    );
    doc.edit_text(1, "猫\nA<B>").unwrap();
    doc.move_text(1, Point { x: 180., y: 130. }).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.objects[0].kind, doc.objects[0].kind);
    assert!(!reopened.partial_view);

    let mut label_doc = Document::new();
    let point = label_doc.add(Kind::Free(Point { x: 100., y: 120. }));
    label_doc.set_point_label(point, "P", true).unwrap();
    let bytes = label_doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        reopened.objects[0]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "P"
    );
    assert!(!reopened.partial_view);
}
#[test]
fn styled_text_imports_and_edits_without_changing_other_runs() {
    for source in [
        include_bytes!("../../../tests/fixtures/text_style_bold_off.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/text_style_italic_on.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/text_style_underline_on.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/text_style_larger.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/text_two_objects_first_bold_off_original.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/text_two_objects_second_bold_off_original.gsp")
            .as_slice(),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
    }

    let original =
        include_bytes!("../../../tests/fixtures/text_two_objects_first_bold_off_original.gsp");
    let mut doc = Document::from_bytes(original, None).unwrap();
    let other = doc.objects[1].kind.clone();
    doc.edit_text(1, "Greetings").unwrap();
    let edited = doc.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[1].kind, other);
    assert!(
        matches!(&reopened.objects[0].kind, Kind::PlainText { text, .. } if text == "Greetings")
    );
    for kind in [0x04b1, 0x04b3, 0x273c] {
        let payloads = |bytes: &[u8]| {
            records(bytes)
                .unwrap()
                .into_iter()
                .filter(|record| record.kind == kind)
                .map(|record| bytes[record.data].to_vec())
                .collect::<Vec<_>>()
        };
        assert_eq!(payloads(&edited), payloads(original));
    }
    doc.move_text(1, Point { x: 110., y: 100. }).unwrap();
    let moved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!moved.partial_view);
    assert_eq!(moved.objects[1].kind, other);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), edited);
}
#[test]
fn text_formatting_keeps_independent_styles_through_save_undo_and_copy() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/text_two_objects_generated.gsp"),
        None,
    )
    .unwrap();
    assert!(doc.text_font_state(1).unwrap().bold);
    doc.selected.insert(1);
    assert_eq!(
        doc.toggle_selected_text_style(TextStyleCommand::Bold)
            .unwrap(),
        1
    );
    assert!(!doc.text_font_state(1).unwrap().bold);
    assert!(doc.text_font_state(2).unwrap().bold);
    let one_formatted = doc.encode().unwrap();
    let reopened = Document::from_bytes(&one_formatted, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(!reopened.text_font_state(1).unwrap().bold);
    assert!(reopened.text_font_state(2).unwrap().bold);
    doc.undo();
    assert!(doc.text_font_state(1).unwrap().bold);
    doc.redo();
    assert!(!doc.text_font_state(1).unwrap().bold);

    doc.selected.clear();
    doc.selected.insert(2);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    doc.toggle_selected_text_style(TextStyleCommand::Underline)
        .unwrap();
    let font = doc.text_font_state(2).unwrap();
    assert!(font.bold && font.italic && font.underline);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(pasted, vec![3]);
    assert_eq!(doc.text_font_state(3).unwrap(), font);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(3).unwrap(), font);
    assert_eq!(reopened.text_font_state(2).unwrap(), font);
    assert!(!reopened.text_font_state(1).unwrap().bold);
}
#[test]
fn mixed_font_families_survive_style_edit_and_copy() {
    let source = include_bytes!("../../../tests/fixtures/text_two_font_cjk_first.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    assert_eq!(doc.text_font_state(1).unwrap().family, "@Noto Sans CJK JP");
    assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
    doc.selected.insert(1);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    let copied = doc.copy_selected().unwrap();
    assert_eq!(doc.paste(&copied).unwrap(), vec![3]);
    doc.selected.clear();
    doc.selected.insert(2);
    doc.toggle_selected_text_style(TextStyleCommand::Bold)
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.text_font_state(1).unwrap().family,
        "@Noto Sans CJK JP"
    );
    assert!(reopened.text_font_state(1).unwrap().italic);
    assert_eq!(reopened.text_font_state(2).unwrap().family, "Tahoma");
    assert!(!reopened.text_font_state(2).unwrap().bold);
    assert_eq!(reopened.text_font_state(3), reopened.text_font_state(1));
}
#[test]
fn native_mixed_format_text_moves_without_losing_runs() {
    let source = include_bytes!("../../../tests/fixtures/text_mixed_bold_italic_he_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    let runs = doc.text_rich_runs(1).unwrap();
    assert_eq!(runs[0][0].text, "He");
    assert_eq!(runs[0][0].style.bold, Some(false));
    assert_eq!(runs[0][0].style.italic, Some(true));
    assert_eq!(runs[0][1].text, "llo");
    doc.edit_text(1, "Hexllo").unwrap();
    let edited = doc.text_rich_runs(1).unwrap();
    assert_eq!(edited[0][0].text, "Hex");
    assert_eq!(edited[0][0].style, runs[0][0].style);
    assert_eq!(edited[0][1].text, "llo");
    let edited_file = doc.encode().unwrap();
    let reopened_edit = Document::from_bytes(&edited_file, None).unwrap();
    assert_eq!(reopened_edit.text_rich_runs(1).unwrap(), edited);
    doc.undo();
    assert_eq!(doc.text_rich_runs(1).unwrap(), runs);
    doc.move_text(1, Point { x: 140., y: 110. }).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap(), runs);
    assert_eq!(reopened.encode().unwrap(), encoded);

    doc.selected.insert(1);
    let clipboard = doc.copy_selected().unwrap();
    assert_eq!(doc.paste(&clipboard).unwrap(), vec![2]);
    assert_eq!(doc.text_rich_runs(2).unwrap(), runs);
    let Kind::PlainText { rect: first, .. } = &doc.objects[0].kind else {
        unreachable!()
    };
    let Kind::PlainText { rect: second, .. } = &doc.objects[1].kind else {
        unreachable!()
    };
    assert_eq!(second.left, first.left + 20);
    assert_eq!(second.top, first.top + 20);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(2).unwrap(), runs);
}
#[test]
fn substring_size_steps_preserve_native_rich_text() {
    let source = include_bytes!("../../../tests/fixtures/text_mixed_size36_he_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    let original = doc.text_rich_runs(1).unwrap();
    assert_eq!(original[0][0].style.size_code, Some(36));
    doc.format_text_range(1, 0, 2, TextStyleCommand::DecreaseSize)
        .unwrap();
    assert_eq!(doc.text_rich_runs(1).unwrap()[0][0].style.size_code, None);
    doc.format_text_range(1, 0, 2, TextStyleCommand::IncreaseSize)
        .unwrap();
    assert_eq!(doc.text_rich_runs(1).unwrap(), original);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap(), original);
}
#[test]
fn native_mixed_family_caption_edits_and_reopens() {
    let source = include_bytes!("../../../tests/fixtures/text_mixed_adventor_he_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
    let runs = doc.text_rich_runs(1).unwrap();
    assert_eq!(runs[0][0].style.family_index, Some(1));
    doc.edit_text(1, "Hexllo").unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap()[0][0].text, "Hex");
    assert_eq!(
        reopened.text_rich_runs(1).unwrap()[0][0].style.family_index,
        Some(1)
    );
}
#[test]
fn selected_range_can_add_a_font_family_and_undo_it() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/text_object_generated_Hello.gsp"),
        None,
    )
    .unwrap();
    doc.format_text_range_family(1, 0, 2, "TeX Gyre Adventor")
        .unwrap();
    assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
        Some(1)
    );
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.text_rich_runs(1).unwrap(),
        doc.text_rich_runs(1).unwrap()
    );
    doc.undo();
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
        None
    );
    assert_eq!(doc.text_font_families().unwrap().len(), 1);
    doc.redo();
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
        Some(1)
    );
    let path =
        std::env::temp_dir().join(format!("sketchpad_run_family_{}.gsp", std::process::id()));
    doc.save_to(&path).unwrap();
    let first = fs::read(&path).unwrap();
    doc.save_to(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), first);
    assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
    fs::remove_file(path).unwrap();
}
#[test]
fn selected_caption_range_writes_native_style_markup() {
    let source = include_bytes!("../../../tests/fixtures/text_object_generated_Hello.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    doc.format_text_range(1, 0, 2, TextStyleCommand::Bold)
        .unwrap();
    let runs = doc.text_rich_runs(1).unwrap();
    assert_eq!(runs[0][0].text, "He");
    assert_eq!(runs[0][0].style.bold, Some(false));
    assert_eq!(runs[0][1].text, "llo");
    let encoded = doc.encode().unwrap();
    assert!(
        encoded
            .windows(b"<VL<H<SW1<T2xHe>><T3xllo>>>".len())
            .any(|part| part == b"<VL<H<SW1<T2xHe>><T3xllo>>>")
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap(), runs);
    doc.format_text_range(1, 0, 2, TextStyleCommand::Italic)
        .unwrap();
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.italic,
        Some(true)
    );
    doc.undo();
    assert_eq!(doc.text_rich_runs(1).unwrap(), runs);
    doc.edit_text(1, "Hexllo").unwrap();
    let edited = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    let edited_runs = edited.text_rich_runs(1).unwrap();
    assert_eq!(edited_runs[0][0].text, "Hex");
    assert_eq!(edited_runs[0][0].style.bold, Some(false));
    assert_eq!(edited_runs[0][1].text, "llo");
}
#[test]
fn selected_text_can_use_new_font_families_independently() {
    let source = include_bytes!("../../../tests/fixtures/text_two_objects_generated.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    doc.selected.insert(1);
    assert_eq!(doc.set_selected_text_family("Arial").unwrap(), 1);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    assert_eq!(doc.text_font_state(1).unwrap().family, "Arial");
    assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
    doc.selected.clear();
    doc.selected.insert(2);
    doc.set_selected_text_family("@Noto Sans CJK JP").unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(1).unwrap().family, "Arial");
    assert!(reopened.text_font_state(1).unwrap().italic);
    assert_eq!(
        reopened.text_font_state(2).unwrap().family,
        "@Noto Sans CJK JP"
    );
    assert_eq!(reopened.encode().unwrap(), encoded);
    assert!(doc.set_selected_text_family("").is_err());
    assert!(doc.set_selected_text_family("Noto 日本語").is_err());
    doc.undo();
    assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
}
#[test]
fn text_size_commands_follow_verified_native_steps() {
    for (source, size) in [
        (
            include_bytes!("../../../tests/fixtures/text_style_smaller_18_original.gsp").as_slice(),
            18,
        ),
        (
            include_bytes!("../../../tests/fixtures/text_style_larger_48_original.gsp").as_slice(),
            48,
        ),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
        assert_eq!(doc.encode().unwrap(), source);
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/text_object_generated_Hello.gsp"),
        None,
    )
    .unwrap();
    doc.selected.insert(1);
    for size in [36, 48, 64] {
        doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
            .unwrap();
        assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
    }
    assert!(
        doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
            .is_err()
    );
    for size in [48, 36, 24, 18, 16, 14] {
        doc.toggle_selected_text_style(TextStyleCommand::DecreaseSize)
            .unwrap();
        assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
    }
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(1).unwrap().size_code, 14);
    doc.undo();
    assert_eq!(doc.text_font_state(1).unwrap().size_code, 16);
}
#[test]
fn point_label_styles_are_independent_and_survive_copy() {
    let source =
        include_bytes!("../../../tests/fixtures/text_two_point_labels_first_bold_off_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    assert!(!doc.text_font_state(1).unwrap().bold);
    assert!(doc.text_font_state(2).unwrap().bold);

    doc.selected.insert(2);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    assert!(doc.text_font_state(2).unwrap().italic);
    assert!(!doc.text_font_state(1).unwrap().bold);
    let copied = doc.copy_selected().unwrap();
    let pasted = doc.paste(&copied).unwrap();
    assert_eq!(pasted, vec![3]);
    assert_eq!(doc.objects[2].label, doc.objects[1].label);
    assert_eq!(doc.text_font_state(3), doc.text_font_state(2));
    doc.selected.clear();
    doc.selected.insert(1);
    doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
        .unwrap();
    assert_eq!(doc.text_font_state(1).unwrap().size_code, 36);
    assert_eq!(doc.text_font_state(2).unwrap().size_code, 24);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(!reopened.text_font_state(1).unwrap().bold);
    assert_eq!(reopened.text_font_state(1).unwrap().size_code, 36);
    assert!(reopened.text_font_state(2).unwrap().italic);
    assert_eq!(reopened.text_font_state(3), reopened.text_font_state(2));
    assert_eq!(reopened.objects[2].label, reopened.objects[1].label);
}
#[test]
fn source_point_label_rename_and_hide_survive_save() {
    let original = include_bytes!("../../../tests/fixtures/text_point_label.gsp");
    let mut doc = Document::from_bytes(original, None).unwrap();
    assert_eq!(doc.encode().unwrap(), original);
    doc.set_point_label(1, "Cat", false).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    let label = reopened.objects[0].label.as_ref().unwrap();
    assert_eq!(label.name_utf8().unwrap(), "Cat");
    assert!(!label.visible());
    assert!(!reopened.partial_view);
}
#[test]
fn wine_dragged_text_preserves_opaque_padding_until_edited() {
    let original =
        include_bytes!("../../../tests/fixtures/text_object_generated_Hello_dragged.gsp");
    let mut doc = Document::from_bytes(original, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), original);
    doc.edit_text(1, "Hello again").unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(
        matches!(&reopened.objects[0].kind, Kind::PlainText { text, .. } if text == "Hello again")
    );
}
