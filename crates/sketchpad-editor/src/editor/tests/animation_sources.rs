use super::*;

#[test]
fn animation_moves_constrained_points_and_their_dependents() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = editor.doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let path = editor.doc.add(Kind::Segment(a, b));
    let moving = editor.doc.add(Kind::PointOnSegment(path, 0.25));
    editor.doc.add(Kind::Segment(a, moving));
    let midpoint = editor.doc.add(Kind::Midpoint(a, moving));
    let center = editor.doc.add(Kind::Free(Point { x: 200., y: 200. }));
    let edge = editor.doc.add(Kind::Free(Point { x: 220., y: 200. }));
    let circle = editor.doc.add(Kind::Circle(center, edge));
    let orbit = editor.doc.add(Kind::PointOnCircle(circle, 1., 0.));
    let plot = editor
        .doc
        .plot_new_function(SimpleFunction::Identity, Point { x: 300., y: 300. })
        .unwrap();
    let curve_point = editor.doc.add(Kind::PointOnCurve(plot, 0.25));
    let sector_center = editor.doc.add(Kind::Free(Point { x: 400., y: 200. }));
    let sector_start = editor.doc.add(Kind::Free(Point { x: 420., y: 200. }));
    let sector_end = editor.doc.add(Kind::Free(Point { x: 400., y: 180. }));
    let arc = editor
        .doc
        .add(Kind::CenterArc(sector_center, sector_start, sector_end));
    let sector = editor.doc.add_sector_boundary(arc).unwrap();
    let sector_point = editor.doc.add_point_on_sector(sector, 0.25).unwrap();
    let curve_before = editor.doc.position(curve_point).unwrap();
    let sector_before = editor.doc.position(sector_point).unwrap();
    assert_eq!(
        editor.doc.animation_button_default_settings(curve_point),
        Some(AnimationSettings::default())
    );
    assert_eq!(
        editor
            .doc
            .animation_button_default_settings(sector_point)
            .unwrap()
            .direction,
        AnimationDirection::Forward
    );
    editor
        .doc
        .selected
        .extend([moving, orbit, curve_point, sector_point]);
    editor.animate_selected();
    assert_eq!(editor.animations.len(), 4);
    for _ in 0..10 {
        editor.tick_animation(0.1);
    }
    assert!((editor.doc.position(moving).unwrap().x - 75.).abs() < 1e-9);
    assert!((editor.doc.position(midpoint).unwrap().x - 37.5).abs() < 1e-9);
    assert!((editor.doc.position(orbit).unwrap().x - 180.).abs() < 1e-9);
    assert!((editor.doc.position(orbit).unwrap().y - 200.).abs() < 1e-9);
    assert_ne!(editor.doc.position(curve_point), Some(curve_before));
    assert_ne!(editor.doc.position(sector_point), Some(sector_before));
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    editor.stop_animation();
    editor.doc.undo();
    assert_eq!(editor.doc.position(moving), Some(Point { x: 25., y: 0. }));
    assert_eq!(editor.doc.position(orbit), Some(Point { x: 220., y: 200. }));
}

#[test]
fn selected_segment_and_circle_animate_their_source_points() {
    for (base, native_bytes) in [
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_segment_base.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/animation_indirect_segment_native.gsp")
                .as_slice(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_circle_base.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/animation_indirect_circle_native.gsp")
                .as_slice(),
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/animation_indirect_segment_hidden_base.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/animation_indirect_segment_hidden_native.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
        let before = [1, 2].map(|id| editor.doc.position(id).unwrap());
        let hidden = !editor.doc.objects[0].visible;
        editor.select_only(3);
        editor.animate_selected();
        assert_eq!(editor.animations.len(), 2);
        assert_eq!(
            editor
                .animations
                .iter()
                .map(|track| track.id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        if hidden {
            assert!(editor.animations.iter().all(|track| track.allow_hidden));
        }
        editor.tick_animation(0.1);
        for (id, initial) in [1, 2].into_iter().zip(before) {
            assert_ne!(editor.doc.position(id).unwrap(), initial);
            assert_eq!(editor.doc.objects[id - 1].visible, !hidden);
        }
        editor.stop_animation();
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_ne!(native.position(1).unwrap(), before[0]);
        assert_ne!(native.position(2).unwrap(), before[1]);
    }
}

#[test]
fn selected_measurement_and_table_have_no_animation_sources() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/animation_indirect_length_base.gsp"),
            None,
        )
        .unwrap(),
    );
    editor.select_only(4);
    assert!(editor.animation_sources(&[4]).is_empty());
    assert!(editor.animation_button_draft().is_none());
    editor.animate_selected();
    assert!(editor.animations.is_empty());
    assert_eq!(editor.doc.position(1), Some(Point { x: 100., y: 100. }));
    assert_eq!(editor.doc.position(2), Some(Point { x: 200., y: 100. }));
    let mut table_editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/animation_indirect_table_base.gsp"),
            None,
        )
        .unwrap(),
    );
    table_editor.select_only(2);
    assert!(table_editor.animation_sources(&[2]).is_empty());
    assert!(table_editor.animation_button_draft().is_none());
    table_editor.animate_selected();
    assert!(table_editor.animations.is_empty());
}

#[test]
fn selected_segment_and_circle_create_buttons_for_reversed_source_order() {
    for (base, port, label) in [
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_segment_base.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_segment_port.gsp"
            )
            .as_slice(),
            "Animate Segment",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_circle_base.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_circle_port.gsp"
            )
            .as_slice(),
            "Animate Circle",
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/animation_indirect_segment_hidden_base.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_segment_hidden_port.gsp"
            )
            .as_slice(),
            "Animate Segment",
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
        editor.select_only(3);
        let draft = editor.animation_button_draft().unwrap();
        assert_eq!(draft.targets, vec![2, 1]);
        assert_eq!(draft.label.as_deref(), Some(label));
        let targets = draft
            .targets
            .iter()
            .copied()
            .zip(draft.settings.iter().copied())
            .collect::<Vec<_>>();
        editor
            .doc
            .add_animation_button_for_targets_with_settings_and_label(
                &targets,
                draft.label.as_deref(),
            )
            .unwrap();
        assert_eq!(editor.doc.encode().unwrap(), port);
    }
}

#[test]
fn indirect_segment_animation_button_moves_both_endpoints() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/action_animation_indirect_segment_port.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_indirect_segment_hidden_port.gsp"
        )
        .as_slice(),
    ] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[3].kind else {
            panic!("Expected Animation button")
        };
        let click = editor
            .action_button_rect(&viewport, *anchor, label)
            .center();
        let before = [1, 2].map(|id| editor.doc.position(id).unwrap());
        let visibility = [1, 2].map(|id| editor.doc.objects[id - 1].visible);
        editor.click(&viewport, click, false);
        assert_eq!(editor.animations.len(), 2);
        editor.tick_animation(0.1);
        for (id, initial) in [1, 2].into_iter().zip(before) {
            assert_ne!(editor.doc.position(id).unwrap(), initial);
            assert_eq!(editor.doc.objects[id - 1].visible, visibility[id - 1]);
        }
    }
}

#[test]
fn selected_lines_rays_and_polygons_create_native_button_targets() {
    for (base, port, source, targets, label) in [
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_line_base.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_line_port.gsp")
                .as_slice(),
            3,
            &[2, 1][..],
            "Animate Line",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_ray_base.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/action_animation_indirect_ray_port.gsp")
                .as_slice(),
            3,
            &[2, 1][..],
            "Animate Ray",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_polygon_base.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_polygon_port.gsp"
            )
            .as_slice(),
            4,
            &[3, 2, 1][..],
            "Animate Triangle",
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/animation_indirect_quadrilateral_base.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_quadrilateral_port.gsp"
            )
            .as_slice(),
            5,
            &[4, 3, 2, 1][..],
            "Animate Quadrilateral",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_pentagon_base.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_pentagon_port.gsp"
            )
            .as_slice(),
            6,
            &[5, 4, 3, 2, 1][..],
            "Animate Pentagon",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_hexagon_base.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_hexagon_port.gsp"
            )
            .as_slice(),
            7,
            &[6, 5, 4, 3, 2, 1][..],
            "Animate Hexagon",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/animation_indirect_heptagon_base.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_heptagon_port.gsp"
            )
            .as_slice(),
            8,
            &[7, 6, 5, 4, 3, 2, 1][..],
            "Animate Polygon",
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
        editor.select_only(source);
        let draft = editor.animation_button_draft().unwrap();
        assert_eq!(draft.targets, targets);
        assert_eq!(draft.label.as_deref(), Some(label));
        let settings = draft
            .targets
            .iter()
            .copied()
            .zip(draft.settings.iter().copied())
            .collect::<Vec<_>>();
        editor
            .doc
            .add_animation_button_for_targets_with_settings_and_label(
                &settings,
                draft.label.as_deref(),
            )
            .unwrap();
        assert_eq!(editor.doc.encode().unwrap(), port);
    }
}
