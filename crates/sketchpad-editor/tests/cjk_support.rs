// Compile font checks independently of the historical native-file fixtures.
#[path = "../src/editor/system_fonts.rs"]
mod system_fonts;

use sketchpad_engine::document::{Document, Kind, Point, TextStyleCommand};

#[test]
fn cjk_text_and_labels_survive_edit_copy_and_save() {
    let original = "中文简体／繁體中文\n日本語：ひらがな・カタカナ\n한국어 한글";
    let edited = format!("{original}\nA < B，。！？");
    let mut doc = Document::new();
    let text = doc.add_text(original, Point { x: 100., y: 120. }).unwrap();
    let point = doc.add(Kind::Free(Point { x: 50., y: 60. }));
    doc.set_point_label(point, "点あ한", true).unwrap();
    doc.edit_text(text, &edited).unwrap();
    doc.undo();
    assert_eq!(doc.text_content(text).unwrap().0, original);
    doc.redo();
    assert_eq!(doc.text_content(text).unwrap().0, edited);
    doc.format_text_range(text, 0, 4, TextStyleCommand::Bold)
        .unwrap();
    doc.selected.insert(text);
    let clipboard = doc.copy_selected().unwrap();
    let copy = doc.paste(&clipboard).unwrap()[0];
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    for id in [text, copy] {
        assert_eq!(reopened.text_content(id).unwrap().0, edited);
        assert_eq!(reopened.text_rich_runs(id), doc.text_rich_runs(text));
    }
    assert_eq!(
        reopened.objects[point - 1]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "点あ한"
    );
}

#[test]
#[ignore = "requires fc-match and installed Chinese, Japanese, and Korean fonts"]
fn system_cjk_glyphs() {
    let mut definitions = egui::FontDefinitions::default();
    system_fonts::add_cjk_fallbacks(&mut definitions);
    let ctx = egui::Context::default();
    ctx.set_fonts(definitions);
    let _ = ctx.run_ui(Default::default(), |ui| {
        for font in [
            egui::FontId::proportional(16.),
            egui::FontId::monospace(16.),
        ] {
            for ch in "中文简体繁體日本語ひらがなカタカナ한국어，。「」한글".chars()
            {
                assert!(
                    ui.fonts_mut(|fonts| fonts.has_glyph(&font, ch)),
                    "{font:?}: missing {ch}"
                );
            }
            let galley = ui.fonts_mut(|fonts| {
                fonts.layout_no_wrap("中文 日本語 한국어".into(), font, egui::Color32::WHITE)
            });
            assert!(galley.size().x > 0. && galley.size().y > 0.);
        }
    });
}
