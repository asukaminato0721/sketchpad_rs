use super::*;

#[test]
fn reflected_polygon_imports_and_tracks_mirror() {
    for (path, polygon, mirror, image) in [
        (
            "../../tests/fixtures/reflection_polygon_original.gsp",
            7,
            3,
            8,
        ),
        ("../../tests/fixtures/reflection.gsp", 9, 4, 13),
    ] {
        let bytes = fs::read(path).unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(
            native.objects[image - 1].kind,
            Kind::ReflectedPolygon(polygon, mirror)
        );
        assert_eq!(native.encode().unwrap(), bytes);
        assert_eq!(
            native.objects[image - 1].visual_style,
            native.objects[polygon - 1].visual_style
        );
    }

    let mut native = Document::open(Path::new(
        "../../tests/fixtures/reflection_polygon_original.gsp",
    ))
    .unwrap();
    let before = native.polygon_points(8).unwrap();
    assert_eq!(
        before,
        vec![
            Point { x: 100., y: 130. },
            Point { x: 60., y: 180. },
            Point { x: 100., y: 230. },
        ]
    );
    assert!(native.move_point(2, Point { x: 170., y: 230. }));
    assert_ne!(native.polygon_points(8).unwrap(), before);
    assert_eq!(
        native.encode().unwrap(),
        fs::read("../../tests/fixtures/reflection_polygon_mirror_dragged_by_wine.gsp").unwrap()
    );

    let mut created = Document::new();
    let mirror_a = created.add(Kind::Free(Point { x: 150., y: 100. }));
    let mirror_b = created.add(Kind::Free(Point { x: 150., y: 230. }));
    let mirror = created.add(Kind::Segment(mirror_a, mirror_b));
    let a = created.add(Kind::Free(Point { x: 200., y: 130. }));
    let b = created.add(Kind::Free(Point { x: 240., y: 180. }));
    let c = created.add(Kind::Free(Point { x: 200., y: 230. }));
    let polygon = created.add(Kind::Polygon(vec![a, b, c]));
    let image = created.add(Kind::ReflectedPolygon(polygon, mirror));
    assert_eq!(
        created.objects[image - 1].visual_style,
        created.objects[polygon - 1].visual_style
    );
    let encoded = created.encode().unwrap();
    let marker = records(&encoded)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x2328)
        .unwrap();
    assert_eq!(u32_at(&encoded, marker.data.start), image as u32);
    assert!(created.move_point(mirror_b, Point { x: 170., y: 230. }));
    let moved = created.encode().unwrap();
    let marker = records(&moved)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x2328)
        .unwrap();
    assert_eq!(u32_at(&moved, marker.data.start), mirror_b as u32);
    let reopened = Document::from_bytes(&moved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.polygon_points(image),
        created.polygon_points(image)
    );
}
#[test]
fn transformed_polygon_fixtures_import_and_track_sources() {
    for (path, image, expected) in [
        (
            "../../tests/fixtures/fixed_rotation_polygon_original.gsp",
            6,
            Kind::RotatedPolygon(5, 1, 90.),
        ),
        (
            "../../tests/fixtures/angle_rotation_polygon_original.gsp",
            9,
            Kind::AngleRotatedPolygon(8, 1, 2, 3, 4),
        ),
        (
            "../../tests/fixtures/fixed_dilation_polygon_original.gsp",
            6,
            Kind::ScaledPolygon(5, 1, 0.5),
        ),
        (
            "../../tests/fixtures/ratio_dilation_polygon_original.gsp",
            9,
            Kind::RatioScaledPolygon(8, 1, 2, 3, 4),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[image - 1].kind, expected);
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        assert_eq!(
            document.objects[image - 1].visual_style,
            document.objects[image - 2].visual_style
        );
        let before = document.polygon_points(image).unwrap();
        assert!(document.move_point(if image == 6 { 2 } else { 4 }, Point { x: 175., y: 170. }));
        assert_ne!(document.polygon_points(image).unwrap(), before);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn vector_translated_polygon_imports_and_tracks_vector() {
    let bytes = fs::read("../../tests/fixtures/vector_translation_polygon_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[7].kind,
        Kind::VectorTranslatedPolygon(7, 1, 2)
    );
    assert_eq!(document.encode().unwrap(), bytes);
    let source_header = document.objects[6].header_offset.unwrap();
    let image_header = document.objects[7].header_offset.unwrap();
    let mut generic = bytes.clone();
    generic[source_header + 8..source_header + 10].copy_from_slice(&15_u16.to_le_bytes());
    generic[image_header + 8..image_header + 10].copy_from_slice(&15_u16.to_le_bytes());
    let generic_document = Document::from_bytes(&generic, None).unwrap();
    assert!(!generic_document.partial_view);
    assert_eq!(
        generic_document.objects[7].kind,
        Kind::VectorTranslatedPolygon(7, 1, 2)
    );
    assert_eq!(generic_document.encode().unwrap(), generic);

    let mut mismatched = bytes.clone();
    mismatched[image_header + 8..image_header + 10].copy_from_slice(&15_u16.to_le_bytes());
    let mismatched = Document::from_bytes(&mismatched, None).unwrap();
    assert!(mismatched.partial_view);
    assert_eq!(mismatched.objects[7].kind, Kind::Unsupported(16));
    assert_eq!(
        document.polygon_points(8),
        Some(vec![
            Point { x: 170., y: 210. },
            Point { x: 210., y: 210. },
            Point { x: 190., y: 250. },
        ])
    );
    assert!(document.move_point(2, Point { x: 250., y: 130. }));
    assert_eq!(
        document.polygon_points(8).unwrap()[0],
        Point { x: 200., y: 210. }
    );
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn fixed_translated_polygon_fixtures_import_and_track_source() {
    for (path, expected) in [
        (
            "../../tests/fixtures/polar_translation_polygon_original.gsp",
            Kind::PolarTranslatedPolygon(7, 90., 37.79527559055118, 1.),
        ),
        (
            "../../tests/fixtures/rectangular_translation_polygon_original.gsp",
            Kind::FixedRectangularTranslatedPolygon(
                7,
                37.79527559055118,
                2.,
                37.79527559055118,
                -0.5,
            ),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[7].kind, expected, "{path}");
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        let before = document.polygon_points(8).unwrap();
        assert!(document.move_point(5, Point { x: 160., y: 200. }));
        assert_ne!(document.polygon_points(8).unwrap(), before);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn marked_distance_polygon_imports_and_tracks_measurement() {
    let bytes =
        fs::read("../../tests/fixtures/marked_distance_polygon_translation_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[7].kind,
        Kind::MarkedDistanceTranslatedPolygon(7, 6, 90.)
    );
    assert_eq!(document.encode().unwrap(), bytes);
    let before = document.polygon_points(8).unwrap();
    assert_eq!(before[0], Point { x: 100., y: 90. });
    assert!(document.move_point(2, Point { x: 300., y: 120. }));
    let moved = document.polygon_points(8).unwrap()[0];
    assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 40.).abs() < 1e-9);
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn polygon_area_imports_and_tracks_vertices() {
    let bytes = fs::read("../../tests/fixtures/polygon_area_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.objects[5].kind, Kind::PolygonArea(5));
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(
        (document.polygon_area_cm2(6).unwrap() - 800. / 37.79527559055118_f64.powi(2)).abs()
            < 1e-10
    );
    assert!(document.move_point(4, Point { x: 190., y: 250. }));
    assert!(
        (document.polygon_area_cm2(6).unwrap() - 1200. / 37.79527559055118_f64.powi(2)).abs()
            < 1e-10
    );
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn transformed_polygon_measurements_import_and_track_source() {
    for (path, expected) in [
        (
            "../../tests/fixtures/transformed_polygon_perimeter_original.gsp",
            Kind::PolygonPerimeter(6),
        ),
        (
            "../../tests/fixtures/transformed_polygon_area_original.gsp",
            Kind::PolygonArea(6),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[6].kind, expected, "{path}");
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        let before = if matches!(expected, Kind::PolygonArea(_)) {
            document.polygon_area_cm2(7).unwrap()
        } else {
            document.perimeter_cm(7).unwrap()
        };
        assert!(document.move_point(4, Point { x: 190., y: 250. }));
        let after = if matches!(expected, Kind::PolygonArea(_)) {
            document.polygon_area_cm2(7).unwrap()
        } else {
            document.perimeter_cm(7).unwrap()
        };
        assert_ne!(before, after);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn rectangular_marked_polygon_imports_and_tracks_both_distances() {
    let bytes = fs::read("../../tests/fixtures/rectangular_marked_polygon_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[11].kind,
        Kind::RectangularMarkedTranslatedPolygon(11, 6, 7)
    );
    assert_eq!(document.encode().unwrap(), bytes);
    assert_eq!(
        document.polygon_points(12).unwrap()[0],
        Point { x: 200., y: 140. }
    );
    assert!(document.move_point(2, Point { x: 300., y: 120. }));
    assert!(document.move_point(4, Point { x: 150., y: 250. }));
    assert_eq!(
        document.polygon_points(12).unwrap()[0],
        Point { x: 250., y: 120. }
    );
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn mixed_rectangular_polygon_fixtures_import_and_track_distance() {
    for (path, expected) in [
        (
            "../../tests/fixtures/rectangular_mixed_polygon_horizontal_original.gsp",
            Kind::MixedRectangularTranslatedPolygon(11, 7, 2., true),
        ),
        (
            "../../tests/fixtures/rectangular_mixed_polygon_vertical_original.gsp",
            Kind::MixedRectangularTranslatedPolygon(11, 6, 2., false),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[11].kind, expected, "{path}");
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        let before = document.polygon_points(12).unwrap();
        let id = if transform_is(
            &expected,
            NativeTransformFlavor::FixedHorizontal,
            &[ElementType::Polygon],
        ) {
            4
        } else {
            2
        };
        assert!(document.move_point(id, Point { x: 300., y: 250. }));
        assert_ne!(document.polygon_points(12).unwrap(), before);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn marked_ratio_dilation_imports_and_tracks_ratio_points() {
    let bytes = fs::read("../../tests/fixtures/ratio_dilation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[5].kind,
        Kind::RatioScaledPoint(5, 1, 2, 3, 4)
    );
    assert_eq!(native.position(6), Some(Point { x: 120., y: 120. }));
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 120. }));
    let origin = created.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = created.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = created.add(Kind::Free(Point { x: 205., y: 100. }));
    let source = created.add(Kind::Free(Point { x: 140., y: 120. }));
    let image = created.add(Kind::RatioScaledPoint(source, center, origin, base, target));
    assert_eq!(created.position(image), Some(Point { x: 120., y: 120. }));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[5].kind, created.objects[5].kind);
    assert!(reopened.move_point(target, Point { x: 255., y: 100. }));
    assert_eq!(reopened.position(image), Some(Point { x: 160., y: 120. }));
    assert!(reopened.move_point(target, Point { x: 155., y: 100. }));
    assert_eq!(reopened.position(image), Some(Point { x: 80., y: 120. }));
    assert!(
        !Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn marked_ratio_circle_dilation_imports_and_tracks_ratio_points() {
    let bytes = fs::read("../../tests/fixtures/ratio_dilation_circle_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[7].kind,
        Kind::RatioScaledCircle(7, 1, 2, 3, 4)
    );
    assert_eq!(
        native.circle_points(8),
        Some((Point { x: 120., y: 120. }, 10.))
    );
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 120. }));
    let origin = created.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = created.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = created.add(Kind::Free(Point { x: 205., y: 100. }));
    let source_center = created.add(Kind::Free(Point { x: 140., y: 120. }));
    let radius_point = created.add(Kind::Free(Point { x: 160., y: 120. }));
    let source = created.add(Kind::Circle(source_center, radius_point));
    let image = created.add(Kind::RatioScaledCircle(
        source, center, origin, base, target,
    ));
    assert_eq!(
        created.circle_points(image),
        Some((Point { x: 120., y: 120. }, 10.))
    );
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
    assert!(reopened.move_point(target, Point { x: 255., y: 100. }));
    assert_eq!(
        reopened.circle_points(image),
        Some((Point { x: 160., y: 120. }, 30.))
    );
    assert!(reopened.move_point(target, Point { x: 155., y: 100. }));
    assert_eq!(
        reopened.circle_points(image),
        Some((Point { x: 80., y: 120. }, 10.))
    );
    assert!(
        !Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn native_polygon_order_is_signed_presentation_preserved_by_edits_and_copy() {
    let original =
        include_bytes!("../../../../../tests/fixtures/polygon_order_original_native.gsp");
    let swapped = include_bytes!("../../../../../tests/fixtures/polygon_order_swapped_native.gsp");
    let negative =
        include_bytes!("../../../../../tests/fixtures/polygon_order_negative_native.gsp");
    for (bytes, ranks) in [
        (original.as_slice(), [0, 1]),
        (swapped.as_slice(), [1, 0]),
        (negative.as_slice(), [0, -1]),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[3].interior_order, Some(ranks[0]));
        assert_eq!(doc.objects[5].interior_order, Some(ranks[1]));
        assert_eq!(doc.polygon_points(4), doc.polygon_points(6));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let mut doc = Document::from_bytes(original, None).unwrap();
    doc.objects.edit(3).interior_order = Some(1);
    doc.objects.edit(5).interior_order = Some(0);
    doc.dirty = true;
    assert_eq!(doc.encode().unwrap(), swapped.as_slice());
    doc.objects.edit(3).interior_order = Some(0);
    doc.objects.edit(5).interior_order = Some(-1);
    assert_eq!(doc.encode().unwrap(), negative.as_slice());
    doc.selected.insert(6);
    let clipboard = doc.copy_selected().unwrap();
    let copied = *doc.paste(&clipboard).unwrap().last().unwrap();
    assert_eq!(doc.objects[copied - 1].interior_order, Some(-1));
    let kind = &doc.objects[copied - 1].kind;
    assert!(transform_is(
        kind,
        NativeTransformFlavor::NumericDilation,
        &[ElementType::Polygon]
    ));
    let source = kind.transform_definition().unwrap().source();
    assert_eq!(doc.objects[source - 1].interior_order, Some(0));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[copied - 1].interior_order, Some(-1));
    assert_eq!(reopened.objects[source - 1].interior_order, Some(0));
    let native_copy = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/polygon_order_copied_native.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(native_copy.objects[10].interior_order, Some(0));
    assert_eq!(native_copy.objects[11].interior_order, Some(1));
}

#[test]
fn native_calculated_rotation_of_vector_polygon_keeps_count_rank_and_live_sources() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/polygon_rotation_of_vector_corpus_native.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.objects[21].kind,
        Kind::VectorTranslatedPolygon(7, 15, 21)
    );
    assert_eq!(
        doc.objects[22].kind,
        Kind::CalculatedAngleRotatedPolygon(22, 21, 17)
    );
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, 23),
        Some(4)
    );
    assert_eq!(doc.objects[21].interior_order, Some(1));
    assert_eq!(doc.objects[22].interior_order, Some(2));
    assert_eq!(doc.encode().unwrap(), bytes);
    let before = doc.polygon_points(23).unwrap();
    doc.set_parameter_value(6, 2.).unwrap();
    assert_ne!(doc.polygon_points(23).unwrap(), before);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.polygon_points(23), doc.polygon_points(23));
    assert_eq!(reopened.objects[22].interior_order, Some(2));
    doc.selected.insert(23);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = *doc.paste(&clipboard).unwrap().last().unwrap();
    assert_eq!(doc.objects[pasted - 1].interior_order, Some(2));
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, pasted),
        Some(4)
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.polygon_points(pasted), doc.polygon_points(pasted));
}

#[test]
fn polygon_vertex_count_rejects_unsupported_cyclic_and_cross_page_chains() {
    let mut doc = Document::new();
    for (x, y) in [(0., 0.), (100., 0.), (0., 100.)] {
        doc.add(Kind::Free(Point { x, y }));
    }
    doc.add(Kind::Polygon(vec![1, 2, 3]));
    let translated = doc.add(Kind::VectorTranslatedPolygon(4, 1, 2));
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, translated),
        Some(3)
    );
    let valid = doc.objects.clone();
    doc.objects.edit(4).kind = Kind::VectorTranslatedPolygon(5, 1, 2);
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, translated),
        None
    );
    doc.objects = valid.clone();
    doc.objects.edit(3).page = 1;
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, translated),
        None
    );
    doc.objects = valid.clone();
    doc.objects.edit(3).deleted = true;
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, translated),
        None
    );
    doc.objects = valid;
    doc.objects.edit(3).kind = Kind::Unsupported(8);
    assert_eq!(
        super::super::kind::polygon_vertex_count(&doc.objects, translated),
        None
    );
}

#[test]
fn legacy_clock_polygon_and_disk_headers_keep_implicit_style_and_rank() {
    let clock = include_bytes!("../../../../../tests/fixtures/legacy_clock_original.gsp");
    let source = Document::from_bytes(clock, None).unwrap();
    assert_eq!(source.objects.len(), 161);
    assert_eq!(source.objects[78].interior_order, None);
    for (id, circle) in [(148, 146), (149, 147)] {
        assert_eq!(source.objects[id - 1].kind, Kind::CircleInterior(circle));
        assert_eq!(source.objects[id - 1].interior_order, None);
        assert!(source.circle_interior_geometry(id).is_some());
        assert_eq!(
            records(clock)
                .unwrap()
                .iter()
                .find(|record| record.data.start == source.objects[id - 1].header_offset.unwrap())
                .unwrap()
                .data
                .len(),
            16
        );
        assert!(matches!(
            source.objects[id - 1].visual_style,
            Some(VisualStyle::Polygon {
                opacity: 0xbfff,
                ..
            })
        ));
    }
    assert_eq!(source.encode().unwrap(), clock);
    // Actual Clock79 header and reference record, rebound only to four known
    // points so its dirty writer can be verified independently of old families.
    let bytes = include_bytes!("../../../../../tests/fixtures/legacy_polygon_rank_original.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects[4].kind, Kind::Polygon(vec![1, 2, 3, 4]));
    assert_eq!(doc.objects[4].interior_order, None);
    let header = doc.objects[4].header_offset.unwrap();
    let raw_header = bytes[header..header + 16].to_vec();
    assert!(doc.move_point(1, Point { x: 110., y: 110. }));
    let edited = doc.encode().unwrap();
    assert_eq!(
        edited,
        include_bytes!(
            "../../../../../tests/fixtures/legacy_polygon_rank_edited_native_verified.gsp"
        )
    );
    assert_eq!(&edited[header..header + 16], raw_header);
    let rs = records(&edited).unwrap();
    assert_eq!(
        rs.iter()
            .find(|record| record.data.start == header)
            .unwrap()
            .data
            .len(),
        16
    );
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.objects[4].interior_order, None);
    assert_eq!(reopened.polygon_points(5), doc.polygon_points(5));
    doc.objects.edit(4).interior_order = Some(0);
    assert!(doc.encode().is_err());
}

#[test]
fn authored_polygon_transform_chains_preserve_native_subtypes_and_live_geometry() {
    for bytes in [
        include_bytes!(
            "../../../../../tests/fixtures/polygon_transform_chains_generated_verified.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/polygon_transform_chains_edited_verified.gsp"
        )
        .as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(
            doc.objects[8].kind,
            Kind::CalculatedRatioScaledPolygon(8, 1, 6)
        );
        assert_eq!(doc.objects[9].kind, Kind::VectorTranslatedPolygon(9, 1, 2));
        assert_eq!(
            doc.objects[10].kind,
            Kind::CalculatedAngleRotatedPolygon(10, 1, 7)
        );
        assert_eq!(
            doc.objects[11].kind,
            Kind::CalculatedRatioScaledPolygon(9, 1, 6)
        );
        assert_eq!(doc.encode().unwrap(), bytes);
        for id in 8..=12 {
            assert_eq!(
                super::super::kind::polygon_vertex_count(&doc.objects, id),
                Some(4)
            );
            assert_eq!(doc.polygon_points(id).unwrap().len(), 4);
            assert_eq!(doc.objects[id - 1].interior_order, Some(1));
        }
        let before = doc.polygon_points(11).unwrap();
        doc.set_parameter_value(6, 0.6).unwrap();
        assert_ne!(doc.polygon_points(11).unwrap(), before);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.polygon_points(11), doc.polygon_points(11));
        doc.selected.insert(11);
        doc.selected.insert(12);
        let clipboard = doc.copy_selected().unwrap();
        let copied = *doc.paste(&clipboard).unwrap().last().unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.polygon_points(copied), doc.polygon_points(copied));
        assert_eq!(reopened.objects[copied - 1].interior_order, Some(1));
        doc.selected.clear();
        doc.selected.insert(6);
        let removed = doc.clear_selected().unwrap();
        assert!(removed.contains(&11));
        assert!(removed.contains(&12));
    }
}

#[test]
fn supported_polygon_transform_families_accept_native_signed_drawing_orders() {
    for path in [
        "reflection_polygon_original.gsp",
        "fixed_rotation_polygon_original.gsp",
        "angle_rotation_polygon_original.gsp",
        "fixed_dilation_polygon_original.gsp",
        "ratio_dilation_polygon_original.gsp",
        "vector_translation_polygon_original.gsp",
        "polar_translation_polygon_original.gsp",
        "rectangular_translation_polygon_original.gsp",
        "marked_distance_polygon_translation_original.gsp",
        "rectangular_marked_polygon_original.gsp",
        "rectangular_mixed_polygon_horizontal_original.gsp",
        "rectangular_mixed_polygon_vertical_original.gsp",
    ] {
        let mut doc = Document::open(Path::new(&format!("../../tests/fixtures/{path}"))).unwrap();
        let polygons = doc
            .objects
            .iter()
            .filter(|o| o.kind.is_polygon())
            .map(|o| (o.id, o.kind.clone(), doc.polygon_points(o.id)))
            .collect::<Vec<_>>();
        assert!(polygons.len() >= 2, "{path}");
        for (index, (id, _, _)) in polygons.iter().enumerate() {
            doc.objects.edit(*id - 1).interior_order = Some(if index == 0 { 7 } else { -3 });
        }
        doc.dirty = true;
        let saved = doc.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view, "{path}");
        for (id, kind, points) in polygons {
            assert_eq!(reopened.objects[id - 1].kind, kind, "{path}");
            assert_eq!(
                reopened.objects[id - 1].interior_order,
                doc.objects[id - 1].interior_order,
                "{path}"
            );
            assert_eq!(reopened.polygon_points(id), points, "{path}");
        }
        assert_eq!(reopened.encode().unwrap(), saved, "{path}");
    }
}

#[test]
fn polygon_opacity_property_is_undoable_and_survives_save() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 100. }));
    let c = document.add(Kind::Free(Point { x: 150., y: 180. }));
    let polygon = document.add(Kind::Polygon(vec![a, b, c]));

    assert!(document.set_object_polygon_opacity(polygon, 49151).unwrap());
    assert!(matches!(
        document.objects[polygon - 1].visual_style,
        Some(VisualStyle::Polygon { opacity: 49151, .. })
    ));
    document.undo();
    assert!(matches!(
        document.objects[polygon - 1].visual_style,
        Some(VisualStyle::Polygon { opacity: 32767, .. })
    ));
    document.redo();

    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(matches!(
        reopened.objects[polygon - 1].visual_style,
        Some(VisualStyle::Polygon { opacity: 49151, .. })
    ));
}
