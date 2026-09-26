use super::*;

#[test]
fn multi_page_points_and_link_append_with_page_local_references() {
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 0;
    let first = document.add(Kind::Free(Point { x: 150., y: 200. }));
    document.active_page = 1;
    let constrained = document.add(Kind::PointOnSegment(4, 0.75));
    let link = document.add_link_page_button(0).unwrap();
    assert_eq!((first, constrained, link), (6, 7, 8));
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_additions_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_mixed_additions_port.gsp")
    );
    assert_eq!(document.objects[1].page, 0);
    assert!(
        matches!(document.objects[6].kind, Kind::PointOnSegment(5, t) if (t - 0.75).abs() < 1e-12)
    );
    assert!(matches!(
        document.objects[7].kind,
        Kind::LinkPageButton { target_page: 0, .. }
    ));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn multi_page_hide_show_buttons_append_with_page_local_targets() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_hide_show_second_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_hide_show_second_port.gsp").as_slice(),
        include_bytes!(
            "../../../tests/fixtures/two_page_hide_show_second_port_clicked_by_wine.gsp"
        )
        .as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_hide_show_both_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_hide_show_immediate_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_new_hide_show_link_port.gsp").as_slice(),
        include_bytes!(
            "../../../tests/fixtures/two_page_new_hide_show_link_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.page_count, 2);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let native = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_hide_show_second_native.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(native.objects[5].kind.dependencies(), vec![5, 4]);
    let local_targets = |bytes: &[u8]| {
        let references = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x07d2)
            .last()
            .unwrap();
        bytes[references.data]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        local_targets(include_bytes!(
            "../../../tests/fixtures/two_page_hide_show_second_native.gsp"
        )),
        vec![2, 4, 3]
    );
    assert_eq!(
        local_targets(include_bytes!(
            "../../../tests/fixtures/two_page_hide_show_second_port.gsp"
        )),
        vec![2, 1, 3]
    );

    let base = include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut second = Document::from_bytes(base, None).unwrap();
    second.active_page = 1;
    assert_eq!(second.add_hide_show_button_for_targets(&[2, 4]).unwrap(), 6);
    assert!(second.add_hide_show_button_for_targets(&[1]).is_err());
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_hide_show_{}.gsp",
        std::process::id()
    ));
    second.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_hide_show_second_port.gsp"),
            second.active_page
        )
    );
    second.activate_hide_show_button(6).unwrap();
    assert!(!second.objects[1].visible);
    assert!(!second.objects[3].visible);
    assert!(second.objects[0].visible);
    assert!(second.objects[2].visible);
    assert!(second.objects[4].visible);

    let mut both = Document::from_bytes(base, None).unwrap();
    both.active_page = 1;
    both.add_hide_show_button_for_targets(&[2, 4]).unwrap();
    both.active_page = 0;
    both.add_hide_show_button_for_targets(&[1]).unwrap();
    both.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_hide_show_both_port.gsp"),
            both.active_page
        )
    );
    assert_eq!(both.objects[1].kind.dependencies(), vec![1]);
    assert_eq!(both.objects[6].kind.dependencies(), vec![3, 5]);

    let mut linked = Document::from_bytes(base, None).unwrap();
    linked.active_page = 0;
    linked.add_hide_show_button_for_targets(&[1]).unwrap();
    linked.active_page = 1;
    linked.add_link_page_button_with_target(0, Some(1)).unwrap();
    linked.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_new_hide_show_link_port.gsp"),
            linked.active_page
        )
    );
    let wine = Document::from_bytes(
        include_bytes!(
            "../../../tests/fixtures/two_page_new_hide_show_link_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(!wine.objects[0].visible);
    assert!(matches!(
        wine.objects[1].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn multi_page_presentation_buttons_use_page_local_actions() {
    let base = include_bytes!("../../../tests/fixtures/two_page_presentation_base_port.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    assert!(
        document
            .can_add_presentation_button_for_targets(&[6, 7])
            .is_ok()
    );
    assert!(
        document
            .can_add_presentation_button_for_targets(&[6, 6])
            .is_err()
    );
    assert!(
        document
            .can_add_presentation_button_for_targets(&[1, 6])
            .is_err()
    );
    assert_eq!(
        document
            .add_presentation_button_for_targets(&[6, 7], PresentationMode::Simultaneous)
            .unwrap(),
        8
    );
    let bytes = document.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../tests/fixtures/two_page_presentation_port.gsp")
    );
    let references = records(&bytes)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x07d2)
        .last()
        .unwrap();
    let refs = bytes[references.data]
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(refs, vec![2, 5, 6]);
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(reopened.encode().unwrap(), bytes);
    assert!(matches!(
        reopened.objects[7].kind,
        Kind::PresentationButton {
            ref targets,
            mode: PresentationMode::Simultaneous,
            ..
        } if targets == &[6, 7]
    ));
    let mut sequential = Document::from_bytes(base, None).unwrap();
    sequential.active_page = 1;
    sequential
        .add_presentation_button_for_targets(&[6, 7], PresentationMode::Sequential)
        .unwrap();
    let sequential_bytes = sequential.encode().unwrap();
    assert_eq!(
        sequential_bytes,
        include_bytes!("../../../tests/fixtures/two_page_presentation_sequential_port.gsp")
    );
    let sequential_reopened = Document::from_bytes(&sequential_bytes, None).unwrap();
    assert!(matches!(
        sequential_reopened.objects[7].kind,
        Kind::PresentationButton {
            ref targets,
            mode: PresentationMode::Sequential,
            ..
        } if targets == &[6, 7]
    ));
    assert_eq!(sequential_reopened.encode().unwrap(), sequential_bytes);
    let clicked = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_presentation_port_clicked_by_wine.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(clicked.encode().unwrap(), clicked.original);
    assert!(clicked.objects[0].visible);
    assert!(!clicked.objects[1].visible);
    assert!(!clicked.objects[2].visible);
    assert!(!clicked.objects[3].visible);
    assert!(clicked.objects[4].visible);
    let sequential_clicked = Document::from_bytes(
        include_bytes!(
            "../../../tests/fixtures/two_page_presentation_sequential_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert_eq!(
        sequential_clicked.encode().unwrap(),
        sequential_clicked.original
    );
    assert_eq!(
        sequential_clicked
            .objects
            .iter()
            .map(|object| object.visible)
            .collect::<Vec<_>>(),
        clicked
            .objects
            .iter()
            .map(|object| object.visible)
            .collect::<Vec<_>>()
    );
}

#[test]
fn multi_page_clear_remaps_action_buttons_and_dependents() {
    let base = include_bytes!("../../../tests/fixtures/two_page_presentation_port.gsp");
    for (selected, removed, expected) in [
        (
            6,
            vec![6, 8],
            include_bytes!("../../../tests/fixtures/two_page_presentation_parent_cleared_port.gsp")
                .as_slice(),
        ),
        (
            2,
            vec![2, 4, 5, 6, 8],
            include_bytes!("../../../tests/fixtures/two_page_presentation_target_cleared_port.gsp")
                .as_slice(),
        ),
        (
            5,
            vec![5],
            include_bytes!("../../../tests/fixtures/two_page_presentation_point_cleared_port.gsp")
                .as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        document.selected.insert(selected);
        assert_eq!(document.clear_selected().unwrap(), removed);
        let encoded = document.encode().unwrap();
        assert_eq!(encoded, expected);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.encode().unwrap(), encoded);
    }
    let clicked = Document::from_bytes(
        include_bytes!(
            "../../../tests/fixtures/two_page_presentation_point_cleared_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert_eq!(clicked.encode().unwrap(), clicked.original);
    assert_eq!(clicked.objects.len(), 7);
    assert!(clicked.objects[0].visible);
    assert!(!clicked.objects[1].visible);
    assert!(!clicked.objects[2].visible);
    assert!(!clicked.objects[3].visible);
    assert!(matches!(
        clicked.objects[6].kind,
        Kind::PresentationButton { ref targets, .. } if targets == &[5, 6]
    ));
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_animation_point_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_movement_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_scroll_port.gsp").as_slice(),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        document.active_page = 1;
        document.selected.insert(6);
        assert_eq!(document.clear_selected().unwrap(), vec![6]);
        let encoded = document.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.objects.len(), 5);
        assert_eq!(reopened.encode().unwrap(), encoded);
    }
}

#[test]
fn multi_page_animation_buttons_append_with_page_local_targets() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_animation_point_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_animation_point_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_animation_segment_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_animation_segment_port.gsp").as_slice(),
        include_bytes!(
            "../../../tests/fixtures/two_page_animation_segment_port_clicked_by_wine.gsp"
        )
        .as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_animation_both_port.gsp").as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.page_count, 2);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let local_targets = |bytes: &[u8]| {
        let references = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x07d2)
            .last()
            .unwrap();
        bytes[references.data]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>()
    };
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_animation_segment_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_animation_segment_port.gsp").as_slice(),
    ] {
        assert_eq!(local_targets(bytes), vec![2, 2, 1]);
    }
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut point = Document::from_bytes(base, None).unwrap();
    point.active_page = 1;
    let point_settings = point.animation_button_default_settings(5).unwrap();
    assert_eq!(
        point
            .add_animation_button_with_settings(5, point_settings)
            .unwrap(),
        6
    );
    assert!(
        point
            .add_animation_button_with_settings(1, point_settings)
            .is_err()
    );
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_animation_{}.gsp",
        std::process::id()
    ));
    point.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_animation_point_port.gsp"),
            point.active_page
        )
    );

    let mut both = Document::from_bytes(base, None).unwrap();
    both.active_page = 1;
    both.add_animation_button_with_settings(5, point_settings)
        .unwrap();
    both.active_page = 0;
    let first_settings = both.animation_button_default_settings(1).unwrap();
    both.add_animation_button_with_settings(1, first_settings)
        .unwrap();
    both.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_animation_both_port.gsp"),
            both.active_page
        )
    );
    assert_eq!(both.objects[1].kind.dependencies(), vec![1]);
    assert_eq!(both.objects[6].kind.dependencies(), vec![6]);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn multi_page_movement_buttons_append_with_page_local_points() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_movement_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_movement_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_movement_port_clicked_by_wine.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_movement_both_port.gsp").as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.page_count, 2);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let local_targets = |bytes: &[u8]| {
        let references = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x07d2)
            .last()
            .unwrap();
        bytes[references.data]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>()
    };
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_movement_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_movement_port.gsp").as_slice(),
    ] {
        assert_eq!(local_targets(bytes), vec![2, 1, 2]);
    }
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut second = Document::from_bytes(base, None).unwrap();
    second.active_page = 1;
    assert_eq!(
        second
            .add_movement_button(2, 3, MovementSpeed::Medium, true)
            .unwrap(),
        6
    );
    assert!(second.can_add_movement_button(1, 3).is_err());
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_movement_{}.gsp",
        std::process::id()
    ));
    second.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_movement_port.gsp"),
            second.active_page
        )
    );

    let mut both = Document::from_bytes(base, None).unwrap();
    both.active_page = 1;
    both.add_movement_button(2, 3, MovementSpeed::Medium, true)
        .unwrap();
    both.active_page = 0;
    assert_eq!(both.add(Kind::Free(Point { x: 200., y: 200. })), 7);
    both.add_movement_button(1, 7, MovementSpeed::Instant, false)
        .unwrap();
    both.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_movement_both_port.gsp"),
            both.active_page
        )
    );
    assert_eq!(both.objects[2].kind.dependencies(), vec![1, 2]);
    assert_eq!(both.objects[7].kind.dependencies(), vec![4, 5]);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn multi_page_scroll_buttons_append_with_page_local_points() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_scroll_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_scroll_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_scroll_both_port.gsp").as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.page_count, 2);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    for bytes in [
        include_bytes!("../../../tests/fixtures/two_page_scroll_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_scroll_port.gsp").as_slice(),
    ] {
        let references = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x07d2)
            .last()
            .unwrap();
        assert_eq!(&bytes[references.data], &[1, 0, 0, 0, 1, 0, 0, 0]);
    }
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut second = Document::from_bytes(base, None).unwrap();
    second.active_page = 1;
    assert_eq!(
        second
            .add_scroll_button(2, ScrollPlacement::UpperLeft)
            .unwrap(),
        6
    );
    assert!(second.can_add_scroll_button(1).is_err());
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_scroll_{}.gsp",
        std::process::id()
    ));
    second.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_scroll_port.gsp"),
            second.active_page
        )
    );
    let mut both = Document::from_bytes(base, None).unwrap();
    both.active_page = 1;
    both.add_scroll_button(2, ScrollPlacement::UpperLeft)
        .unwrap();
    both.active_page = 0;
    both.add_scroll_button(1, ScrollPlacement::Center).unwrap();
    both.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        expected_page(
            include_bytes!("../../../tests/fixtures/two_page_scroll_both_port.gsp"),
            both.active_page
        )
    );
    assert_eq!(both.objects[1].kind.dependencies(), vec![1]);
    assert_eq!(both.objects[6].kind.dependencies(), vec![3]);
    std::fs::remove_file(path).unwrap();
}
