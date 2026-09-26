use super::*;

#[test]
fn polygon_interior_reflection_command() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 150., y: 100. }));
    let b = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
    let mirror = editor.doc.add(Kind::Segment(a, b));
    let p = editor.doc.add(Kind::Free(Point { x: 200., y: 130. }));
    let q = editor.doc.add(Kind::Free(Point { x: 240., y: 180. }));
    let r = editor.doc.add(Kind::Free(Point { x: 200., y: 230. }));
    let polygon = editor.doc.add(Kind::Polygon(vec![p, q, r]));
    editor.marked_mirror = Some(mirror);
    editor.select_only(polygon);
    editor.reflect_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::ReflectedPolygon(polygon, mirror)
    );
    assert_eq!(
        editor
            .doc
            .polygon_points(editor.doc.objects.len())
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn transformed_circles_are_selectable_on_canvas() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let source_center = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
    let radius_point = editor.doc.add(Kind::Free(Point { x: 200., y: 100. }));
    let circle = editor.doc.add(Kind::Circle(source_center, radius_point));
    let rotated = editor.doc.add(Kind::RotatedCircle(circle, center, 90.));
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 150., y: 100. },
    };
    assert_eq!(
        editor.hit(&viewport, viewport.screen(Point { x: 120., y: 20. })),
        Some(rotated)
    );
}

#[test]
fn polygon_locus_interior_is_selectable_on_canvas() {
    let base = include_bytes!("../../../tests/fixtures/locus_polygon_target_base.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    let locus = document.construct_locus(4, 7).unwrap();
    let editor = Editor::new(document);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 210., y: 140. },
    };
    assert_eq!(
        editor.hit(&viewport, viewport.screen(Point { x: 140., y: 150. })),
        Some(locus)
    );
    assert_eq!(
        editor.hit(
            &viewport,
            viewport.screen(Point {
                x: 180.72,
                y: 179.9
            })
        ),
        Some(locus)
    );
}

#[test]
fn polygon_rotation_and_dilation_commands_use_live_interiors() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 100., y: 120. }));
    let first = editor.doc.add(Kind::Free(Point { x: 230., y: 160. }));
    let vertex = editor.doc.add(Kind::Free(Point { x: 230., y: 220. }));
    let last = editor.doc.add(Kind::Free(Point { x: 290., y: 220. }));
    let a = editor.doc.add(Kind::Free(Point { x: 140., y: 150. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 150. }));
    let c = editor.doc.add(Kind::Free(Point { x: 160., y: 190. }));
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    let origin = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = editor.doc.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = editor.doc.add(Kind::Free(Point { x: 205., y: 100. }));
    editor.marked_center = Some(center);
    editor.marked_angle = Some(MarkedAngle::Points([first, vertex, last]));
    editor.marked_ratio = Some([origin, base, target]);

    editor.rotation_angle = 90.;
    editor.select_only(polygon);
    editor.rotate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::RotatedPolygon(polygon, center, 90.)
    );

    editor.rotation_use_marked = true;
    editor.select_only(polygon);
    editor.rotate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::AngleRotatedPolygon(polygon, center, first, vertex, last)
    );

    editor.dilation_factor = 0.5;
    editor.select_only(polygon);
    editor.dilate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::ScaledPolygon(polygon, center, 0.5)
    );

    editor.dilation_use_marked = true;
    editor.select_only(polygon);
    editor.dilate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::RatioScaledPolygon(polygon, center, origin, base, target)
    );
    let before = editor.doc.polygon_points(image).unwrap();
    assert!(editor.doc.move_point(target, Point { x: 255., y: 100. }));
    assert_ne!(editor.doc.polygon_points(image).unwrap(), before);
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn marked_vector_translates_selected_polygon() {
    let mut editor = Editor::new(Document::new());
    let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = editor.doc.add(Kind::Free(Point { x: 220., y: 130. }));
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let b = editor.doc.add(Kind::Free(Point { x: 140., y: 200. }));
    let c = editor.doc.add(Kind::Free(Point { x: 120., y: 240. }));
    let source = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    editor.marked_vector = Some((start, end));
    editor.translation_mode = TranslationMode::MarkedVector;
    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::VectorTranslatedPolygon(source, start, end)
    );
    assert_eq!(
        editor.doc.polygon_points(image).unwrap()[0],
        Point { x: 170., y: 210. }
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );

    editor.translation_mode = TranslationMode::FixedPolar;
    editor.translation_angle = 90.;
    editor.translation_distance = 1.;
    editor.select_only(source);
    editor.translate_selected();
    let polar = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::PolarTranslatedPolygon(source, 90., 37.79527559055118, 1.)
    );
    assert!(
        (editor.doc.polygon_points(polar).unwrap()[0].y - (200. - 37.79527559055118)).abs() < 1e-9
    );

    editor.translation_mode = TranslationMode::FixedRectangular;
    editor.translation_horizontal = 2.;
    editor.translation_vertical = -0.5;
    editor.select_only(source);
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::FixedRectangularTranslatedPolygon(
            source,
            37.79527559055118,
            2.,
            37.79527559055118,
            -0.5
        )
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn marked_distance_translates_selected_polygon() {
    let mut editor = Editor::new(Document::new());
    let first = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let last = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 190. }));
    let b = editor.doc.add(Kind::Free(Point { x: 140., y: 190. }));
    let c = editor.doc.add(Kind::Free(Point { x: 120., y: 230. }));
    let distance = editor.doc.add(Kind::PointDistance(first, last));
    let source = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    editor.marked_distance = Some(distance);
    editor.translation_mode = TranslationMode::MarkedDistance;
    editor.translation_angle = 90.;
    editor.select_only(source);
    editor.translate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::MarkedDistanceTranslatedPolygon(source, distance, 90.)
    );
    assert!((editor.doc.polygon_points(image).unwrap()[0].y - 90.).abs() < 1e-9);
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn area_command_measures_selected_polygon() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
    let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
    let c = editor.doc.add(Kind::Free(Point { x: 190., y: 230. }));
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    editor.select_only(polygon);
    editor.measure_circle(CircleMeasure::Area);
    let area = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[area - 1].kind,
        Kind::PolygonArea(polygon)
    );
    assert!(
        editor
            .overlay_rows()
            .iter()
            .any(|(_, text)| text == "Area = 0.56 cm²")
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );

    let rotated = editor.doc.add(Kind::RotatedPolygon(polygon, a, 90.));
    editor.select_only(rotated);
    editor.measure_perimeter();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::PolygonPerimeter(rotated)
    );
    editor.select_only(rotated);
    editor.measure_circle(CircleMeasure::Area);
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::PolygonArea(rotated)
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn rectangular_marked_and_mixed_translate_selected_polygon() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let c = editor.doc.add(Kind::Free(Point { x: 150., y: 180. }));
    let d = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
    let horizontal = editor.doc.add(Kind::PointDistance(a, b));
    let vertical = editor.doc.add(Kind::PointDistance(c, d));
    let p = editor.doc.add(Kind::Free(Point { x: 100., y: 190. }));
    let q = editor.doc.add(Kind::Free(Point { x: 140., y: 190. }));
    let r = editor.doc.add(Kind::Free(Point { x: 120., y: 230. }));
    let source = editor.doc.add(Kind::Polygon(vec![p, q, r]));
    editor.marked_rectangular_distances = Some((horizontal, vertical));

    editor.translation_mode = TranslationMode::RectangularMarked;
    editor.select_only(source);
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical)
    );

    editor.translation_horizontal = 2.;
    editor.translation_mode = TranslationMode::RectangularHorizontalFixed;
    editor.select_only(source);
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::MixedRectangularTranslatedPolygon(source, vertical, 2., true)
    );

    editor.translation_vertical = 2.;
    editor.translation_mode = TranslationMode::RectangularVerticalFixed;
    editor.select_only(source);
    editor.translate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::MixedRectangularTranslatedPolygon(source, horizontal, 2., false)
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn marked_ratio_drives_dynamic_dilation() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 100., y: 120. }));
    let origin = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = editor.doc.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = editor.doc.add(Kind::Free(Point { x: 205., y: 100. }));
    let source = editor.doc.add(Kind::Free(Point { x: 140., y: 120. }));
    let before = editor.doc.encode().unwrap();

    editor.select_only(center);
    editor.mark_center_selected();
    editor.select_only(origin);
    editor.selection_order.extend([base, target]);
    editor.doc.selected.extend([base, target]);
    editor.mark_ratio_selected();
    assert_eq!(editor.marked_ratio, Some([origin, base, target]));
    assert!(editor.selection_order.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);

    editor.select_only(source);
    editor.dilate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::RatioScaledPoint(source, center, origin, base, target)
    );
    assert_eq!(editor.doc.position(image), Some(Point { x: 120., y: 120. }));
    assert!(editor.doc.move_point(target, Point { x: 255., y: 100. }));
    assert_eq!(editor.doc.position(image), Some(Point { x: 160., y: 120. }));

    let radius_point = editor.doc.add(Kind::Free(Point { x: 160., y: 120. }));
    let circle = editor.doc.add(Kind::Circle(source, radius_point));
    editor.select_only(circle);
    editor.dilate_selected();
    let circle_image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[circle_image - 1].kind,
        Kind::RatioScaledCircle(circle, center, origin, base, target)
    );
    assert_eq!(
        editor.doc.circle_points(circle_image),
        Some((Point { x: 160., y: 120. }, 30.))
    );
}

#[test]
fn polygon_boundary_point_command_animates_and_saves() {
    let mut editor = Editor::new(Document::new());
    let vertices = [(0., 0.), (120., 0.), (120., 60.), (0., 60.)]
        .into_iter()
        .map(|(x, y)| editor.doc.add(Kind::Free(Point { x, y })))
        .collect::<Vec<_>>();
    let polygon = editor.doc.add(Kind::Polygon(vertices));
    editor.select_only(polygon);
    editor.point_on_object();
    let point = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[point - 1].kind,
        Kind::PointOnPolygon(polygon, 0, 0.5)
    );
    editor.select_only(point);
    editor.animate_selected();
    assert_eq!(editor.animations.len(), 1);
    let before = editor.doc.position(point).unwrap();
    editor.tick_animation(0.1);
    assert_ne!(editor.doc.position(point).unwrap(), before);
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(point), editor.doc.position(point));
}
