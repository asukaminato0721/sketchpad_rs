use super::*;

fn isolated_binding(state: &str, binding: usize) -> Editor {
    let bytes = std::fs::read(format!(
        "../../tests/fixtures/scoped_locus67_{state}_native_saved.gsp"
    ))
    .unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view, "{state}");
    // Independent Native witnesses remain in the fixture. Isolate the output
    // for editor picking so no witness or original source can pass a hit.
    for object in &mut doc.objects {
        object.visible = object.id == binding;
    }
    Editor::new(doc)
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(3000., 2400.)),
        scale: 3.,
        center: Point { x: 500., y: 400. },
    }
}

fn member(q: f64, external: f64, t: f64, gap: bool, translated: bool) -> (Point, Point) {
    let f = (q + external)
        * if gap {
            0.03 / (t - 0.5).powi(2) + 0.5
        } else {
            1.
        };
    let (dx, dy) = if translated { (30., -20.) } else { (0., 0.) };
    (
        Point {
            x: 100. + (100. + 40. * t) * f + dx,
            y: 100. + 80. * f + dy,
        },
        Point {
            x: 100. + 130. * f + dx,
            y: 100. + 120. * f + dy,
        },
    )
}

fn stroke_rows(svg: &str) -> Vec<&str> {
    svg.lines()
        .filter(|line| line.starts_with("<line ") || line.starts_with("<polyline "))
        .collect()
}

#[test]
fn native_scoped_locus_members_pick_sampled_extents_and_svg_preserves_style_roles() {
    let view = viewport();
    for (state, binding, entry, external, depth, gap, direct_color, translated) in [
        ("live", 18, 0., 1., 3, false, false, false),
        ("frozen", 18, 0., 1., 3, false, false, false),
        ("edited", 18, 0.5, 1.25, 3, false, false, false),
        ("undefined_gap", 18, 0., 1., 3, true, false, false),
        (
            "extra_frozen_entry_edited",
            18,
            0.5,
            1.25,
            3,
            false,
            false,
            false,
        ),
        ("extra_frozen_depth1", 18, 0., 1., 1, false, false, false),
        ("extra_colorized", 22, 0., 1., 3, false, true, false),
        (
            "extra_transform_colorized",
            25,
            0.,
            1.,
            3,
            false,
            false,
            true,
        ),
    ] {
        let editor = isolated_binding(state, binding);
        for outer in 1..=depth {
            for t in [0., 0.25, 0.5, 0.75, 1.] {
                if gap && t == 0.5 {
                    continue;
                }
                let (a, b) = member(entry + f64::from(outer), external, t, gap, translated);
                let at = Point {
                    x: a.x * 0.65 + b.x * 0.35,
                    y: a.y * 0.65 + b.y * 0.35,
                };
                assert_eq!(
                    editor.hit(&view, view.screen(at)),
                    Some(binding),
                    "{state}: outer{outer}, local{t}"
                );
            }
        }
        assert_eq!(
            editor.hit(&view, view.screen(Point { x: 50., y: 600. })),
            None,
            "{state}: exterior"
        );
        if gap {
            // This sits between the finite .25/.75 members; its minimum
            // distance from all12 finite strokes is7.295 world pixels.
            assert_eq!(
                editor.hit(
                    &view,
                    view.screen(Point {
                        x: 343.04,
                        y: 288.16
                    })
                ),
                None,
                "undefined local .5 must not be bridged"
            );
        } else if entry == 0. && external == 1. {
            let (a, b) = member(1., 1., 0.125, false, translated);
            assert_eq!(
                editor.hit(
                    &view,
                    view.screen(Point {
                        x: a.x * 0.65 + b.x * 0.35,
                        y: a.y * 0.65 + b.y * 0.35
                    })
                ),
                None,
                "{state}: discrete fan gap"
            );
        }

        let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
        let rows = stroke_rows(&svg);
        assert_eq!(
            rows.len(),
            depth as usize * if gap { 4 } else { 5 },
            "{state}"
        );
        assert!(!svg.contains("NaN"));
        if direct_color {
            // Native's measured colors belong to the direct Colorized target;
            // the t1 green channel is204, not the initial manifest's203.
            for red in [102_u8, 153, 204] {
                for green in [25_u8, 70, 114, 159, 204] {
                    let variants = if green == 204 {
                        vec![203, 204]
                    } else {
                        vec![green]
                    };
                    let count = variants
                        .iter()
                        .map(|&g| {
                            let color = format!("stroke=\"#{red:02x}{g:02x}33\"");
                            rows.iter().filter(|row| row.contains(&color)).count()
                        })
                        .sum::<usize>();
                    // Native observed204 for t=1; the current float codec can
                    // floor that channel to203. This is the sole tolerance.
                    assert_eq!(count, 1, "{state}: red{red} green{green}");
                }
            }
        } else {
            assert!(
                rows.iter().all(|row| row.contains("stroke=\"#000000\"")),
                "{state}: binding BLACK must replace source GREEN/deep Colorized style"
            );
        }
    }
}

#[test]
fn scoped_locus_binding_visibility_is_independent_of_parent_and_source_deletion_undo() {
    let view = viewport();
    for (state, source, root, binding) in [("live", 15, 17, 18), ("extra_colorized", 19, 21, 22)] {
        let mut editor = isolated_binding(state, binding);
        let (a, b) = member(1., 1., 0.25, false, false);
        let at = view.screen(Point {
            x: a.x * 0.65 + b.x * 0.35,
            y: a.y * 0.65 + b.y * 0.35,
        });
        assert!(!editor.doc.objects[source - 1].visible && !editor.doc.objects[root - 1].visible);
        assert_eq!(editor.hit(&view, at), Some(binding));
        editor.select_only(root);
        editor.doc.set_selected_visibility(true);
        editor.doc.set_selected_visibility(false);
        assert_eq!(
            editor.hit(&view, at),
            Some(binding),
            "{state}: parent hiding is not output hiding"
        );
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        editor.doc.undo();

        editor.select_only(binding);
        editor.doc.set_selected_visibility(false);
        assert_eq!(editor.hit(&view, at), None);
        assert!(stroke_rows(&crate::render::svg_page(&editor.doc, &view).unwrap()).is_empty());
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert_eq!(editor.hit(&view, at), Some(binding));

        editor.select_only(source);
        editor.clear_selected();
        assert!(
            editor.doc.objects[source - 1].deleted,
            "{state}: {}",
            editor.status
        );
        assert!(editor.doc.objects[binding - 1].deleted);
        assert_eq!(editor.hit(&view, at), None);
        assert!(stroke_rows(&crate::render::svg_page(&editor.doc, &view).unwrap()).is_empty());
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert_eq!(editor.hit(&view, at), Some(binding));
        editor.doc.redo();
        assert_eq!(editor.hit(&view, at), None);
        editor.doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_final_only_scoped_locus_renders_five_last_generation_members() {
    let view = viewport();
    for (state, binding, direct_color) in [("live", 18, false), ("colorized", 22, true)] {
        let bytes = std::fs::read(format!(
            "../../tests/fixtures/scoped_locus67_final_final_{state}_native_saved.gsp"
        )).unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        for object in &mut doc.objects { object.visible = object.id == binding; }
        let editor = Editor::new(doc);
        let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
        let rows = stroke_rows(&svg);
        assert_eq!(rows.len(), 5, "{state}");
        for (t, row) in [0., 0.25, 0.5, 0.75, 1.].into_iter().zip(rows) {
            let (a, b) = member(3., 1., t, false, false);
            let at = Point { x: a.x * 0.65 + b.x * 0.35, y: a.y * 0.65 + b.y * 0.35 };
            assert_eq!(editor.hit(&view, view.screen(at)), Some(binding));
            if direct_color {
                let green = match t { 0. => 25, 0.25 => 70, 0.5 => 114, 0.75 => 159, _ => 204 };
                let color = format!("stroke=\"#cc{:02x}33\"", green);
                let rounded = "stroke=\"#cccb33\"";
                assert!(row.contains(&color) || (t == 1. && row.contains(rounded)), "{row}");
            } else {
                assert!(row.contains("stroke=\"#000000\""));
            }
        }
    }
}

#[test]
fn scalar_selection_authors_scoped_locus_binding_and_preserves_source_through_reopen_undo() {
    let view = viewport();
    let mut editor = isolated_binding("live", 18);
    let source = 15;
    let source_before = editor.doc.objects[source - 1].kind.clone();
    editor.select_only(17);
    editor.clear_selected();
    assert!(editor.doc.objects[16].deleted && editor.doc.objects[17].deleted);
    let before_authoring = editor.doc.encode().unwrap();
    assert_eq!(editor.doc.objects[source - 1].kind, source_before);

    editor.selection_order = vec![7, 8, source, 16];
    editor
        .doc
        .selected
        .extend(editor.selection_order.iter().copied());
    editor.iterate_selected(9);
    let root = *editor
        .doc
        .selected
        .iter()
        .next()
        .expect("new iteration selected");
    assert!(
        matches!(&editor.doc.objects[root - 1].kind,
        Kind::MappedPointIteration { seeds, branches, depth: IterationDepth::Value(16), .. }
            if seeds == &[7] && branches == &[vec![8]]),
        "{}",
        editor.status
    );
    let binding = root + 1;
    assert_eq!(
        editor.doc.objects[binding - 1].kind,
        Kind::IterationBinding(source, root)
    );
    assert_eq!(editor.doc.objects[source - 1].kind, source_before);
    assert_eq!(
        editor
            .doc
            .mapped_collection_strokes(binding, 4)
            .unwrap()
            .len(),
        15
    );
    let (a, b) = member(1., 1., 0.25, false, false);
    let at = view.screen(Point {
        x: a.x * 0.65 + b.x * 0.35,
        y: a.y * 0.65 + b.y * 0.35,
    });
    assert_eq!(editor.hit(&view, at), Some(binding));
    editor.doc.verify_roundtrip().unwrap();
    let saved = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        reopened
            .objects
            .iter()
            .any(|object| matches!(object.kind, Kind::Locus(..)))
    );
    let reopened_binding = reopened
        .objects
        .iter()
        .find_map(|object| {
            if let Kind::IterationBinding(image, _) = object.kind
                && matches!(reopened.objects[image - 1].kind, Kind::Locus(..))
            {
                Some(object.id)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        reopened
            .mapped_collection_strokes(reopened_binding, 4)
            .unwrap()
            .len(),
        15
    );
    assert_eq!(Editor::new(reopened).hit(&view, at), Some(reopened_binding));
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), before_authoring);
    assert_eq!(editor.doc.objects[source - 1].kind, source_before);
}
