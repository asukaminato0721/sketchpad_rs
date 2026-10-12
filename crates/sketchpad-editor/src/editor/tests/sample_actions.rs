use super::*;

#[test]
fn parameter_buttons_convert_native_ranges_and_rates_to_display_units() {
    use sketchpad_engine::action_records::ParameterAnimationSettings;
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/numeric_typed_parameters_native_verified.gsp"
    );
    for target in 1..=3 {
        for discrete in [false, true] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            doc.set_parameter_value(target, 1.).unwrap();
            let Kind::Parameter(group) = &doc.objects()[target - 1].kind else {
                panic!("Expected parameter");
            };
            let scale = group.native_unit_scale();
            let settings = AnimationSettings {
                direction: AnimationDirection::Forward,
                speed: if discrete { 10. } else { (10. * scale) as f32 },
                once: false,
                parameter: Some(ParameterAnimationSettings {
                    discrete,
                    step: scale as f32,
                    min: 0.,
                    max: (100. * scale) as f32,
                }),
            };
            let button = doc
                .add_animation_button_with_settings(target, settings)
                .unwrap();
            let mut editor = Editor::new(doc);
            editor.animate_button_targets(button, &[target], &[settings]);
            editor.tick_animation(0.1);
            assert_eq!(editor.animations.len(), 1, "{}", editor.status);
            assert!(
                (editor.doc.numeric_value(target).unwrap() - 2.).abs() < 1e-5,
                "target {target}, discrete={discrete}"
            );
            editor.doc.verify_roundtrip().unwrap();
        }
    }
}

#[test]
fn reset_button_replaces_another_buttons_animation_in_one_click() {
    use sketchpad_engine::action_records::ParameterAnimationSettings;
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/animation_parameter_base.gsp"),
        None,
    )
    .unwrap();
    let run = doc
        .add_animation_button_with_settings(
            1,
            AnimationSettings {
                direction: AnimationDirection::Forward,
                speed: 10.,
                once: false,
                parameter: Some(ParameterAnimationSettings {
                    discrete: false,
                    step: 1.,
                    min: 8.5,
                    max: 8.50001,
                }),
            },
        )
        .unwrap();
    let reset = doc
        .add_animation_button_with_settings(
            1,
            AnimationSettings {
                direction: AnimationDirection::Bidirectional,
                speed: 1000.,
                once: false,
                parameter: Some(ParameterAnimationSettings {
                    discrete: false,
                    step: 1.,
                    min: 0.,
                    max: 0.001,
                }),
            },
        )
        .unwrap();
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: editor.center,
    };
    editor.activate_referenced_action(run, &viewport).unwrap();
    let before = editor.doc.numeric_value(1).unwrap();
    editor.tick_animation(0.1);
    assert_eq!(editor.doc.numeric_value(1), Some(before + 1.));
    editor.activate_referenced_action(reset, &viewport).unwrap();
    assert_eq!(
        editor.animations.len(),
        1,
        "Reset must start on its first click"
    );
    editor.tick_animation(0.1);
    let Kind::Parameter(group) = &editor.doc.objects()[0].kind else {
        unreachable!()
    };
    assert!((0. ..=0.001).contains(&group.value()));
    editor.activate_referenced_action(reset, &viewport).unwrap();
    assert!(
        editor.animations.is_empty(),
        "A second click on the same button stops it"
    );
}

#[test]
fn compact_buttons_receive_clicks_before_overlapping_captions() {
    use sketchpad_engine::action_records::{VisibilityAction, VisibilityOptions};
    let mut doc = Document::new();
    let sources: Vec<_> = (0..3)
        .map(|x| doc.add(Kind::Free(Point { x: x as f64, y: 0. })))
        .collect();
    let buttons: Vec<_> = sources
        .iter()
        .zip([("复原", (40, 40)), ("开始", (80, 40)), ("加速", (40, 61))])
        .map(|(&target, (label, anchor))| {
            doc.add(Kind::HideShowButton {
                targets: vec![target],
                action: VisibilityAction::Hide,
                options: VisibilityOptions::default(),
                caption_flags: Some(7),
                label: label.into(),
                anchor,
            })
        })
        .collect();
    doc.add(Kind::PlainText {
        text: "Background caption".into(),
        rect: sketchpad_engine::text_object::TextRect {
            left: 0,
            top: 0,
            right: 400,
            bottom: 300,
        },
    });
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    editor.button_view_center = viewport.center;
    for (source, button) in sources.into_iter().zip(buttons) {
        let Kind::HideShowButton { label, anchor, .. } = &editor.doc.objects()[button - 1].kind
        else {
            unreachable!()
        };
        let rect = editor.action_button_rect(&viewport, *anchor, label);
        assert_eq!(editor.hit(&viewport, rect.center()), Some(button));
        assert_eq!(
            editor.hit(&viewport, rect.max - egui::vec2(1., 1.)),
            Some(button)
        );
        editor.click(&viewport, rect.center(), false);
        assert!(!editor.doc.objects()[source - 1].visible);
        assert_eq!(editor.status, format!("Activated button {button}"));
    }
}

#[test]
fn presentation_runs_hidden_movement_and_nested_presentation() {
    let mut doc = Document::new();
    let source = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let destination = doc.add(Kind::Free(Point { x: 120., y: 0. }));
    let movement = doc
        .add_movement_button(source, destination, MovementSpeed::Medium, true)
        .unwrap();
    let nested = doc
        .add_presentation_button_for_targets(&[movement], PresentationMode::Sequential)
        .unwrap();
    let parent = doc
        .add_presentation_button_for_targets(&[nested], PresentationMode::Sequential)
        .unwrap();
    doc.edit_object((movement - 1) + 1, |object| object.visible = false)
        .unwrap();
    doc.edit_object((nested - 1) + 1, |object| object.visible = false)
        .unwrap();
    let mut editor = Editor::new(doc);
    editor.scale = 1.;
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: editor.center,
    };
    editor.viewport_rect = Some(viewport.rect);
    assert!(
        editor
            .activate_presentation_button(nested, &viewport, 0)
            .is_err()
    );
    editor
        .activate_presentation_button(parent, &viewport, 0)
        .unwrap();
    assert_eq!(editor.movements.len(), 1);
    for _ in 0..12 {
        editor.tick_movement(0.1);
        editor.tick_presentations(0.1);
    }
    assert_eq!(
        editor.doc.position(source),
        editor.doc.position(destination)
    );
    assert!(editor.movements.is_empty());
    assert!(editor.presentations.is_empty());
    assert!(!editor.doc.objects()[movement - 1].visible);
    assert!(!editor.doc.objects()[nested - 1].visible);
    editor
        .doc
        .edit_object((movement - 1) + 1, |object| object.deleted = true)
        .unwrap();
    assert!(
        editor
            .activate_presentation_button(parent, &viewport, 0)
            .is_err()
    );
    editor
        .doc
        .edit_object((movement - 1) + 1, |object| object.deleted = false)
        .unwrap();
    editor
        .doc
        .edit_object((movement - 1) + 1, |object| object.page = 1)
        .unwrap();
    assert!(
        editor
            .activate_presentation_button(parent, &viewport, 0)
            .is_err()
    );
}

#[test]
#[ignore = "Set SKETCHPAD_ACTION_SAMPLE to a local GSP file"]
fn external_sample_action_buttons() {
    let bytes = std::fs::read(std::env::var("SKETCHPAD_ACTION_SAMPLE").unwrap()).unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    let mut editor = Editor::new(doc.clone());
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100., 750.)),
        scale: 1.,
        center: editor.center,
    };
    let mut checked = 0;
    for object in doc
        .objects()
        .iter()
        .filter(|object| object.visible && !object.deleted)
    {
        let (label, anchor) = match &object.kind {
            Kind::AnimationButton { label, anchor, .. }
            | Kind::HideShowButton { label, anchor, .. }
            | Kind::MovementButton { label, anchor, .. }
            | Kind::PresentationButton { label, anchor, .. } => (label, anchor),
            _ => continue,
        };
        editor.doc = doc.clone();
        editor.doc.active_page = object.page;
        editor.clear_animation_tracks();
        editor.movements.clear();
        editor.presentations.clear();
        editor.status.clear();
        let rect = editor.action_button_rect(&viewport, *anchor, label);
        let hit = editor.hit(&viewport, rect.center());
        let text_hit = editor.hit(&viewport, rect.min + egui::vec2(16., 12.));
        assert_eq!(hit, Some(object.id), "page {}: {label}", object.page + 1);
        assert_eq!(
            text_hit,
            Some(object.id),
            "page {}: {label}",
            object.page + 1
        );
        let result = editor.activate_referenced_action(object.id, &viewport);
        assert!(
            result.is_ok(),
            "page {}: {label}: {result:?}",
            object.page + 1
        );
        let tracks = editor.animations.len();
        if let Kind::AnimationButton { targets, .. } = &object.kind {
            assert_eq!(tracks, targets.len(), "page {}: {label}", object.page + 1);
        }
        let before_values = editor
            .animations
            .iter()
            .filter_map(|track| {
                editor
                    .doc
                    .numeric_value(track.id)
                    .map(|value| (track.id, value))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for _ in 0..3 {
            editor.tick_animation(0.05);
            editor.tick_movement(0.05);
            editor.tick_presentations(0.05);
        }
        if let Kind::AnimationButton {
            targets, settings, ..
        } = &object.kind
        {
            for (&target, setting) in targets.iter().zip(settings) {
                if !setting.once {
                    assert!(
                        editor.animations.iter().any(|track| track.id == target),
                        "page {}: {label}: target {target} stopped: {}",
                        object.page + 1,
                        editor.status
                    );
                }
                if let Kind::Parameter(group) = &editor.doc.objects()[target - 1].kind {
                    let domain = setting.parameter.unwrap();
                    let scale = group.native_unit_scale();
                    let tolerance = 10_f64.powi(-(group.precision() as i32)) / 2.;
                    let before = before_values[&target];
                    let min = f64::from(domain.min) / scale;
                    let max = f64::from(domain.max) / scale;
                    assert!(
                        group.value() >= min.min(before) - tolerance
                            && group.value() <= max.max(before) + tolerance,
                        "page {}: {label}: parameter {target} = {}",
                        object.page + 1,
                        group.value()
                    );
                    if before < min - tolerance {
                        assert!(
                            group.value() > before,
                            "{label}: parameter {target} did not advance"
                        );
                    } else if before > max + tolerance {
                        assert!(
                            group.value() < before,
                            "{label}: parameter {target} did not advance"
                        );
                    }
                }
            }
        }
        eprintln!(
            "page={} id={} label={label:?} hit={hit:?} text_hit={text_hit:?} result={result:?} tracks={tracks} status={:?}",
            object.page + 1,
            object.id,
            editor.status
        );
        checked += 1;
    }
    eprintln!("Checked {checked} buttons");
    // Check save after a parameter animation without changing the source file.
    if let Some(parameter) = doc
        .objects()
        .iter()
        .find(|object| matches!(object.kind, Kind::Parameter(_)))
    {
        editor.doc = doc.clone();
        editor.doc.active_page = parameter.page;
        let before = editor.doc.numeric_value(parameter.id).unwrap();
        editor
            .doc
            .animate_parameter_value(parameter.id, before + 0.25)
            .unwrap();
        editor.doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.numeric_value(parameter.id),
            editor.doc.numeric_value(parameter.id)
        );
        for (before, after) in doc.objects().iter().zip(reopened.objects().iter()) {
            if matches!(before.kind, Kind::Image { .. }) {
                assert_eq!(before.kind, after.kind, "Image {} changed", before.id);
            }
        }
    }
}
