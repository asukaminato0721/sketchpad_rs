use super::*;

fn close_point(actual: Option<Point>, expected: Point) {
    let actual = actual.unwrap();
    assert!(actual.distance(expected) < 1e-9, "{actual:?} != {expected:?}");
}

#[test]
fn independently_native_constrained_arc_and_measurements_follow_linear_and_undefined_states() {
    for (bytes, defined) in [
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_linear_native_verified.gsp").as_slice(), true),
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_outside_native_verified.gsp").as_slice(), false),
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_repeated_native_verified.gsp").as_slice(), false),
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_outside_low_native_verified.gsp").as_slice(), false),
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_middle_end_native_verified.gsp").as_slice(), false),
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_endpoints_equal_native_verified.gsp").as_slice(), false),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(doc.objects.iter().all(|object| !matches!(object.kind, Kind::Unsupported(_))));
        if defined {
            close_point(doc.position(5), Point { x: 245., y: 180. });
            assert_eq!(doc.arc_angle_degrees(9), Some(0.));
            assert!((doc.arc_length_cm(8).unwrap() - 60. / 37.79527559055118).abs() < 1e-12);
            assert!(doc.line_points(7).is_some());
        } else {
            assert_eq!(doc.position(5), None);
            assert_eq!(doc.line_points(7), None);
            assert_eq!(doc.arc_angle_degrees(9), None);
            assert_eq!(doc.arc_length_cm(8), None);
        }
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.selected = (1..=9).collect();
        let clipboard = doc.copy_selected().unwrap();
        assert_eq!(doc.paste(&clipboard).unwrap().len(), 9);
        assert_eq!(doc.position(14).is_some(), defined);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn constrained_arc_writer_labels_source_edits_and_drag_match_native_controls() {
    let circular = include_bytes!("../../../tests/fixtures/arc_constrained_measure_circular_native_verified.gsp");
    let linear = include_bytes!("../../../tests/fixtures/arc_constrained_measure_linear_native_verified.gsp");
    let outside = include_bytes!("../../../tests/fixtures/arc_constrained_measure_outside_native_verified.gsp");
    let native_dragged = include_bytes!("../../../tests/fixtures/arc_constrained_measure_dragged_native.gsp");
    let dragged = Document::from_bytes(native_dragged, None).unwrap();
    assert_eq!(dragged.objects[4].kind, Kind::PointOnArc(4, 0.75));
    close_point(dragged.position(5), Point { x: 215., y: 180. });
    assert_eq!(dragged.encode().unwrap(), native_dragged);
    for (bytes, t, position) in [
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_drag_start_native.gsp").as_slice(), 0., Point { x: 260., y: 180. }),
        (include_bytes!("../../../tests/fixtures/arc_constrained_measure_drag_end_native.gsp").as_slice(), 1., Point { x: 200., y: 180. }),
    ] {
        let endpoint = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(endpoint.objects[4].kind, Kind::PointOnArc(4, t));
        close_point(endpoint.position(5), position);
        assert_eq!(endpoint.encode().unwrap(), bytes);
    }
    let mut doc = Document::new();
    for point in [Point { x: 260., y: 180. }, Point { x: 242.42640687119285, y: 137.57359312880715 }, Point { x: 200., y: 120. }] { doc.add(Kind::Free(point)); }
    doc.add(Kind::ThreePointArc(1, 2, 3));
    let kind = doc.constrained_point(4, Point { x: 255.4327719506772, y: 157.0389940580946 }).unwrap();
    doc.add(kind);
    doc.set_point_label(5, "ConstrainedQuarter", true).unwrap();
    doc.add(Kind::Free(Point { x: 170., y: 230. }));
    doc.add(Kind::Segment(6, 5));
    doc.add(Kind::ArcLength(4));
    doc.add(Kind::ArcAngle(4));
    assert_eq!(doc.encode().unwrap(), circular);
    let mut imported = Document::from_bytes(include_bytes!("../../../tests/fixtures/arc_constrained_measure_unlabeled_native_verified.gsp"), None).unwrap();
    imported.set_point_label(5, "ConstrainedQuarter", true).unwrap();
    assert_eq!(imported.encode().unwrap(), circular);
    doc.checkpoint();
    assert!(doc.move_point(2, Point { x: 240., y: 180. }));
    assert!(doc.move_point(3, Point { x: 200., y: 180. }));
    assert_eq!(doc.encode().unwrap(), linear);
    assert_eq!(doc.constrained_point(4, Point { x: 280., y: 190. }), Some(Kind::PointOnArc(4, 0.)));
    assert_eq!(doc.constrained_point(4, Point { x: 175., y: 190. }), Some(Kind::PointOnArc(4, 1.)));
    doc.checkpoint();
    assert!(doc.move_point(2, Point { x: 290., y: 180. }));
    assert_eq!(doc.encode().unwrap(), outside);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), linear);
    doc.redo();
    assert_eq!(doc.position(5), None);
    assert!(doc.move_point(2, Point { x: 240., y: 180. }));
    assert!(doc.move_point(5, Point { x: 215., y: 175. }));
    assert_eq!(doc.objects[4].kind, Kind::PointOnArc(4, 0.75));
    close_point(doc.position(5), Point { x: 215., y: 180. });
    assert_eq!(doc.encode().unwrap(), include_bytes!("../../../tests/fixtures/arc_constrained_measure_authored_drag_native_verified.gsp"));
    doc.verify_roundtrip().unwrap();
    let mut copied = doc.clone();
    copied.selected = (1..=9).collect();
    let clipboard = copied.copy_selected().unwrap();
    copied.paste(&clipboard).unwrap();
    assert_eq!(copied.encode().unwrap(), include_bytes!("../../../tests/fixtures/arc_constrained_measure_authored_copy_native_verified.gsp"));
    let mut undefined_copy = Document::from_bytes(outside, None).unwrap();
    undefined_copy.selected = (1..=9).collect();
    let clipboard = undefined_copy.copy_selected().unwrap();
    undefined_copy.paste(&clipboard).unwrap();
    assert_eq!(undefined_copy.encode().unwrap(), include_bytes!("../../../tests/fixtures/arc_constrained_measure_authored_undefined_copy_native_verified.gsp"));
    if let Ok(directory) = std::env::var("SKETCHPAD_ARC_CONSTRAINT_GOLDEN_DIR") {
        fs::write(Path::new(&directory).join("arc_constraint_drag_authored.gsp"), doc.encode().unwrap()).unwrap();
        doc.selected = (1..=9).collect();
        let clipboard = doc.copy_selected().unwrap();
        doc.paste(&clipboard).unwrap();
        fs::write(Path::new(&directory).join("arc_constraint_copied_authored.gsp"), doc.encode().unwrap()).unwrap();
        let mut undefined = Document::from_bytes(outside, None).unwrap();
        undefined.selected = (1..=9).collect();
        let clipboard = undefined.copy_selected().unwrap();
        undefined.paste(&clipboard).unwrap();
        fs::write(Path::new(&directory).join("arc_constraint_undefined_copied_authored.gsp"), undefined.encode().unwrap()).unwrap();
    }
}

#[test]
fn native_known_arc_plots_use_normalized_sweep_and_preserve_undefined_domain() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/arc_circle_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_uneven_middle_plot_native_verified.gsp").as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects.len(), 21);
        assert!(doc.objects.iter().all(|object| !matches!(object.kind, Kind::Unsupported(_))));
        close_point(doc.position(10), Point { x: 200. + 60. * (std::f64::consts::PI / 8.).cos(), y: 180. - 60. * (std::f64::consts::PI / 8.).sin() });
        close_point(doc.position(16), Point { x: 200. + 60. / 2_f64.sqrt(), y: 180. - 60. / 2_f64.sqrt() });
        for point in [12, 14, 18, 20] {
            assert_eq!(doc.position(point), None);
            assert!(doc.objects[point - 1].visible);
            assert_eq!(doc.line_points(point + 1), None);
        }
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
        doc.set_parameter_value(8, 0.75).unwrap();
        assert!(doc.position(12).is_some() && doc.line_points(13).is_some());
        doc.undo();
        assert_eq!(doc.position(12), None);
        doc.redo();
        assert!(doc.position(12).is_some());
        let encoded = doc.encode().unwrap();
        assert_eq!(Document::from_bytes(&encoded, None).unwrap().position(12), doc.position(12));
    }
    for bytes in [
        include_bytes!("../../../tests/fixtures/arc_circle_endpoint_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_endpoint_plot_native_verified.gsp").as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        for point in [10, 16] { close_point(doc.position(point), Point { x: 260., y: 180. }); }
        for point in [12, 18] { close_point(doc.position(point), Point { x: 200., y: 120. }); }
        for point in [14, 20] { assert_eq!(doc.position(point), None); }
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_collapsed_arc_plots_are_undefined_and_dependencies_remain_guarded() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/arc_circle_collapsed_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_circle_repeated_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_repeated_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_outside_middle_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_outside_end_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_repeated_coordinates_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_repeated_end_coordinates_plot_native_verified.gsp").as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        for point in [10, 12, 14, 16, 18, 20] {
            assert_eq!(doc.position(point), None);
            assert_eq!(doc.line_points(point + 1), None);
        }
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.selected = (1..=21).collect();
        let copied = doc.copy_selected().unwrap();
        assert_eq!(doc.paste(&copied).unwrap().len(), 21);
        doc.verify_roundtrip().unwrap();
    }
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 120., y: 100. }));
    let c = doc.add(Kind::Free(Point { x: 140., y: 100. + 1e-12 }));
    let nearly = doc.add(Kind::ThreePointArc(a, b, c));
    assert!(!doc.undefined_arc_plot(nearly, 0.5));
    let before = (doc.objects.len(), doc.undo.len());
    assert!(doc.plot_literal_on_path(0.5, Some(nearly)).is_err());
    assert!(doc.plot_literal_on_path(f64::NAN, Some(nearly)).is_err());
    assert_eq!((doc.objects.len(), doc.undo.len()), before);
    doc.objects[a - 1].deleted = true;
    assert!(!doc.undefined_arc_plot(nearly, -0.25));
    assert!(doc.plot_literal_on_path(-0.25, Some(nearly)).is_err());
    let mut cyclic = Document::new();
    cyclic.add(Kind::PathLiteralPoint(4, 0.5));
    cyclic.add(Kind::Free(Point { x: 120., y: 100. }));
    cyclic.add(Kind::Free(Point { x: 140., y: 120. }));
    cyclic.add(Kind::ThreePointArc(1, 2, 3));
    assert!(!cyclic.plot_arc_host_available(4));
    let before = (cyclic.objects.len(), cyclic.undo.len());
    assert!(cyclic.plot_literal_on_path(0.5, Some(4)).is_err());
    assert_eq!((cyclic.objects.len(), cyclic.undo.len()), before);
}

fn authored_arcs() -> Document {
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 150., y: 160. }));
    doc.add(Kind::Free(Point { x: 210., y: 160. }));
    doc.add(Kind::Circle(1, 2));
    doc.add(Kind::PointOnCircle(3, 1., 0.));
    doc.add(Kind::PointOnCircle(3, 0., -1.));
    doc.add(Kind::ArcOnCircle(3, 4, 5));
    doc.selected = [3].into_iter().collect();
    doc.set_selected_visibility(false);
    doc.add(Kind::Free(Point { x: 130., y: 240. }));
    doc.add(Kind::Free(Point { x: 90. + 40. / 2_f64.sqrt(), y: 240. - 40. / 2_f64.sqrt() }));
    doc.add(Kind::Free(Point { x: 90., y: 200. }));
    doc.add(Kind::ThreePointArc(7, 8, 9));
    for (name, value, y) in [("a", 0.25, 10), ("b", -0.25, 35), ("c", 1.25, 60)] {
        doc.add_named_parameter(name, value, (10, y)).unwrap();
    }
    for (path, anchor, prefix) in [(6, 1, "Circle"), (10, 9, "Three")] {
        for (source, suffix) in [(11, "Quarter"), (12, "Negative"), (13, "Over")] {
            let point = doc.plot_parameter_on_path(source, Some(path)).unwrap();
            doc.set_point_label(point, &format!("{prefix}Value{suffix}"), true).unwrap();
            doc.add(Kind::Segment(anchor, point));
        }
        for (value, suffix) in [(0.5, "Half"), (-0.5, "Negative"), (1.5, "Over")] {
            let point = doc.plot_literal_on_path(value, Some(path)).unwrap();
            doc.set_point_label(point, &format!("{prefix}Literal{suffix}"), true).unwrap();
            doc.add(Kind::Segment(anchor, point));
        }
    }
    doc
}

#[test]
fn authored_arc_plots_follow_source_edits_copy_and_collapsed_recovery() {
    let mut doc = authored_arcs();
    assert_eq!(doc.objects.len(), 37);
    let generated = doc.encode().unwrap();
    assert_eq!(generated, include_bytes!("../../../tests/fixtures/arc_plots_authored_generated_native_verified.gsp"));
    let before = (doc.objects.len(), doc.undo.len(), generated.clone());
    assert!(doc.plot_literal_on_path(f64::INFINITY, Some(6)).is_err());
    assert!(doc.plot_parameter_on_path(1, Some(10)).is_err());
    assert_eq!((doc.objects.len(), doc.undo.len(), doc.encode().unwrap()), before);
    doc.checkpoint();
    assert!(doc.move_point(2, Point { x: 230., y: 160. }));
    assert!(doc.move_point(8, Point { x: 118., y: 220. }));
    doc.set_parameter_value(11, 0.75).unwrap();
    doc.set_parameter_value(12, 0.25).unwrap();
    for point in [14, 16, 20, 26, 28, 32] { assert!(doc.position(point).is_some()); }
    for point in [18, 22, 24, 30, 34, 36] { assert_eq!(doc.position(point), None); }
    let edited = doc.encode().unwrap();
    assert_eq!(edited, include_bytes!("../../../tests/fixtures/arc_plots_authored_edited_native_verified.gsp"));
    let mut copy = doc.clone();
    copy.selected = (1..=37).collect();
    let clipboard = copy.copy_selected().unwrap();
    let pasted = copy.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 37);
    for old in [14, 16, 20, 26, 28, 32] {
        let point = doc.position(old).unwrap();
        close_point(copy.position(old + 37), Point { x: point.x + 20., y: point.y + 20. });
    }
    for old in [18, 22, 24, 30, 34, 36] { assert_eq!(copy.position(old + 37), None); }
    let copied = copy.encode().unwrap();
    assert_eq!(copied, include_bytes!("../../../tests/fixtures/arc_plots_authored_copied_native_verified.gsp"));
    copy.verify_roundtrip().unwrap();
    doc.checkpoint();
    assert!(doc.move_point(5, doc.position(4).unwrap()));
    doc.checkpoint();
    assert!(doc.move_point(8, Point { x: 110., y: 220. }));
    for point in [14, 16, 18, 20, 22, 24, 30, 34, 36] {
        assert_eq!(doc.position(point), None);
        assert_eq!(doc.line_points(point + 1), None);
    }
    for (point, expected) in [(26, Point { x: 100., y: 210. }),
        (28, Point { x: 120., y: 230. }), (32, Point { x: 110., y: 220. })] {
        close_point(doc.position(point), expected);
        assert!(doc.line_points(point + 1).is_some());
    }
    let collapsed = doc.encode().unwrap();
    assert_eq!(collapsed, include_bytes!("../../../tests/fixtures/arc_plots_authored_collapsed_native_verified.gsp"));
    doc.undo();
    assert!(doc.position(26).is_some());
    assert_eq!(doc.position(14), None);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), collapsed);
    doc.verify_roundtrip().unwrap();
    if let Ok(directory) = std::env::var("SKETCHPAD_ARC_GOLDEN_DIR") {
        for (stage, bytes) in [("generated", generated), ("edited", edited), ("copied", copied), ("collapsed", collapsed)] {
            fs::write(Path::new(&directory).join(format!("arc_plots_authored_{stage}.gsp")), bytes).unwrap();
        }
    }
}

#[test]
fn graph_plot_eligibility_matches_api_families_and_scalar_categories() {
    let mut doc = authored_arcs();
    for id in [3, 6, 10] { assert!(doc.can_plot_value_on_path(id)); }
    for id in [1, 11, 14, 0, 999] { assert!(!doc.can_plot_value_on_path(id)); }
    let polygon = doc.add(Kind::Polygon(vec![1, 2, 7]));
    let ray = doc.add(Kind::Ray(1, 2));
    let line = doc.add(Kind::Line(1, 2));
    let rotated_ray = doc.add(Kind::RotatedRay(ray, 1, 45.));
    let scaled_line = doc.add(Kind::ScaledLine(line, 1, 2.));
    for id in [polygon, ray, line, rotated_ray, scaled_line] {
        assert!(doc.can_plot_value_on_path(id));
        doc.plot_literal_on_path(0.5, Some(id)).unwrap();
    }
    let source = doc.add_parameter_calculation(11, "a*2", (10, 90)).unwrap();
    assert!(doc.is_scalar_plot_source(source));
    doc.plot_parameter_on_path(source, Some(6)).unwrap();
    let angle = doc.add_parameter_calculation(11, "360°/a", (10, 115)).unwrap();
    assert!(!doc.is_scalar_plot_source(angle));
    for id in [11, 12, 13] { assert!(doc.is_scalar_plot_source(id)); }
    for id in [0, 1, 6, 999] { assert!(!doc.is_scalar_plot_source(id)); }
    doc.objects[source - 1].page = 1;
    doc.objects[rotated_ray - 1].deleted = true;
    assert!(!doc.is_scalar_plot_source(source));
    assert!(!doc.can_plot_value_on_path(rotated_ray));
    let unknown80 = doc.add(Kind::Unsupported(80));
    assert!(!doc.can_plot_value_on_path(unknown80));
}

#[test]
fn native_distinct_collinear_arc_plots_and_host_use_linear_fallback() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/arc_three_point_collapsed_plot_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/arc_three_point_uneven_linear_middle_plot_native_verified.gsp").as_slice(),
    ] {
    let doc = Document::from_bytes(bytes, None).unwrap();
    close_point(doc.position(10), Point { x: 245., y: 180. });
    close_point(doc.position(16), Point { x: 230., y: 180. });
    assert_eq!(doc.linear_three_point_arc(5), Some((Point { x: 260., y: 180. }, Point { x: 200., y: 180. })));
    assert_eq!(doc.arc_points(5, 128), Some(vec![Point { x: 260., y: 180. }, Point { x: 200., y: 180. }]));
    for point in [12, 14, 18, 20] { assert_eq!(doc.position(point), None); }
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_linear_arc_svg_uses_host_stroke_and_zero_circle_arc_is_absent() {
    let viewport = crate::render::Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300., 300.)),
        scale: 1., center: Point { x: 150., y: 150. },
    };
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/arc_three_point_collapsed_plot_native_verified.gsp"), None).unwrap();
    for object in &mut doc.objects { object.visible = object.id == 5; }
    let svg = crate::render::svg_page(&doc, &viewport).unwrap();
    assert!(svg.contains("d=\"M 260 180 L 200 180\""));
    assert_eq!(svg.matches("<path").count(), 1);
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/arc_circle_collapsed_plot_native_verified.gsp"), None).unwrap();
    for object in &mut doc.objects { object.visible = object.id == 6; }
    assert!(doc.arc_points(6, 128).is_none());
    assert!(!crate::render::svg_page(&doc, &viewport).unwrap().contains("<path"));
}
