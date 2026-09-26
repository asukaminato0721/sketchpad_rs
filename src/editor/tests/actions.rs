use super::*;

#[test]
fn page_link_activates_destination_hide_show_button() {
    for (index, bytes) in [
        include_bytes!(
            "../../../tests/fixtures/action_link_page_activate_hide_show_native_created.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
        )
        .as_slice(),
        include_bytes!("../../../tests/fixtures/two_page_new_hide_show_link_port.gsp").as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_link_click_save_{}_{}.gsp",
            std::process::id(),
            index
        ));
        editor.doc.path = Some(path.clone());
        editor.doc.active_page = 1;
        editor.fit();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: editor.scale,
            center: editor.center,
        };
        let Kind::LinkPageButton { label, anchor, .. } = &editor.doc.objects.last().unwrap().kind
        else {
            panic!();
        };
        let click = editor
            .action_button_rect(&viewport, *anchor, label)
            .center();
        editor.click(&viewport, click, false);
        assert_eq!(editor.doc.active_page, 0);
        assert!(!editor.doc.objects[0].visible);
        assert!(matches!(
            editor.doc.objects[1].kind,
            Kind::HideShowButton {
                action: sketchpad_rs::action_records::VisibilityAction::Show,
                ..
            }
        ));
        assert!(editor.save(false), "{}", editor.status);
        let reopened = Document::open(&path).unwrap();
        assert!(!reopened.objects[0].visible);
        assert!(matches!(
            reopened.objects[1].kind,
            Kind::HideShowButton {
                action: sketchpad_rs::action_records::VisibilityAction::Show,
                ..
            }
        ));
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn scroll_buttons_place_target_at_corner_or_center() {
    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let corner = doc
        .add_scroll_button(point, ScrollPlacement::UpperLeft)
        .unwrap();
    let centered = doc
        .add_scroll_button(point, ScrollPlacement::Center)
        .unwrap();
    let mut editor = Editor::new(doc);
    editor.scale = 2.;
    let viewport = |center| Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 2.,
        center,
    };
    let v = viewport(editor.center);
    let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[corner - 1].kind else {
        panic!()
    };
    let click = sketchpad_rs::render::action_button_rect(&v, *anchor, label).center();
    editor.click(&v, click, false);
    assert_eq!(editor.center, Point { x: 250., y: 195. });
    let scrolled = viewport(editor.center);
    let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[corner - 1].kind else {
        panic!()
    };
    assert_eq!(
        editor.action_button_rect(&scrolled, *anchor, label).min,
        egui::pos2(-190., -145.)
    );
    editor.fit();
    editor.scale = 2.;
    let v = viewport(editor.center);
    let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[centered - 1].kind else {
        panic!()
    };
    let click = sketchpad_rs::render::action_button_rect(&v, *anchor, label).center();
    editor.click(&v, click, false);
    assert_eq!(editor.center, Point { x: 150., y: 120. });
}

#[test]
fn page_two_scroll_button_moves_only_the_view() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../tests/fixtures/two_page_scroll_port.gsp"),
            None,
        )
        .unwrap(),
    );
    editor.doc.active_page = 1;
    editor.scale = 1.;
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: editor.center,
    };
    let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[5].kind else {
        panic!("Expected Scroll button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    assert!(editor.doc.can_add_scroll_button(2).is_ok());
    let saved = editor.doc.encode().unwrap();
    editor.click(&viewport, click, false);
    assert_eq!(editor.center, Point { x: 311., y: 261. });
    assert_eq!(editor.doc.encode().unwrap(), saved);
}

#[test]
fn movement_button_reaches_instant_and_initial_destinations() {
    let mut doc = Document::new();
    let source = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let destination = doc.add(Kind::Free(Point { x: 120., y: 0. }));
    let instant = doc
        .add_movement_button(source, destination, MovementSpeed::Instant, true)
        .unwrap();
    let mut editor = Editor::new(doc);
    editor.scale = 1.;
    editor.activate_movement_button(instant);
    assert_eq!(editor.doc.position(source), Some(Point { x: 120., y: 0. }));
    assert!(editor.movements.is_empty());

    editor.doc.move_point(source, Point { x: 0., y: 0. });
    let initial = editor
        .doc
        .add_movement_button(source, destination, MovementSpeed::Medium, false)
        .unwrap();
    editor.activate_movement_button(initial);
    editor.doc.move_point(destination, Point { x: 200., y: 0. });
    for _ in 0..10 {
        editor.tick_movement(0.1);
    }
    assert_eq!(editor.doc.position(source), Some(Point { x: 120., y: 0. }));
    assert!(editor.movements.is_empty());
}

#[test]
fn page_two_movement_button_runs_and_saves() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../tests/fixtures/two_page_movement_port.gsp"),
            None,
        )
        .unwrap(),
    );
    editor.doc.active_page = 1;
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 200., y: 150. },
    };
    let Kind::MovementButton { label, anchor, .. } = &editor.doc.objects[5].kind else {
        panic!("Expected Movement button")
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    let first_page = editor.doc.position(1).unwrap();
    editor.click(&viewport, click, false);
    for _ in 0..100 {
        editor.tick_movement(0.1);
    }
    assert_eq!(editor.doc.position(2), editor.doc.position(3));
    assert_eq!(editor.doc.position(1), Some(first_page));
    let saved = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(saved.position(2), saved.position(3));
    assert_eq!(saved.position(1), Some(first_page));
    let clicked = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_movement_port_clicked_by_wine.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(clicked.position(2), clicked.position(3));
    assert_eq!(clicked.position(1), Some(first_page));
}
