use super::*;

#[test]
fn four_source_polygon_button_moves_all_vertices() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_quadrilateral_port.gsp"
            ),
            None,
        )
        .unwrap(),
    );
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[5].kind else {
        panic!("Expected Animation button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    let before = [1, 2, 3, 4].map(|id| editor.doc.position(id).unwrap());
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 4);
    editor.tick_animation(0.1);
    for (id, initial) in [1, 2, 3, 4].into_iter().zip(before) {
        assert_ne!(editor.doc.position(id).unwrap(), initial);
    }
    let clicked = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_indirect_quadrilateral_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(!clicked.partial_view);
    for (id, initial) in [1, 2, 3, 4].into_iter().zip(before) {
        assert_ne!(clicked.position(id).unwrap(), initial);
    }
}

#[test]
fn six_source_polygon_button_moves_all_vertices() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!(
                "../../../../../tests/fixtures/action_animation_indirect_hexagon_port.gsp"
            ),
            None,
        )
        .unwrap(),
    );
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[7].kind else {
        panic!("Expected Animation button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    let before = [1, 2, 3, 4, 5, 6].map(|id| editor.doc.position(id).unwrap());
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 6);
    editor.tick_animation(0.1);
    for (id, initial) in [1, 2, 3, 4, 5, 6].into_iter().zip(before) {
        assert_ne!(editor.doc.position(id).unwrap(), initial);
    }
    let clicked = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/action_animation_indirect_hexagon_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(!clicked.partial_view);
    for (id, initial) in [1, 2, 3, 4, 5, 6].into_iter().zip(before) {
        assert_ne!(clicked.position(id).unwrap(), initial);
    }
}

#[test]
fn page_two_segment_animation_button_creates_and_runs() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_native.gsp"),
            None,
        )
        .unwrap(),
    );
    editor.doc.active_page = 1;
    editor.select_only(4);
    let draft = editor.animation_button_draft().unwrap();
    assert_eq!(draft.targets, vec![3, 2]);
    let settings = draft
        .targets
        .iter()
        .copied()
        .zip(draft.settings.iter().copied())
        .collect::<Vec<_>>();
    assert!(
        editor
            .doc
            .can_add_animation_button_for_targets_with_settings_and_label(
                &settings,
                draft.label.as_deref(),
            )
            .is_ok()
    );
    editor
        .doc
        .add_animation_button_for_targets_with_settings_and_label(&settings, draft.label.as_deref())
        .unwrap();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_animation_segment_port.gsp")
    );
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[5].kind else {
        panic!("Expected Animation button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    let before = [2, 3].map(|id| editor.doc.position(id).unwrap());
    let other_page = editor.doc.position(1).unwrap();
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 2);
    editor.tick_animation(0.1);
    for (id, initial) in [2, 3].into_iter().zip(before) {
        assert_ne!(editor.doc.position(id).unwrap(), initial);
    }
    assert_eq!(editor.doc.position(1).unwrap(), other_page);
    let saved = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(saved.position(1).unwrap(), other_page);
    for id in [2, 3] {
        assert_eq!(saved.position(id), editor.doc.position(id));
    }
    let clicked = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/two_page_animation_segment_port_clicked_by_wine.gsp"
        ),
        None,
    )
    .unwrap();
    for (id, initial) in [2, 3].into_iter().zip(before) {
        assert_ne!(clicked.position(id).unwrap(), initial);
    }
    assert_eq!(clicked.position(1).unwrap(), other_page);
}

#[test]
fn ray_and_line_points_animate_within_visible_path_and_save() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = editor.doc.add(Kind::Free(Point { x: 200., y: 100. }));
    let ray = editor.doc.add(Kind::Ray(a, b));
    let line = editor.doc.add(Kind::Line(a, b));
    let on_ray = editor.doc.add(Kind::PointOnRay(ray, 0.25));
    let on_line = editor.doc.add(Kind::PointOnLine(line, -0.25));
    editor.doc.add(Kind::Segment(on_ray, on_line));
    let dependent = editor.doc.add(Kind::Midpoint(on_ray, on_line));
    editor.animation_bounds = Some((Point { x: 0., y: 0. }, Point { x: 300., y: 200. }));
    editor.doc.selected.extend([on_ray, on_line]);
    editor.animate_selected();
    assert_eq!(editor.animations.len(), 2);
    assert!(
        editor
            .animations
            .iter()
            .all(|track| track.straight_range.is_some())
    );
    editor.tick_animation(0.1);
    let ray_x = editor.doc.position(on_ray).unwrap().x;
    let line_x = editor.doc.position(on_line).unwrap().x;
    assert!((ray_x - 134.5).abs() < 1e-9);
    assert!((line_x - 89.).abs() < 1e-9);
    assert_eq!(
        editor.doc.position(dependent).unwrap().x,
        (ray_x + line_x) / 2.
    );
    for _ in 0..100 {
        editor.tick_animation(0.1);
        let ray_x = editor.doc.position(on_ray).unwrap().x;
        let line_x = editor.doc.position(on_line).unwrap().x;
        assert!((100. ..=290.).contains(&ray_x));
        assert!((10. ..=290.).contains(&line_x));
    }
    editor.stop_animation();
    let saved = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(on_ray), editor.doc.position(on_ray));
    assert_eq!(reopened.position(on_line), editor.doc.position(on_line));
}

#[test]
fn arc_animation_bounces_and_pause_holds_position() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let b = editor.doc.add(Kind::Free(Point { x: 70., y: 70. }));
    let c = editor.doc.add(Kind::Free(Point { x: 0., y: 100. }));
    let arc = editor.doc.add(Kind::ThreePointArc(a, b, c));
    let moving = editor.doc.add(Kind::PointOnArc(arc, 0.9));
    editor.select_only(moving);
    editor.animate_selected();
    editor.animation_speed = 2.;
    editor.tick_animation(0.1);
    assert!((editor.animations[0].phase - 1.).abs() < 1e-9);
    editor.tick_animation(0.1);
    assert!((editor.animations[0].phase - 0.9).abs() < 1e-9);
    assert_eq!(editor.animations[0].direction, -1.);
    let at = editor.doc.position(moving);
    editor.animation_paused = true;
    editor.tick_animation(0.1);
    assert_eq!(editor.doc.position(moving), at);
    editor.stop_animation();
    assert!(editor.animations.is_empty());
}

#[test]
fn animation_stopped_file_from_desktop_roundtrips() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/animation_segment_locus_saved_by_port.gsp");
    let document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    let Kind::PointOnSegment(3, t) = document.objects[3].kind else {
        panic!("Animated point lost its segment constraint");
    };
    assert!((0.6..0.9).contains(&t));
    assert!((document.position(4).unwrap().x - (120. + 180. * t)).abs() < 1e-9);
}

#[test]
fn free_point_animation_wanders_within_canvas_and_saves() {
    let mut editor = Editor::new(Document::new());
    let moving = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
    editor.animation_bounds = Some((Point { x: 0., y: 0. }, Point { x: 200., y: 200. }));
    editor.select_only(moving);
    editor.animate_selected();
    assert!(editor.animations[0].free_velocity.is_some());
    for _ in 0..150 {
        editor.tick_animation(0.1);
        let at = editor.doc.position(moving).unwrap();
        assert!((10.0..=190.).contains(&at.x));
        assert!((10.0..=190.).contains(&at.y));
    }
    let stopped_at = editor.doc.position(moving).unwrap();
    assert_ne!(stopped_at, Point { x: 100., y: 100. });
    editor.animation_paused = true;
    editor.tick_animation(0.1);
    assert_eq!(editor.doc.position(moving), Some(stopped_at));
    editor.stop_animation();
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(moving), Some(stopped_at));
    editor.doc.undo();
    assert_eq!(
        editor.doc.position(moving),
        Some(Point { x: 100., y: 100. })
    );
}

#[test]
fn free_point_animation_desktop_file_roundtrips() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/animation_free_point_saved_by_port.gsp");
    let document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    let at = document.position(1).unwrap();
    assert_ne!(at, Point { x: 200., y: 150. });
    assert!((at.x - 169.69428734643552).abs() < 1e-9);
    assert!((at.y - 146.60481682616518).abs() < 1e-9);
}
