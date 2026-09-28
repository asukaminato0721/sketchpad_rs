use super::*;

fn broad_binding_graph(numeric_depth: bool) -> (Document, usize, [usize; 5]) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 80., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 120., y: 100. }));
    let u = doc.add(Kind::Free(Point { x: 20., y: 200. }));
    let v = doc.add(Kind::Free(Point { x: 40., y: 200. }));
    let seed_host = doc.add(Kind::Segment(a, b));
    let seed = doc.add(Kind::SegmentMidpoint(seed_host));
    let image = doc.add(Kind::VectorTranslatedPoint(seed, u, v));
    let top = doc.add(Kind::PolarTranslatedPoint(seed, 90., 1., 20.));
    let circle = doc.add(Kind::Circle(seed, image));
    let polygon = doc.add(Kind::Polygon(vec![seed, image, top]));
    let ray = doc.add(Kind::Ray(seed, image));
    let line = doc.add(Kind::Line(seed, top));
    let arc = doc.add(Kind::CenterArc(seed, image, top));
    let depth = if numeric_depth {
        IterationDepth::Value(doc.add_named_parameter("n", 3., (10, 100)).unwrap())
    } else {
        IterationDepth::Fixed(3)
    };
    let sources = [circle, polygon, ray, line, arc];
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![image]],
            &[image, top, circle, polygon, ray, line, arc],
            &sources,
            depth,
        )
        .unwrap();
    assert!(matches!(
        doc.objects[parent - 1].kind,
        Kind::MappedPointIteration { .. }
    ));
    (doc, parent, sources)
}

#[test]
fn generic_shape_bindings_share_derived_seed_substitution_and_preserve_source_kinds() {
    for numeric_depth in [false, true] {
        let (doc, parent, sources) = broad_binding_graph(numeric_depth);
        let original_kinds: Vec<_> = sources
            .iter()
            .map(|id| format!("{:?}", doc.objects[id - 1].kind))
            .collect();
        let circles = doc.circle_iteration_images(parent + 1).unwrap();
        let polygons = doc.polygon_iteration_images(parent + 2).unwrap();
        let rays = doc.ray_iteration_images(parent + 3).unwrap();
        let lines = doc.line_iteration_images(parent + 4).unwrap();
        let arcs = doc.arc_iteration_images(parent + 5, 16).unwrap();
        for generation in 0..3 {
            let center = Point {
                x: 120. + generation as f64 * 20.,
                y: 100.,
            };
            assert_eq!(circles[generation], (center, 20.));
            assert_eq!(polygons[generation].0[0], center);
            assert_eq!(
                polygons[generation].0[1],
                Point {
                    x: center.x + 20.,
                    y: 100.
                }
            );
            assert_eq!(polygons[generation].1, 1.);
            assert_eq!(
                rays[generation],
                (
                    center,
                    Point {
                        x: center.x + 20.,
                        y: 100.
                    }
                )
            );
            assert_eq!(lines[generation].0, center);
            assert!(
                lines[generation].1.distance(Point {
                    x: center.x,
                    y: 80.
                }) < 1e-10
            );
            assert_eq!(
                arcs[generation].first().copied(),
                Some(Point {
                    x: center.x + 20.,
                    y: 100.
                })
            );
            assert!(
                arcs[generation].last().unwrap().distance(Point {
                    x: center.x,
                    y: 80.
                }) < 1e-10
            );
        }
        assert_eq!(doc.position(6), Some(Point { x: 100., y: 100. }));
        assert_eq!(
            original_kinds,
            sources
                .iter()
                .map(|id| format!("{:?}", doc.objects[id - 1].kind))
                .collect::<Vec<_>>()
        );
        doc.verify_roundtrip().unwrap();
        let bytes = doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(reopened.circle_iteration_images(parent + 1), Some(circles));
        assert_eq!(
            reopened.polygon_iteration_images(parent + 2),
            Some(polygons)
        );
        assert_eq!(reopened.ray_iteration_images(parent + 3), Some(rays));
        assert_eq!(reopened.line_iteration_images(parent + 4), Some(lines));
        assert_eq!(reopened.arc_iteration_images(parent + 5, 16), Some(arcs));
    }
}

#[test]
fn broad_binding_source_edits_and_final_level_selection_have_exact_structural_undo() {
    let (mut doc, parent, _) = broad_binding_graph(true);
    let original = doc.encode().unwrap();
    let circles = doc.circle_iteration_images(parent + 1).unwrap();
    doc.checkpoint();
    doc.move_point(4, Point { x: 50., y: 200. });
    let edited = doc.encode().unwrap();
    let changed = doc.circle_iteration_images(parent + 1).unwrap();
    assert_eq!(changed[2], (Point { x: 190., y: 100. }, 30.));
    assert_ne!(changed, circles);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), edited);
    doc.set_mapped_iteration_final_only(parent, true).unwrap();
    assert_eq!(
        doc.circle_iteration_images(parent + 1),
        Some(vec![changed[2]])
    );
    assert_eq!(doc.polygon_iteration_images(parent + 2).unwrap().len(), 1);
    assert_eq!(doc.ray_iteration_images(parent + 3).unwrap().len(), 1);
    assert_eq!(doc.line_iteration_images(parent + 4).unwrap().len(), 1);
    assert_eq!(doc.arc_iteration_images(parent + 5, 8).unwrap().len(), 1);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), edited);
    doc.redo();
    doc.verify_roundtrip().unwrap();
}

#[test]
fn undefined_circle_generation_does_not_hide_other_bindings_or_valid_generations() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 80., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 120., y: 100. }));
    let fixed = doc.add(Kind::Free(Point { x: 140., y: 100. }));
    let seed_host = doc.add(Kind::Segment(a, b));
    let seed = doc.add(Kind::SegmentMidpoint(seed_host));
    let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 1., 20.));
    let circle = doc.add(Kind::Circle(seed, fixed));
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![image]],
            &[image, circle],
            &[circle, image],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    assert_eq!(
        doc.circle_iteration_images(parent + 1),
        Some(vec![
            (Point { x: 120., y: 100. }, 20.),
            (Point { x: 160., y: 100. }, 20.),
        ])
    );
    assert_eq!(
        doc.mapped_point_binding_images(parent + 2),
        Some(vec![
            Point { x: 140., y: 100. },
            Point { x: 160., y: 100. },
            Point { x: 180., y: 100. },
        ])
    );
    doc.verify_roundtrip().unwrap();
}

#[test]
fn omitted_circle_host_keeps_original_value_while_carried_host_recomputes() {
    for carried_host in [false, true] {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 80., y: 100. }));
        let b = doc.add(Kind::Free(Point { x: 120., y: 100. }));
        let seed_host = doc.add(Kind::Segment(a, b));
        let seed = doc.add(Kind::SegmentMidpoint(seed_host));
        let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 1., 20.));
        let host_segment = doc.add(Kind::Segment(seed, b));
        let host = doc.add(Kind::SegmentMidpoint(host_segment));
        let circle = doc.add(Kind::Circle(host, image));
        let mut carried = vec![image];
        if carried_host {
            carried.extend([host_segment, host]);
        }
        carried.push(circle);
        let parent = doc
            .add_mapped_point_iteration_with_bindings(
                &[seed],
                &[vec![image]],
                &carried,
                &[circle],
                IterationDepth::Fixed(3),
            )
            .unwrap();
        let images = doc.circle_iteration_images(parent + 1).unwrap();
        assert_eq!(images.len(), 3);
        assert_eq!(
            images[2],
            if carried_host {
                (Point { x: 140., y: 100. }, 40.)
            } else {
                (Point { x: 110., y: 100. }, 70.)
            }
        );
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_original_generic_binding_families_preserve_bytes_and_resolve_their_source_geometry() {
    let mut errors = Vec::new();
    for (name, bytes, binding) in [
        (
            "circle",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_original_circle_rolling_native_verified.gsp"
            )
            .as_slice(),
            9,
        ),
        (
            "circle_segment",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_original_circle_segment_native_verified.gsp"
            )
            .as_slice(),
            13,
        ),
        (
            "polygon",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_original_polygon_scalar_native_verified.gsp"
            )
            .as_slice(),
            22,
        ),
        (
            "arc",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_original_arc_spiral_native_verified.gsp"
            )
            .as_slice(),
            25,
        ),
        (
            "ray",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_original_ray_axis_native_verified.gsp"
            )
            .as_slice(),
            16,
        ),
        (
            "line",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_original_line_reflection_native_verified.gsp"
            )
            .as_slice(),
            27,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(
            matches!(doc.objects[binding - 1].kind, Kind::IterationBinding(..)),
            "{name}: {:?}", doc.objects[binding - 1].kind
        );
        let count = match name {
            "circle" | "circle_segment" => doc.circle_iteration_images(binding).unwrap_or_else(|| panic!("undefined {name} binding{binding}")).len(),
            "polygon" => doc.polygon_iteration_images(binding).unwrap().len(),
            "arc" => doc.arc_iteration_images(binding, 24).unwrap().len(),
            "ray" => doc.ray_iteration_images(binding).unwrap().len(),
            "line" => doc.line_iteration_images(binding).unwrap().len(),
            _ => unreachable!(),
        };
        assert!(count > 0, "{name}");
        assert_eq!(doc.encode().unwrap().as_slice(), bytes, "{name}");
        if let Err(error) = doc.verify_roundtrip() { errors.push(format!("{name}: {error}")); }

    }
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn numeric_depth_shape_binding_styles_survive_independent_source_and_binding_edits() {
    let (mut doc, parent, sources) = broad_binding_graph(true);
    for (index, source) in sources.iter().enumerate() {
        assert_eq!(
            doc.objects[parent + index].visual_style,
            doc.objects[source - 1].visual_style
        );
        assert!(doc.objects[parent + index].visual_style.is_some());
    }
    assert!(matches!(
        doc.objects[parent + 1].visual_style,
        Some(VisualStyle::Polygon { .. })
    ));
    for binding in [parent + 1, parent + 3, parent + 4, parent + 5] {
        assert!(matches!(
            doc.objects[binding - 1].visual_style,
            Some(VisualStyle::Stroke { .. })
        ));
    }
    let initial = doc.encode().unwrap();
    let inherited = doc.objects[parent + 1].visual_style;
    doc.selected = [sources[1]].into_iter().collect();
    assert_eq!(
        doc.set_selected_color(RgbColor {
            red: 20,
            green: 100,
            blue: 180
        })
        .unwrap(),
        1
    );
    assert_eq!(doc.objects[parent + 1].visual_style, inherited);
    assert_ne!(doc.objects[sources[1] - 1].visual_style, inherited);
    let source_edited = doc.encode().unwrap();
    doc.selected = [parent + 2, parent + 5].into_iter().collect();
    assert_eq!(
        doc.set_selected_color(RgbColor {
            red: 200,
            green: 60,
            blue: 30
        })
        .unwrap(),
        2
    );
    doc.selected = [parent + 5].into_iter().collect();
    assert_eq!(doc.set_selected_line_width(LineWidth::Thick).unwrap(), 1);
    doc.verify_roundtrip().unwrap();
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    for id in sources.into_iter().chain(parent + 1..=parent + 5) {
        assert_eq!(
            reopened.objects[id - 1].visual_style,
            doc.objects[id - 1].visual_style
        );
    }
    doc.undo();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), source_edited);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), initial);
}

#[test]
fn generic_binding_import_rejects_noncarried_sources_and_noniteration_parents_losslessly() {
    let (doc, parent, _) = broad_binding_graph(false);
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    let at = reopened.objects[parent].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let start = rs
        .iter()
        .position(|r| r.kind == 0x07d0 && r.data.start == at)
        .unwrap();
    let refs = rs[start..].iter().find(|r| r.kind == 0x07d2).unwrap();
    for (offset, reference) in [(4, 1usize), (8, 1usize)] {
        let mut malformed = bytes.clone();
        let position = refs.data.start + offset;
        malformed[position..position + 4].copy_from_slice(&(reference as u32).to_le_bytes());
        let imported = Document::from_bytes(&malformed, None).unwrap();
        assert!(matches!(
            imported.objects[parent].kind,
            Kind::Unsupported(77)
        ));
        assert_eq!(imported.encode().unwrap(), malformed);
    }
}

fn native_binding_control(state: &str) -> (Document, usize) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = doc.add(Kind::Free(if state.contains("edited") {
        Point { x: 180., y: 200. }
    } else {
        Point { x: 160., y: 180. }
    }));
    let c = doc.add(Kind::Free(Point { x: 100., y: 240. }));
    let origin = doc.add(Kind::Free(Point { x: 80., y: 80. }));
    let target = doc.add(Kind::Free(Point { x: 200., y: 100. }));
    let images: Vec<_> = [a, b, c]
        .into_iter()
        .map(|seed| {
            let scaled = doc.add(Kind::ScaledPoint(seed, origin, 0.75));
            doc.add(Kind::VectorTranslatedPoint(scaled, origin, target))
        })
        .collect();
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let circle_segment = doc.add(Kind::CircleBySegment(a, segment));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let ray = doc.add(Kind::Ray(a, b));
    let line = doc.add(Kind::Line(a, c));
    let arc = doc.add(Kind::CenterArc(a, b, c));
    let mut frozen = None;
    let extra = match state {
        "transformed_circle" => Some(doc.add(Kind::ScaledCircle(circle, origin, 0.5))),
        "transformed_polygon" => Some(doc.add(Kind::ScaledPolygon(polygon, origin, 0.5))),
        "frozen_circle" => {
            let center = doc.add(Kind::SegmentMidpoint(segment));
            frozen = Some(center);
            Some(doc.add(Kind::Circle(center, b)))
        }
        _ => None,
    };
    let source = extra.unwrap_or_else(|| match state {
        "circle_segment" => circle_segment,
        "polygon" => polygon,
        "ray" => ray,
        "line" => line,
        "arc" | "arc_final" => arc,
        _ => circle,
    });
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[a, b, c],
            &[images],
            &(4..=doc.objects.len())
                .filter(|id| Some(*id) != frozen)
                .collect::<Vec<_>>(),
            &[source],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    if state.contains("final") {
        doc.set_mapped_iteration_final_only(parent, true).unwrap();
    }
    for object in doc.objects.iter_mut().take(parent - 1) {
        object.visible = false;
    }
    (doc, parent)
}

#[test]
fn fresh_generic_shape_apis_reconstruct_independently_verified_native_controls() {
    for (state, bytes) in [
        (
            "circle",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_circle_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "circle_segment",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_circle_segment_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "polygon",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_polygon_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "ray",
            include_bytes!("../../../../../tests/fixtures/iteration_binding47_ray_native_verified.gsp")
                .as_slice(),
        ),
        (
            "line",
            include_bytes!("../../../../../tests/fixtures/iteration_binding47_line_native_verified.gsp")
                .as_slice(),
        ),
        (
            "arc",
            include_bytes!("../../../../../tests/fixtures/iteration_binding47_arc_native_verified.gsp")
                .as_slice(),
        ),
        (
            "circle_edited",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_circle_edited_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "circle_final",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_circle_final_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "arc_final",
            include_bytes!(
                "../../../../../tests/fixtures/iteration_binding47_arc_final_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let (doc, parent) = native_binding_control(state);
        assert_eq!(doc.encode().unwrap().as_slice(), bytes, "{state}");
        doc.verify_roundtrip().unwrap();
        if state == "circle" || state == "circle_segment" {
            assert_eq!(
                doc.circle_iteration_images(parent + 1),
                Some(vec![
                    (Point { x: 215., y: 175. }, 45.),
                    (
                        Point {
                            x: 301.25,
                            y: 171.25
                        },
                        33.75
                    ),
                    (
                        Point {
                            x: 365.9375,
                            y: 168.4375
                        },
                        25.3125
                    ),
                ])
            );
        }
    }
}

#[test]
fn derived_seed_and_numeric_depth_edits_update_every_generic_shape_binding() {
    let (mut doc, parent, _) = broad_binding_graph(true);
    let Kind::MappedPointIteration {
        depth: IterationDepth::Value(depth),
        ..
    } = doc.objects[parent - 1].kind
    else {
        panic!("numeric mapped root");
    };
    let before = doc.encode().unwrap();
    doc.set_parameter_value(depth, 5.).unwrap();
    assert_eq!(doc.circle_iteration_images(parent + 1).unwrap().len(), 5);
    assert_eq!(doc.polygon_iteration_images(parent + 2).unwrap().len(), 5);
    assert_eq!(doc.ray_iteration_images(parent + 3).unwrap().len(), 5);
    assert_eq!(doc.line_iteration_images(parent + 4).unwrap().len(), 5);
    assert_eq!(doc.arc_iteration_images(parent + 5, 12).unwrap().len(), 5);
    doc.checkpoint();
    doc.move_point(1, Point { x: 80., y: 120. });
    assert_eq!(
        doc.circle_iteration_images(parent + 1).unwrap()[4],
        (Point { x: 200., y: 110. }, 20.)
    );
    assert_eq!(
        doc.polygon_iteration_images(parent + 2).unwrap()[4].0[0],
        Point { x: 200., y: 110. }
    );
    assert_eq!(
        doc.ray_iteration_images(parent + 3).unwrap()[4].0,
        Point { x: 200., y: 110. }
    );
    assert_eq!(
        doc.line_iteration_images(parent + 4).unwrap()[4].0,
        Point { x: 200., y: 110. }
    );
    assert!(
        doc.arc_iteration_images(parent + 5, 12).unwrap()[4][0]
            .distance(Point { x: 220., y: 110. })
            < 1e-10
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
}

fn native_derived_binding_control(state: &str) -> (Document, usize) {
    let mut d = Document::new();
    let l = d.add(Kind::Free(Point {
        x: if state.contains("edited") { 100. } else { 80. },
        y: 180.,
    }));
    let r = d.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = d.add(Kind::Free(Point { x: 160., y: 180. }));
    let c = d.add(Kind::Free(Point { x: 100., y: 240. }));
    let host = d.add(Kind::Segment(l, r));
    let a = d.add(Kind::SegmentMidpoint(host));
    let origin = d.add(Kind::Free(Point { x: 80., y: 80. }));
    let target = d.add(Kind::Free(Point { x: 200., y: 100. }));
    let mut images = vec![];
    for seed in [a, b, c] {
        let scaled = d.add(Kind::ScaledPoint(seed, origin, 0.75));
        images.push(d.add(Kind::VectorTranslatedPoint(scaled, origin, target)));
    }
    let segment = d.add(Kind::Segment(a, b));
    let circle = d.add(Kind::Circle(a, b));
    let circle_segment = d.add(Kind::CircleBySegment(a, segment));
    let polygon = d.add(Kind::Polygon(vec![a, b, c]));
    let ray = d.add(Kind::Ray(a, b));
    let line = d.add(Kind::Line(a, c));
    let arc = d.add(Kind::CenterArc(a, b, c));
    let bindings = if state.contains("multi") {
        vec![circle, circle_segment, polygon, ray, line, arc]
    } else {
        vec![circle]
    };
    let parent = d
        .add_mapped_point_iteration_with_bindings(
            &[a, b, c],
            &[images],
            &(origin..=arc).collect::<Vec<_>>(),
            &bindings,
            IterationDepth::Fixed(3),
        )
        .unwrap();
    for o in d.objects.iter_mut().take(parent - 1) {
        o.visible = false;
    }
    (d, parent)
}

#[test]
fn native_derived_seed_controls_verify_shared_shape_outputs_and_live_host_edits() {
    for (state, bytes) in [
        ("derived_circle", include_bytes!("../../../../../tests/fixtures/iteration_binding47_derived_circle_native_verified.gsp").as_slice()),
        ("derived_circle_edited", include_bytes!("../../../../../tests/fixtures/iteration_binding47_derived_circle_edited_native_verified.gsp").as_slice()),
        ("derived_multi", include_bytes!("../../../../../tests/fixtures/iteration_binding47_derived_multi_native_verified.gsp").as_slice()),
    ] {
        let (doc, parent) = native_derived_binding_control(state);
        assert_eq!(doc.encode().unwrap().as_slice(), bytes, "{state}");
        doc.verify_roundtrip().unwrap();
        assert!(matches!(doc.objects[5].kind, Kind::Midpoint(1, 2)));
        assert_eq!(doc.objects[5].midpoint_host, Some(5));
        let circles = doc.circle_iteration_images(parent + 1).unwrap();
        assert_eq!(circles.len(), 3);
        assert_eq!(circles[2], if state.contains("edited") {
            (Point { x: 370.15625, y: 168.4375 }, 21.09375)
        } else {
            (Point { x: 365.9375, y: 168.4375 }, 25.3125)
        });
        if state == "derived_multi" {
            assert_eq!(doc.circle_iteration_images(parent + 2), Some(circles));
            assert_eq!(doc.polygon_iteration_images(parent + 3).unwrap().len(), 3);
            assert_eq!(doc.ray_iteration_images(parent + 4).unwrap().len(), 3);
            assert_eq!(doc.line_iteration_images(parent + 5).unwrap().len(), 3);
            assert_eq!(doc.arc_iteration_images(parent + 6, 24).unwrap().len(), 3);
        }
    }
    let (mut doc, parent) = native_derived_binding_control("derived_circle");
    doc.checkpoint();
    doc.move_point(1, Point { x: 100., y: 180. });
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/iteration_binding47_derived_circle_edited_native_verified.gsp"
        )
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/iteration_binding47_derived_circle_native_verified.gsp"
        )
    );
    assert_eq!(
        doc.circle_iteration_images(parent + 1).unwrap()[0],
        (Point { x: 215., y: 175. }, 45.)
    );
}

#[test]
fn native_transformed_bindings_preserve_transform_definitions_and_omitted_host_values() {
    for (state, bytes) in [
        ("transformed_circle", include_bytes!("../../../../../tests/fixtures/iteration_binding47_transformed_circle_native_verified.gsp").as_slice()),
        ("transformed_polygon", include_bytes!("../../../../../tests/fixtures/iteration_binding47_transformed_polygon_native_verified.gsp").as_slice()),
        ("frozen_circle", include_bytes!("../../../../../tests/fixtures/iteration_binding47_frozen_circle_native_verified.gsp").as_slice()),
    ] {
        let (doc, parent) = native_binding_control(state);
        assert_eq!(doc.encode().unwrap().as_slice(), bytes, "{state}");
        doc.verify_roundtrip().unwrap();
        if state == "transformed_polygon" {
            assert!(doc.objects[18].kind.transform_definition().is_some());
            let polygons = doc.polygon_iteration_images(parent + 1).unwrap();
            assert_eq!(polygons[0].0[0], Point { x: 147.5, y: 127.5 });
            assert_eq!(polygons[2].0[0], Point { x: 222.96875, y: 124.21875 });
        } else {
            let circles = doc.circle_iteration_images(parent + 1).unwrap();
            if state == "transformed_circle" {
                assert!(doc.objects[18].kind.transform_definition().is_some());
                assert_eq!(circles[0], (Point { x: 147.5, y: 127.5 }, 22.5));
                assert_eq!(circles[2], (Point { x: 222.96875, y: 124.21875 }, 12.65625));
            } else {
                assert!(matches!(doc.objects[18].kind, Kind::Midpoint(1, 2)));
                assert!(circles.iter().all(|(center, _)| *center == Point { x: 130., y: 180. }));
                assert!((circles[2].1 - 261.25f64.hypot(11.5625)).abs() < 1e-10);
            }
        }
    }
}

#[test]
fn native_undefined_numeric_ratio_hides_only_middle_circle_generation() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/iteration_binding47_undefined_circle_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(
        doc.objects[23].kind,
        Kind::IterationBinding(22, 23)
    ));
    assert!(doc.objects[21].kind.transform_definition().is_some());
    let circles = doc.circle_iteration_images(24).unwrap();
    assert_eq!(
        circles,
        vec![
            (Point { x: 215., y: 175. }, 45.),
            (
                Point {
                    x: 365.9375,
                    y: 168.4375
                },
                25.3125
            ),
        ]
    );
    assert_eq!(doc.unitless_numeric_value(19), Some(0.));
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
    doc.verify_roundtrip().unwrap();
}

#[test]
fn typed_length_circle_radius_updates_geometry_downstream_points_and_iteration_images() {
    let original = Document::from_bytes(include_bytes!("../../../../../tests/fixtures/iteration_binding47_original_circle_segment_native_verified.gsp"), None).unwrap();
    assert_eq!(
        original.objects[2].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    let mut doc = Document::new();
    let center = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let parameter = doc.add(original.objects[2].kind.clone());
    let circle = doc.add(Kind::CircleBySegment(center, parameter));
    let point = doc.add(Kind::PointOnCircle(circle, 1., 0.));
    let image = doc.add(Kind::PolarTranslatedPoint(center, 0., 1., 20.));
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[center],
            &[vec![image]],
            &[circle, image],
            &[circle],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    let radius = 37.79527559055118;
    assert_eq!(
        doc.circle_points(circle),
        Some((Point { x: 100., y: 180. }, radius))
    );
    assert!(
        doc.position(point).unwrap().distance(Point {
            x: 100. + radius,
            y: 180.
        }) < 1e-10
    );
    assert_eq!(
        doc.circle_iteration_images(parent + 1).unwrap()[2],
        (Point { x: 160., y: 180. }, radius)
    );
    let before = doc.encode().unwrap();
    doc.set_parameter_value(parameter, 2.).unwrap();
    assert_eq!(
        doc.circle_points(circle),
        Some((Point { x: 100., y: 180. }, radius * 2.))
    );
    assert!(
        doc.position(point).unwrap().distance(Point {
            x: 100. + radius * 2.,
            y: 180.
        }) < 1e-10
    );
    assert!(
        doc.circle_iteration_images(parent + 1)
            .unwrap()
            .iter()
            .all(|(_, r)| *r == radius * 2.)
    );
    assert!(matches!(
        doc.objects[circle - 1].kind,
        Kind::CircleBySegment(1, 2)
    ));
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
    assert_eq!(doc.circle_points(circle).unwrap().1, radius);
}
