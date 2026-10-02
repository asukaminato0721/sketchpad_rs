use super::*;

#[test]
fn source_point_style_undoes_and_survives_save() {
    let mut document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/style_blue_point.gsp"),
        None,
    )
    .unwrap();
    document.selected.insert(1);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Large)
            .unwrap(),
        1
    );
    assert_eq!(
        document.objects[0].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Large,
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 255
            },
        })
    );
    document.undo();
    assert!(matches!(
        document.objects[0].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            ..
        })
    ));
    document.redo();
    let check = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(
        check.objects[0].visual_style,
        document.objects[0].visual_style
    );
    assert_eq!(check.objects[0].kind, document.objects[0].kind);
    let path =
        std::env::temp_dir().join(format!("sketchpad-style-point-{}.gsp", std::process::id()));
    document.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        reopened.objects[0].visual_style,
        document.objects[0].visual_style
    );
    assert!(!reopened.partial_view);
}

#[test]
fn new_styled_segment_and_copied_points_reopen() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 100., y: 120. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 120. }));
    let segment = document.add(Kind::Segment(a, b));
    document.selected.insert(segment);
    assert_eq!(
        document.set_selected_line_width(LineWidth::Thick).unwrap(),
        1
    );
    assert_eq!(
        document
            .set_selected_line_pattern(LinePattern::Dashed)
            .unwrap(),
        1
    );
    assert_eq!(
        document
            .set_selected_color(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
            .unwrap(),
        1
    );
    document.selected.clear();
    document.selected.insert(a);
    document
        .set_selected_point_style(PointStyle::Small)
        .unwrap();
    let copied = document.copy_selected().unwrap();
    let pasted = document.paste(&copied).unwrap();
    assert_eq!(
        document.objects[pasted[0] - 1].visual_style,
        document.objects[a - 1].visual_style
    );
    let path = std::env::temp_dir().join(format!(
        "sketchpad-style-segment-{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        reopened.objects[segment - 1].visual_style,
        Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Dashed
            },
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 255
            },
        })
    );
    assert_eq!(
        reopened.objects[pasted[0] - 1].visual_style,
        document.objects[a - 1].visual_style
    );
}
#[test]
fn imported_picture_moves_undoes_and_survives_save() {
    let png = include_bytes!("../../../../../tests/fixtures/image_tiny.png");
    let mut document = Document::new();
    document.add(Kind::Free(Point { x: 80., y: 90. }));
    let image = document
        .import_png(png, Point { x: 120., y: 130. })
        .unwrap();
    assert_eq!(image, 2);
    document
        .move_image(image, Point { x: 160., y: 155. })
        .unwrap();
    assert!(
        matches!(&document.objects[1].kind, Kind::Image { placement: ImagePlacement::Independent(transform), .. } if transform[2] == 160.)
    );
    document.undo();
    assert!(
        matches!(&document.objects[1].kind, Kind::Image { placement: ImagePlacement::Independent(transform), .. } if transform[2] == 120.)
    );
    document.redo();
    let path = std::env::temp_dir().join(format!("sketchpad-image-{}.gsp", std::process::id()));
    document.save_to(&path).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 2);
    assert_eq!(reopened.objects[1].kind, document.objects[1].kind);

    let top_left = document.add(Kind::Free(Point { x: 16., y: 26. }));
    let bottom_left = document.add(Kind::Free(Point { x: 16., y: 90. }));
    if let Kind::Image { placement, .. } = &mut document.objects.edit(image - 1).kind {
        *placement = ImagePlacement::Rectangle {
            bottom_right: 1,
            top_left,
        };
    }
    assert_eq!(
        document.image_transform(image),
        Some([4., 0., 16., 0., 4., 26.])
    );
    assert_eq!(document.objects[image - 1].kind.dependencies(), vec![1, 3]);
    if let Kind::Image { placement, .. } = &mut document.objects.edit(image - 1).kind {
        *placement = ImagePlacement::Parallelogram {
            top_left,
            bottom_right: 1,
            bottom_left,
        };
    }
    assert_eq!(
        document.image_transform(image),
        Some([4., 0., 16., 0., 4., 26.])
    );
    assert_eq!(
        document.objects[image - 1].kind.dependencies(),
        vec![3, 1, 4]
    );
    let vector_start = document.add(Kind::Free(Point { x: 10., y: 20. }));
    let vector_end = document.add(Kind::Free(Point { x: 15., y: 12. }));
    let translated = document.add(Kind::Image {
        png: png.to_vec(),
        placement: ImagePlacement::Transformed(
            TransformDefinition::new(
                image,
                ElementType::Image,
                Transform::Translate(DisplacementExpr::Vector {
                    start: vector_start,
                    end: vector_end,
                }),
                NativeTransformFlavor::Vector,
            )
            .unwrap(),
        ),
    });
    assert_eq!(
        document.image_transform(translated),
        Some([4., 0., 21., 0., 4., 18.])
    );
    assert_eq!(
        document.objects[translated - 1].kind.dependencies(),
        vec![image, vector_start, vector_end]
    );
    let scaled = document.add(Kind::Image {
        png: png.to_vec(),
        placement: ImagePlacement::Transformed(
            TransformDefinition::new(
                image,
                ElementType::Image,
                Transform::Dilate {
                    center: top_left,
                    ratio: NumericExpr::Literal {
                        magnitude: 2.,
                        unit: NumericUnit::Scalar,
                        scale: 1.,
                    },
                },
                NativeTransformFlavor::FixedDilation,
            )
            .unwrap(),
        ),
    });
    assert_eq!(
        document.image_transform(scaled),
        Some([8., 0., 16., 0., 8., 26.])
    );
    assert_eq!(
        document.objects[scaled - 1].kind.dependencies(),
        vec![image, top_left]
    );
    if let Kind::Image { placement, .. } = &mut document.objects.edit(image - 1).kind {
        *placement = ImagePlacement::Point {
            anchor: 1,
            relative: [0., 1., -8., -1., 0., 8.],
        };
    }
    assert_eq!(
        document.image_transform(image),
        Some([0., 1., 72., -1., 0., 98.])
    );
    assert_eq!(document.objects[image - 1].kind.dependencies(), vec![1]);
}

#[test]
fn picture_top_edge_defines_a_native_data_function() {
    let png = include_bytes!("../../../../../tests/fixtures/image_tiny.png");
    let mut document = Document::new();
    let picture = document
        .import_png(png, Point { x: 150., y: 150. })
        .unwrap();
    let function = document
        .add_picture_function(picture, Point { x: 199., y: 125. })
        .unwrap();

    assert_eq!(function, 8);
    assert!(matches!(
        document.objects[function - 1].kind,
        Kind::DataFunction {
            source: 1,
            frame: 7,
            smoothing: 3
        }
    ));
    assert_eq!(document.dependencies_of(function).unwrap(), vec![1, 7]);
    assert!(document.function_value(function, -1.1).is_some());
    assert!(document.function_value(function, 0.0).is_none());

    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[function - 1].kind,
        document.objects[function - 1].kind
    );
    assert_eq!(
        reopened.function_value(function, -1.1),
        document.function_value(function, -1.1)
    );
}

#[test]
fn geometric_labels_keep_native_placement_parameters() {
    let mut native_ray = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/text_ray_label_hide.gsp"),
        None,
    )
    .unwrap();
    assert!(!native_ray.partial_view);
    let ray_label = native_ray.objects[5].label.as_ref().unwrap();
    assert_eq!(ray_label.name_utf8().unwrap(), "k");
    assert!(!ray_label.visible());
    native_ray.selected.insert(6);
    assert_eq!(
        native_ray
            .set_selected_geometry_label_visibility(true)
            .unwrap(),
        1
    );
    assert!(native_ray.objects[5].label.as_ref().unwrap().visible());
    native_ray.undo();
    assert!(!native_ray.objects[5].label.as_ref().unwrap().visible());

    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 100., y: 80. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 80. }));
    let center = document.add(Kind::Free(Point { x: 320., y: 100. }));
    let radius = document.add(Kind::Free(Point { x: 360., y: 100. }));
    let arc_end = document.add(Kind::Free(Point { x: 320., y: 60. }));
    let segment = document.add(Kind::Segment(a, b));
    let ray = document.add(Kind::Ray(a, b));
    let line = document.add(Kind::Line(a, b));
    let circle = document.add(Kind::Circle(center, radius));
    let arc = document.add(Kind::ArcOnCircle(circle, radius, arc_end));
    let transformed_segment = document.add(Kind::FixedRectangularTranslatedSegment(
        segment,
        37.79527559055118,
        3.,
        37.79527559055118,
        0.,
    ));
    let transformed_circle = document.add(Kind::FixedRectangularTranslatedCircle(
        circle,
        37.79527559055118,
        3.,
        37.79527559055118,
        0.,
    ));
    let inch_transformed_segment = document.add(Kind::FixedRectangularTranslatedSegment(
        segment, 96., 0.5, 96., -0.25,
    ));
    let transformed_ray = document.add(Kind::RotatedRay(ray, a, 45.));
    let transformed_line = document.add(Kind::RotatedLine(line, a, 45.));
    let angle = document
        .add_named_parameter("theta", 45., (20, 20))
        .unwrap();
    let transformed_arc = document
        .add_calculated_angle_rotation(arc, center, angle)
        .unwrap();
    let [
        colorized_segment,
        colorized_ray,
        colorized_line,
        colorized_circle,
        colorized_arc,
    ] = [segment, ray, line, circle, arc].map(|source| {
        document
            .add_colorized(
                source,
                crate::document::colorized::ColorBinding::Spectrum { value: angle },
                0.,
                100.,
                crate::document::colorized::ColorRangeBehavior::Clamp,
            )
            .unwrap()
    });

    assert!(document.set_object_label(segment, "s", true).unwrap());
    assert!(document.set_object_label(circle, "c", true).unwrap());
    assert!(document.set_object_label(arc, "a[1]", true).unwrap());
    assert!(
        document
            .set_object_label(transformed_segment, "t[1]", true)
            .unwrap()
    );
    assert!(
        document
            .set_object_label(transformed_circle, "d[1]", true)
            .unwrap()
    );
    assert!(
        document
            .set_object_label(transformed_ray, "r[1]", true)
            .unwrap()
    );
    assert!(
        document
            .set_object_label(transformed_line, "l[1]", true)
            .unwrap()
    );
    assert!(
        document
            .set_object_label(transformed_arc, "b[1]", true)
            .unwrap()
    );
    for (id, name) in [
        (colorized_segment, "u[1]"),
        (colorized_ray, "r[2]"),
        (colorized_line, "l[2]"),
        (colorized_circle, "v[1]"),
        (colorized_arc, "w[1]"),
    ] {
        assert!(document.set_object_label(id, name, true).unwrap());
    }
    assert_eq!(
        document.object_label_anchor(segment),
        Some(Point { x: 125., y: 80. })
    );
    let circle_anchor = document.object_label_anchor(circle).unwrap();
    assert!((circle_anchor.x - 348.2842712474619).abs() < 1e-12);
    assert!((circle_anchor.y - 128.2842712474619).abs() < 1e-12);
    assert_eq!(
        document.object_label_anchor(arc),
        document.point_on_arc(arc, 0.25)
    );
    assert_eq!(
        document.object_label_anchor(transformed_segment),
        document
            .line_points(transformed_segment)
            .map(|(start, end, _)| Point {
                x: start.x + 0.25 * (end.x - start.x),
                y: start.y + 0.25 * (end.y - start.y),
            })
    );
    for straight in [transformed_ray, transformed_line] {
        assert_eq!(
            document.object_label_anchor(straight),
            document.line_points(straight).map(|(start, end, _)| Point {
                x: start.x + 0.25 * (end.x - start.x),
                y: start.y + 0.25 * (end.y - start.y),
            })
        );
    }
    let transformed_circle_anchor = document.object_label_anchor(transformed_circle).unwrap();
    let (transformed_center, transformed_radius) =
        document.circle_points(transformed_circle).unwrap();
    let transformed_angle = 7.0 * std::f64::consts::FRAC_PI_4;
    assert!(
        (transformed_circle_anchor.x
            - (transformed_center.x + transformed_radius * transformed_angle.cos()))
        .abs()
            < 1e-12
    );
    assert!(
        (transformed_circle_anchor.y
            - (transformed_center.y - transformed_radius * transformed_angle.sin()))
        .abs()
            < 1e-12
    );
    assert_eq!(
        document.object_label_anchor(transformed_arc),
        document.point_on_arc(transformed_arc, 0.25)
    );
    for straight in [colorized_segment, colorized_ray, colorized_line] {
        assert_eq!(
            document.object_label_anchor(straight),
            document.line_points(straight).map(|(start, end, _)| Point {
                x: start.x + 0.25 * (end.x - start.x),
                y: start.y + 0.25 * (end.y - start.y),
            })
        );
    }
    assert!(document.object_label_anchor(colorized_circle).is_some());
    assert_eq!(
        document.object_label_anchor(colorized_arc),
        document.point_on_arc(colorized_arc, 0.25)
    );

    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    for id in [
        segment,
        circle,
        arc,
        transformed_segment,
        transformed_ray,
        transformed_line,
        transformed_circle,
        transformed_arc,
        colorized_segment,
        colorized_ray,
        colorized_line,
        colorized_circle,
        colorized_arc,
        inch_transformed_segment,
    ] {
        assert_eq!(
            reopened.objects[id - 1].label,
            document.objects[id - 1].label
        );
        assert_eq!(
            reopened.object_label_anchor(id),
            document.object_label_anchor(id)
        );
    }

    let records = records(&encoded).unwrap();
    let header_offset = reopened.objects[inch_transformed_segment - 1]
        .header_offset
        .unwrap();
    let start = records
        .iter()
        .position(|record| record.kind == 0x07d0 && record.data.start == header_offset)
        .unwrap();
    let constraint = records[start..]
        .iter()
        .find(|record| record.kind == 0x07d3)
        .unwrap();
    assert_eq!(u32_at(&encoded, constraint.data.start + 20), 2);
    assert_eq!(u32_at(&encoded, constraint.data.start + 40), 2);
    let definition = reopened.objects[inch_transformed_segment - 1]
        .kind
        .transform_definition()
        .unwrap();
    assert_eq!(
        definition.wire_flavor(),
        NativeTransformFlavor::FixedCartesian
    );
    let (source_a, source_b, _) = reopened.line_points(segment).unwrap();
    let (image_a, image_b, _) = reopened.line_points(inch_transformed_segment).unwrap();
    for (source, image) in [(source_a, image_a), (source_b, image_b)] {
        assert_eq!(image.x - source.x, 48.);
        assert_eq!(image.y - source.y, 24.);
    }
}

#[test]
fn picture_import_rejects_bad_png_and_preserves_existing_resources() {
    let mut document = Document::new();
    assert!(
        document
            .import_png(b"broken", Point { x: 0., y: 0. })
            .is_err()
    );
    assert!(document.objects.is_empty());
    let png = include_bytes!("../../../../../tests/fixtures/image_tiny.png");
    document.import_png(png, Point { x: 0., y: 0. }).unwrap();
    assert!(document.import_png(png, Point { x: 20., y: 20. }).is_err());
    assert_eq!(document.objects.len(), 1);
    for source in [
        include_bytes!("../../../../../tests/fixtures/image_original_cropped.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/image_original_pasted.gsp").as_slice(),
    ] {
        let mut read_only = Document::from_bytes(source, None).unwrap();
        assert_eq!(read_only.encode().unwrap(), source);
        read_only.checkpoint();
        read_only.can_save().unwrap();
        assert!(read_only.import_png(png, Point { x: 20., y: 20. }).is_err());
        assert_eq!(read_only.encode().unwrap(), source);
    }
}

#[test]
fn picture_crop_writes_native_layout_and_reopens_as_a_live_dependency() {
    let source = include_bytes!("../../../../../tests/fixtures/image_port_polygon.gsp");
    let mut document = Document::from_bytes(source, None).unwrap();
    let crop = document.crop_picture_to_polygon(1, 5).unwrap();
    assert_eq!(crop, 6);
    assert!(!document.objects[0].visible);
    assert!(matches!(
        document.objects[5].kind,
        Kind::CroppedImage {
            source: 1,
            polygon: 5
        }
    ));
    let written = document.encode().unwrap();
    let image_file = crate::image_records::ImageFile::parse(&written).unwrap();
    assert_eq!(image_file.crops().len(), 1);
    assert_eq!(image_file.crops()[0].source, 1);
    assert_eq!(image_file.crops()[0].polygon, 5);
    assert_eq!(image_file.crops()[0].dimensions, (16, 16));
    let reopened = Document::from_bytes(&written, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.cropped_image(6).is_some());

    let native = include_bytes!("../../../../../tests/fixtures/image_original_cropped.gsp");
    let native_document = Document::from_bytes(native, None).unwrap();
    assert!(!native_document.partial_view);
    assert!(matches!(
        native_document.objects[5].kind,
        Kind::CroppedImage {
            source: 1,
            polygon: 5
        }
    ));
    assert_eq!(native_document.encode().unwrap(), native);
}

#[test]
fn original_drag_of_port_picture_reopens_and_remains_editable() {
    let source =
        include_bytes!("../../../../../tests/fixtures/image_port_imported_dragged_wine.gsp");
    let mut document = Document::from_bytes(source, None).unwrap();
    assert!(!document.partial_view);
    assert!(
        matches!(&document.objects[1].kind, Kind::Image { placement: ImagePlacement::Independent(transform), .. }
        if transform[2] == 184. && transform[5] == 170.)
    );
    document.move_image(2, Point { x: 204., y: 180. }).unwrap();
    let written = document.encode().unwrap();
    let reopened = Document::from_bytes(&written, None).unwrap();
    assert!(
        matches!(&reopened.objects[1].kind, Kind::Image { placement: ImagePlacement::Independent(transform), .. }
        if transform[2] == 204. && transform[5] == 180.)
    );
}
#[test]
fn plain_text_edit_move_and_label_survive_save() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/text_object_original_Hello.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/text_object_original_Hello.gsp")
    );
    doc.edit_text(1, "猫\nA<B>").unwrap();
    doc.move_text(1, Point { x: 180., y: 130. }).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.objects[0].kind, doc.objects[0].kind);
    assert!(!reopened.partial_view);

    let mut label_doc = Document::new();
    let point = label_doc.add(Kind::Free(Point { x: 100., y: 120. }));
    label_doc.set_point_label(point, "P", true).unwrap();
    let bytes = label_doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        reopened.objects[0]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "P"
    );
    assert!(!reopened.partial_view);
}
#[test]
fn styled_text_imports_and_edits_without_changing_other_runs() {
    for source in [
        include_bytes!("../../../../../tests/fixtures/text_style_bold_off.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/text_style_italic_on.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/text_style_underline_on.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/text_style_larger.gsp").as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/text_two_objects_first_bold_off_original.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/text_two_objects_second_bold_off_original.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
    }

    let original = include_bytes!(
        "../../../../../tests/fixtures/text_two_objects_first_bold_off_original.gsp"
    );
    let mut doc = Document::from_bytes(original, None).unwrap();
    let other = doc.objects[1].kind.clone();
    doc.edit_text(1, "Greetings").unwrap();
    let edited = doc.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[1].kind, other);
    assert!(
        matches!(&reopened.objects[0].kind, Kind::PlainText { text, .. } if text == "Greetings")
    );
    for kind in [0x04b1, 0x04b3, 0x273c] {
        let payloads = |bytes: &[u8]| {
            records(bytes)
                .unwrap()
                .into_iter()
                .filter(|record| record.kind == kind)
                .map(|record| bytes[record.data].to_vec())
                .collect::<Vec<_>>()
        };
        assert_eq!(payloads(&edited), payloads(original));
    }
    doc.move_text(1, Point { x: 110., y: 100. }).unwrap();
    let moved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!moved.partial_view);
    assert_eq!(moved.objects[1].kind, other);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), edited);
}
#[test]
fn text_formatting_keeps_independent_styles_through_save_undo_and_copy() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/text_two_objects_generated.gsp"),
        None,
    )
    .unwrap();
    assert!(doc.text_font_state(1).unwrap().bold);
    doc.selected.insert(1);
    assert_eq!(
        doc.toggle_selected_text_style(TextStyleCommand::Bold)
            .unwrap(),
        1
    );
    assert!(!doc.text_font_state(1).unwrap().bold);
    assert!(doc.text_font_state(2).unwrap().bold);
    let one_formatted = doc.encode().unwrap();
    let reopened = Document::from_bytes(&one_formatted, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(!reopened.text_font_state(1).unwrap().bold);
    assert!(reopened.text_font_state(2).unwrap().bold);
    doc.undo();
    assert!(doc.text_font_state(1).unwrap().bold);
    doc.redo();
    assert!(!doc.text_font_state(1).unwrap().bold);

    doc.selected.clear();
    doc.selected.insert(2);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    doc.toggle_selected_text_style(TextStyleCommand::Underline)
        .unwrap();
    let font = doc.text_font_state(2).unwrap();
    assert!(font.bold && font.italic && font.underline);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(pasted, vec![3]);
    assert_eq!(doc.text_font_state(3).unwrap(), font);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(3).unwrap(), font);
    assert_eq!(reopened.text_font_state(2).unwrap(), font);
    assert!(!reopened.text_font_state(1).unwrap().bold);
}
#[test]
fn mixed_font_families_survive_style_edit_and_copy() {
    let source = include_bytes!("../../../../../tests/fixtures/text_two_font_cjk_first.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    assert_eq!(doc.text_font_state(1).unwrap().family, "@Noto Sans CJK JP");
    assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
    doc.selected.insert(1);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    let copied = doc.copy_selected().unwrap();
    assert_eq!(doc.paste(&copied).unwrap(), vec![3]);
    doc.selected.clear();
    doc.selected.insert(2);
    doc.toggle_selected_text_style(TextStyleCommand::Bold)
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.text_font_state(1).unwrap().family,
        "@Noto Sans CJK JP"
    );
    assert!(reopened.text_font_state(1).unwrap().italic);
    assert_eq!(reopened.text_font_state(2).unwrap().family, "Tahoma");
    assert!(!reopened.text_font_state(2).unwrap().bold);
    assert_eq!(reopened.text_font_state(3), reopened.text_font_state(1));
}
#[test]
fn native_mixed_format_text_moves_without_losing_runs() {
    let source =
        include_bytes!("../../../../../tests/fixtures/text_mixed_bold_italic_he_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    let runs = doc.text_rich_runs(1).unwrap();
    assert_eq!(runs[0][0].text, "He");
    assert_eq!(runs[0][0].style.bold, Some(false));
    assert_eq!(runs[0][0].style.italic, Some(true));
    assert_eq!(runs[0][1].text, "llo");
    doc.edit_text(1, "Hexllo").unwrap();
    let edited = doc.text_rich_runs(1).unwrap();
    assert_eq!(edited[0][0].text, "Hex");
    assert_eq!(edited[0][0].style, runs[0][0].style);
    assert_eq!(edited[0][1].text, "llo");
    let edited_file = doc.encode().unwrap();
    let reopened_edit = Document::from_bytes(&edited_file, None).unwrap();
    assert_eq!(reopened_edit.text_rich_runs(1).unwrap(), edited);
    doc.undo();
    assert_eq!(doc.text_rich_runs(1).unwrap(), runs);
    doc.move_text(1, Point { x: 140., y: 110. }).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap(), runs);
    assert_eq!(reopened.encode().unwrap(), encoded);

    doc.selected.insert(1);
    let clipboard = doc.copy_selected().unwrap();
    assert_eq!(doc.paste(&clipboard).unwrap(), vec![2]);
    assert_eq!(doc.text_rich_runs(2).unwrap(), runs);
    let Kind::PlainText { rect: first, .. } = &doc.objects[0].kind else {
        unreachable!()
    };
    let Kind::PlainText { rect: second, .. } = &doc.objects[1].kind else {
        unreachable!()
    };
    assert_eq!(second.left, first.left + 20);
    assert_eq!(second.top, first.top + 20);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(2).unwrap(), runs);
}
#[test]
fn substring_size_steps_preserve_native_rich_text() {
    let source = include_bytes!("../../../../../tests/fixtures/text_mixed_size36_he_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    let original = doc.text_rich_runs(1).unwrap();
    assert_eq!(original[0][0].style.size_code, Some(36));
    doc.format_text_range(1, 0, 2, TextStyleCommand::DecreaseSize)
        .unwrap();
    assert_eq!(doc.text_rich_runs(1).unwrap()[0][0].style.size_code, None);
    doc.format_text_range(1, 0, 2, TextStyleCommand::IncreaseSize)
        .unwrap();
    assert_eq!(doc.text_rich_runs(1).unwrap(), original);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap(), original);
}
#[test]
fn native_mixed_family_caption_edits_and_reopens() {
    let source =
        include_bytes!("../../../../../tests/fixtures/text_mixed_adventor_he_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
    let runs = doc.text_rich_runs(1).unwrap();
    assert_eq!(runs[0][0].style.family_index, Some(1));
    doc.edit_text(1, "Hexllo").unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap()[0][0].text, "Hex");
    assert_eq!(
        reopened.text_rich_runs(1).unwrap()[0][0].style.family_index,
        Some(1)
    );
}
#[test]
fn selected_range_can_add_a_font_family_and_undo_it() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/text_object_generated_Hello.gsp"),
        None,
    )
    .unwrap();
    doc.format_text_range_family(1, 0, 2, "TeX Gyre Adventor")
        .unwrap();
    assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
        Some(1)
    );
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.text_rich_runs(1).unwrap(),
        doc.text_rich_runs(1).unwrap()
    );
    doc.undo();
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
        None
    );
    assert_eq!(doc.text_font_families().unwrap().len(), 1);
    doc.redo();
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
        Some(1)
    );
    let path =
        std::env::temp_dir().join(format!("sketchpad_run_family_{}.gsp", std::process::id()));
    doc.save_to(&path).unwrap();
    let first = fs::read(&path).unwrap();
    doc.save_to(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), first);
    assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
    fs::remove_file(path).unwrap();
}
#[test]
fn selected_caption_range_writes_native_style_markup() {
    let source = include_bytes!("../../../../../tests/fixtures/text_object_generated_Hello.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    doc.format_text_range(1, 0, 2, TextStyleCommand::Bold)
        .unwrap();
    let runs = doc.text_rich_runs(1).unwrap();
    assert_eq!(runs[0][0].text, "He");
    assert_eq!(runs[0][0].style.bold, Some(false));
    assert_eq!(runs[0][1].text, "llo");
    let encoded = doc.encode().unwrap();
    assert!(
        encoded
            .windows(b"<VL<H<SW1<T2xHe>><T3xllo>>>".len())
            .any(|part| part == b"<VL<H<SW1<T2xHe>><T3xllo>>>")
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_rich_runs(1).unwrap(), runs);
    doc.format_text_range(1, 0, 2, TextStyleCommand::Italic)
        .unwrap();
    assert_eq!(
        doc.text_rich_runs(1).unwrap()[0][0].style.italic,
        Some(true)
    );
    doc.undo();
    assert_eq!(doc.text_rich_runs(1).unwrap(), runs);
    doc.edit_text(1, "Hexllo").unwrap();
    let edited = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    let edited_runs = edited.text_rich_runs(1).unwrap();
    assert_eq!(edited_runs[0][0].text, "Hex");
    assert_eq!(edited_runs[0][0].style.bold, Some(false));
    assert_eq!(edited_runs[0][1].text, "llo");
}
#[test]
fn selected_text_can_use_new_font_families_independently() {
    let source = include_bytes!("../../../../../tests/fixtures/text_two_objects_generated.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    doc.selected.insert(1);
    assert_eq!(doc.set_selected_text_family("Arial").unwrap(), 1);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    assert_eq!(doc.text_font_state(1).unwrap().family, "Arial");
    assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
    doc.selected.clear();
    doc.selected.insert(2);
    doc.set_selected_text_family("@Noto Sans CJK JP").unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(1).unwrap().family, "Arial");
    assert!(reopened.text_font_state(1).unwrap().italic);
    assert_eq!(
        reopened.text_font_state(2).unwrap().family,
        "@Noto Sans CJK JP"
    );
    assert_eq!(reopened.encode().unwrap(), encoded);
    assert!(doc.set_selected_text_family("").is_err());
    doc.set_selected_text_family("Noto 日本語").unwrap();
    let unicode = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(unicode.text_font_state(2).unwrap().family, "Noto 日本語");
    doc.undo();
    doc.undo();
    assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
}
#[test]
fn text_size_commands_follow_verified_native_steps() {
    for (source, size) in [
        (
            include_bytes!("../../../../../tests/fixtures/text_style_smaller_18_original.gsp")
                .as_slice(),
            18,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/text_style_larger_48_original.gsp")
                .as_slice(),
            48,
        ),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
        assert_eq!(doc.encode().unwrap(), source);
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/text_object_generated_Hello.gsp"),
        None,
    )
    .unwrap();
    doc.selected.insert(1);
    for size in [36, 48, 64] {
        doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
            .unwrap();
        assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
    }
    assert!(
        doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
            .is_err()
    );
    for size in [48, 36, 24, 18, 16, 14] {
        doc.toggle_selected_text_style(TextStyleCommand::DecreaseSize)
            .unwrap();
        assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
    }
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(1).unwrap().size_code, 14);
    doc.undo();
    assert_eq!(doc.text_font_state(1).unwrap().size_code, 16);
}
#[test]
fn point_label_styles_are_independent_and_survive_copy() {
    let source = include_bytes!(
        "../../../../../tests/fixtures/text_two_point_labels_first_bold_off_original.gsp"
    );
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), source);
    assert!(!doc.text_font_state(1).unwrap().bold);
    assert!(doc.text_font_state(2).unwrap().bold);

    doc.selected.insert(2);
    doc.toggle_selected_text_style(TextStyleCommand::Italic)
        .unwrap();
    assert!(doc.text_font_state(2).unwrap().italic);
    assert!(!doc.text_font_state(1).unwrap().bold);
    let copied = doc.copy_selected().unwrap();
    let pasted = doc.paste(&copied).unwrap();
    assert_eq!(pasted, vec![3]);
    assert_eq!(doc.objects[2].label, doc.objects[1].label);
    assert_eq!(doc.text_font_state(3), doc.text_font_state(2));
    doc.selected.clear();
    doc.selected.insert(1);
    doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
        .unwrap();
    assert_eq!(doc.text_font_state(1).unwrap().size_code, 36);
    assert_eq!(doc.text_font_state(2).unwrap().size_code, 24);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(!reopened.text_font_state(1).unwrap().bold);
    assert_eq!(reopened.text_font_state(1).unwrap().size_code, 36);
    assert!(reopened.text_font_state(2).unwrap().italic);
    assert_eq!(reopened.text_font_state(3), reopened.text_font_state(2));
    assert_eq!(reopened.objects[2].label, reopened.objects[1].label);
}
#[test]
fn source_point_label_rename_and_hide_survive_save() {
    let original = include_bytes!("../../../../../tests/fixtures/text_point_label.gsp");
    let mut doc = Document::from_bytes(original, None).unwrap();
    assert_eq!(doc.encode().unwrap(), original);
    doc.set_point_label(1, "Cat", false).unwrap();
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    let label = reopened.objects[0].label.as_ref().unwrap();
    assert_eq!(label.name_utf8().unwrap(), "Cat");
    assert!(!label.visible());
    assert!(!reopened.partial_view);
}
#[test]
fn wine_dragged_text_preserves_opaque_padding_until_edited() {
    let original =
        include_bytes!("../../../../../tests/fixtures/text_object_generated_Hello_dragged.gsp");
    let mut doc = Document::from_bytes(original, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), original);
    doc.edit_text(1, "Hello again").unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(
        matches!(&reopened.objects[0].kind, Kind::PlainText { text, .. } if text == "Hello again")
    );
}

#[test]
fn multi_page_literal_text_creation_edit_move_and_clipboard() {
    let native = include_bytes!("../../../../../tests/fixtures/text_object_original_multiline.gsp");
    let mut document = Document::from_bytes(native, None).unwrap();
    let text_id = document
        .objects
        .iter()
        .find(|object| matches!(object.kind, Kind::PlainText { .. }))
        .unwrap()
        .id;
    document.add_blank_page().unwrap();
    document.active_page = 0;
    document
        .edit_text(text_id, "Changed source caption\nwith another line")
        .unwrap();
    document
        .move_text(text_id, Point { x: 45., y: 70. })
        .unwrap();
    document.selected.insert(text_id);
    document
        .toggle_selected_text_style(TextStyleCommand::Bold)
        .unwrap();
    document
        .format_text_range(text_id, 0, 7, TextStyleCommand::Italic)
        .unwrap();
    let clipboard = document.copy_selected().unwrap();
    document.active_page = 1;
    let pasted = document.paste(&clipboard).unwrap();
    document
        .add_text("Second page\r\nUnicode 日本語", Point { x: 100., y: 150. })
        .unwrap();
    // Add on the earlier page after additions on page 2 to exercise page order.
    document.active_page = 0;
    document
        .add_text("First page addition", Point { x: 30., y: 250. })
        .unwrap();
    document.verify_roundtrip().unwrap();
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    let texts = reopened
        .objects
        .iter()
        .filter_map(|object| match &object.kind {
            Kind::PlainText { text, rect } => Some((object.id, object.page, text.as_str(), *rect)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 4);
    assert_eq!(texts[0].1, 0);
    assert_eq!(texts[0].2, "Changed source caption\nwith another line");
    assert_eq!(texts[0].3.left, 45);
    assert_eq!(texts[1].2, "First page addition");
    assert_eq!(texts[2].1, 1);
    assert_eq!(texts[2].2, texts[0].2);
    assert_eq!(texts[2].3.left, 65);
    assert!(reopened.text_font_state(texts[0].0).unwrap().bold);
    assert!(reopened.text_font_state(texts[2].0).unwrap().bold);
    assert_eq!(
        reopened.text_rich_runs(texts[2].0),
        document.text_rich_runs(pasted[0])
    );
    assert_eq!(texts[3].2, "Second page\nUnicode 日本語");
    assert_eq!(reopened.encode().unwrap(), encoded);
}

#[test]
fn multi_page_literal_text_source_edits_preserve_other_pages_and_clear() {
    let mut document = Document::new();
    document
        .add_text("Original", Point { x: 35., y: 50. })
        .unwrap();
    document.add_blank_page().unwrap();
    let second = document
        .add_text("Second page", Point { x: 200., y: 150. })
        .unwrap();
    document.selected.insert(second);
    document.set_selected_text_family("DejaVu Sans").unwrap();
    document
        .format_text_range_family(second, 0, 6, "DejaVu Serif")
        .unwrap();
    let bytes = document.encode().unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    let original_second = document.objects[1].clone();
    document.active_page = 0;
    let before_edit = document.encode().unwrap();
    document
        .edit_text(1, "A much longer source caption on the first page")
        .unwrap();
    document.verify_roundtrip().unwrap();
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.objects[1].kind, original_second.kind);
    assert_eq!(reopened.text_font_state(2), document.text_font_state(2));
    assert_eq!(reopened.text_rich_runs(2), document.text_rich_runs(2));
    document.undo();
    assert_eq!(document.encode().unwrap(), before_edit);
    document.redo();
    assert_eq!(document.encode().unwrap(), encoded);
    document.selected.clear();
    document.selected.insert(1);
    document.clear_selected().unwrap();
    document.verify_roundtrip().unwrap();
    let cleared = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(cleared.objects.len(), 1);
    assert_eq!(cleared.objects[0].page, 1);
    assert_eq!(cleared.objects[0].kind, original_second.kind);
}

#[test]
fn multi_page_literal_text_native_saved_fixture_remains_editable() {
    let native =
        include_bytes!("../../../../../tests/fixtures/two_page_text_port_saved_by_wine.gsp");
    let mut document = Document::from_bytes(native, None).unwrap();
    assert_eq!(document.page_count, 2);
    assert_eq!(document.objects.len(), 4);
    assert_eq!(document.encode().unwrap(), native);
    document.active_page = 1;
    assert!(document.edit_text(1, "Wrong page").is_err());
    let before_runs = document.text_rich_runs(3).unwrap();
    let before_font = document.text_font_state(3).unwrap();
    document
        .edit_text(3, "Source caption expanded\nSecond line added")
        .unwrap();
    document.move_text(3, Point { x: 120., y: 80. }).unwrap();
    document.verify_roundtrip().unwrap();
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.text_rich_runs(3).unwrap()[0], before_runs[0]);
    assert_eq!(reopened.text_font_state(3).unwrap(), before_font);
    assert_eq!(reopened.objects[0].kind, document.objects[0].kind);
    assert_eq!(reopened.objects[1].kind, document.objects[1].kind);
}

#[test]
fn native_measurement_caption_position_font_and_clipboard_are_preserved() {
    let native = include_bytes!("../../../../../tests/fixtures/measurement_copy_native.gsp");
    let mut doc = Document::from_bytes(native, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), native);
    assert_eq!(doc.objects[3].measurement_anchor, Some((10, 5)));
    assert_eq!(doc.objects[7].measurement_anchor, Some((30, 25)));
    assert_eq!(doc.text_font_state(4).unwrap().size_code, 18);
    let font = doc.text_font_state(4).unwrap();
    doc.move_measurement_caption(4, (-5, 120)).unwrap();
    doc.verify_roundtrip().unwrap();
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(reopened.objects[3].measurement_anchor, Some((-5, 120)));
    assert_eq!(reopened.text_font_state(4), Some(font));
    doc.undo();
    assert_eq!(doc.encode().unwrap(), native);
    doc.selected.insert(4);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|&&id| matches!(doc.objects[id - 1].kind, Kind::SegmentLength(_)))
        .unwrap();
    assert_eq!(doc.objects[copied - 1].measurement_anchor, Some((30, 25)));
    doc.verify_roundtrip().unwrap();
}

#[test]
fn measurement_caption_font_edit_preserves_geometry_and_saved_position() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/measurement_copy_native.gsp"),
        None,
    )
    .unwrap();
    let geometry = doc.position(2);
    let value = doc.numeric_value(4);
    doc.selected.insert(4);
    assert_eq!(
        doc.toggle_selected_text_style(TextStyleCommand::Bold)
            .unwrap(),
        1
    );
    doc.set_selected_text_family("Arial").unwrap();
    doc.verify_roundtrip().unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    let font = reopened.text_font_state(4).unwrap();
    assert!(font.bold);
    assert_eq!(font.family, "Arial");
    assert_eq!(reopened.objects[3].measurement_anchor, Some((10, 5)));
    assert_eq!(reopened.position(2), geometry);
    assert_eq!(reopened.numeric_value(4), value);
}

#[test]
fn native_measurement_caption_reads_its_actual_indexed_font() {
    let native =
        include_bytes!("../../../../../tests/fixtures/measurement_font_native_verified.gsp");
    let doc = Document::from_bytes(native, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), native);
    let font = doc.text_font_state(4).unwrap();
    assert_eq!(font.size_code, 24);
    assert!(font.bold);
}

#[test]
fn formatted_measurement_caption_replay_matches_native_verified_file() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/wine_segment_length.gsp"),
        None,
    )
    .unwrap();
    doc.selected.insert(4);
    doc.toggle_selected_text_style(TextStyleCommand::Bold)
        .unwrap();
    doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
        .unwrap();
    doc.set_selected_text_family("Arial").unwrap();
    let expected =
        include_bytes!("../../../../../tests/fixtures/measurement_caption_formatted_verified.gsp");
    assert_eq!(doc.encode().unwrap(), expected);
    let reopened = Document::from_bytes(expected, None).unwrap();
    assert!(!reopened.partial_view);
    let font = reopened.text_font_state(4).unwrap();
    assert_eq!(font.family, "Arial");
    assert_eq!(font.size_code, 24);
    assert!(font.bold);
}

#[test]
fn multi_page_measurement_caption_moves_atomically_and_preserves_other_page() {
    let native =
        include_bytes!("../../../../../tests/fixtures/two_page_point_measurements_port.gsp");
    let mut doc = Document::from_bytes(native, None).unwrap();
    let id = doc
        .objects
        .iter()
        .find(|o| o.measurement_anchor.is_some())
        .unwrap()
        .id;
    doc.active_page = doc.objects[id - 1].page;
    let before = doc.encode().unwrap();
    let original = doc.objects.clone();
    let value = doc.numeric_value(id);
    let font = doc.text_font_state(id);
    doc.move_measurement_caption(id, (-20, 150)).unwrap();
    let saved = doc.encode().unwrap();
    assert_eq!(
        saved,
        include_bytes!(
            "../../../../../tests/fixtures/two_page_measurement_caption_moved_native_verified.gsp"
        )
    );
    doc.verify_roundtrip().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(
        reopened.objects[id - 1].measurement_anchor,
        Some((-20, 150))
    );
    assert_eq!(reopened.numeric_value(id), value);
    assert_eq!(reopened.text_font_state(id), font);
    for object in &original {
        if object.id != id {
            assert_eq!(reopened.objects[object.id - 1], *object);
        }
    }
    if std::env::var_os("SKETCHPAD_WRITE_CAPTION_FIXTURE").is_some() {
        fs::write("/tmp/multi_page_measurement_caption_moved_port.gsp", &saved).unwrap();
    }
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), saved);
    doc.active_page = 1 - doc.active_page;
    let off_page = doc.encode().unwrap();
    assert!(doc.move_measurement_caption(id, (25, 80)).is_err());
    assert_eq!(doc.encode().unwrap(), off_page);
    doc.active_page = 1 - doc.active_page;
    let mut unknown = doc.clone();
    unknown.objects.edit(0).kind = Kind::Unsupported(0xffff);
    let snapshot = unknown.objects.clone();
    assert!(unknown.move_measurement_caption(id, (25, 80)).is_err());
    assert_eq!(unknown.objects, snapshot);
}
