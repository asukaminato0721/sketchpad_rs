use super::*;

#[test]
fn native_page_links_import_with_destinations_and_preserve_file() {
    for (bytes, source_page, destination_page) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_link_page_default_original.gsp")
                .as_slice(),
            1,
            0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_link_page_two_original.gsp").as_slice(),
            0,
            1,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.page_count, 2);
        assert_eq!(document.objects[1].page, source_page);
        assert!(matches!(&document.objects[1].kind,
            Kind::LinkPageButton { target_page, label, anchor, .. }
            if *target_page == destination_page && label == "链接" && *anchor == (10, 5)
        ));
        assert_eq!(document.encode().unwrap(), bytes);
    }
}

#[test]
fn page_link_appends_to_last_page_of_native_document() {
    let bytes = include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    document.active_page = 1;
    let button = document.add_link_page_button(0).unwrap();
    let url_button = document
        .add_link_url_button("https://example.org/")
        .unwrap();
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.page_count, 2);
    assert_eq!(reopened.objects[button - 1].page, 1);
    assert!(matches!(&reopened.objects[button - 1].kind,
        Kind::LinkPageButton { target_page: 0, label, .. } if label == "Go to Page 1"
    ));
    assert!(matches!(&reopened.objects[url_button - 1].kind,
        Kind::LinkUrlButton { url, .. } if url == "https://example.org/"
    ));
    assert_eq!(reopened.encode().unwrap(), encoded);
}
#[test]
fn links_on_earlier_pages_preserve_later_objects_and_ids() {
    let bytes = include_bytes!("../../../../../tests/fixtures/two_page_two_points_native.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(document.objects.len(), 2);
    document.active_page = 0;
    document.add_link_page_button(1).unwrap();
    document
        .add_link_url_button("https://example.org/")
        .unwrap();
    document.active_page = 1;
    document.add_link_page_button(0).unwrap();
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_page_links_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(document.objects.len(), 5);
    assert!(matches!(document.objects[0].kind, Kind::Free(_)));
    assert!(matches!(
        &document.objects[1].kind,
        Kind::LinkPageButton { target_page: 1, .. }
    ));
    assert!(matches!(&document.objects[2].kind,
        Kind::LinkUrlButton { url, .. } if url == "https://example.org/"
    ));
    assert!(matches!(document.objects[3].kind, Kind::Free(_)));
    assert_eq!(document.objects[3].page, 1);
    assert!(matches!(
        &document.objects[4].kind,
        Kind::LinkPageButton { target_page: 0, .. }
    ));
    let saved = std::fs::read(&path).unwrap();
    assert_eq!(document.encode().unwrap(), saved);
    document.active_page = 0;
    document.add_link_page_button(1).unwrap();
    document.save_to(&path).unwrap();
    assert_eq!(document.objects.len(), 6);
    assert!(matches!(
        document.objects[3].kind,
        Kind::LinkPageButton { .. }
    ));
    assert!(matches!(document.objects[4].kind, Kind::Free(_)));
    std::fs::remove_file(path).unwrap();
}
#[test]
fn page_link_destination_button_imports_and_writes_native_activation_flag() {
    let native = include_bytes!(
        "../../../../../tests/fixtures/action_link_page_activate_hide_show_native_created.gsp"
    );
    let imported = Document::from_bytes(native, None).unwrap();
    assert!(matches!(
        imported.objects[3].kind,
        Kind::LinkPageButton {
            target_page: 0,
            target_button_ordinal: Some(1),
            ..
        }
    ));
    assert_eq!(imported.encode().unwrap(), native);

    let base = include_bytes!("../../../../../tests/fixtures/two_page_hide_show_first_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    document
        .add_link_page_button_with_target(0, Some(1))
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_link_destination_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let saved = std::fs::read(&path).unwrap();
    assert_eq!(
        saved,
        expected_page(
            include_bytes!(
                "../../../../../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
            ),
            document.active_page
        )
    );
    assert!(matches!(
        document.objects[3].kind,
        Kind::LinkPageButton {
            target_button_ordinal: Some(1),
            ..
        }
    ));
    std::fs::remove_file(path).unwrap();
}
#[test]
fn multi_page_hide_show_state_saves_without_changing_other_pages() {
    for (index, base) in [
        include_bytes!(
            "../../../../../tests/fixtures/action_link_page_activate_hide_show_native_created.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
        )
        .as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 0;
        document.activate_hide_show_button(2).unwrap();
        assert!(document.can_save().is_ok());
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_hide_show_{}_{}.gsp",
            std::process::id(),
            index
        ));
        document.save_to(&path).unwrap();
        let saved = std::fs::read(&path).unwrap();
        assert!(!document.objects[0].visible);
        assert!(matches!(
            document.objects[1].kind,
            Kind::HideShowButton {
                action: VisibilityAction::Show,
                ..
            }
        ));
        let original_records = records(base).unwrap();
        let saved_records = records(&saved).unwrap();
        let original_page_2 = original_records
            .iter()
            .filter(|record| record.kind == 0x3e8)
            .nth(1)
            .unwrap()
            .start;
        let saved_page_2 = saved_records
            .iter()
            .filter(|record| record.kind == 0x3e8)
            .nth(1)
            .unwrap()
            .start;
        assert_eq!(&saved[saved_page_2..], &base[original_page_2..]);

        document.activate_hide_show_button(2).unwrap();
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            expected_page(base, document.active_page)
        );
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
fn multi_page_link_and_destination_state_save_together() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_hide_show_first_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    document
        .add_link_page_button_with_target(0, Some(1))
        .unwrap();
    document.active_page = 0;
    document.activate_hide_show_button(2).unwrap();
    assert!(document.can_save().is_ok());
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_link_with_state_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/action_link_page_activate_hide_show_port_clicked.gsp"
        )
    );
    std::fs::remove_file(path).unwrap();
}
#[test]
fn free_points_move_and_save_on_both_pages() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_two_points_native.gsp");
    for (index, page, id, position, expected) in [
        (
            0,
            1,
            2,
            Point { x: 261., y: 171. },
            include_bytes!("../../../../../tests/fixtures/two_page_point_moved_port.gsp").as_slice(),
        ),
        (
            1,
            0,
            1,
            Point { x: 363., y: 247. },
            include_bytes!("../../../../../tests/fixtures/two_page_point_moved_link_port.gsp").as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = page;
        assert!(document.move_point(id, position));
        if index == 1 {
            document.add_link_page_button(1).unwrap();
        }
        assert!(document.can_save().is_ok());
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_move_{}_{}.gsp",
            std::process::id(),
            index
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            expected_page(expected, document.active_page)
        );
        assert_eq!(document.position(id), Some(position));
        assert_eq!(document.objects[id - 1].page, page);
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
fn constrained_points_move_and_save_on_second_page() {
    let cases: &[(&[u8], usize, Point, &[u8])] = &[
        (
            include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_native.gsp"),
            5,
            Point { x: 246., y: 111. },
            include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_moved_port.gsp"),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/two_page_circle_constrained_native.gsp"),
            5,
            Point { x: 111., y: 111. },
            include_bytes!("../../../../../tests/fixtures/two_page_circle_constrained_moved_port.gsp"),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/two_page_arc_constrained_native.gsp"),
            6,
            Point { x: 241., y: 64. },
            include_bytes!("../../../../../tests/fixtures/two_page_arc_constrained_moved_port.gsp"),
        ),
    ];
    for (index, &(base, id, target, expected)) in cases.iter().enumerate() {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        assert!(document.move_point(id, target));
        assert!(document.can_save().is_ok());
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_constrained_{}_{}.gsp",
            std::process::id(),
            index
        ));
        document.save_to(&path).unwrap();
        let saved = std::fs::read(&path).unwrap();
        assert_eq!(saved, expected);
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert_eq!(reopened.page_count, 2);
        assert_eq!(reopened.objects[id - 1].page, 1);
        match (index, &reopened.objects[id - 1].kind) {
            (0, Kind::PointOnSegment(4, t)) => assert!((*t - 0.75).abs() < 1e-12),
            (1, Kind::PointOnCircle(4, x, y)) => {
                assert!((*x + 1.).abs() < 1e-12);
                assert!(y.abs() < 1e-12);
            }
            (2, Kind::PointOnArc(5, t)) => assert!((*t - 0.6888239941915089).abs() < 1e-4),
            _ => panic!("Unexpected constrained point after multi-page save"),
        }
        if index == 0 {
            assert_eq!(
                saved,
                include_bytes!(
                    "../../../../../tests/fixtures/two_page_segment_constrained_dragged_native.gsp"
                )
            );
        }
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
fn multi_page_constraint_writer_rejects_invalid_edits() {
    for (base, index, replacement) in [
        (
            include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_native.gsp")
                .as_slice(),
            4,
            Kind::PointOnSegment(2, 0.5),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/two_page_arc_constrained_native.gsp")
                .as_slice(),
            5,
            Kind::PointOnArc(5, 1.1),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/two_page_circle_constrained_native.gsp")
                .as_slice(),
            4,
            Kind::PointOnCircle(4, 0.5, 0.5),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.checkpoint();
        document.objects[index].kind = replacement;
        assert!(document.can_save().is_err());
        assert!(document.encode().is_err());
    }
}
#[test]
fn multi_page_visual_styles_save_on_both_pages() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    assert!(document.partial_view);
    let blue = RgbColor {
        red: 0,
        green: 0,
        blue: 255,
    };
    let purple = RgbColor {
        red: 128,
        green: 0,
        blue: 128,
    };
    let green = RgbColor {
        red: 0,
        green: 128,
        blue: 0,
    };
    document.active_page = 0;
    document.selected.insert(1);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Large)
            .unwrap(),
        1
    );
    assert_eq!(document.set_selected_color(blue).unwrap(), 1);
    document.active_page = 1;
    document.selected.clear();
    document.selected.insert(5);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Small)
            .unwrap(),
        1
    );
    assert_eq!(document.set_selected_color(purple).unwrap(), 1);
    document.selected.clear();
    document.selected.insert(4);
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
    assert_eq!(document.set_selected_color(green).unwrap(), 1);
    assert!(document.can_save().is_ok());
    let saved = document.encode().unwrap();
    assert_eq!(
        saved,
        include_bytes!("../../../../../tests/fixtures/two_page_styled_port.gsp")
    );
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(
        reopened.objects[0].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Large,
            color: blue
        })
    );
    assert_eq!(
        reopened.objects[3].visual_style,
        Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Dashed
            },
            color: green
        })
    );
    assert_eq!(
        reopened.objects[4].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Small,
            color: purple
        })
    );
    assert!(document.move_point(5, Point { x: 246., y: 111. }));
    document.add_link_page_button(0).unwrap();
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_style_link_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let combined = Document::open(&path).unwrap();
    assert_eq!(
        combined.objects[0].visual_style,
        reopened.objects[0].visual_style
    );
    assert_eq!(
        combined.objects[3].visual_style,
        reopened.objects[3].visual_style
    );
    assert_eq!(
        combined.objects[4].visual_style,
        reopened.objects[4].visual_style
    );
    assert!(
        matches!(combined.objects[4].kind, Kind::PointOnSegment(4, t) if (t - 0.75).abs() < 1e-12)
    );
    assert!(matches!(
        combined.objects[5].kind,
        Kind::LinkPageButton { target_page: 0, .. }
    ));
    std::fs::remove_file(path).unwrap();

    let mut invalid = Document::from_bytes(base, None).unwrap();
    invalid.checkpoint();
    invalid.objects[0].visual_style = Some(VisualStyle::Stroke {
        line: SegmentLineStyle {
            width: LineWidth::Thick,
            pattern: LinePattern::Dashed,
        },
        color: green,
    });
    assert!(invalid.can_save().is_err());
}
#[test]
fn multi_page_trace_flags_save_and_toggle_back() {
    let base = include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    document.selected.extend([4, 5]);
    assert_eq!(document.set_selected_tracing(true).unwrap(), 2);
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_trace_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_traced_port.gsp")
    );
    assert!(document.objects[3].tracing);
    assert!(document.objects[4].tracing);
    assert_eq!(document.set_selected_tracing(false).unwrap(), 2);
    document.save_to(&path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), base);
    std::fs::remove_file(path).unwrap();
}
