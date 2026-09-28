use super::*;

#[test]
fn hide_show_button_survives_original_click_and_port_edit() {
    let mut doc = Document::open(Path::new("../../tests/fixtures/action_hide_show.gsp")).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects[3].kind.dependencies(), vec![3]);
    doc.activate_hide_show_button(4).unwrap();
    assert!(!doc.objects[2].visible);
    assert!(matches!(
        doc.objects[3].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.objects[2].visible, doc.objects[2].visible);
    assert_eq!(reopened.objects[3].kind, doc.objects[3].kind);
    doc.undo();
    assert!(doc.objects[2].visible);
    doc.redo();
    assert!(!doc.objects[2].visible);
    let mut wine =
        Document::open(Path::new("../../tests/fixtures/action_port_wine_clicked.gsp")).unwrap();
    assert!(!wine.partial_view);
    assert!(!wine.objects[2].visible);
    assert!(matches!(
        wine.objects[3].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    wine.activate_hide_show_button(4).unwrap();
    let path = std::env::temp_dir().join(format!("sketchpad_action_{}.gsp", std::process::id()));
    wine.save_to(&path).unwrap();
    let port_saved = Document::open(&path).unwrap();
    assert!(port_saved.objects[2].visible);
    assert!(matches!(
        port_saved.objects[3].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Hide,
            ..
        }
    ));
    fs::remove_file(path).unwrap();
}

#[test]
fn two_target_hide_show_button_import_create_and_toggle() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_hide_show_two_points_original.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_hide_show_two_points_clicked.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_hide_show_two_points_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[3].kind.dependencies(), vec![1, 2]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let mut native = Document::open(Path::new(
        "../../tests/fixtures/action_hide_show_two_points_original.gsp",
    ))
    .unwrap();
    native.activate_hide_show_button(4).unwrap();
    assert!(!native.objects[0].visible);
    assert!(!native.objects[1].visible);
    assert!(native.objects[2].visible);
    let hidden = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
    assert_eq!(hidden.objects[3].kind.dependencies(), vec![1, 2]);
    assert!(matches!(
        hidden.objects[3].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    native.undo();
    assert!(native.objects[0].visible && native.objects[1].visible);

    let mut generated = Document::new();
    let first = generated.add(Kind::Free(Point { x: 150., y: 120. }));
    let second = generated.add(Kind::Free(Point { x: 270., y: 120. }));
    generated.add(Kind::Segment(first, second));
    let button = generated
        .add_hide_show_button_for_targets(&[first, second])
        .unwrap();
    assert!(generated.add_hide_show_button_for_targets(&[]).is_err());
    assert!(
        generated
            .add_hide_show_button_for_targets(&[first, first])
            .is_err()
    );
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.objects[button - 1].kind.dependencies(),
        vec![first, second]
    );

    let mut source = Document::open(Path::new(
        "../../tests/fixtures/action_hide_show_two_points_original.gsp",
    ))
    .unwrap();
    source.selected.insert(4);
    let clipboard = source.copy_selected().unwrap();
    let mut destination = Document::new();
    let pasted = destination.paste(&clipboard).unwrap();
    let pasted_button = *pasted.last().unwrap();
    assert_eq!(
        destination.objects[pasted_button - 1].kind.dependencies(),
        vec![1, 2]
    );
    assert!(
        !Document::from_bytes(&destination.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn three_target_hide_show_button_matches_native_click() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_hide_show_three_points_native.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_hide_show_three_points_port.gsp").as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_hide_show_three_points_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[3].kind.dependencies(), vec![1, 2, 3]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let mut generated = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/action_hide_show_three_points_port.gsp"),
        None,
    )
    .unwrap();
    generated.activate_hide_show_button(4).unwrap();
    assert!(generated.objects[..3].iter().all(|object| !object.visible));
    assert!(matches!(
        generated.objects[3].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    let wine = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/action_hide_show_three_points_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(wine.objects[..3].iter().all(|object| !object.visible));
    assert!(matches!(
        wine.objects[3].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
}
#[test]
fn mixed_target_hide_show_button_matches_native_click() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_hide_show_mixed_native.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_hide_show_mixed_port.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_hide_show_mixed_port_clicked_by_wine.gsp")
            .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[6].kind.dependencies(), vec![4, 5, 6]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let mut generated = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/action_hide_show_mixed_port.gsp"),
        None,
    )
    .unwrap();
    generated.activate_hide_show_button(7).unwrap();
    assert!(generated.objects[..3].iter().all(|object| object.visible));
    assert!(generated.objects[3..6].iter().all(|object| !object.visible));
    assert!(matches!(
        generated.objects[6].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    let wine = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/action_hide_show_mixed_port_clicked_by_wine.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        generated
            .objects
            .iter()
            .map(|object| object.visible)
            .collect::<Vec<_>>(),
        wine.objects
            .iter()
            .map(|object| object.visible)
            .collect::<Vec<_>>()
    );
}
#[test]
fn animation_button_imports_click_state_and_generated_file_round_trips() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_original.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_clicked.gsp").as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(
            &doc.objects[4].kind,
            Kind::AnimationButton { targets, label, anchor: (10, 5), .. }
                if targets == &[4] && label == "动画点"
        ));
        assert_eq!(doc.objects[4].kind.dependencies(), vec![4]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let base = include_bytes!("../../../../../tests/fixtures/animation_button_base.gsp");
    let mut doc = Document::from_bytes(base, None).unwrap();
    let button = doc.add_animation_button(4).unwrap();
    assert_eq!(button, 5);
    let generated = doc.encode().unwrap();
    let reopened = Document::from_bytes(&generated, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), generated);
    assert!(matches!(
        &reopened.objects[4].kind,
        Kind::AnimationButton { targets, label, .. } if targets == &[4] && label == "Animate Point"
    ));
    let clicked =
        include_bytes!("../../../../../tests/fixtures/action_animation_port_clicked_by_wine.gsp");
    let clicked_doc = Document::from_bytes(clicked, None).unwrap();
    assert!(!clicked_doc.partial_view);
    assert_ne!(clicked_doc.position(4), doc.position(4));
    assert_eq!(clicked_doc.encode().unwrap(), clicked);
    doc.selected.insert(button);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let pasted_button = *pasted.last().unwrap();
    assert!(matches!(
        doc.objects[pasted_button - 1].kind,
        Kind::AnimationButton { ref targets, .. } if targets.len() == 1 && targets[0] != 4 && targets[0] < pasted_button
    ));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[pasted_button - 1].kind,
        Kind::AnimationButton { .. }
    ));
}

#[test]
fn ray_and_line_animation_buttons_match_native_defaults() {
    use crate::action_records::AnimationDirection;
    for (base, native) in [
        (
            include_bytes!("../../../../../tests/fixtures/point_on_ray_native.gsp").as_slice(),
            include_bytes!("../../../../../tests/fixtures/action_animation_ray_original.gsp").as_slice(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point_on_line_native.gsp").as_slice(),
            include_bytes!("../../../../../tests/fixtures/action_animation_line_original.gsp").as_slice(),
        ),
    ] {
        let imported = Document::from_bytes(native, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.encode().unwrap(), native);
        assert!(matches!(
            &imported.objects[4].kind,
            Kind::AnimationButton { targets, settings, .. }
                if targets == &[4]
                    && settings.len() == 1
                    && settings[0].direction == AnimationDirection::Bidirectional
                    && settings[0].speed == 24.
        ));
        let mut generated = Document::from_bytes(base, None).unwrap();
        let button = generated.add_animation_button(4).unwrap();
        assert_eq!(button, 5);
        let encoded = generated.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), encoded);
    }
    let clicked =
        include_bytes!("../../../../../tests/fixtures/action_animation_ray_port_clicked_by_wine.gsp");
    let clicked_doc = Document::from_bytes(clicked, None).unwrap();
    assert!(!clicked_doc.partial_view);
    assert_eq!(clicked_doc.encode().unwrap(), clicked);
    assert!(matches!(clicked_doc.objects[3].kind, Kind::PointOnRay(3, t) if t != 0.3));
}

#[test]
fn movement_button_imports_and_generates_editable_file() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_movement_default_original.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_movement_slow_original.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_movement_fast_original.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_movement_instant_original.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_movement_initial_original.gsp").as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert!(matches!(
            &document.objects[3].kind,
            Kind::MovementButton {
                pairs,
                ..
            } if pairs == &vec![(1, 2)]
        ));
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let mut document = Document::new();
    let source = document.add(Kind::Free(Point { x: 150., y: 120. }));
    let destination = document.add(Kind::Free(Point { x: 270., y: 120. }));
    let id = document
        .add_movement_button(source, destination, MovementSpeed::Fast, false)
        .unwrap();
    let bytes = document.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        &reopened.objects[id - 1].kind,
        Kind::MovementButton {
            pairs,
            speed: MovementSpeed::Fast,
            follow: false,
            ..
        } if pairs == &vec![(1, 2)]
    ));
    assert_eq!(reopened.encode().unwrap(), bytes);
}

#[test]
fn scroll_button_imports_and_generates_editable_file() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_scroll_top_left_original.gsp")
                .as_slice(),
            ScrollPlacement::UpperLeft,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_scroll_center_original.gsp").as_slice(),
            ScrollPlacement::Center,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert!(
            matches!(document.objects[3].kind, Kind::ScrollButton { target: 1, placement, .. } if placement == expected)
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let mut document = Document::new();
    let target = document.add(Kind::Free(Point { x: 150., y: 120. }));
    let button = document
        .add_scroll_button(target, ScrollPlacement::Center)
        .unwrap();
    let bytes = document.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[button - 1].kind,
        Kind::ScrollButton {
            target: 1,
            placement: ScrollPlacement::Center,
            ..
        }
    ));
    assert_eq!(reopened.encode().unwrap(), bytes);
}

#[test]
fn presentation_button_imports_and_generates_editable_file() {
    for (bytes, expected, targets) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_presentation_default_original.gsp")
                .as_slice(),
            PresentationMode::Simultaneous,
            vec![3, 4],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/action_presentation_sequential_scroll_original.gsp"
            )
            .as_slice(),
            PresentationMode::Sequential,
            vec![4, 5],
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let last = document.objects.last().unwrap();
        assert!(
            matches!(&last.kind, Kind::PresentationButton { targets: actual, mode, .. }
            if actual == &targets && *mode == expected)
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let base = include_bytes!("../../../../../tests/fixtures/presentation_two_hide_show_base.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    let button = document
        .add_presentation_button_for_targets(&[3, 4], PresentationMode::Sequential)
        .unwrap();
    let bytes = document.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(&reopened.objects[button - 1].kind, Kind::PresentationButton { targets, mode: PresentationMode::Sequential, .. }
        if targets == &[3, 4])
    );
    assert_eq!(reopened.encode().unwrap(), bytes);
}

#[test]
fn presentation_button_clicks_in_native_and_desktop_match() {
    for (bytes, mode) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/action_presentation_port_simultaneous_clicked_by_wine.gsp"
            )
            .as_slice(),
            PresentationMode::Simultaneous,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/action_presentation_port_sequential_clicked_by_wine.gsp"
            )
            .as_slice(),
            PresentationMode::Sequential,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_presentation_port_desktop_clicked.gsp")
                .as_slice(),
            PresentationMode::Simultaneous,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert!(!document.objects[0].visible);
        assert!(!document.objects[1].visible);
        for button in &document.objects[2..4] {
            assert!(matches!(
                button.kind,
                Kind::HideShowButton {
                    action: VisibilityAction::Show,
                    ..
                }
            ));
        }
        assert!(matches!(
            document.objects[4].kind,
            Kind::PresentationButton { mode: actual, .. } if actual == mode
        ));
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
#[test]
fn parameter_animation_button_uses_native_settings_and_anchor() {
    use crate::action_records::AnimationDirection;
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_parameter_native.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_parameter_port.gsp").as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_parameter_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::AnimationButton {
            targets,
            anchor,
            settings,
            ..
        } = &document.objects[1].kind
        else {
            panic!("Expected Animation button")
        };
        assert_eq!(targets, &[1]);
        assert_eq!(*anchor, (10, 47));
        assert_eq!(
            settings,
            &[AnimationSettings {
                direction: AnimationDirection::Bidirectional,
                speed: 1.,
                once: false,
                parameter: Some(crate::action_records::ParameterAnimationSettings::default(),),
            }]
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/parameter_animated_native_saved.gsp"),
        None,
    )
    .unwrap();
    assert!(!document.partial_view);
    let Kind::Parameter(group) = &document.objects[0].kind else {
        panic!("Expected animated parameter")
    };
    assert!((group.value() - 1.957).abs() < 1e-12);
    let clicked = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_parameter_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    let Kind::Parameter(group) = &clicked.objects[0].kind else {
        panic!("Expected clicked parameter")
    };
    assert!((group.value() - 1.).abs() > 0.01);
    for (bytes, rate, discrete, step, min, max) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_parameter_native_rate2.gsp")
                .as_slice(),
            2.,
            false,
            1.,
            -100.,
            100.,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_parameter_native_discrete.gsp"
            )
            .as_slice(),
            2.,
            true,
            0.5,
            -3.,
            7.,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_parameter_custom_continuous.gsp"
            )
            .as_slice(),
            2.5,
            false,
            1.,
            -3.,
            7.,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::AnimationButton { settings, .. } = &document.objects[1].kind else {
            panic!("Expected parameter button")
        };
        assert_eq!(settings[0].speed, rate);
        let options = settings[0].parameter.unwrap();
        assert_eq!(
            (options.discrete, options.step, options.min, options.max),
            (discrete, step, min, max)
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let clicked = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_parameter_custom_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(!clicked.partial_view);
    assert!(matches!(&clicked.objects[0].kind, Kind::Parameter(group) if group.value() == 3.));
    let mut mixed = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_parameter_base.gsp"),
        None,
    )
    .unwrap();
    let point = mixed.add(Kind::Free(Point { x: 200., y: 150. }));
    let settings = [
        (1, mixed.animation_button_default_settings(1).unwrap()),
        (
            point,
            mixed.animation_button_default_settings(point).unwrap(),
        ),
    ];
    assert!(
        mixed
            .add_animation_button_for_targets_with_settings(&settings)
            .is_err()
    );
}

#[test]
fn indirect_segment_and_circle_animation_buttons_keep_native_targets() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_indirect_segment_native.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_indirect_segment_port.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_indirect_circle_native.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_indirect_circle_port.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_indirect_segment_hidden_native.gsp"
        )
        .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_indirect_segment_hidden_port.gsp")
            .as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::AnimationButton {
            targets,
            anchor,
            settings,
            ..
        } = &document.objects[3].kind
        else {
            panic!("Expected Animation button")
        };
        assert_eq!(targets, &[2, 1]);
        assert_eq!(*anchor, (10, 5));
        assert_eq!(settings.len(), 2);
        assert!(settings.iter().all(|setting| {
            setting.direction == crate::action_records::AnimationDirection::Forward
                && setting.speed == 24.
                && !setting.once
                && setting.parameter.is_none()
        }));
        assert_eq!(document.encode().unwrap(), bytes);
    }
}

#[test]
fn line_ray_and_polygon_animation_buttons_keep_native_targets() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_line_native.gsp")
                .as_slice(),
            &[2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_line_port.gsp")
                .as_slice(),
            &[2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_ray_native.gsp")
                .as_slice(),
            &[2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_ray_port.gsp")
                .as_slice(),
            &[2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_polygon_native.gsp")
                .as_slice(),
            &[3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_polygon_port.gsp")
                .as_slice(),
            &[3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_quadrilateral_native.gsp")
                .as_slice(),
            &[4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_quadrilateral_port.gsp")
                .as_slice(),
            &[4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_pentagon_native.gsp")
                .as_slice(),
            &[5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_pentagon_port.gsp")
                .as_slice(),
            &[5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_hexagon_native.gsp")
                .as_slice(),
            &[6, 5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_hexagon_port.gsp")
                .as_slice(),
            &[6, 5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_heptagon_native.gsp")
                .as_slice(),
            &[7, 6, 5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_heptagon_port.gsp")
                .as_slice(),
            &[7, 6, 5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_hexagon_port_clicked_by_wine.gsp")
                .as_slice(),
            &[6, 5, 4, 3, 2, 1][..],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_quadrilateral_port_clicked_by_wine.gsp"
            )
            .as_slice(),
            &[4, 3, 2, 1][..],
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::AnimationButton {
            targets, settings, ..
        } = &document.objects.last().unwrap().kind
        else {
            panic!("Expected Animation button")
        };
        assert_eq!(targets, expected);
        assert_eq!(settings.len(), expected.len());
        assert_eq!(document.encode().unwrap(), bytes);
    }
}

#[test]
fn link_url_buttons_import_and_generate_editable_files() {
    for (bytes, url) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_link_url_default_original.gsp")
                .as_slice(),
            "http://kejianyuan.com",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_link_url_custom_original.gsp")
                .as_slice(),
            "https://example.invalid/sketchpad",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_link_url_even_original.gsp").as_slice(),
            "https://example.org/",
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert!(matches!(&document.objects[1].kind,
            Kind::LinkUrlButton { url: actual, label, anchor }
            if actual == url && label == url && *anchor == (10, 5)
        ));
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let base = include_bytes!("../../../../../tests/fixtures/point.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    let button = document
        .add_link_url_button("https://example.org/")
        .unwrap();
    let bytes = document.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(&reopened.objects[button - 1].kind,
        Kind::LinkUrlButton { url, label, .. }
        if url == "https://example.org/" && label == url
    ));
    assert_eq!(reopened.encode().unwrap(), bytes);
}

#[test]
fn original_multi_pair_and_optional_movement_layouts_remain_lossless() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/movement46_cube_missing_label_native_verified.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(
        matches!(&doc.objects[41].kind, Kind::MovementButton { pairs, .. } if pairs.len() == 3)
    );
    assert!(
        matches!(&doc.objects[61].kind, Kind::MovementButton { pairs, label, .. } if pairs.len() == 1 && label.is_empty())
    );
    assert_eq!(doc.encode().unwrap(), bytes);
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/movement46_course_empty_metadata_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(
        matches!(&doc.objects[366].kind, Kind::MovementButton { pairs, .. } if pairs.len() == 1)
    );
    assert_eq!(doc.encode().unwrap(), bytes);
}
