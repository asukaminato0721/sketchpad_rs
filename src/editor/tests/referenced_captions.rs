use super::*;

#[test]
fn referenced_caption_dispatches_hidden_action_and_shift_selects_text() {
    let mut editor=Editor::new(Document::new());
    let point=editor.doc.add(Kind::Free(Point{x:100.,y:100.}));
    let button=editor.doc.add_hide_show_button(point).unwrap();
    editor.doc.objects[button-1].visible=false;
    let caption=editor.doc.add_referenced_text(vec![button],"<H<T4xRun ><?1x1<T5xSTALE>>>",(200,150)).unwrap();
    let viewport=Viewport{rect:egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(800.,600.)),scale:1.,center:Point{x:400.,y:300.}};
    let size = f64::from(editor.doc.text_font_state(caption).unwrap().size_code);
    let click=viewport.screen(Point{x:200. + 4.5*size*0.65,y:150. + size*0.3});
    assert_eq!(editor.referenced_action_hit(&viewport,click),Some(button));
    editor.click(&viewport,click,false);
    assert!(!editor.doc.objects[point-1].visible);
    assert!(!editor.doc.objects[button-1].visible);
    editor.doc.verify_roundtrip().unwrap();
    editor.click(&viewport,click,true);
    assert!(editor.doc.selected.contains(&caption));
    assert!(!editor.doc.objects[point-1].visible);
    editor.doc.undo();
    assert!(editor.doc.objects[point-1].visible);
}

#[test]
fn mapped_numeric_labels_are_rendered_and_picked_at_substituted_anchors() {
    let doc=Document::from_bytes(include_bytes!("../../../tests/fixtures/mapped_numeric_label51_spaced_baseline_native_verified.gsp"),None).unwrap();
    let binding=doc.objects.iter().find(|object|doc.mapped_numeric_label_images(object.id).is_some()).unwrap().id;
    let label=doc.mapped_numeric_label_images(binding).unwrap()[1].clone();
    let mut editor=Editor::new(doc);
    let viewport=Viewport{rect:egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1200.,800.)),scale:1.,center:Point{x:600.,y:400.}};
    let at=viewport.screen(label.0);
    assert_eq!(editor.hit(&viewport,at),Some(binding));
    editor.click(&viewport,at,false);
    assert!(editor.doc.selected.contains(&binding));
    assert!(sketchpad_rs::render::svg_page(&editor.doc,&viewport).unwrap().contains(&label.1));
}
