use super::*;
use crate::editor::colorized::ColorModel;

#[test]
fn color_command_batches_geometry_into_one_undo_and_keeps_live_values() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 120., y: 160. }));
    let b = editor.doc.add(Kind::Free(Point { x: 220., y: 160. }));
    let line = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    let value = editor.doc.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    let before = editor.doc.encode().unwrap();
    editor.selection_order = vec![line, value, circle];
    editor.colorize_selected(ColorModel::Grayscale);
    let outputs = editor.selection_order.clone();
    assert_eq!(outputs.len(), 2);
    assert!(!editor.doc.is_displayed(line));
    assert!(!editor.doc.is_displayed(circle));
    assert!(outputs.iter().all(|id| editor.doc.is_displayed(*id)));
    editor.doc.verify_roundtrip().unwrap();
    let authored = editor.doc.encode().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), before);
    editor.doc.redo();
    assert_eq!(editor.doc.encode().unwrap(), authored);
    editor.doc.set_parameter_value(value, 0.75).unwrap();
    assert_eq!(editor.doc.colorized_color(outputs[0]).unwrap().red, 191);
    assert_eq!(
        editor.doc.circle_points(outputs[1]),
        editor.doc.circle_points(circle)
    );
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.colorized_color(outputs[1]).unwrap().blue, 191);
}

#[test]
fn color_command_preserves_channel_selection_order_and_rejects_invalid_batch() {
    let mut editor = Editor::new(Document::new());
    let point = editor.doc.add(Kind::Free(Point { x: 120., y: 160. }));
    let r = editor.doc.add_named_parameter("r", 0.75, (10, 10)).unwrap();
    let g = editor.doc.add_named_parameter("g", 0.5, (10, 30)).unwrap();
    let b = editor.doc.add_named_parameter("b", 0.25, (10, 50)).unwrap();
    editor.selection_order = vec![r, point, g, b];
    editor.colorize_selected(ColorModel::Rgb);
    let color = editor
        .doc
        .colorized_color(editor.selection_order[0])
        .unwrap();
    assert_eq!((color.red, color.green, color.blue), (191, 127, 63));
    let before = editor.doc.encode().unwrap();
    editor.selection_order = vec![point, r, g];
    editor.colorize_selected(ColorModel::Rgb);
    assert_eq!(editor.doc.encode().unwrap(), before);
    let caption = editor.doc.add(Kind::Unsupported(999));
    let count = editor.doc.objects().len();
    editor.selection_order = vec![point, caption, r];
    editor.colorize_selected(ColorModel::Grayscale);
    assert_eq!(editor.doc.objects().len(), count);
}
