use super::*;

fn selected_geometry_iteration(shape: usize) -> (Editor, usize, Viewport) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 80., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 120., y: 100. }));
    let u = doc.add(Kind::Free(Point { x: 20., y: 200. }));
    let v = doc.add(Kind::Free(Point { x: 40., y: 200. }));
    let host = doc.add(Kind::Segment(a, b));
    let seed = doc.add(Kind::SegmentMidpoint(host));
    let image = doc.add(Kind::VectorTranslatedPoint(seed, u, v));
    let top = doc.add(Kind::PolarTranslatedPoint(seed, 90., 1., 20.));
    let source = doc.add(match shape {
        0 => Kind::Circle(seed, image),
        1 => Kind::Polygon(vec![seed, image, top]),
        2 => Kind::Ray(seed, image),
        3 => Kind::Line(seed, top),
        _ => Kind::CenterArc(seed, image, top),
    });
    let mut editor = Editor::new(doc);
    editor.selection_order = vec![seed, image, source];
    editor.doc.selected.extend([seed, image, source]);
    editor.iterate_selected(3);
    assert!(
        editor.status.starts_with("Created iteration "),
        "{}",
        editor.status
    );
    let parent = *editor.doc.selected.iter().next().unwrap();
    assert!(matches!(
        editor.doc.objects[parent - 1].kind,
        Kind::MappedPointIteration { .. }
    ));
    let binding = parent + 1;
    for mut object in editor.doc.objects.iter_mut() {
        object.visible = object.id == binding;
    }
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 160., y: 100. },
    };
    (editor, binding, viewport)
}

#[test]
fn selected_geometry_iterations_render_pick_and_keep_independent_styles() {
    for shape in 0..5 {
        let (mut editor, binding, vp) = selected_geometry_iteration(shape);
        let point = match shape {
            0 => Point { x: 160., y: 120. },
            1 => Point { x: 164., y: 96. },
            2 => Point { x: 280., y: 100. },
            3 => Point { x: 160., y: 220. },
            _ => editor.doc.arc_iteration_images(binding, 96).unwrap()[2][48],
        };
        assert_eq!(
            editor.hit(&vp, vp.screen(point)),
            Some(binding),
            "shape{shape}"
        );
        editor.select_only(binding);
        let color = sketchpad_engine::style_records::RgbColor {
            red: 190,
            green: 20,
            blue: 90,
        };
        assert_eq!(editor.doc.set_selected_color(color).unwrap(), 1);
        if shape != 1 {
            assert_eq!(
                editor
                    .doc
                    .set_selected_line_pattern(sketchpad_engine::style_records::LinePattern::Dashed)
                    .unwrap(),
                1
            );
        }
        let svg = crate::render::svg_page(&editor.doc, &vp).unwrap();
        assert!(svg.contains("#be145a"), "shape{shape}: {svg}");
        match shape {
            0 => assert_eq!(svg.matches("<circle").count(), 3),
            1 => assert_eq!(svg.matches("<polygon").count(), 3),
            2 | 3 => assert_eq!(svg.matches("<line").count(), 3),
            _ => assert_eq!(svg.matches("<polyline").count(), 3),
        }
        let saved = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
        assert_eq!(
            saved.objects[binding - 1].visual_style,
            editor.doc.objects[binding - 1].visual_style
        );
        editor.doc.set_selected_visibility(false);
        assert_ne!(editor.hit(&vp, vp.screen(point)), Some(binding));
    }
}
