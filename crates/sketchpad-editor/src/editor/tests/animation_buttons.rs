use super::*;

#[test]
fn clicking_animation_button_starts_its_target_without_controller() {
    let doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_button_base.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let button = editor.doc.add_animation_button(4).unwrap();
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 250., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[button - 1].kind else {
        panic!("Expected animation button")
    };
    let click = crate::render::action_button_rect(&viewport, *anchor, label).center();
    let before = editor.doc.position(4).unwrap();
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 1);
    assert!(!editor.show_motion_controller);
    editor.tick_animation(0.1);
    assert_ne!(editor.doc.position(4).unwrap(), before);
}

#[test]
fn animation_button_direction_speed_once_and_second_click_control_motion() {
    let base = include_bytes!("../../../../../tests/fixtures/animation_button_base.gsp");
    for (direction, speed, expected_phase) in [
        (AnimationDirection::Forward, 24., 0.55),
        (AnimationDirection::Backward, 24., 0.45),
        (AnimationDirection::Forward, 80., 0.5 + 0.05 * 80. / 24.),
        (AnimationDirection::Forward, 4., 0.5 + 0.05 * 4. / 24.),
    ] {
        let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
        let settings = AnimationSettings {
            direction,
            speed,
            once: false,
            ..AnimationSettings::default()
        };
        editor
            .doc
            .add_animation_button_with_settings(4, settings)
            .unwrap();
        editor.animate_button_targets(&[4], &[settings]);
        editor.tick_animation(0.1);
        assert!((editor.animations[0].phase - expected_phase).abs() < 1e-9);
        editor.animate_button_targets(&[4], &[settings]);
        assert!(editor.animations.is_empty());
    }
    let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
    let settings = AnimationSettings {
        direction: AnimationDirection::Forward,
        speed: 80.,
        once: true,
        ..AnimationSettings::default()
    };
    editor.animate_button_targets(&[4], &[settings]);
    for _ in 0..8 {
        editor.tick_animation(0.1);
    }
    assert!(editor.animations.is_empty());
    assert!((editor.doc.position(4).unwrap().x - 250.).abs() < 1e-9);
}

#[test]
fn two_point_animation_button_starts_and_stops_both_tracks() {
    let doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_button_two_points_base.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let settings = [
        AnimationSettings::default(),
        AnimationSettings {
            direction: AnimationDirection::Backward,
            ..AnimationSettings::default()
        },
    ];
    let button = editor
        .doc
        .add_animation_button_for_targets_with_settings(&[(4, settings[0]), (5, settings[1])])
        .unwrap();
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 250., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[button - 1].kind else {
        panic!("Expected Animation button")
    };
    let click = crate::render::action_button_rect(&viewport, *anchor, label).center();
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 2);
    assert!(!editor.show_motion_controller);
    editor.tick_animation(0.1);
    assert!((editor.doc.position(4).unwrap().x - 235.).abs() < 1e-9);
    assert!((editor.doc.position(5).unwrap().x - 265.).abs() < 1e-9);
    editor.click(&viewport, click, false);
    assert!(editor.animations.is_empty());
}

#[test]
fn three_point_animation_button_moves_all_targets() {
    let doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/action_animation_three_points_port.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[6].kind else {
        panic!("Expected Animation button")
    };
    let click = crate::render::action_button_rect(&viewport, *anchor, label).center();
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 3);
    editor.tick_animation(0.1);
    for (id, expected_x) in [(4, 140.), (5, 200.), (6, 260.)] {
        assert!((editor.doc.position(id).unwrap().x - expected_x).abs() < 1e-9);
    }
    editor.click(&viewport, click, false);
    assert!(editor.animations.is_empty());
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(6), editor.doc.position(6));
}

#[test]
fn mixed_point_animation_button_moves_each_point_by_its_path() {
    let doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/action_animation_mixed_points_port.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[9].kind else {
        panic!("Expected Animation button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    let before = [1, 5, 9].map(|id| editor.doc.position(id).unwrap());
    assert_eq!(editor.hit(&viewport, click), Some(10));
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 3);
    editor.tick_animation(0.1);
    for (id, initial) in [1, 5, 9].into_iter().zip(before) {
        assert_ne!(editor.doc.position(id).unwrap(), initial);
    }
    assert!((editor.doc.position(5).unwrap().x - 210.).abs() < 1e-9);
    let on_circle = editor.doc.position(9).unwrap();
    assert!((on_circle.x - (300. + 30. * (0.3 * std::f64::consts::TAU).cos())).abs() < 1e-9);
    editor.click(&viewport, click, false);
    assert!(editor.animations.is_empty());
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn selected_parameter_animates_and_saves_with_one_undo_step() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/animation_parameter_base.gsp"),
            None,
        )
        .unwrap(),
    );
    editor.select_only(1);
    editor.animate_selected();
    assert_eq!(editor.animations.len(), 1);
    assert_eq!(editor.animations[0].parameter_domain, Some((-100., 100.)));
    editor.tick_animation(0.1);
    let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
        panic!("Expected parameter")
    };
    assert!((group.value() - 1.1).abs() < 1e-9);
    editor.stop_animation();
    let saved = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    let Kind::Parameter(group) = &reopened.objects[0].kind else {
        panic!("Expected parameter")
    };
    assert!((group.value() - 1.1).abs() < 1e-9);
    editor.doc.undo();
    let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
        panic!("Expected parameter")
    };
    assert!((group.value() - 1.).abs() < 1e-9);
}

#[test]
fn parameter_animation_button_click_starts_and_stops_motion() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/action_animation_parameter_port.gsp"),
            None,
        )
        .unwrap(),
    );
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[1].kind else {
        panic!("Expected Animation button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    assert_eq!(editor.hit(&viewport, click), Some(2));
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 1);
    assert!(!editor.show_motion_controller);
    editor.tick_animation(0.1);
    let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
        panic!("Expected parameter")
    };
    assert!((group.value() - 1.1).abs() < 1e-9);
    editor.click(&viewport, click, false);
    assert!(editor.animations.is_empty());
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn parameter_animation_bounces_at_default_domain_edge() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/animation_parameter_base.gsp"),
            None,
        )
        .unwrap(),
    );
    editor.doc.set_parameter_value(1, 99.9).unwrap();
    editor.select_only(1);
    editor.animate_selected();
    editor.tick_animation(0.1);
    assert_eq!(editor.animations.len(), 1);
    let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
        panic!("Expected parameter")
    };
    assert!((group.value() - 100.).abs() < 1e-9);
    editor.tick_animation(0.1);
    assert_eq!(editor.animations.len(), 1);
    let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
        panic!("Expected parameter")
    };
    assert!((group.value() - 99.9).abs() < 1e-9);
}

#[test]
fn custom_parameter_button_uses_rate_domain_and_discrete_step() {
    use sketchpad_engine::action_records::ParameterAnimationSettings;
    for (discrete, expected) in [(false, 1.2), (true, 1.0)] {
        let mut editor = Editor::new(
            Document::from_bytes(
                include_bytes!("../../../../../tests/fixtures/animation_parameter_base.gsp"),
                None,
            )
            .unwrap(),
        );
        let settings = AnimationSettings {
            direction: AnimationDirection::Bidirectional,
            speed: 2.,
            once: false,
            parameter: Some(ParameterAnimationSettings {
                discrete,
                step: 0.5,
                min: -3.,
                max: 7.,
            }),
        };
        let button = editor
            .doc
            .add_animation_button_with_settings(1, settings)
            .unwrap();
        let encoded = editor.doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(&reopened.objects[button - 1].kind, Kind::AnimationButton { settings: actual, .. } if actual == &[settings])
        );
        editor.animate_button_targets(&[1], &[settings]);
        assert_eq!(editor.animations[0].parameter_domain, Some((-3., 7.)));
        editor.tick_animation(0.1);
        let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
            panic!("Expected parameter")
        };
        assert!((group.value() - expected).abs() < 1e-9);
        if discrete {
            for _ in 0..4 {
                editor.tick_animation(0.1);
            }
            let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
                panic!("Expected parameter")
            };
            assert!((group.value() - 1.5).abs() < 1e-9);
            for _ in 0..15 {
                editor.tick_animation(0.1);
            }
            let Kind::Parameter(group) = &editor.doc.objects[0].kind else {
                panic!("Expected parameter")
            };
            assert!((group.value() - 3.).abs() < 1e-9);
        }
    }
}
