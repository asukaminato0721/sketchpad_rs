use super::*;

#[test]
fn arc_commands_share_live_transforms_picking_constraints_styles_and_measurements() {
    use sketchpad_engine::document::transform_model::ElementType;
    for element in [
        ElementType::Arc,
        ElementType::Disk,
        ElementType::Sector,
        ElementType::CircularSegment,
    ] {
        let operations = if element == ElementType::CircularSegment {
            vec![5, 7, 8, 11]
        } else {
            (0..14).collect()
        };
        for operation in operations {
            let mut doc = Document::new();
            for point in [
                Point { x: 100., y: 180. },
                Point { x: 140., y: 180. },
                Point { x: 100., y: 140. },
            ] {
                doc.add(Kind::Free(point));
            }
            let source = match element {
                ElementType::Disk => {
                    let circle = doc.add(Kind::Circle(1, 2));
                    doc.add(Kind::CircleInterior(circle))
                }
                ElementType::Sector => {
                    let arc = doc.add(Kind::CenterArc(1, 2, 3));
                    doc.add(Kind::SectorBoundary(arc))
                }
                ElementType::CircularSegment => {
                    let arc = doc.add(Kind::CenterArc(1, 2, 3));
                    doc.add(Kind::CircularSegmentBoundary(arc))
                }
                _ => doc.add(Kind::CenterArc(1, 2, 3)),
            };
            let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
            let vector_start = doc.add(Kind::Free(Point { x: 10., y: 10. }));
            let vector_end = doc.add(Kind::Free(Point { x: 30., y: 10. }));
            let mirror = doc.add(Kind::Segment(origin, vector_start));
            let angle = doc.add(Kind::AngleMeasurement(1, 2, 3));
            let ratio = doc.add_named_parameter("n", -0.5, (10, 100)).unwrap();
            let horizontal = doc.add(Kind::SegmentLength(mirror));
            let vertical = doc.add(Kind::PointDistance(1, 2));
            let mut editor = Editor::new(doc);
            editor.select_only(source);
            editor.marked_center = Some(origin);
            editor.marked_vector = Some((vector_start, vector_end));
            editor.marked_mirror = Some(mirror);
            editor.marked_distance = Some(horizontal);
            editor.marked_rectangular_distances = Some((horizontal, vertical));
            editor.translation_angle = 30.;
            editor.translation_distance = 1.;
            editor.translation_horizontal = 1.;
            editor.translation_vertical = 2.;
            let before = editor.doc.encode().unwrap();
            match operation {
                0..=6 => {
                    editor.translation_mode = [
                        TranslationMode::FixedPolar,
                        TranslationMode::FixedRectangular,
                        TranslationMode::MarkedDistance,
                        TranslationMode::RectangularMarked,
                        TranslationMode::RectangularHorizontalFixed,
                        TranslationMode::RectangularVerticalFixed,
                        TranslationMode::MarkedVector,
                    ][operation];
                    editor.translate_selected();
                }
                7 => editor.reflect_selected(),
                8..=10 => {
                    editor.rotation_angle = 30.;
                    editor.rotation_use_marked = operation != 8;
                    editor.marked_angle = Some(if operation == 9 {
                        MarkedAngle::Points([1, 2, 3])
                    } else {
                        MarkedAngle::Value(angle)
                    });
                    editor.rotate_selected();
                }
                _ => {
                    editor.dilation_factor = -0.5;
                    editor.dilation_use_marked = operation != 11;
                    editor.marked_ratio = Some([origin, vector_start, vector_end]);
                    editor.marked_ratio_value = (operation == 13).then_some(ratio);
                    editor.dilate_selected();
                }
            }
            let image = *editor.doc.selected.iter().next().unwrap();
            assert_ne!(image, source, "operation{operation}: {}", editor.status);
            let definition = editor.doc.objects()[image - 1]
                .kind
                .transform_definition()
                .unwrap();
            assert_eq!(definition.element(), element);
            assert_eq!(definition.source(), source);
            assert_eq!(
                definition.wire_flavor().code(),
                [21, 17, 24, 20, 18, 19, 16, 34, 27, 28, 29, 30, 32, 33][operation]
            );
            assert_eq!(
                editor.doc.objects()[image - 1].visual_style,
                editor.doc.objects()[source - 1].visual_style
            );
            editor.doc.verify_roundtrip().unwrap();
            editor.doc.undo();
            assert_eq!(editor.doc.encode().unwrap(), before);
            editor.doc.redo();
            for id in 1..=editor.doc.objects().len() {
                editor
                    .doc
                    .edit_object(id, |object| object.visible = id == image)
                    .unwrap();
            }
            if element != ElementType::Arc {
                let outline = editor.doc.filled_region_points(image).unwrap();
                let interior = Point {
                    x: outline.iter().map(|p| p.x).sum::<f64>() / outline.len() as f64,
                    y: outline.iter().map(|p| p.y).sum::<f64>() / outline.len() as f64,
                };
                let viewport = Viewport {
                    rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
                    scale: 1.,
                    center: interior,
                };
                assert_eq!(
                    editor.hit(&viewport, viewport.screen(interior)),
                    Some(image),
                    "{element:?} operation{operation}"
                );
                let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
                assert_eq!(svg.matches("<polygon").count(), 1);
                continue;
            }
            let middle = editor.doc.point_on_arc(image, 0.5).unwrap();
            let viewport = Viewport {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
                scale: 1.,
                center: middle,
            };
            assert_eq!(
                editor.hit(&viewport, viewport.screen(middle)),
                Some(image),
                "operation{operation}"
            );
            let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
            assert_eq!(svg.matches("<path").count(), 1);
            editor.select_only(image);
            editor
                .doc
                .set_selected_line_pattern(sketchpad_engine::style_records::LinePattern::Dashed)
                .unwrap();
            editor.point_on_object();
            let point = *editor.doc.selected.iter().next().unwrap();
            assert!(
                matches!(editor.doc.objects()[point - 1].kind, Kind::PointOnArc(host, _) if host == image)
            );
            editor.select_only(image);
            editor.measure_arc_angle();
            let measurement = *editor.doc.selected.iter().next().unwrap();
            assert!((editor.doc.numeric_value(measurement).unwrap() - 90.).abs() < 1e-9);
            editor.select_only(image);
            editor.measure_arc_length();
            assert_ne!(*editor.doc.selected.iter().next().unwrap(), image);
            editor.doc.verify_roundtrip().unwrap();
        }
    }
}
