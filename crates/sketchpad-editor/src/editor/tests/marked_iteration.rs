use super::*;

#[test]
fn scalar_values_can_be_marked_for_point_and_shape_dilation() {
    for family in 0..4 {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_polygon_base_port.gsp"),
            None,
        )
        .unwrap();
        let source = match family {
            0 => 3,
            1 => doc.add(Kind::Segment(3, 4)),
            2 => doc.add(Kind::Circle(3, 4)),
            _ => 6,
        };
        let mut editor = Editor::new(doc);
        editor.doc.selected.insert(1);
        editor.selection_order = vec![1];
        editor.mark_ratio_selected();
        assert_eq!(editor.marked_ratio_value, Some(1));
        assert!(editor.marked_ratio.is_none());
        editor.doc.selected.extend([source, 2]);
        editor.selection_order = vec![source, 2];
        editor.dilate_selected();
        let image = *editor.doc.selected.iter().next().unwrap();
        assert_eq!(
            editor.doc.objects()[image - 1].kind,
            match family {
                0 => Kind::CalculatedRatioScaledPoint(source, 2, 1),
                1 => Kind::CalculatedRatioScaledSegment(source, 2, 1),
                2 => Kind::CalculatedRatioScaledCircle(source, 2, 1),
                _ => Kind::CalculatedRatioScaledPolygon(source, 2, 1),
            }
        );
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        let pick = match family {
            0 => Point { x: 160., y: 180. },
            1 => Point { x: 160., y: 190. },
            2 => Point { x: 140., y: 180. },
            _ => Point { x: 165., y: 195. },
        };
        assert_eq!(editor.hit(&viewport, viewport.screen(pick)), Some(image));
        if family == 0 {
            editor.doc.selected.insert(source);
            editor.selection_order = vec![image, source];
            editor.iterate_selected(3);
            let iteration = *editor.doc.selected.iter().next().unwrap();
            assert_eq!(
                editor.doc.point_iteration_images(iteration).unwrap().len(),
                3
            );
        }
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn selected_shapes_rotate_by_marked_numeric_angles() {
    for family in 0..3 {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/numeric_angle_polygon_port.gsp"),
            None,
        )
        .unwrap();
        let source = match family {
            0 => doc.add(Kind::Segment(6, 7)),
            1 => doc.add(Kind::Circle(6, 7)),
            _ => 9,
        };
        let mut editor = Editor::new(doc);
        editor.selection_order = vec![5];
        editor.doc.selected.insert(5);
        editor.mark_angle_selected();
        editor.selection_order = vec![source, 1];
        editor.doc.selected.extend([source, 1]);
        editor.rotate_selected();
        let transformed = *editor.doc.selected.iter().next().unwrap();
        assert_eq!(
            editor.doc.objects()[transformed - 1].kind,
            match family {
                0 => Kind::CalculatedAngleRotatedSegment(source, 1, 5),
                1 => Kind::CalculatedAngleRotatedCircle(source, 1, 5),
                _ => Kind::CalculatedAngleRotatedPolygon(source, 1, 5),
            }
        );
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        let pick = match family {
            0 => Point { x: 180., y: 120. },
            1 => Point { x: 120., y: 120. },
            _ => Point { x: 180., y: 110. },
        };
        assert_eq!(
            editor.hit(&viewport, viewport.screen(pick)),
            Some(transformed)
        );
        if family == 0 {
            editor.measure_length();
            let measurement = *editor.doc.selected.iter().next().unwrap();
            assert_eq!(
                editor.doc.objects()[measurement - 1].kind,
                Kind::SegmentLength(transformed)
            );
        }
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn measured_angle_and_arc_angle_can_be_marked_and_iterated() {
    for arc in [false, true] {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 220., y: 180. }));
        let center = doc.add(Kind::Free(Point { x: 160., y: 180. }));
        let first = doc.add(Kind::Free(Point { x: 250., y: 160. }));
        let vertex = doc.add(Kind::Free(Point { x: 200., y: 160. }));
        let last = doc.add(Kind::Free(Point { x: 200., y: 110. }));
        let angle = if arc {
            let path = doc.add(Kind::ThreePointArc(first, vertex, last));
            doc.add(Kind::ArcAngle(path))
        } else {
            doc.add(Kind::AngleMeasurement(first, vertex, last))
        };
        let mut editor = Editor::new(doc);
        editor.selection_order = vec![angle];
        editor.doc.selected.insert(angle);
        editor.mark_angle_selected();
        assert_eq!(editor.marked_angle, Some(MarkedAngle::Value(angle)));
        editor.selection_order = vec![seed, center];
        editor.doc.selected.extend([seed, center]);
        editor.rotate_selected();
        let image = *editor.doc.selected.iter().next().unwrap();
        assert_eq!(
            editor.doc.objects()[image - 1].kind,
            Kind::CalculatedAngleRotatedPoint(seed, center, angle)
        );
        editor.selection_order = vec![seed, image];
        editor.doc.selected.insert(seed);
        editor.iterate_selected(3);
        let iteration = *editor.doc.selected.iter().next().unwrap();
        assert_eq!(
            editor.doc.point_iteration_images(iteration).unwrap().len(),
            3
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn selected_marked_images_create_live_geometry_iterations() {
    for mapping in 0..4 {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 120., y: 180. }));
        let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
        let b = doc.add(Kind::Free(Point { x: 100., y: 160. }));
        let distance = doc.add(Kind::PointDistance(a, b));
        let image = doc.add(match mapping {
            0 => Kind::MarkedDistanceTranslatedPoint(seed, distance, 0.),
            1 => Kind::MixedRectangularTranslatedPoint(seed, distance, 0.25, true),
            2 => Kind::RectangularMarkedTranslatedPoint(seed, distance, distance),
            _ => Kind::AngleRotatedPoint(seed, a, seed, a, b),
        });
        let mut editor = Editor::new(doc);
        editor.selection_order = vec![image, seed];
        editor.doc.selected.extend([seed, image]);
        editor.iterate_selected(3);
        let iteration = *editor.doc.selected.iter().next().unwrap();
        assert!(matches!(
            editor.doc.objects()[iteration - 1].kind,
            Kind::PointIteration(..)
        ));
        assert_eq!(
            editor.doc.point_iteration_images(iteration).unwrap().len(),
            3
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn selected_reflection_and_shared_vector_create_iterations() {
    for vector in [false, true] {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 100., y: 180. }));
        let a = doc.add(Kind::Free(Point { x: 140., y: 80. }));
        let b = doc.add(Kind::Free(Point { x: 140., y: 260. }));
        let mirror = doc.add(Kind::Line(a, b));
        let image = doc.add(if vector {
            Kind::VectorTranslatedPoint(seed, seed, b)
        } else {
            Kind::ReflectedPoint(seed, mirror)
        });
        let mut editor = Editor::new(doc);
        editor.selection_order = vec![image, seed];
        editor.doc.selected.extend([seed, image]);
        editor.iterate_selected(3);
        let points = editor.doc.point_iteration_images(6).unwrap();
        if vector {
            assert_eq!(points, vec![Point { x: 140., y: 260. }; 3]);
        } else {
            assert_eq!(
                points,
                vec![
                    Point { x: 100., y: 180. },
                    Point { x: 180., y: 180. },
                    Point { x: 100., y: 180. }
                ]
            );
        }
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn selected_dilated_image_creates_point_and_circle_iterations() {
    let mut doc = Document::new();
    let seed = doc.add(Kind::Free(Point { x: 220., y: 180. }));
    let center = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let image = doc.add(Kind::ScaledPoint(seed, center, 0.5));
    let circle = doc.add(Kind::Circle(seed, image));
    let mut editor = Editor::new(doc);
    editor.selection_order = vec![image, seed];
    editor.doc.selected.extend([seed, image]);
    editor.iterate_selected(4);
    assert_eq!(
        editor.doc.point_iteration_images(5).unwrap()[0],
        Point { x: 130., y: 180. }
    );
    editor.selection_order = vec![circle, image, seed];
    editor.doc.selected = [circle, image, seed].into_iter().collect();
    editor.iterate_selected(3);
    let binding = editor
        .doc
        .objects()
        .iter()
        .find(|o| matches!(o.kind, Kind::IterationBinding(source, _) if source == circle))
        .unwrap()
        .id;
    assert_eq!(
        editor.doc.circle_iteration_images(binding).unwrap()[0],
        (Point { x: 160., y: 180. }, 30.)
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
