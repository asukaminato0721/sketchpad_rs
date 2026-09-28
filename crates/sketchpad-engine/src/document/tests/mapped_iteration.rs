use super::*;

fn mapped_example(depth: IterationDepth) -> (Document, usize, usize) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 160. }));
    let b = doc.add(Kind::Free(Point { x: 140., y: 180. }));
    // Simultaneous A -> B and B -> B + (B-A) must retain a 40x20 vector.
    let image = doc.add(Kind::VectorTranslatedPoint(b, a, b));
    let segment = doc.add(Kind::Segment(a, b));
    let id = doc
        .add_mapped_point_iteration(&[a, b], &[vec![b, image]], &[segment, image], depth)
        .unwrap();
    (doc, id, id + 1)
}

#[test]
fn mapped_iteration_simultaneous_substitution_movement_and_save() {
    let (mut doc, id, binding) = mapped_example(IterationDepth::Fixed(3));
    let lines = doc.branch_segment_iteration_images(binding).unwrap();
    assert_eq!(lines.len(), 3);
    for (step, (a, b)) in lines.iter().enumerate() {
        assert_eq!(
            *a,
            Point {
                x: 140. + 40. * step as f64,
                y: 180. + 20. * step as f64
            }
        );
        assert_eq!(
            *b,
            Point {
                x: 180. + 40. * step as f64,
                y: 200. + 20. * step as f64
            }
        );
    }
    let bytes = doc.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_FIXTURE").is_some() {
        fs::write("/tmp/mapped_point_iteration_generated.gsp", &bytes).unwrap();
    }
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.branch_segment_iteration_images(binding),
        Some(lines)
    );
    doc.move_point(2, Point { x: 150., y: 170. });
    let lines = doc.branch_segment_iteration_images(binding).unwrap();
    assert_eq!(
        lines[2],
        (Point { x: 250., y: 190. }, Point { x: 300., y: 200. })
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.branch_segment_iteration_images(binding),
        Some(lines)
    );
    doc.selected.insert(id);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = pasted
        .into_iter()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::MappedPointIteration { .. }))
        .unwrap();
    assert_ne!(doc.objects[copied - 1].kind, doc.objects[id - 1].kind);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    let mut deleted = mapped_example(IterationDepth::Fixed(3)).0;
    deleted.selected.insert(1);
    assert_eq!(deleted.clear_selected().unwrap(), vec![1, 3, 4, 5, 6]);
    assert_eq!(
        Document::from_bytes(&deleted.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        1
    );
}

#[test]
fn mapped_iteration_live_depth_and_undefined_domain() {
    let mut fresh = Document::new();
    let parameter = fresh.add_named_parameter("n", 2., (10, 10)).unwrap();
    let a = fresh.add(Kind::Free(Point { x: 100., y: 160. }));
    let b = fresh.add(Kind::Free(Point { x: 140., y: 180. }));
    let image = fresh.add(Kind::VectorTranslatedPoint(b, a, b));
    let segment = fresh.add(Kind::Segment(a, b));
    let definition = fresh
        .add_mapped_point_iteration(
            &[a, b],
            &[vec![b, image]],
            &[segment, image],
            IterationDepth::Value(parameter),
        )
        .unwrap();
    let binding = definition + 1;
    assert_eq!(
        fresh
            .branch_segment_iteration_images(binding)
            .unwrap()
            .len(),
        2
    );
    fresh.set_parameter_value(parameter, 4.).unwrap();
    assert_eq!(
        fresh
            .branch_segment_iteration_images(binding)
            .unwrap()
            .len(),
        4
    );
    let bytes = fresh.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_FIXTURE").is_some() {
        fs::write("/tmp/mapped_point_iteration_live_generated.gsp", &bytes).unwrap();
    }
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened
            .branch_segment_iteration_images(binding)
            .unwrap()
            .len(),
        4
    );
    fresh.set_parameter_value(parameter, 0.).unwrap();
    assert!(fresh.branch_segment_iteration_images(binding).is_none());
    assert!(fresh.encode().is_ok());
}

#[test]
fn mapped_iteration_ant_native_import_and_mapping_graph() {
    let bytes = include_bytes!("../../../../../tests/fixtures/mapped_point_iteration_ant_native.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(
        doc.objects
            .iter()
            .all(|o| !matches!(o.kind, Kind::Unsupported(_)))
    );
    for id in [37usize, 98] {
        assert!(
            matches!(&doc.objects[id-1].kind,Kind::LinkUrlButton { url,label,anchor } if url=="http://exjh.com" && label==url && *anchor==(33,575))
        );
    }
    let mut edited = doc.clone();
    assert!(edited.move_point(1, Point { x: 407., y: 290. }));
    let saved = edited.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(reopened.position(1), Some(Point { x: 407., y: 290. }));
    assert_eq!(
        reopened.branch_segment_iteration_images(29),
        edited.branch_segment_iteration_images(29)
    );
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_FIXTURE").is_some() {
        fs::write("/tmp/mapped_point_iteration_ant_edited.gsp", &saved).unwrap();
    }
    let url_edited = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/mapped_point_iteration_ant_url_edited_verified.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(
        url_edited
            .objects
            .iter()
            .all(|o| !matches!(o.kind, Kind::Unsupported(_)))
    );
    assert!(
        matches!(&url_edited.objects[36].kind,Kind::LinkUrlButton {url,label,anchor} if url=="http://exjk.com" && label==url && *anchor==(90,575))
    );
    let definitions: Vec<_> = doc
        .objects
        .iter()
        .filter(|o| matches!(o.kind, Kind::MappedPointIteration { .. }))
        .collect();
    assert_eq!(definitions.len(), 2);
    for object in definitions {
        let Kind::MappedPointIteration {
            seeds,
            branches,
            carried,
            depth,
            ..
        } = &object.kind
        else {
            unreachable!()
        };
        assert_eq!(seeds.len(), 4);
        assert_eq!(branches.len(), 4);
        assert!(carried.len() >= 20);
        let source = carried
            .iter()
            .find(|id| matches!(doc.objects[**id - 1].kind, Kind::Segment(_, _)))
            .unwrap();
        let binding = doc
            .objects
            .iter()
            .find(|o| o.kind == Kind::IterationBinding(*source, object.id))
            .unwrap()
            .id;
        let lines = doc.branch_segment_iteration_images(binding).unwrap();
        let (a, b, _) = doc.line_points(*source).unwrap();
        for (x, y) in lines {
            assert!((x.distance(y) - a.distance(b)).abs() < 1e-8);
        }
        assert!(matches!(depth, IterationDepth::Value(_)));
    }
}

#[test]
fn mapped_iteration_two_branches_share_coincident_states_and_keep_segments() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 180., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 220., y: 200. }));
    let right = doc.add(Kind::VectorTranslatedPoint(b, a, b));
    let left = doc.add(Kind::VectorTranslatedPoint(a, b, a));
    let segment = doc.add(Kind::Segment(a, b));
    let iteration = doc
        .add_mapped_point_iteration(
            &[a, b],
            &[vec![b, right], vec![left, a]],
            &[segment, right, left],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    let images = doc.branch_segment_iteration_images(iteration + 1).unwrap();
    assert_eq!(images.len(), 9); // 2, 3, and 4 distinct simultaneous states.
    assert_eq!(
        images[0],
        (Point { x: 220., y: 200. }, Point { x: 260., y: 220. })
    );
    assert_eq!(
        images[1],
        (Point { x: 140., y: 160. }, Point { x: 180., y: 180. })
    );
    assert!(images.contains(&(Point { x: 60., y: 120. }, Point { x: 100., y: 140. })));
    assert!(images.contains(&(Point { x: 300., y: 240. }, Point { x: 340., y: 260. })));
    let vp = crate::render::Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(0., 0.), egui::vec2(800., 600.)),
        scale: 1.,
        center: Point { x: 200., y: 200. },
    };
    let on = vp.screen(Point { x: 80., y: 130. });
    assert!(crate::render::branch_iteration_hit(
        &vp,
        &doc,
        iteration + 1,
        on
    ));
    assert!(!crate::render::branch_iteration_hit(
        &vp,
        &doc,
        iteration + 1,
        on + egui::vec2(0., 30.)
    ));
    assert_eq!(
        crate::render::svg_page(&doc, &vp)
            .unwrap()
            .matches("<line")
            .count(),
        10
    );
    let bytes = doc.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_FIXTURE").is_some() {
        fs::write("/tmp/mapped_point_iteration_branches_generated.gsp", &bytes).unwrap();
    }
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.branch_segment_iteration_images(iteration + 1),
        Some(images)
    );
    assert!(
        doc.add_mapped_point_iteration(
            &[a, a],
            &[vec![b, right]],
            &[segment],
            IterationDepth::Fixed(2)
        )
        .is_err()
    );
    assert!(
        doc.add_mapped_point_iteration(
            &[a, b],
            &[vec![right]],
            &[segment],
            IterationDepth::Fixed(2)
        )
        .is_err()
    );
}

#[test]
fn mapped_iteration_native_verified_generation_and_binding_styles() {
    let (doc, _, binding) = mapped_example(IterationDepth::Fixed(3));
    let expected =
        include_bytes!("../../../../../tests/fixtures/mapped_point_iteration_generated_verified.gsp");
    assert_eq!(doc.encode().unwrap(), expected);
    for bytes in [
        expected.as_slice(),
        include_bytes!("../../../../../tests/fixtures/mapped_point_iteration_branches_verified.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/mapped_point_iteration_live_verified.gsp")
            .as_slice(),
    ] {
        let mut opened = Document::from_bytes(bytes, None).unwrap();
        assert!(!opened.partial_view);
        let binding = opened
            .objects
            .iter()
            .find(|o| matches!(o.kind, Kind::IterationBinding(_, _)))
            .unwrap()
            .id;
        assert!(matches!(
            opened.objects[binding - 1].visual_style,
            Some(VisualStyle::Stroke { .. })
        ));
        opened.selected.insert(binding);
        assert_eq!(
            opened
                .set_selected_color(RgbColor {
                    red: 20,
                    green: 160,
                    blue: 60
                })
                .unwrap(),
            1
        );
        let saved = Document::from_bytes(&opened.encode().unwrap(), None).unwrap();
        assert_eq!(
            saved.objects[binding - 1].visual_style,
            opened.objects[binding - 1].visual_style
        );
    }
    assert_eq!(
        doc.objects[binding - 1].visual_style,
        doc.objects[3].visual_style
    );
}

#[test]
fn multi_page_ant_parameter_depth_edit_is_local_atomic_and_undoable() {
    let bytes = include_bytes!("../../../../../tests/fixtures/mapped_point_iteration_ant_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(
        doc.objects
            .iter()
            .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
    );
    doc.active_page = 0;
    assert_eq!(doc.numeric_value(25), Some(3.));
    assert_eq!(doc.numeric_value(73), Some(4.));
    let first_images = doc.branch_segment_iteration_images(29).unwrap();
    let second_images = doc.branch_segment_iteration_images(76).unwrap();
    let untouched = doc.objects[37..].to_vec();
    assert!(doc.set_parameter_value(73, 2.).is_err());
    assert!(doc.set_parameter_value(25, f64::NAN).is_err());
    assert_eq!(doc.objects[37..], untouched);
    assert!(!doc.dirty);
    assert_eq!(doc.numeric_value(25), Some(3.));
    doc.set_parameter_value(25, 2.).unwrap();
    assert_eq!(doc.numeric_value(25), Some(2.));
    assert_eq!(doc.objects[37..], untouched);
    assert!(doc.branch_segment_iteration_images(29).unwrap().len() < first_images.len());
    assert_eq!(
        doc.branch_segment_iteration_images(76).unwrap(),
        second_images
    );
    let edited = doc.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.partial_view, doc.partial_view);
    assert_eq!(reopened.numeric_value(25), Some(2.));
    assert_eq!(reopened.numeric_value(73), Some(4.));
    assert_eq!(
        reopened.branch_segment_iteration_images(29),
        doc.branch_segment_iteration_images(29)
    );
    assert_eq!(
        reopened.branch_segment_iteration_images(76).unwrap(),
        second_images
    );
    doc.undo();
    assert_eq!(doc.numeric_value(25), Some(3.));
    assert_eq!(
        doc.branch_segment_iteration_images(29).unwrap(),
        first_images
    );
    doc.redo();
    assert_eq!(doc.numeric_value(25), Some(2.));
    doc.active_page = 1;
    doc.set_parameter_value(73, 3.).unwrap();
    assert_eq!(doc.numeric_value(25), Some(2.));
    assert_eq!(doc.numeric_value(73), Some(3.));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.numeric_value(25), Some(2.));
    assert_eq!(reopened.numeric_value(73), Some(3.));
}

#[test]
fn mapped_midpoint_triangle_native_point_bindings_and_simultaneous_geometry() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/mapped_midpoint_triangle_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(
        matches!(&doc.objects[9].kind, Kind::MappedPointIteration { seeds, branches, carried, depth, .. }
        if seeds == &[1,2,3] && branches == &[vec![8,9,7]] && carried == &[4,5,6,7,8,9]
            && *depth == IterationDepth::Fixed(3))
    );
    for id in 11..=16 {
        assert_eq!(doc.objects[id - 1].kind, Kind::IterationBinding(id - 7, 10));
    }
    let lines = doc.branch_segment_iteration_images(11).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0],
        (Point { x: 751., y: 467.5 }, Point { x: 853., y: 319.5 })
    );
    let points = doc.mapped_point_binding_images(14).unwrap();
    assert_eq!(points.len(), 3);
    assert_eq!(points[0], Point { x: 802., y: 393.5 });
    assert!(matches!(
        doc.objects[13].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            color: RgbColor {
                red: 255,
                green: 0,
                blue: 0
            }
        })
    ));
    doc.selected.insert(14);
    assert_eq!(doc.set_selected_point_style(PointStyle::Large).unwrap(), 1);
    assert_eq!(
        doc.set_selected_color(RgbColor {
            red: 14,
            green: 150,
            blue: 210
        })
        .unwrap(),
        1
    );
    doc.move_point(1, Point { x: 800., y: 200. });
    assert_ne!(doc.mapped_point_binding_images(14).unwrap(), points);
    doc.verify_roundtrip().unwrap();
    let saved = doc.encode().unwrap();
    assert_eq!(
        saved.as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/mapped_midpoint_triangle_edited_native_verified.gsp"
        )
    );
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_MIDPOINT_FIXTURE").is_some() {
        fs::write("/tmp/mapped_midpoint_triangle_edited.gsp", &saved).unwrap();
    }
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(
        reopened.mapped_point_binding_images(14),
        doc.mapped_point_binding_images(14)
    );
    assert_eq!(
        reopened.objects[13].visual_style,
        doc.objects[13].visual_style
    );
    let vp = crate::render::Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 800.)),
        scale: 1.,
        center: Point { x: 750., y: 350. },
    };
    let svg = crate::render::svg_page(&reopened, &vp).unwrap();
    assert_eq!(
        svg.lines()
            .filter(|line| line.contains("r=\"6\" fill=\"#0e96d2\""))
            .count(),
        3
    );
    let at = vp.screen(reopened.mapped_point_binding_images(14).unwrap()[0]);
    assert!(crate::render::point_iteration_hit(&vp, &reopened, 14, at));
    assert!(!crate::render::point_iteration_hit(
        &vp,
        &reopened,
        14,
        at + egui::vec2(20., 0.)
    ));
}

#[test]
fn mapped_midpoint_triangle_authors_explicit_bindings_and_keeps_prerequisites_distinct() {
    let mut doc = Document::new();
    for p in [
        Point { x: 200., y: 100. },
        Point { x: 100., y: 300. },
        Point { x: 400., y: 250. },
    ] {
        doc.add(Kind::Free(p));
    }
    for (a, b) in [(1, 2), (2, 3), (3, 1)] {
        doc.add(Kind::Segment(a, b));
    }
    for (a, b) in [(1, 2), (2, 3), (3, 1)] {
        doc.add(Kind::Midpoint(a, b));
    }
    let carried = [4, 5, 6, 7, 8, 9];
    let id = doc
        .add_mapped_point_iteration_with_bindings(
            &[1, 2, 3],
            &[vec![8, 9, 7]],
            &carried,
            &carried,
            IterationDepth::Fixed(2),
        )
        .unwrap();
    assert_eq!(id, 10);
    assert_eq!(doc.objects.len(), 16);
    for binding in 11..=13 {
        assert_eq!(
            doc.branch_segment_iteration_images(binding).unwrap().len(),
            2
        );
    }
    for binding in 14..=16 {
        assert_eq!(doc.point_iteration_images(binding).unwrap().len(), 2);
    }
    doc.verify_roundtrip().unwrap();
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/mapped_midpoint_triangle_generated_native_verified.gsp"
        )
    );
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_MIDPOINT_FIXTURE").is_some() {
        fs::write(
            "/tmp/mapped_midpoint_triangle_generated.gsp",
            doc.encode().unwrap(),
        )
        .unwrap();
    }
    doc.selected.insert(id);
    let clip = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clip).unwrap();
    let parent = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::MappedPointIteration { .. }))
        .unwrap();
    assert_eq!(
        doc.objects
            .iter()
            .filter(|o| matches!(o.kind,Kind::IterationBinding(_,p)if p==parent))
            .count(),
        6
    );
    doc.verify_roundtrip().unwrap();
    let before = doc.objects.len();
    assert!(
        doc.add_mapped_point_iteration_with_bindings(
            &[1, 2, 3],
            &[vec![8, 9, 7]],
            &carried,
            &[1],
            IterationDepth::Fixed(2)
        )
        .is_err()
    );
    assert_eq!(doc.objects.len(), before);
}

#[test]
fn mapped_calculated_angle_native_parameter_updates_and_point_segment_bindings() {
    let full_bytes = include_bytes!(
        "../../../../../tests/fixtures/mapped_calculated_angle_iteration_full_native_verified.gsp"
    );
    let full = Document::from_bytes(full_bytes, None).unwrap();
    assert!(!full.partial_view);
    assert!(matches!(
        full.objects[6].kind,
        Kind::MappedPointIteration { .. }
    ));
    assert!(matches!(full.objects[10].kind, Kind::ReferencedText { .. }));
    assert_eq!(full.encode().unwrap(), full_bytes);
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/mapped_calculated_angle_iteration_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects.len(), 9);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(
        matches!(&doc.objects[6].kind,Kind::MappedPointIteration{seeds,branches,carried,depth,..}
        if seeds==&[1,2]&&branches==&[vec![2,6]]&&carried==&[3,6]&&*depth==IterationDepth::Value(4))
    );
    assert_eq!(doc.objects[7].kind, Kind::IterationBinding(6, 7));
    assert_eq!(doc.objects[8].kind, Kind::IterationBinding(3, 7));
    for n in [17., 5.] {
        doc.set_parameter_value(4, n).unwrap();
        let theta = (180. / n).to_radians();
        let lines = doc.branch_segment_iteration_images(9).unwrap();
        assert_eq!(lines.len(), n as usize);
        assert_eq!(lines[0].0, Point { x: 687., y: 345. });
        assert!((lines[0].1.x - (687. - 180. * theta.cos())).abs() < 1e-9);
        assert!((lines[0].1.y - (345. + 180. * theta.sin())).abs() < 1e-9);
        assert!(
            lines
                .iter()
                .all(|(a, b)| (a.distance(*b) - 180.).abs() < 1e-9)
        );
        assert_eq!(
            doc.mapped_point_binding_images(8).unwrap().len(),
            n as usize
        );
        doc.verify_roundtrip().unwrap();
    }
    doc.move_point(1, Point { x: 487., y: 325. });
    doc.selected.insert(8);
    doc.set_selected_color(RgbColor {
        red: 14,
        green: 150,
        blue: 210,
    })
    .unwrap();
    let saved = doc.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_ANGLE_FIXTURE").is_some() {
        fs::write("/tmp/mapped_calculated_angle_iteration_edited.gsp", &saved).unwrap();
    }
    assert_eq!(
        saved.as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/mapped_calculated_angle_iteration_edited_native_verified.gsp"
        )
    );
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(
        reopened.branch_segment_iteration_images(9),
        doc.branch_segment_iteration_images(9)
    );
    doc.selected.clear();
    doc.selected.insert(7);
    let clip = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clip).unwrap();
    let parent = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::MappedPointIteration { .. }))
        .unwrap();
    assert_eq!(
        doc.objects
            .iter()
            .filter(|o| matches!(o.kind,Kind::IterationBinding(_,p)if p==parent))
            .count(),
        2
    );
    doc.verify_roundtrip().unwrap();
    let mut undefined = reopened;
    undefined.set_parameter_value(4, 0.).unwrap();
    assert!(undefined.branch_segment_iteration_images(9).is_none());
    assert!(undefined.encode().is_ok());
}

#[test]
fn mapped_calculated_angle_authored_typed_measurement_inputs_and_live_depth() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 250., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 430., y: 180. }));
    let segment = doc.add(Kind::Segment(a, b));
    let n = doc.add_named_parameter("N", 5., (10, 30)).unwrap();
    let angle = doc
        .add_parameter_calculation(n, "180°/N", (10, 60))
        .unwrap();
    let image = doc.add_calculated_angle_rotation(a, b, angle).unwrap();
    let iteration = doc
        .add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[vec![b, image]],
            &[segment, image],
            &[image, segment],
            IterationDepth::Value(n),
        )
        .unwrap();
    assert_eq!(iteration, 7);
    assert_eq!(doc.point_iteration_images(8).unwrap().len(), 5);
    doc.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_MAPPED_ANGLE_FIXTURE").is_some() {
        fs::write(
            "/tmp/mapped_calculated_angle_iteration_generated.gsp",
            doc.encode().unwrap(),
        )
        .unwrap();
    }
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/mapped_calculated_angle_iteration_generated_native_verified.gsp"
        )
    );
    let measured = doc.add(Kind::AngleMeasurement(a, b, image));
    let measured_image = doc.add_calculated_angle_rotation(a, b, measured).unwrap();
    let measured_iteration = doc
        .add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[vec![b, measured_image]],
            &[segment, measured_image],
            &[segment],
            IterationDepth::Fixed(2),
        )
        .unwrap();
    let measured_lines = doc
        .branch_segment_iteration_images(measured_iteration + 1)
        .unwrap();
    let formula_lines = doc.branch_segment_iteration_images(9).unwrap();
    for ((a, b), (c, d)) in measured_lines.iter().zip(&formula_lines) {
        assert!(a.distance(*c) < 1e-9 && b.distance(*d) < 1e-9);
    }
    doc.verify_roundtrip().unwrap();
    // Scalar inputs do not satisfy a degree rotation's source contract.
    let wrong_image = doc.add(Kind::CalculatedAngleRotatedPoint(a, b, n));
    let count = doc.objects.len();
    assert!(
        doc.add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[vec![b, wrong_image]],
            &[segment, wrong_image],
            &[segment],
            IterationDepth::Fixed(2),
        )
        .is_err()
    );
    assert_eq!(doc.objects.len(), count);
}

#[test]
fn mapped_iteration_final_level_flag_matches_native_and_preserves_raw_definition() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/mapped_midpoint_triangle_generated_native_verified.gsp"
    );
    let final_bytes = include_bytes!(
        "../../../../../tests/fixtures/mapped_midpoint_triangle_final_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(original, None).unwrap();
    let all_lines = doc.branch_segment_iteration_images(11).unwrap();
    let all_points = doc.point_iteration_images(14).unwrap();
    assert_eq!(all_lines.len(), 2);
    assert_eq!(all_points.len(), 2);
    doc.set_mapped_iteration_final_only(10, true).unwrap();
    assert_eq!(
        doc.branch_segment_iteration_images(11).unwrap(),
        vec![*all_lines.last().unwrap()]
    );
    assert_eq!(
        doc.point_iteration_images(14).unwrap(),
        vec![*all_points.last().unwrap()]
    );
    assert_eq!(doc.encode().unwrap(), final_bytes);
    let reopened = Document::from_bytes(final_bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[9].kind,
        Kind::MappedPointIteration {
            final_only: true,
            ..
        }
    ));
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), final_bytes);
    doc.selected.insert(10);
    let clip = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clip).unwrap();
    let parent = pasted
        .iter()
        .copied()
        .find(|id| {
            matches!(
                doc.objects[*id - 1].kind,
                Kind::MappedPointIteration {
                    final_only: true,
                    ..
                }
            )
        })
        .unwrap();
    let binding=doc.objects.iter().find(|o|matches!(o.kind,Kind::IterationBinding(source,p)if p==parent&&doc.line_points(source).is_some())).unwrap().id;
    assert_eq!(
        doc.branch_segment_iteration_images(binding).unwrap().len(),
        1
    );
    doc.verify_roundtrip().unwrap();
    let mut unknown = final_bytes.to_vec();
    let definition = records(&unknown)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x090a)
        .unwrap();
    unknown[definition.data.start + 4] = 2;
    let mut unsupported = Document::from_bytes(&unknown, None).unwrap();
    assert_eq!(unsupported.objects[9].kind, Kind::Unsupported(76));
    assert_eq!(unsupported.encode().unwrap(), unknown);
    assert!(
        unsupported
            .set_mapped_iteration_final_only(10, false)
            .is_err()
    );
}

#[test]
fn mapped_iteration_final_level_keeps_all_branches_and_live_depth() {
    let final_bytes =
        include_bytes!("../../../../../tests/fixtures/mapped_branches_final_native_verified.gsp");
    let mut doc = Document::from_bytes(final_bytes, None).unwrap();
    let parent = doc
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::MappedPointIteration { .. }))
        .unwrap()
        .id;
    let binding = parent + 1;
    assert_eq!(
        doc.branch_segment_iteration_images(binding).unwrap().len(),
        4
    );
    doc.set_mapped_iteration_final_only(parent, false).unwrap();
    assert_eq!(
        doc.branch_segment_iteration_images(binding).unwrap().len(),
        9
    );
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/mapped_point_iteration_branches_verified.gsp")
    );
    let (mut live, parent, binding) = {
        let mut d = Document::new();
        let n = d.add_named_parameter("n", 2., (10, 10)).unwrap();
        let a = d.add(Kind::Free(Point { x: 100., y: 160. }));
        let b = d.add(Kind::Free(Point { x: 140., y: 180. }));
        let image = d.add(Kind::VectorTranslatedPoint(b, a, b));
        let segment = d.add(Kind::Segment(a, b));
        let parent = d
            .add_mapped_point_iteration(
                &[a, b],
                &[vec![b, image]],
                &[segment, image],
                IterationDepth::Value(n),
            )
            .unwrap();
        (d, parent, parent + 1)
    };
    live.set_mapped_iteration_final_only(parent, true).unwrap();
    assert_eq!(
        live.branch_segment_iteration_images(binding).unwrap(),
        vec![(Point { x: 180., y: 200. }, Point { x: 220., y: 220. })]
    );
    live.set_parameter_value(1, 4.).unwrap();
    assert_eq!(
        live.branch_segment_iteration_images(binding).unwrap(),
        vec![(Point { x: 260., y: 240. }, Point { x: 300., y: 260. })]
    );
    live.verify_roundtrip().unwrap();
    assert_eq!(
        live.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/mapped_final_level_live_native_verified.gsp")
    );
    if std::env::var_os("SKETCHPAD_WRITE_FINAL_ITERATION_FIXTURE").is_some() {
        fs::write("/tmp/mapped_final_level_live.gsp", live.encode().unwrap()).unwrap();
    }
}

#[test]
fn numeric_iteration_parameter_seed_remains_scalar_until_typed_native_proof() {
    for parameter in [1, 2, 3] {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/numeric_typed_parameters_native_verified.gsp"),
            None,
        )
        .unwrap();
        let seed = doc.add(Kind::Free(Point { x: 100., y: 160. }));
        let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 37.79527559055118, 1.));
        let group =
            crate::table_records::ParameterCalculationGroup::from_expression_with_source_unit(
                parameter as u32,
                "v",
                "v/v",
                (10, 100),
                doc.objects[parameter - 1].kind.numeric_unit().unwrap(),
            )
            .unwrap();
        let calculation = doc.add(Kind::ParameterCalculation(parameter, group));
        let label = doc.add(Kind::PointNumericLabel(image, calculation));
        let before = doc.objects.len();
        let result = doc.add_point_parameter_iteration(
            seed,
            parameter,
            image,
            calculation,
            label,
            IterationDepth::Fixed(2),
        );
        if parameter == 3 {
            assert!(result.is_ok());
            doc.verify_roundtrip().unwrap();
        } else {
            assert!(result.is_err());
            assert_eq!(doc.objects.len(), before);
        }
    }
}

fn derived_seed_carried_transform(dilate: bool) -> (Document, usize, usize) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 80., y: 160. }));
    let b = doc.add(Kind::Free(Point { x: 160., y: 160. }));
    doc.add(Kind::Segment(a, b));
    let midpoint = doc.add(Kind::Midpoint(a, b));
    let fixed = doc.add(Kind::Free(Point { x: 120., y: 200. }));
    let image = doc.add(Kind::VectorTranslatedPoint(midpoint, a, b));
    let other_image = doc.add(Kind::VectorTranslatedPoint(fixed, a, b));
    let segment = doc.add(Kind::Segment(midpoint, fixed));
    let transformed = doc.add(if dilate {
        Kind::ScaledSegment(segment, midpoint, 0.5)
    } else {
        Kind::RotatedSegment(segment, midpoint, 90.)
    });
    let iteration = doc
        .add_mapped_point_iteration_with_bindings(
            &[midpoint, fixed],
            &[vec![image, fixed]],
            &[segment, transformed, image, other_image],
            &[transformed],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    (doc, iteration, iteration + 1)
}

#[test]
fn mapped_derived_seed_recomputes_transformed_segment_and_preserves_live_source() {
    for dilate in [false, true] {
        let (mut doc, iteration, binding) = derived_seed_carried_transform(dilate);
        let initial_kind = doc.objects[3].kind.clone();
        let images = doc.branch_segment_iteration_images(binding).unwrap();
        assert_eq!(images.len(), 3);
        for (step, (a, b)) in images.iter().enumerate() {
            let x = 200. + 80. * step as f64;
            assert!(a.distance(Point { x, y: 160. }) < 1e-9);
            let end = if dilate {
                Point {
                    x: (x + 120.) / 2.,
                    y: 180.,
                }
            } else {
                Point {
                    x: x + 40.,
                    y: x + 40.,
                }
            };
            assert!(b.distance(end) < 1e-9, "{step}: {b:?}, expected {end:?}");
        }
        // Evaluating an overridden midpoint must leave the source DAG intact.
        assert_eq!(doc.objects[3].kind, initial_kind);
        assert_eq!(doc.position(4), Some(Point { x: 120., y: 160. }));
        doc.verify_roundtrip().unwrap();
        let original = doc.encode().unwrap();
        let native = if dilate {
            include_bytes!(
                "../../../../../tests/fixtures/mapped_derived43_two_dilation_native_verified.gsp"
            )
            .as_slice()
        } else {
            include_bytes!(
                "../../../../../tests/fixtures/mapped_derived43_two_rotation_native_verified.gsp"
            )
            .as_slice()
        };
        assert_eq!(original.as_slice(), native);
        let reopened = Document::from_bytes(&original, None).unwrap();
        assert_eq!(
            reopened.branch_segment_iteration_images(binding),
            Some(images.clone())
        );
        doc.checkpoint();
        doc.move_point(1, Point { x: 100., y: 160. });
        let changed = doc.branch_segment_iteration_images(binding).unwrap();
        assert_eq!(changed[0].0, Point { x: 190., y: 160. });
        assert_eq!(changed[2].0, Point { x: 310., y: 160. });
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), original);
        assert_eq!(doc.branch_segment_iteration_images(binding), Some(images));
        doc.redo();
        assert_eq!(doc.branch_segment_iteration_images(binding), Some(changed));
        doc.set_mapped_iteration_final_only(iteration, true)
            .unwrap();
        assert_eq!(
            doc.branch_segment_iteration_images(binding).unwrap().len(),
            1
        );
        doc.verify_roundtrip().unwrap();
    }
}

fn mapped_scalar_fibonacci(calculated_seeds: bool) -> (Document, usize) {
    mapped_scalar_fibonacci_with_depth(calculated_seeds, false)
}

fn mapped_scalar_fibonacci_with_depth(calculated_seeds: bool, live: bool) -> (Document, usize) {
    let mut doc = Document::new();
    let a = if calculated_seeds {
        doc.add_calculation_expression("1", (10, 30)).unwrap()
    } else {
        doc.add_named_parameter("a", 1., (10, 30)).unwrap()
    };
    let b = if calculated_seeds {
        doc.add_calculation_expression("1", (10, 60)).unwrap()
    } else {
        doc.add_named_parameter("b", 1., (10, 60)).unwrap()
    };
    let sum = doc
        .add_multi_calculation(&[a, b], "v1+v2", (10, 90))
        .unwrap();
    let frame = doc
        .add_default_coordinate_system(Point { x: 160., y: 360. })
        .unwrap();
    let point = doc.add(Kind::NumericCoordinatePoint(a, b, frame));
    let image = doc.add(Kind::NumericCoordinatePoint(b, sum, frame));
    let segment = doc.add(Kind::Segment(point, image));
    let depth = if live {
        IterationDepth::Value(doc.add_named_parameter("n", 3., (10, 120)).unwrap())
    } else {
        IterationDepth::Fixed(3)
    };
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[vec![b, sum]],
            &[sum, point, image, segment],
            &[point, segment],
            depth,
        )
        .unwrap();
    (doc, parent)
}

#[test]
fn mapped_scalar_states_substitute_simultaneously_and_recompute_coordinate_dag() {
    for calculated in [false, true] {
        let (mut doc, iteration) = mapped_scalar_fibonacci(calculated);
        let source_kinds: Vec<_> = doc
            .objects
            .iter()
            .take(3)
            .map(|object| object.kind.clone())
            .collect();
        let to_world = |x: f64, y: f64| Point {
            x: 160. + x * 37.79527559055118,
            y: 360. - y * 37.79527559055118,
        };
        let points = doc.point_iteration_images(iteration + 1).unwrap();
        let lines = doc.branch_segment_iteration_images(iteration + 2).unwrap();
        for ((point, (first, last)), ((a, b), (c, d))) in points.iter().zip(&lines).zip([
            ((1., 2.), (2., 3.)),
            ((2., 3.), (3., 5.)),
            ((3., 5.), (5., 8.)),
        ]) {
            assert!(point.distance(to_world(a, b)) < 1e-9);
            assert!(first.distance(to_world(a, b)) < 1e-9);
            assert!(last.distance(to_world(c, d)) < 1e-9);
        }
        assert_eq!(points.len(), 3);
        assert_eq!(lines.len(), 3);
        assert_eq!(
            doc.objects
                .iter()
                .take(3)
                .map(|object| object.kind.clone())
                .collect::<Vec<_>>(),
            source_kinds
        );
        assert_eq!(doc.numeric_value(3), Some(2.));
        doc.verify_roundtrip().unwrap();
        let bytes = doc.encode().unwrap();
        let native = if calculated {
            include_bytes!(
                "../../../../../tests/fixtures/numeric_mapped43_api_calculated_native_verified.gsp"
            )
            .as_slice()
        } else {
            include_bytes!(
                "../../../../../tests/fixtures/numeric_mapped43_api_fibonacci_native_verified.gsp"
            )
            .as_slice()
        };
        assert_eq!(bytes.as_slice(), native);
        assert_eq!(
            Document::from_bytes(&bytes, None)
                .unwrap()
                .point_iteration_images(iteration + 1),
            Some(points.clone())
        );
        if !calculated {
            doc.set_parameter_value(1, 2.).unwrap();
            assert!(
                doc.point_iteration_images(iteration + 1).unwrap()[1].distance(to_world(3., 4.))
                    < 1e-9
            );
            doc.verify_roundtrip().unwrap();
            doc.undo();
            assert_eq!(doc.encode().unwrap(), bytes);
            doc.redo();
            assert!(
                doc.point_iteration_images(iteration + 1).unwrap()[1].distance(to_world(3., 4.))
                    < 1e-9
            );
        }
        doc.set_mapped_iteration_final_only(iteration, true)
            .unwrap();
        assert_eq!(doc.point_iteration_images(iteration + 1).unwrap().len(), 1);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn mapped_scalar_roles_are_checked_and_undefined_states_recover_without_mutating_sources() {
    let (mut doc, _) = mapped_scalar_fibonacci(false);
    let count = doc.objects.len();
    let point = 10;
    assert!(
        doc.add_mapped_point_iteration_with_bindings(
            &[point, 1],
            &[vec![1, point]],
            &[12],
            &[12],
            IterationDepth::Fixed(2),
        )
        .is_err()
    );
    assert_eq!(doc.objects.len(), count);
    // A runtime pole can arise after a valid first generation; it suppresses
    // the whole sequence without changing the editable original calculation.
    let mut pole = Document::new();
    let a = pole.add_named_parameter("a", 1., (10, 30)).unwrap();
    let b = pole.add_named_parameter("b", 2., (10, 60)).unwrap();
    let image = pole
        .add_multi_calculation(&[a, b], "v1/(v2-1)", (10, 90))
        .unwrap();
    let frame = pole
        .add_default_coordinate_system(Point { x: 160., y: 360. })
        .unwrap();
    let plotted = pole.add(Kind::NumericCoordinatePoint(a, b, frame));
    let parent = pole
        .add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[vec![b, image]],
            &[plotted, image],
            &[plotted],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    assert!(pole.point_iteration_images(parent + 1).is_none());
    assert_eq!(pole.numeric_value(image), Some(1.));
    pole.verify_roundtrip().unwrap();
    pole.set_parameter_value(b, 3.).unwrap();
    assert_eq!(pole.point_iteration_images(parent + 1).unwrap().len(), 3);
    assert_eq!(pole.numeric_value(image), Some(0.5));
    pole.verify_roundtrip().unwrap();
    pole.undo();
    assert!(pole.point_iteration_images(parent + 1).is_none());
    pole.redo();
    assert_eq!(pole.point_iteration_images(parent + 1).unwrap().len(), 3);
    // The API must reject malformed construction graphs before checkpointing.
    let segment = pole.add(Kind::Segment(a, b));
    pole.objects[segment - 1].kind = Kind::Segment(segment, segment);
    let count = pole.objects.len();
    assert!(
        pole.add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[vec![b, image]],
            &[segment],
            &[segment],
            IterationDepth::Fixed(3),
        )
        .is_err()
    );
    assert_eq!(pole.objects.len(), count);
}

#[test]
fn mapped_mixed_point_scalar_states_share_one_substitution_step() {
    let mut doc = Document::new();
    let seed = doc.add(Kind::Free(Point { x: 100., y: 300. }));
    let scalar = doc.add_named_parameter("b", 1., (10, 60)).unwrap();
    let scalar_image = doc
        .add_parameter_calculation(scalar, "b+1", (10, 90))
        .unwrap();
    let frame = doc
        .add_default_coordinate_system(Point { x: 160., y: 360. })
        .unwrap();
    let image = doc.add(Kind::NumericCoordinatePoint(scalar, scalar, frame));
    let segment = doc.add(Kind::Segment(seed, image));
    let iteration = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed, scalar],
            &[vec![image, scalar_image]],
            &[scalar_image, seed, image, segment],
            &[seed, segment],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    let to_world = |n: f64| Point {
        x: 160. + n * 37.79527559055118,
        y: 360. - n * 37.79527559055118,
    };
    let points = doc.point_iteration_images(iteration + 1).unwrap();
    let lines = doc.branch_segment_iteration_images(iteration + 2).unwrap();
    for (at, (point, (first, last))) in points.iter().zip(&lines).enumerate() {
        assert!(point.distance(to_world(at as f64 + 1.)) < 1e-9);
        assert!(first.distance(*point) < 1e-9);
        assert!(last.distance(to_world(at as f64 + 2.)) < 1e-9);
    }
    assert_eq!(points.len(), 3);
    assert_eq!(doc.position(seed), Some(Point { x: 100., y: 300. }));
    assert_eq!(doc.numeric_value(scalar), Some(1.));
    doc.verify_roundtrip().unwrap();
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!("../../../../../tests/fixtures/numeric_mapped43_api_mixed_native_verified.gsp")
    );
    doc.set_parameter_value(scalar, 2.).unwrap();
    assert!(doc.point_iteration_images(iteration + 1).unwrap()[0].distance(to_world(2.)) < 1e-9);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.point_iteration_images(iteration + 1), Some(points));
}

#[test]
fn mapped_scalar_live_depth_and_edits_reconstruct_native_outputs() {
    let (mut doc, parent) = mapped_scalar_fibonacci_with_depth(false, true);
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!("../../../../../tests/fixtures/numeric_mapped43_api_live_native_verified.gsp")
    );
    doc.set_parameter_value(1, 2.).unwrap();
    doc.set_parameter_value(2, 1.).unwrap();
    doc.set_parameter_value(13, 2.).unwrap();
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!("../../../../../tests/fixtures/numeric_mapped43_api_edited_native_verified.gsp")
    );
    let images = doc.point_iteration_images(parent + 1).unwrap();
    assert_eq!(images.len(), 2);
    let to_world = |x: f64, y: f64| Point {
        x: 160. + x * 37.79527559055118,
        y: 360. - y * 37.79527559055118,
    };
    assert!(images[0].distance(to_world(1., 3.)) < 1e-9);
    assert!(images[1].distance(to_world(3., 4.)) < 1e-9);
    doc.verify_roundtrip().unwrap();
    doc.set_parameter_value(13, 0.).unwrap();
    assert!(doc.point_iteration_images(parent + 1).is_none());
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.point_iteration_images(parent + 1), Some(images));
}

#[test]
fn mapped_pentagon_corpus_recomputes_rotation_dilation_graph_for_every_branch() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/mapped_pentagon43_corpus_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
    assert!(
        matches!(&doc.objects[15].kind,Kind::MappedPointIteration{seeds,branches,depth:IterationDepth::Fixed(2),final_only:true,..}
        if seeds==&[1,2] && branches==&[vec![10,6],vec![6,8],vec![8,9],vec![9,7],vec![7,10]])
    );
    // The Native control shows five clusters of five final pentagons.
    // Each branch joins adjacent inner vertices, scaling the seed side by φ⁻²;
    // two substitutions therefore scale it by φ⁻⁴.
    let original = doc.position(1).unwrap().distance(doc.position(2).unwrap());
    let ratio = (5_f64.sqrt() - 1.) / 2.;
    for binding in 17..=21 {
        let lines = doc.branch_segment_iteration_images(binding).unwrap();
        assert_eq!(lines.len(), 25);
        for (a, b) in lines {
            assert!((a.distance(b) - original * ratio.powi(4)).abs() < 1e-8);
        }
    }
    // Each branch's adjacent carried sides meet after the same simultaneous
    // seed substitution, including their dependent rotation centers.
    let all = (17..=21)
        .map(|id| doc.branch_segment_iteration_images(id).unwrap())
        .collect::<Vec<_>>();
    for generation in 0..25 {
        for side in 0..5 {
            assert!(
                all[side][generation]
                    .1
                    .distance(all[(side + 1) % 5][generation].0)
                    < 1e-8
            );
        }
    }
    assert_eq!(doc.position(1), Some(Point { x: 291., y: 486. }));
    doc.verify_roundtrip().unwrap();
    // The referenced caption now completes the original graph, so this verified
    // iteration edit is available and retains the original presentation bytes.
    doc.set_mapped_iteration_final_only(16, false).unwrap();
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
}
