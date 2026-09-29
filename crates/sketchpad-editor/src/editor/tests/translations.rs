use super::*;

#[test]
fn circle_construction_host_keeps_exact_transform_flavors() {
    for kind in [
        Kind::Circle(1, 2),
        Kind::CircleBySegment(1, 2),
        Kind::ReflectedCircle(1, 2),
        Kind::ScaledCircle(1, 2, 3.),
        Kind::RatioScaledCircle(1, 2, 3, 4, 5),
    ] {
        assert!(super::super::is_circle_construction_host(&kind), "{kind:?}");
    }
    for kind in [
        Kind::RotatedCircle(1, 2, 3.),
        Kind::VectorTranslatedCircle(1, 2, 3),
        Kind::CalculatedRatioScaledCircle(1, 2, 3),
        Kind::ReflectedPoint(1, 2),
        Kind::ScaledPolygon(1, 2, 3.),
    ] {
        assert!(
            !super::super::is_circle_construction_host(&kind),
            "{kind:?}"
        );
    }
}

#[test]
fn marked_center_and_mirror_drive_transforms_without_changing_source_records() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let source = editor.doc.add(Kind::Free(Point { x: 30., y: 0. }));
    let axis_point = editor.doc.add(Kind::Free(Point { x: 0., y: 40. }));
    let mirror = editor.doc.add(Kind::Line(center, axis_point));
    let before = editor.doc.encode().unwrap();

    editor.select_only(center);
    editor.mark_center_selected();
    editor.select_only(mirror);
    editor.mark_mirror_selected();
    assert_eq!(editor.doc.encode().unwrap(), before);

    editor.select_only(source);
    editor.rotation_angle = 90.;
    editor.rotate_selected();
    assert!(editor.doc.objects.last().unwrap().kind == Kind::RotatedPoint(source, center, 90.));

    editor.select_only(source);
    editor.dilation_factor = 2.;
    editor.dilate_selected();
    assert!(editor.doc.objects.last().unwrap().kind == Kind::ScaledPoint(source, center, 2.));

    editor.select_only(source);
    editor.reflect_selected();
    assert!(editor.doc.objects.last().unwrap().kind == Kind::ReflectedPoint(source, mirror));
    assert!(Document::from_bytes(&editor.doc.encode().unwrap(), None).is_ok());
}

#[test]
fn marked_vector_drives_dynamic_translation() {
    let mut editor = Editor::new(Document::new());
    let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = editor.doc.add(Kind::Free(Point { x: 220., y: 130. }));
    let source = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let before = editor.doc.encode().unwrap();

    editor.select_only(start);
    editor.selection_order.push(end);
    editor.doc.selected.insert(end);
    editor.mark_vector_selected();
    assert_eq!(editor.marked_vector, Some((start, end)));
    assert!(editor.selection_order.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);

    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::VectorTranslatedPoint(source, start, end)
    );
    assert_eq!(editor.doc.position(image), Some(Point { x: 170., y: 210. }));
    assert!(editor.doc.move_point(end, Point { x: 250., y: 130. }));
    assert_eq!(editor.doc.position(image), Some(Point { x: 200., y: 210. }));
}

#[test]
fn marked_distance_drives_dynamic_polar_translation() {
    let mut editor = Editor::new(Document::new());
    let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let source = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
    let distance = editor.doc.add(Kind::PointDistance(start, end));
    let before = editor.doc.encode().unwrap();

    editor.select_only(distance);
    editor.mark_distance_selected();
    assert_eq!(editor.marked_distance, Some(distance));
    assert_eq!(editor.translation_mode, TranslationMode::MarkedDistance);
    assert!(editor.selection_order.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);

    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::MarkedDistanceTranslatedPoint(source, distance, 90.)
    );
    assert_eq!(editor.doc.position(image), Some(Point { x: 100., y: 120. }));
    assert!(editor.doc.move_point(end, Point { x: 300., y: 120. }));
    let moved = editor.doc.position(image).unwrap();
    assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 70.).abs() < 1e-9);
}
#[test]
fn marked_distance_translates_selected_segment_and_circle() {
    let mut editor = Editor::new(Document::new());
    let measure_start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let measure_end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let source_start = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
    let source_end = editor.doc.add(Kind::Free(Point { x: 200., y: 220. }));
    let distance = editor
        .doc
        .add(Kind::PointDistance(measure_start, measure_end));
    let segment = editor.doc.add(Kind::Segment(source_start, source_end));
    let circle = editor.doc.add(Kind::Circle(source_start, source_end));
    editor.marked_distance = Some(distance);
    editor.translation_mode = TranslationMode::MarkedDistance;

    editor.select_only(segment);
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::MarkedDistanceTranslatedSegment(segment, distance, 90.)
    );
    editor.select_only(circle);
    editor.translate_selected();
    let translated_circle = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::MarkedDistanceTranslatedCircle(circle, distance, 90.)
    );
    assert!(
        editor
            .doc
            .move_point(measure_end, Point { x: 300., y: 120. })
    );
    let (center, _) = editor.doc.circle_points(translated_circle).unwrap();
    assert!((center.y - 70.).abs() < 1e-9);
}

#[test]
fn two_marked_distances_drive_rectangular_translation() {
    let mut editor = Editor::new(Document::new());
    let h_start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let h_end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let v_start = editor.doc.add(Kind::Free(Point { x: 150., y: 180. }));
    let v_end = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
    let source = editor.doc.add(Kind::Free(Point { x: 100., y: 210. }));
    let horizontal = editor.doc.add(Kind::PointDistance(h_start, h_end));
    let vertical = editor.doc.add(Kind::PointDistance(v_start, v_end));
    let before = editor.doc.encode().unwrap();

    editor.select_only(horizontal);
    editor.doc.selected.insert(vertical);
    editor.selection_order.push(vertical);
    editor.mark_distance_selected();
    assert_eq!(
        editor.marked_rectangular_distances,
        Some((horizontal, vertical))
    );
    assert_eq!(editor.translation_mode, TranslationMode::RectangularMarked);
    assert!(editor.selection_order.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);

    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
    );
    assert_eq!(editor.doc.position(image), Some(Point { x: 200., y: 160. }));
}

#[test]
fn fixed_rectangular_translation_uses_signed_offsets() {
    let mut editor = Editor::new(Document::new());
    let source = editor.doc.add(Kind::Free(Point { x: 100., y: 210. }));
    editor.select_only(source);
    editor.translation_mode = TranslationMode::FixedRectangular;
    editor.translation_horizontal = 2.;
    editor.translation_vertical = -0.5;
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::FixedRectangularTranslatedPoint(
            source,
            37.79527559055118,
            2.,
            37.79527559055118,
            -0.5
        )
    );
    let translated = editor.doc.position(image).unwrap();
    assert!((translated.x - 175.59055118110236).abs() < 1e-9);
    assert!((translated.y - 228.89763779527559).abs() < 1e-9);
}

#[test]
fn mixed_rectangular_translation_uses_the_other_marked_axis() {
    let mut editor = Editor::new(Document::new());
    let h_start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let h_end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let v_start = editor.doc.add(Kind::Free(Point { x: 150., y: 180. }));
    let v_end = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
    let source = editor.doc.add(Kind::Free(Point { x: 100., y: 210. }));
    let horizontal = editor.doc.add(Kind::PointDistance(h_start, h_end));
    let vertical = editor.doc.add(Kind::PointDistance(v_start, v_end));
    editor.marked_rectangular_distances = Some((horizontal, vertical));
    editor.translation_horizontal = 2.;
    editor.translation_vertical = -0.5;

    editor.select_only(source);
    editor.translation_mode = TranslationMode::RectangularHorizontalFixed;
    editor.translate_selected();
    let fixed_x = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[fixed_x - 1].kind,
        Kind::MixedRectangularTranslatedPoint(source, vertical, 2., true)
    );

    editor.select_only(source);
    editor.translation_mode = TranslationMode::RectangularVerticalFixed;
    editor.translate_selected();
    let fixed_y = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[fixed_y - 1].kind,
        Kind::MixedRectangularTranslatedPoint(source, horizontal, -0.5, false)
    );
    assert!(editor.doc.move_point(h_end, Point { x: 300., y: 120. }));
    assert!(editor.doc.move_point(v_end, Point { x: 150., y: 250. }));
    let fixed_x_position = editor.doc.position(fixed_x).unwrap();
    let fixed_y_position = editor.doc.position(fixed_y).unwrap();
    assert!((fixed_x_position.x - (100. + 2. * 37.79527559055118)).abs() < 1e-9);
    assert_eq!(fixed_x_position.y, 140.);
    assert_eq!(fixed_y_position.x, 250.);
    assert!((fixed_y_position.y - (210. + 0.5 * 37.79527559055118)).abs() < 1e-9);
}

#[test]
fn fixed_polar_translation_accepts_a_segment() {
    let mut editor = Editor::new(Document::new());
    let start = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let end = editor.doc.add(Kind::Free(Point { x: 200., y: 200. }));
    let segment = editor.doc.add(Kind::Segment(start, end));
    editor.select_only(segment);
    editor.translation_mode = TranslationMode::FixedPolar;
    editor.translation_angle = 90.;
    editor.translation_distance = 1.;
    editor.translate_selected();
    let translated = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[translated - 1].kind,
        Kind::PolarTranslatedSegment(segment, 90., 37.79527559055118, 1.)
    );
    let (a, b, mode) = editor.doc.line_points(translated).unwrap();
    assert_eq!(mode, sketchpad_engine::document::LineMode::Segment);
    assert!((a.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert!((b.y - (200. - 37.79527559055118)).abs() < 1e-9);
}
#[test]
fn fixed_polar_circle_and_rectangular_segment_translation() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let radius_point = editor.doc.add(Kind::Free(Point { x: 150., y: 200. }));
    let circle = editor.doc.add(Kind::Circle(center, radius_point));
    let end = editor.doc.add(Kind::Free(Point { x: 200., y: 200. }));
    let segment = editor.doc.add(Kind::Segment(center, end));

    editor.select_only(circle);
    editor.translation_mode = TranslationMode::FixedPolar;
    editor.translate_selected();
    let translated_circle = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[translated_circle - 1].kind,
        Kind::PolarTranslatedCircle(circle, 90., 37.79527559055118, 1.)
    );
    let (translated_center, radius) = editor.doc.circle_points(translated_circle).unwrap();
    assert!((translated_center.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert_eq!(radius, 50.);

    editor.select_only(segment);
    editor.translation_mode = TranslationMode::FixedRectangular;
    editor.translation_horizontal = 2.;
    editor.translation_vertical = -0.5;
    editor.translate_selected();
    let translated_segment = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[translated_segment - 1].kind,
        Kind::FixedRectangularTranslatedSegment(
            segment,
            37.79527559055118,
            2.,
            37.79527559055118,
            -0.5
        )
    );
    let (a, b, _) = editor.doc.line_points(translated_segment).unwrap();
    assert!((a.x - (100. + 2. * 37.79527559055118)).abs() < 1e-9);
    assert!((b.y - (200. + 0.5 * 37.79527559055118)).abs() < 1e-9);
}
#[test]
fn rectangular_circle_and_vector_shape_translation() {
    let mut editor = Editor::new(Document::new());
    let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = editor.doc.add(Kind::Free(Point { x: 220., y: 130. }));
    let segment = editor.doc.add(Kind::Segment(start, end));
    let center = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let radius_point = editor.doc.add(Kind::Free(Point { x: 150., y: 200. }));
    let circle = editor.doc.add(Kind::Circle(center, radius_point));
    editor.marked_vector = Some((start, end));

    editor.select_only(segment);
    editor.translation_mode = TranslationMode::MarkedVector;
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::VectorTranslatedSegment(segment, start, end)
    );

    editor.select_only(circle);
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::VectorTranslatedCircle(circle, start, end)
    );

    editor.select_only(circle);
    editor.translation_mode = TranslationMode::FixedRectangular;
    editor.translate_selected();
    let translated = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::FixedRectangularTranslatedCircle(
            circle,
            37.79527559055118,
            1.,
            37.79527559055118,
            1.
        )
    );
    assert!(editor.doc.circle_points(translated).is_some());
}

#[test]
fn marked_and_mixed_rectangular_translation_accepts_segments_and_circles() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let b = editor.doc.add(Kind::Free(Point { x: 150., y: 200. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    let distance = editor.doc.add(Kind::PointDistance(a, b));
    editor.marked_rectangular_distances = Some((distance, distance));
    for mode in [
        TranslationMode::RectangularMarked,
        TranslationMode::RectangularHorizontalFixed,
        TranslationMode::RectangularVerticalFixed,
    ] {
        editor.translation_mode = mode;
        for source in [segment, circle] {
            editor.select_only(source);
            let before = editor.doc.objects.len();
            editor.translate_selected();
            let id = editor.doc.objects.len();
            assert_eq!(id, before + 1, "{}", editor.status);
            assert!(editor.doc.line_points(id).is_some() || editor.doc.circle_points(id).is_some());
        }
    }
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn typed_length_parameter_can_be_marked_and_drives_editor_translation() {
    let doc = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/numeric_typed_parameters_native_verified.gsp"
        ),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let source = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
    let before = editor.doc.encode().unwrap();
    editor.select_only(2);
    editor.mark_distance_selected();
    assert_eq!(editor.marked_distance, Some(2));
    assert_eq!(editor.translation_mode, TranslationMode::MarkedDistance);
    assert!(editor.selection_order.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);
    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::MarkedDistanceTranslatedPoint(source, 2, 90.)
    );
    assert!(
        editor.doc.position(image).unwrap().distance(Point {
            x: 100.,
            y: 220. - 37.79527559055118,
        }) < 1e-9
    );
    let baseline = editor.doc.encode().unwrap();
    editor.doc.set_parameter_value(2, 2.).unwrap();
    assert!(
        editor.doc.position(image).unwrap().distance(Point {
            x: 100.,
            y: 220. - 2. * 37.79527559055118,
        }) < 1e-9
    );
    editor.doc.verify_roundtrip().unwrap();
    let edited = editor.doc.encode().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), baseline);
    editor.doc.redo();
    assert_eq!(editor.doc.encode().unwrap(), edited);
}

#[test]
fn scalar_parameter_can_be_marked_as_centimeters_and_translated_with_sign() {
    let mut doc = Document::new();
    let distance = doc.add_named_parameter("distance", -3., (10, 20)).unwrap();
    let source = doc.add(Kind::Free(Point { x: 350., y: 220. }));
    let mut editor = Editor::new(doc);
    let before = editor.doc.encode().unwrap();
    editor.select_only(distance);
    editor.mark_distance_selected();
    assert_eq!(editor.marked_distance, Some(distance));
    assert_eq!(editor.doc.encode().unwrap(), before);
    editor.translation_angle = 0.;
    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::MarkedDistanceTranslatedPoint(source, distance, 0.)
    );
    assert!(
        editor.doc.position(image).unwrap().distance(Point {
            x: 350. - 3. * 37.79527559055118,
            y: 220.
        }) < 1e-9
    );
    editor.doc.set_parameter_value(distance, 0.).unwrap();
    assert_eq!(editor.doc.position(image), editor.doc.position(source));
    editor.doc.verify_roundtrip().unwrap();
}
