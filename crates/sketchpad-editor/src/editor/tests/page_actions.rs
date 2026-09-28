use super::*;

#[test]
fn presentation_buttons_activate_parents_and_wait_for_motion() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/presentation_two_hide_show_base.gsp"),
        None,
    )
    .unwrap();
    let presentation = doc
        .add_presentation_button_for_targets(&[3, 4], PresentationMode::Simultaneous)
        .unwrap();
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: editor.center,
    };
    let Kind::PresentationButton { label, anchor, .. } = &editor.doc.objects[presentation - 1].kind
    else {
        panic!()
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    editor.click(&viewport, click, false);
    assert!(!editor.doc.objects[0].visible);
    assert!(!editor.doc.objects[1].visible);
    assert!(editor.presentations.is_empty());

    let mut doc = Document::new();
    let source = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let destination = doc.add(Kind::Free(Point { x: 120., y: 0. }));
    let movement = doc
        .add_movement_button(source, destination, MovementSpeed::Medium, true)
        .unwrap();
    let hide = doc.add_hide_show_button(destination).unwrap();
    let presentation = doc
        .add_presentation_button_for_targets(&[movement, hide], PresentationMode::Sequential)
        .unwrap();
    let mut editor = Editor::new(doc);
    editor.scale = 1.;
    editor.viewport_rect = Some(viewport.rect);
    editor
        .activate_presentation_button(presentation, &viewport, 0)
        .unwrap();
    assert_eq!(editor.presentations.len(), 1);
    assert!(editor.doc.objects[destination - 1].visible);
    for _ in 0..9 {
        editor.tick_movement(0.1);
        editor.tick_presentations();
    }
    assert_eq!(editor.doc.position(source), Some(Point { x: 120., y: 0. }));
    assert!(!editor.doc.objects[destination - 1].visible);
    assert!(editor.presentations.is_empty());
}

#[test]
fn sketchdoc_link_button_opens_sibling_document() {
    let path = std::path::Path::new("../../tests/fixtures/action_link_url_port_sketchdoc.gsp");
    let mut editor = Editor::new(Document::open(path).unwrap());
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: editor.scale,
        center: editor.center,
    };
    let Kind::LinkUrlButton { label, anchor, .. } = &editor.doc.objects[1].kind else {
        panic!();
    };
    let click = editor
        .action_button_rect(&viewport, *anchor, label)
        .center();
    editor.click(&viewport, click, false);
    assert_eq!(editor.doc.objects.len(), 1);
    assert_eq!(
        editor.doc.path.as_ref().unwrap().file_name().unwrap(),
        "point.gsp"
    );
}

#[test]
fn page_link_buttons_switch_pages() {
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
        (
            include_bytes!("../../../../../tests/fixtures/action_link_page_port_generated.gsp")
                .as_slice(),
            1,
            0,
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        editor.doc.active_page = source_page;
        editor.fit();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: editor.scale,
            center: editor.center,
        };
        let Kind::LinkPageButton { label, anchor, .. } = &editor.doc.objects[1].kind else {
            panic!();
        };
        let click = editor
            .action_button_rect(&viewport, *anchor, label)
            .center();
        editor.click(&viewport, click, false);
        assert_eq!(editor.doc.active_page, destination_page);
        assert_eq!(
            Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .active_page,
            destination_page
        );
    }
}
