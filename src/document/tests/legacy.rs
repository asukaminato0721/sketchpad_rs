use super::*;

#[test]
fn legacy_paste_upgrades_every_header_and_undo_restores_original_source() {
    let original = include_bytes!("../../../tests/fixtures/legacy_polygon_rank_original.gsp");
    let mut document = Document::from_bytes(original, None).unwrap();
    document.selected.insert(5);
    let clipboard = document.copy_selected().unwrap();
    assert_eq!(document.paste(&clipboard).unwrap(), vec![6, 7, 8, 9, 10]);
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!("../../../tests/fixtures/legacy_polygon_same_document_native_verified.gsp")
    );
    let rs = records(&encoded).unwrap();
    assert!(
        rs.iter()
            .filter(|r| r.kind == 0x7d0)
            .all(|r| r.data.len() == 28)
    );
    let version = rs.iter().find(|r| r.kind == 0x384).unwrap();
    assert_eq!(
        &encoded[version.data.start..version.data.start + 2],
        &[5, 0]
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.objects.len(), 10);
    assert_eq!(reopened.objects[4].interior_order, Some(0));
    for id in [1, 2, 3, 4, 6, 7, 8, 9] {
        assert!(matches!(
            reopened.objects[id - 1].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Small,
                ..
            })
        ));
    }
    if std::env::var_os("SKETCHPAD_WRITE_LEGACY_PASTE_FIXTURE").is_some() {
        fs::write("/tmp/legacy_polygon_same_document_port.gsp", &encoded).unwrap();
    }
    document.undo();
    assert_eq!(document.objects.len(), 5);
    assert_eq!(document.objects[4].interior_order, None);
    assert_eq!(document.encode().unwrap(), original);
    document.redo();
    assert_eq!(document.encode().unwrap(), encoded);
    document.verify_roundtrip().unwrap();
}

#[test]
fn legacy_paste_keeps_prior_edits_and_independent_undo_steps() {
    let original = include_bytes!("../../../tests/fixtures/legacy_polygon_rank_original.gsp");
    let mut document = Document::from_bytes(original, None).unwrap();
    document.selected.insert(5);
    document
        .set_selected_color(RgbColor {
            red: 0,
            green: 128,
            blue: 255,
        })
        .unwrap();
    let edited = document.encode().unwrap();
    let clipboard = document.copy_selected().unwrap();
    document.paste(&clipboard).unwrap();
    let first_paste = document.encode().unwrap();
    assert_eq!(
        document.objects[4].visual_style,
        document.objects[9].visual_style
    );
    document.paste(&clipboard).unwrap();
    assert_eq!(document.objects.len(), 15);
    document.verify_roundtrip().unwrap();
    document.undo();
    assert_eq!(document.encode().unwrap(), first_paste);
    document.undo();
    assert_eq!(document.encode().unwrap(), edited);
    document.undo();
    assert_eq!(document.encode().unwrap(), original);
}

#[test]
fn legacy_paste_rejects_unproved_upgrade_atomically() {
    let mut bytes =
        include_bytes!("../../../tests/fixtures/legacy_polygon_rank_original.gsp").to_vec();
    let header = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x7d0)
        .unwrap();
    bytes[header.data.start + 10] = 2;
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    let mut modern = Document::new();
    let point = modern.add(Kind::Free(Point { x: 20., y: 30. }));
    modern.selected.insert(point);
    let clipboard = modern.copy_selected().unwrap();
    let before = document.encode().unwrap();
    assert!(document.paste(&clipboard).is_err());
    assert_eq!(document.encode().unwrap(), before);
    assert!(document.undo.is_empty());
}

#[test]
fn legacy_point_size_edit_upgrades_storage_and_undo_restores_short_headers() {
    let original = include_bytes!("../../../tests/fixtures/legacy_polygon_rank_original.gsp");
    let mut document = Document::from_bytes(original, None).unwrap();
    document.selected.insert(1);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Small)
            .unwrap(),
        0
    );
    assert_eq!(document.encode().unwrap(), original);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Medium)
            .unwrap(),
        1
    );
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!("../../../tests/fixtures/legacy_point_medium_upgraded_native_verified.gsp")
    );
    assert!(
        records(&encoded)
            .unwrap()
            .iter()
            .filter(|r| r.kind == 0x7d0)
            .all(|r| r.data.len() == 28)
    );
    assert_eq!(document.selected, [1].into_iter().collect());
    assert!(matches!(
        document.objects[0].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            ..
        })
    ));
    let native_ui = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/legacy_point_medium_native_ui_verified.gsp"),
        None,
    )
    .unwrap();
    for (actual, native) in document.objects.iter().zip(&native_ui.objects) {
        assert_eq!(actual.visual_style, native.visual_style);
        assert_eq!(actual.interior_order, native.interior_order);
    }
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_LEGACY_POINT_STYLE_FIXTURE").is_some() {
        fs::write("/tmp/legacy_point_medium_upgraded_port.gsp", &encoded).unwrap();
    }
    document.undo();
    assert_eq!(document.encode().unwrap(), original);
    assert_eq!(document.objects[4].interior_order, None);
    document.redo();
    assert_eq!(document.encode().unwrap(), encoded);
}

#[test]
fn legacy_stroke_paste_and_new_patterns_keep_native_pen_meaning() {
    for (input, dashed) in [
        (
            include_bytes!("../../../tests/fixtures/legacy_stroke_styles_original.gsp").as_slice(),
            false,
        ),
        (
            include_bytes!("../../../tests/fixtures/legacy_stroke_dashed_original.gsp").as_slice(),
            true,
        ),
    ] {
        let mut document = Document::from_bytes(input, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.objects.len(), 10);
        let styles: Vec<_> = document.objects.iter().map(|o| o.visual_style).collect();
        document.selected = (1..=10).collect();
        let clipboard = document.copy_selected().unwrap();
        document.paste(&clipboard).unwrap();
        for (index, style) in styles.iter().enumerate() {
            assert_eq!(document.objects[index].visual_style, *style);
            assert_eq!(document.objects[index + 10].visual_style, *style);
        }
        document.verify_roundtrip().unwrap();
        let pasted = document.encode().unwrap();
        let golden = if dashed {
            include_bytes!(
                "../../../tests/fixtures/legacy_dashed_paste_upgraded_native_verified.gsp"
            )
            .as_slice()
        } else {
            include_bytes!(
                "../../../tests/fixtures/legacy_stroke_paste_upgraded_native_verified.gsp"
            )
            .as_slice()
        };
        assert_eq!(pasted, golden);
        if std::env::var_os("SKETCHPAD_WRITE_LEGACY_STROKE_FIXTURE").is_some() {
            fs::write(
                if dashed {
                    "/tmp/legacy_dashed_paste_port.gsp"
                } else {
                    "/tmp/legacy_stroke_paste_port.gsp"
                },
                &pasted,
            )
            .unwrap();
        }
        document.undo();
        assert_eq!(document.encode().unwrap(), input);
        document.selected = [9].into_iter().collect();
        assert_eq!(
            document
                .set_selected_line_pattern(LinePattern::Dotted)
                .unwrap(),
            1
        );
        let styled = document.encode().unwrap();
        assert!(
            records(&styled)
                .unwrap()
                .iter()
                .filter(|r| r.kind == 0x7d0)
                .all(|r| r.data.len() == 28)
        );
        assert!(matches!(
            document.objects[8].visual_style,
            Some(VisualStyle::Stroke {
                line: SegmentLineStyle {
                    pattern: LinePattern::Dotted,
                    ..
                },
                ..
            })
        ));
        document.verify_roundtrip().unwrap();
        if !dashed {
            assert_eq!(
                styled,
                include_bytes!(
                    "../../../tests/fixtures/legacy_stroke_dotted_upgraded_native_verified.gsp"
                )
            );
        }
        if !dashed && std::env::var_os("SKETCHPAD_WRITE_LEGACY_STROKE_FIXTURE").is_some() {
            fs::write("/tmp/legacy_stroke_dotted_edit_port.gsp", &styled).unwrap();
        }
        document.undo();
        assert_eq!(document.encode().unwrap(), input);
        document.redo();
        assert_eq!(document.encode().unwrap(), styled);
    }
}

#[test]
fn legacy_style_upgrade_remaps_survivors_after_pending_deletion() {
    let mut document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/legacy_polygon_rank_original.gsp"),
        None,
    )
    .unwrap();
    document.selected.insert(2);
    document.clear_selected().unwrap();
    let before = document.encode().unwrap();
    document.selected = [3].into_iter().collect();
    let point = document.position(3).unwrap();
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Medium)
            .unwrap(),
        1
    );
    assert_eq!(document.objects.len(), 3);
    assert_eq!(document.selected, [2].into_iter().collect());
    assert_eq!(document.position(2), Some(point));
    assert!(matches!(
        document.objects[1].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            ..
        })
    ));
    document.verify_roundtrip().unwrap();
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
}
