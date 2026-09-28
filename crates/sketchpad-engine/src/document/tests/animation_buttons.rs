use super::*;

#[test]
fn animation_button_variants_import_and_preserve_settings() {
    use crate::action_records::AnimationDirection;
    for (bytes, direction, speed, once) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_forward_original.gsp")
                .as_slice(),
            AnimationDirection::Forward,
            24.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_backward_original.gsp")
                .as_slice(),
            AnimationDirection::Backward,
            24.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_random_original.gsp")
                .as_slice(),
            AnimationDirection::Random,
            24.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_slow_original.gsp").as_slice(),
            AnimationDirection::Bidirectional,
            4.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_fast_original.gsp").as_slice(),
            AnimationDirection::Bidirectional,
            80.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_forward_once_original.gsp")
                .as_slice(),
            AnimationDirection::Forward,
            24.,
            true,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(
            doc.objects[4].kind,
            Kind::AnimationButton { ref settings, .. }
                if settings.len() == 1 && settings[0] == AnimationSettings { direction, speed, once, parameter: None }
        ));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_button_base.gsp"),
        None,
    )
    .unwrap();
    let settings = AnimationSettings {
        direction: AnimationDirection::Backward,
        speed: 80.,
        once: true,
        ..AnimationSettings::default()
    };
    let id = doc.add_animation_button_with_settings(4, settings).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(reopened.objects[id-1].kind, Kind::AnimationButton { settings: ref actual, .. } if actual == &[settings])
    );
    assert_eq!(reopened.encode().unwrap(), encoded);
    for (bytes, target, button) in [
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_circle_original.gsp")
                .as_slice(),
            4,
            5,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/action_animation_free_original.gsp").as_slice(),
            1,
            2,
        ),
    ] {
        let native = Document::from_bytes(bytes, None).unwrap();
        assert!(!native.partial_view);
        assert!(
            matches!(native.objects[button-1].kind, Kind::AnimationButton { ref targets, ref settings, .. } if targets == &[target] && settings == &[AnimationSettings { direction: AnimationDirection::Forward, speed: 24., once: false, parameter: None }])
        );
        assert_eq!(native.encode().unwrap(), bytes);
    }
    let mut circle_doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_button_circle_base.gsp"),
        None,
    )
    .unwrap();
    let circle_id = circle_doc.add_animation_button(4).unwrap();
    assert!(matches!(
        circle_doc.objects[circle_id - 1].kind,
        Kind::AnimationButton { ref settings, .. }
            if settings.len() == 1 && settings[0].direction == AnimationDirection::Forward
    ));
    let mut free_doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_free_point_base.gsp"),
        None,
    )
    .unwrap();
    let free_id = free_doc.add_animation_button(1).unwrap();
    assert!(matches!(
        free_doc.objects[free_id - 1].kind,
        Kind::AnimationButton { ref settings, .. }
            if settings.len() == 1 && settings[0].direction == AnimationDirection::Forward
    ));
}

#[test]
fn two_point_animation_button_imports_creates_and_copies() {
    use crate::action_records::AnimationDirection;
    let mut native = Document::open(Path::new(
        "../../tests/fixtures/action_animation_two_points_second_backward.gsp",
    ))
    .unwrap();
    let expected_settings = vec![
        AnimationSettings::default(),
        AnimationSettings {
            direction: AnimationDirection::Backward,
            ..AnimationSettings::default()
        },
    ];
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_two_points_original.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_two_points_second_backward.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_two_points_clicked.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_two_points_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[5].kind.dependencies(), vec![4, 5]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    assert!(
        matches!(&native.objects[5].kind, Kind::AnimationButton { settings, .. } if settings == &expected_settings)
    );
    native.selected.insert(6);
    let clipboard = native.copy_selected().unwrap();
    let mut destination = Document::new();
    let pasted = destination.paste(&clipboard).unwrap();
    assert_eq!(
        destination.objects[pasted.last().unwrap() - 1]
            .kind
            .dependencies(),
        vec![4, 5]
    );
    assert!(
        !Document::from_bytes(&destination.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );

    let mut generated = Document::open(Path::new(
        "../../tests/fixtures/animation_button_two_points_base.gsp",
    ))
    .unwrap();
    let button = generated
        .add_animation_button_for_targets_with_settings(&[
            (4, expected_settings[0]),
            (5, expected_settings[1]),
        ])
        .unwrap();
    assert_eq!(button, 6);
    assert!(
        generated
            .add_animation_button_for_targets_with_settings(&[
                (4, expected_settings[0]),
                (4, expected_settings[1])
            ])
            .is_err()
    );
    let encoded = generated.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(
        matches!(&reopened.objects[5].kind, Kind::AnimationButton { settings, .. } if settings == &expected_settings)
    );
}
#[test]
fn three_point_animation_button_imports_edits_and_copies() {
    use crate::action_records::AnimationDirection;
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_three_points_native.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_three_points_port.gsp").as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_three_points_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert!(matches!(
            &document.objects[6].kind,
            Kind::AnimationButton { targets, settings, .. }
                if targets == &[4, 5, 6]
                    && settings == &[AnimationSettings::default(); 3]
        ));
        assert!(
            records(bytes)
                .unwrap()
                .iter()
                .any(|record| record.kind == 0x07d3 && record.data.len() == 100)
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let mut document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_button_three_points_base.gsp"),
        None,
    )
    .unwrap();
    let settings = [
        AnimationSettings::default(),
        AnimationSettings {
            direction: AnimationDirection::Backward,
            speed: 4.,
            once: false,
            ..AnimationSettings::default()
        },
        AnimationSettings {
            direction: AnimationDirection::Forward,
            speed: 80.,
            once: true,
            ..AnimationSettings::default()
        },
    ];
    let button = document
        .add_animation_button_for_targets_with_settings(&[
            (4, settings[0]),
            (5, settings[1]),
            (6, settings[2]),
        ])
        .unwrap();
    let generated = document.encode().unwrap();
    assert_eq!(
        generated,
        include_bytes!("../../../../../tests/fixtures/action_animation_three_points_settings_port.gsp")
    );
    let reopened = Document::from_bytes(&generated, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        &reopened.objects[button - 1].kind,
        Kind::AnimationButton { targets, settings: actual, .. }
            if targets == &[4, 5, 6] && actual == &settings
    ));
    document.selected.insert(button);
    let clipboard = document.copy_selected().unwrap();
    let mut destination = Document::new();
    let pasted = destination.paste(&clipboard).unwrap();
    assert_eq!(pasted.last(), Some(&7));
    assert_eq!(destination.objects[6].kind.dependencies(), vec![4, 5, 6]);
    assert!(
        !Document::from_bytes(&destination.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn mixed_point_animation_button_keeps_per_target_defaults() {
    use crate::action_records::AnimationDirection;
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_mixed_points_native.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/action_animation_mixed_points_port.gsp").as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_mixed_points_port_clicked_by_wine.gsp"
        )
        .as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::AnimationButton {
            targets, settings, ..
        } = &document.objects[9].kind
        else {
            panic!("Expected Animation button")
        };
        let mut defaults = targets
            .iter()
            .copied()
            .zip(settings.iter().map(|setting| setting.direction))
            .collect::<Vec<_>>();
        defaults.sort_by_key(|(target, _)| *target);
        assert_eq!(
            defaults,
            vec![
                (1, AnimationDirection::Forward),
                (5, AnimationDirection::Bidirectional),
                (9, AnimationDirection::Forward),
            ]
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
