use super::*;

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.)),
        scale: 1.,
        center: Point { x: 400., y: 300. },
    }
}

#[test]
fn public_pi_suffix_caption_tracks_live_value_in_canvas_svg_and_picking() {
    let mut document = Document::new();
    let parameter = document.add_named_parameter("n", 1., (10, 30)).unwrap();
    let caption = document
        .add_referenced_text(vec![parameter], "<H<?1x1<H<T1x1>>><!70>>", (240, 180))
        .unwrap();
    let mut editor = Editor::new(document);
    let view = viewport();

    // The canvas consumes these same live rich runs; the cached "1" is not
    // painted as a second value, and the native suffix is a separate run.
    let runs = editor.doc.text_rich_runs(caption).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs[0]
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "1.0π"
    );
    assert_eq!(editor.doc.text_content(caption).unwrap().0, "1.0π");
    assert!(
        crate::render::svg_page(&editor.doc, &view)
            .unwrap()
            .contains("1.0π")
    );

    editor.doc.set_parameter_value(parameter, 2.).unwrap();
    assert_eq!(editor.doc.text_content(caption).unwrap().0, "2.0π");
    assert_eq!(
        editor.doc.text_rich_runs(caption).unwrap()[0]
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "2.0π"
    );
    let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
    assert!(svg.contains("2.0π"));
    assert!(!svg.contains("1.0π"));

    let size = f32::from(editor.doc.text_font_state(caption).unwrap().size_code);
    let suffix_hit = view.screen(Point {
        x: 240. + 3.5 * f64::from(size) * 0.65,
        y: 180. + f64::from(size) * 0.3,
    });
    assert_eq!(editor.hit(&view, suffix_hit), Some(caption));
    editor.click(&view, suffix_hit, false);
    assert!(editor.doc.selected.contains(&caption));

    let saved = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), saved);
    assert_eq!(reopened.text_content(caption).unwrap().0, "2.0π");
    assert!(
        crate::render::svg_page(&reopened, &view)
            .unwrap()
            .contains("2.0π")
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.text_content(caption).unwrap().0, "1.0π");
}

#[test]
fn original_pi_caption_is_readable_while_partial_view_still_blocks_svg() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/向忠作品/函数图象描点法作图演示平台.gsp"
    ))
    .unwrap();
    let document = Document::from_bytes(&bytes, None).unwrap();
    assert!(document.partial_view);
    assert!(matches!(
        document.objects[215].kind,
        Kind::ReferencedText { .. }
    ));
    assert_eq!(
        document.text_content(216).unwrap().0,
        format!("{}π", document.text_content(217).unwrap().0)
    );
    assert!(crate::render::svg_page(&document, &viewport()).is_err());
    assert_eq!(document.encode().unwrap(), bytes);
}

#[test]
fn native_subscript_caption_imports_live_text_and_remains_pickable() {
    for (bytes, expected) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/referenced_token71_dash_original_native_saved.gsp"
            )
            .as_slice(),
            "A₃",
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/referenced_token71_dash_rebind6_native_saved.gsp"
            )
            .as_slice(),
            "A₅.₀₀",
        ),
    ] {
        let editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        assert!(matches!(
            editor.doc.objects[8].kind,
            Kind::ReferencedText { .. }
        ));
        assert_eq!(editor.doc.text_content(9).unwrap().0, expected);
        assert_eq!(
            editor.doc.text_rich_runs(9).unwrap()[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            expected
        );
        let (_, rect) = editor.doc.text_content(9).unwrap();
        let hit = viewport().screen(Point {
            x: f64::from(rect.left) + 3.,
            y: f64::from(rect.top) + 3.,
        });
        assert_eq!(editor.hit(&viewport(), hit), Some(9));
        if editor.doc.partial_view {
            assert!(crate::render::svg_page(&editor.doc, &viewport()).is_err());
        } else {
            assert!(
                crate::render::svg_page(&editor.doc, &viewport())
                    .unwrap()
                    .contains(expected)
            );
        }
        assert_eq!(editor.doc.encode().unwrap(), bytes);
    }
}

#[test]
fn public_subscript_caption_updates_value_and_precision_through_svg_and_reopen() {
    let mut document = Document::new();
    let parameter = document.add_named_parameter("n", 3., (10, 30)).unwrap();
    let calculation = document
        .add_parameter_calculation(parameter, "n+0", (10, 65))
        .unwrap();
    document.set_numeric_precision(&[calculation], 0).unwrap();
    let caption = document
        .add_referenced_text(
            vec![calculation],
            "<VL<-<T1xA><?1x1<H<T1x3>>>>>",
            (240, 180),
        )
        .unwrap();
    let mut editor = Editor::new(document);
    assert_eq!(editor.doc.text_content(caption).unwrap().0, "A₃");
    assert!(
        crate::render::svg_page(&editor.doc, &viewport())
            .unwrap()
            .contains("A₃")
    );

    editor.doc.set_parameter_value(parameter, 5.).unwrap();
    editor.doc.set_numeric_precision(&[calculation], 2).unwrap();
    assert_eq!(editor.doc.text_content(caption).unwrap().0, "A₅.₀₀");
    let canvas_runs = editor.doc.text_rich_runs(caption).unwrap();
    assert_eq!(
        canvas_runs[0]
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "A₅.₀₀"
    );
    let svg = crate::render::svg_page(&editor.doc, &viewport()).unwrap();
    assert!(svg.contains("A₅.₀₀"));
    assert!(!svg.contains("A₃"));
    assert_eq!(
        editor.hit(&viewport(), viewport().screen(Point { x: 244., y: 184. })),
        Some(caption)
    );

    let saved = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_content(caption).unwrap().0, "A₅.₀₀");
    assert_eq!(reopened.encode().unwrap(), saved);
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.text_content(caption).unwrap().0, "A₅");
}
