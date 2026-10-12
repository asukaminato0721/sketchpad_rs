use super::*;
use crate::render::TraceMark;

#[test]
fn chord_scan_button_retains_curve_history() {
    let bytes = include_bytes!("../../../../../tests/fixtures/chord_scan_original.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(doc.objects()[55].tracing);
    let mut editor = Editor::new(doc);
    let expected = editor.doc.sampled_mapped_styled_strokes(56, 384).unwrap();
    assert_eq!(expected.len(), 384);
    assert!(
        expected
            .windows(2)
            .any(|pair| pair[0].style != pair[1].style)
    );
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.)),
        scale: 1.,
        center: editor.center,
    };
    editor.activate_referenced_action(44, &viewport).unwrap();
    let before = editor.doc.position(41).unwrap();
    editor.tick_animation(0.1);
    assert_ne!(editor.doc.position(41).unwrap(), before);
    for _ in 1..24 {
        editor.tick_animation(0.1);
    }
    assert_eq!(editor.trace_marks.len(), 24);
    let TraceMark::Strokes(first) = &editor.trace_marks[0].mark else {
        panic!("Expected a sampled curve trace");
    };
    assert_eq!(
        first, &expected,
        "Past positions and colors must stay fixed"
    );
    let TraceMark::Strokes(last) = &editor.trace_marks[23].mark else {
        panic!("Expected a sampled curve trace");
    };
    assert_ne!(first[0].start, last[0].start);

    editor.stop_animation();
    editor.tick_animation(0.1);
    assert_eq!(editor.trace_marks.len(), 24);
    editor.doc.selected.insert(56);
    assert_eq!(editor.doc.set_selected_tracing(false).unwrap(), 1);
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 24);
    editor.trace_marks.clear();
    assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 1);
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 1);

    editor
        .doc
        .edit_object(56, |object| object.visible = false)
        .unwrap();
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 1);
    editor
        .doc
        .edit_object(56, |object| object.visible = true)
        .unwrap();
    editor.doc.active_page = 1;
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 1);
}

#[test]
fn sampled_curve_without_color_mapping_retains_its_stroke_style() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let path = doc.add(Kind::Segment(a, b));
    let seed = doc.add(Kind::PointOnSegment(path, 0.2));
    let target = doc.add(Kind::Midpoint(seed, a));
    let curve = doc
        .add_sampled_mapped(target, path, seed, &[target], 20)
        .unwrap();
    doc.selected.insert(curve);
    assert_eq!(doc.set_selected_tracing(true).unwrap(), 1);
    let expected_style = doc.effective_visual_style(curve).unwrap();
    let mut editor = Editor::new(doc);
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 1);
    let TraceMark::Strokes(strokes) = &editor.trace_marks[0].mark else {
        panic!("Expected a sampled curve trace");
    };
    assert_eq!(strokes.len(), 19);
    assert_eq!(strokes[0].start, Point { x: 0., y: 0. });
    assert_eq!(strokes[18].end, Point { x: 50., y: 50. });
    assert!(strokes.iter().all(|stroke| stroke.style == expected_style));
}
