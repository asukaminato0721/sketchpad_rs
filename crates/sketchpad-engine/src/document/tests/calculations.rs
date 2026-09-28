use super::*;

#[test]
fn polygon_perimeter_matches_original_and_updates_after_drag() {
    let original = Document::open(Path::new("../../tests/fixtures/wine_polygon_perimeter.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::Polygon(vec![1, 2, 3]));
    assert_eq!(original.objects[4].kind, Kind::PolygonPerimeter(4));
    assert!((original.perimeter_cm(5).unwrap() - 9.525).abs() < 0.001);
    let edited = Document::open(Path::new(
        "../../tests/fixtures/wine_moved_generated_polygon_perimeter.gsp",
    ))
    .unwrap();
    assert_eq!(edited.objects[4].kind, Kind::PolygonPerimeter(4));
    assert!((edited.perimeter_cm(5).unwrap() - 10.64).abs() < 0.01);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 270., y: 120. }));
    let c = doc.add(Kind::Free(Point { x: 270., y: 210. }));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let perimeter = doc.add(Kind::PolygonPerimeter(polygon));
    assert_eq!(doc.objects[polygon - 1].kind.dependencies(), vec![a, b, c]);
    assert_eq!(
        doc.objects[perimeter - 1].kind.dependencies(),
        vec![polygon]
    );
    let path = std::env::temp_dir().join(format!("sketchpad_polygon_{}.gsp", std::process::id()));
    doc.save_to(&path).unwrap();
    let mut reopened = Document::open(&path).unwrap();
    assert_eq!(
        reopened.objects[polygon - 1].kind,
        Kind::Polygon(vec![a, b, c])
    );
    assert!((reopened.perimeter_cm(perimeter).unwrap() - 9.525).abs() < 0.001);
    reopened.checkpoint();
    assert!(reopened.move_point(c, Point { x: 310., y: 210. }));
    let changed = reopened.perimeter_cm(perimeter).unwrap();
    assert!((changed - 10.64).abs() < 0.01);
    reopened.undo();
    assert!((reopened.perimeter_cm(perimeter).unwrap() - 9.525).abs() < 0.001);
    reopened.redo();
    assert!((reopened.perimeter_cm(perimeter).unwrap() - changed).abs() < 1e-9);
    reopened.save_to(&path).unwrap();
    let saved = Document::open(&path).unwrap();
    assert!((saved.perimeter_cm(perimeter).unwrap() - changed).abs() < 1e-9);
    fs::remove_file(path).unwrap();
}
#[test]
fn measured_length_calculation_import_creation_and_iteration() {
    let native = include_bytes!("../../../../../tests/fixtures/calculation_length_times_two_native.gsp");
    let imported = Document::from_bytes(native, None).unwrap();
    assert!(!imported.partial_view);
    assert_eq!(imported.encode().unwrap(), native);
    let length = imported.segment_length_cm(4).unwrap();
    assert!((imported.measured_calculation_cm(5).unwrap() - length * 2.).abs() < 1e-10);

    let mut generated = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/wine_segment_length.gsp"),
        None,
    )
    .unwrap();
    let id = generated
        .add_measured_length_calculation(4, 3, (10, 34))
        .unwrap();
    assert_eq!(id, 5);
    assert!(
        (generated.measured_calculation_cm(id).unwrap()
            - generated.segment_length_cm(4).unwrap() * 3.)
            .abs()
            < 1e-10
    );
    assert!(generated.move_point(2, Point { x: 400., y: 218. }));
    let moved = generated.measured_calculation_cm(id).unwrap();
    assert!((moved - generated.segment_length_cm(4).unwrap() * 3.).abs() < 1e-10);
    let bytes = generated.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!((reopened.measured_calculation_cm(id).unwrap() - moved).abs() < 1e-10);
    assert!(
        crate::render::number_lines_for_object(&reopened, id)
            .unwrap()
            .1[0]
            .contains("cm")
    );

    let mut shifted = Document::new();
    shifted.add(Kind::Free(Point { x: 20., y: 20. }));
    let a = shifted.add(Kind::Free(Point { x: 168., y: 304. }));
    let b = shifted.add(Kind::Free(Point { x: 336., y: 218. }));
    let segment = shifted.add(Kind::Segment(a, b));
    let source = shifted.add(Kind::SegmentLength(segment));
    shifted
        .add_measured_length_calculation(source, 0, (10, 34))
        .unwrap();
    shifted.selected.insert(1);
    shifted.clear_selected().unwrap();
    let compact = Document::from_bytes(&shifted.encode().unwrap(), None).unwrap();
    assert!(!compact.partial_view);
    assert!(matches!(&compact.objects[4].kind,
        Kind::MeasuredCalculation(4, group) if group.source_id() == 4 && group.factor() == 0));
    assert_eq!(compact.measured_calculation_cm(5), Some(0.));

    let distance_native =
        include_bytes!("../../../../../tests/fixtures/calculation_distance_times_two_native.gsp");
    let distance = Document::from_bytes(distance_native, None).unwrap();
    assert!(!distance.partial_view);
    assert_eq!(distance.encode().unwrap(), distance_native);
    assert!(
        (distance.measured_calculation_cm(7).unwrap()
            - distance.point_distance_cm(6).unwrap() * 2.)
            .abs()
            < 1e-10
    );
    let distance_port =
        include_bytes!("../../../../../tests/fixtures/calculation_distance_times_three_port.gsp");
    let mut distance = Document::from_bytes(distance_port, None).unwrap();
    assert!(!distance.partial_view);
    assert_eq!(distance.encode().unwrap(), distance_port);
    assert_eq!(distance.length_measurement_name(6).as_deref(), Some("AB"));
    assert!(distance.move_point(2, Point { x: 300., y: 120. }));
    assert!(
        (distance.measured_calculation_cm(7).unwrap()
            - distance.point_distance_cm(6).unwrap() * 3.)
            .abs()
            < 1e-10
    );

    let perimeter_native =
        include_bytes!("../../../../../tests/fixtures/calculation_perimeter_times_two_native.gsp");
    let perimeter = Document::from_bytes(perimeter_native, None).unwrap();
    assert!(!perimeter.partial_view);
    assert_eq!(perimeter.encode().unwrap(), perimeter_native);
    assert!(
        (perimeter.measured_calculation_cm(6).unwrap() - perimeter.perimeter_cm(5).unwrap() * 2.)
            .abs()
            < 1e-10
    );
    let perimeter_port =
        include_bytes!("../../../../../tests/fixtures/calculation_perimeter_times_three_port.gsp");
    let mut perimeter = Document::from_bytes(perimeter_port, None).unwrap();
    assert!(!perimeter.partial_view);
    assert_eq!(perimeter.encode().unwrap(), perimeter_port);
    assert_eq!(
        perimeter.length_measurement_name(5).as_deref(),
        Some("Perimeter")
    );
    assert!(perimeter.move_point(3, Point { x: 300., y: 240. }));
    assert!(
        (perimeter.measured_calculation_cm(6).unwrap() - perimeter.perimeter_cm(5).unwrap() * 3.)
            .abs()
            < 1e-10
    );

    for (native, port, expected_name, measure) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/calculation_circumference_times_two_native.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/calculation_circumference_times_three_port.gsp"
            )
            .as_slice(),
            "Circumference",
            Document::circle_circumference_cm as fn(&Document, usize) -> Option<f64>,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/calculation_radius_times_two_native.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/calculation_radius_times_three_port.gsp")
                .as_slice(),
            "Radius",
            Document::circle_radius_cm as fn(&Document, usize) -> Option<f64>,
        ),
    ] {
        let imported = Document::from_bytes(native, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.encode().unwrap(), native);
        assert!(
            (imported.measured_calculation_cm(5).unwrap() - measure(&imported, 4).unwrap() * 2.)
                .abs()
                < 1e-10
        );
        let mut generated = Document::from_bytes(port, None).unwrap();
        assert!(!generated.partial_view);
        assert_eq!(generated.encode().unwrap(), port);
        assert_eq!(
            generated.length_measurement_name(4).as_deref(),
            Some(expected_name)
        );
        assert!(generated.move_point(2, Point { x: 300., y: 120. }));
        assert!(
            (generated.measured_calculation_cm(5).unwrap() - measure(&generated, 4).unwrap() * 3.)
                .abs()
                < 1e-10
        );
    }

    let arc_native =
        include_bytes!("../../../../../tests/fixtures/calculation_arc_length_times_two_native.gsp");
    let arc = Document::from_bytes(arc_native, None).unwrap();
    assert!(!arc.partial_view);
    assert_eq!(arc.encode().unwrap(), arc_native);
    assert!(
        (arc.measured_calculation_cm(6).unwrap() - arc.arc_length_cm(5).unwrap() * 2.).abs()
            < 1e-10
    );
    let arc_port =
        include_bytes!("../../../../../tests/fixtures/calculation_arc_length_times_three_port.gsp");
    let mut arc = Document::from_bytes(arc_port, None).unwrap();
    assert!(!arc.partial_view);
    assert_eq!(arc.encode().unwrap(), arc_port);
    assert_eq!(
        arc.length_measurement_name(5).as_deref(),
        Some("Arc Length")
    );
    assert!(arc.move_point(2, Point { x: 260., y: 110. }));
    assert!(
        (arc.measured_calculation_cm(6).unwrap() - arc.arc_length_cm(5).unwrap() * 3.).abs()
            < 1e-10
    );

    let marked_bytes =
        include_bytes!("../../../../../tests/fixtures/calculation_marked_distance_port.gsp");
    let mut marked = Document::from_bytes(marked_bytes, None).unwrap();
    assert!(!marked.partial_view);
    assert_eq!(marked.encode().unwrap(), marked_bytes);
    assert!((marked.position(6).unwrap().x - 264.).abs() < 1e-9);
    assert!(marked.move_point(2, Point { x: 210., y: 304. }));
    assert!((marked.position(6).unwrap().x - 294.).abs() < 1e-9);

    let mut copied = Document::from_bytes(native, None).unwrap();
    copied.selected.insert(5);
    let clipboard = copied.copy_selected().unwrap();
    let pasted = copied.paste(&clipboard).unwrap();
    let clone_id = *pasted.last().unwrap();
    let cloned = Document::from_bytes(&copied.encode().unwrap(), None).unwrap();
    assert!(!cloned.partial_view);
    assert!(matches!(&cloned.objects[clone_id - 1].kind,
        Kind::MeasuredCalculation(source, group) if *source != 4 && group.source_id() as usize == *source));
    assert!(cloned.measured_calculation_cm(clone_id).is_some());

    let divided_native =
        include_bytes!("../../../../../tests/fixtures/calculation_length_divide_two_native.gsp");
    let divided = Document::from_bytes(divided_native, None).unwrap();
    assert!(!divided.partial_view);
    assert_eq!(divided.encode().unwrap(), divided_native);
    assert!(
        (divided.measured_calculation_cm(5).unwrap() - divided.segment_length_cm(4).unwrap() / 2.)
            .abs()
            < 1e-10
    );
    let divided_port =
        include_bytes!("../../../../../tests/fixtures/calculation_length_divide_three_port.gsp");
    let mut divided = Document::from_bytes(divided_port, None).unwrap();
    assert!(!divided.partial_view);
    assert_eq!(divided.encode().unwrap(), divided_port);
    assert!(
        matches!(&divided.objects[4].kind, Kind::MeasuredCalculation(4, group) if group.divides() && group.factor() == 3)
    );
    assert!(divided.move_point(2, Point { x: 400., y: 218. }));
    assert!(
        (divided.measured_calculation_cm(5).unwrap() - divided.segment_length_cm(4).unwrap() / 3.)
            .abs()
            < 1e-10
    );
    assert!(
        divided
            .add_measured_length_division(4, 0, (10, 100))
            .is_err()
    );

    for (native, port, factor, divides) in [
        (
            include_bytes!("../../../../../tests/fixtures/calculation_length_times_twelve_native.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/calculation_length_times_twenty_three_port.gsp"
            )
            .as_slice(),
            23u8,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/calculation_length_divide_twelve_native.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/calculation_length_divide_twelve_port.gsp")
                .as_slice(),
            12u8,
            true,
        ),
    ] {
        let imported = Document::from_bytes(native, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.encode().unwrap(), native);
        let mut generated = Document::from_bytes(port, None).unwrap();
        assert!(!generated.partial_view);
        assert_eq!(generated.encode().unwrap(), port);
        assert!(matches!(&generated.objects[4].kind,
            Kind::MeasuredCalculation(4, group) if group.factor() == factor && group.divides() == divides));
        assert!(generated.move_point(2, Point { x: 400., y: 218. }));
        let expected = generated.segment_length_cm(4).unwrap();
        let expected = if divides {
            expected / f64::from(factor)
        } else {
            expected * f64::from(factor)
        };
        assert!((generated.measured_calculation_cm(5).unwrap() - expected).abs() < 1e-10);
    }
    assert!(crate::table_records::MeasuredCalculationGroup::multiply(4, 99, (10, 34)).is_ok());
    assert!(crate::table_records::MeasuredCalculationGroup::multiply(4, 100, (10, 34)).is_err());
    assert!(crate::table_records::MeasuredCalculationGroup::divide(4, 99, (10, 34)).is_ok());
    assert!(crate::table_records::MeasuredCalculationGroup::divide(4, 0, (10, 34)).is_err());
}

#[test]
fn two_length_product_and_ratio_keep_units_and_update() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/calculation_two_lengths_product_ratio_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    let unit: f64 = 37.79527559055118;
    assert!((doc.length_pair_calculation_value(11).unwrap() - 20000. / unit.powi(2)).abs() < 1e-10);
    assert_eq!(doc.length_pair_calculation_value(12), Some(0.5));
    assert!(doc.distance_value_pixels(11).is_none());
    assert!(doc.distance_value_pixels(12).is_none());
    let product = crate::render::number_lines_for_object(&doc, 11)
        .unwrap()
        .1
        .join(" ");
    let ratio = crate::render::number_lines_for_object(&doc, 12)
        .unwrap()
        .1
        .join(" ");
    assert!(product.ends_with("cm²"));
    assert!(!ratio.contains("cm"));
    assert!(
        doc.add_measured_length_calculation(11, 2, (10, 232))
            .is_err()
    );
    assert!(
        doc.add_length_pair_calculation(11, 4, false, (10, 232))
            .is_err()
    );
    doc.move_point(2, Point { x: 300., y: 180. });
    assert_eq!(doc.length_pair_calculation_value(12), Some(0.75));
    doc.selected.insert(12);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let cloned = *pasted.last().unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.length_pair_calculation_value(cloned), Some(0.75));
    doc.move_point(6, doc.position(5).unwrap());
    assert_eq!(doc.length_pair_calculation_value(12), None);
    assert!(crate::render::number_lines_for_object(&doc, 12).unwrap().1[0].contains("undefined"));
    assert!(doc.add_length_pair_operation(4, 8, 3, (10, 232)).is_err());
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn two_length_calculations_update_copy_and_delete() {
    let native =
        include_bytes!("../../../../../tests/fixtures/calculation_two_lengths_sum_native_verified.gsp");
    let native = Document::from_bytes(native, None).unwrap();
    assert!(!native.partial_view);
    assert!(
        (native.length_pair_calculation_cm(9).unwrap() - 300. / 37.79527559055118).abs() < 1e-10
    );
    let bytes = include_bytes!("../../../../../tests/fixtures/calculation_two_lengths_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(
        matches!(&doc.objects[8].kind, Kind::LengthPairCalculation(4,8,group) if !group.subtracts())
    );
    assert!(
        matches!(&doc.objects[9].kind, Kind::LengthPairCalculation(4,8,group) if group.subtracts())
    );
    assert!((doc.length_pair_calculation_cm(10).unwrap() + 100. / 37.79527559055118).abs() < 1e-10);
    assert!(crate::render::number_lines_for_object(&doc, 10).is_some());
    doc.move_point(2, Point { x: 300., y: 180. });
    assert!((doc.length_pair_calculation_cm(9).unwrap() - 350. / 37.79527559055118).abs() < 1e-10);
    assert!((doc.length_pair_calculation_cm(10).unwrap() + 50. / 37.79527559055118).abs() < 1e-10);
    doc.selected.insert(10);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let clone_id = *pasted.last().unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(&reopened.objects[clone_id-1].kind,Kind::LengthPairCalculation(a,b,group) if *a!=4 && *b!=8 && group.sources()==[*a as u32,*b as u32])
    );
    assert!(
        (reopened.length_pair_calculation_cm(clone_id).unwrap() + 50. / 37.79527559055118).abs()
            < 1e-10
    );
    let scaled = doc
        .add_measured_length_calculation(9, 2, (10, 200))
        .unwrap();
    assert!(
        (doc.measured_calculation_cm(scaled).unwrap() - 700. / 37.79527559055118).abs() < 1e-10
    );
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(4);
    assert_eq!(cleared.clear_selected().unwrap(), vec![4, 9, 10]);
    assert!(
        !Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    assert!(
        cleared
            .add_length_pair_calculation(1, 8, false, (10, 200))
            .is_err()
    );
}

#[test]
fn chained_measured_calculations_update_and_remap() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/calculation_length_chained_division_port.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(matches!(
        document.objects[5].kind,
        Kind::MeasuredCalculation(5, _)
    ));
    assert!(matches!(
        document.objects[6].kind,
        Kind::MeasuredCalculation(6, _)
    ));
    let base = document.segment_length_cm(4).unwrap();
    assert!((document.measured_calculation_cm(6).unwrap() - base * 6.).abs() < 1e-10);
    assert!((document.measured_calculation_cm(7).unwrap() - base * 2.).abs() < 1e-10);
    assert!(document.length_measurement_name(7).unwrap().contains("AB"));
    assert!(document.move_point(2, Point { x: 400., y: 218. }));
    let moved_base = document.segment_length_cm(4).unwrap();
    assert!((document.measured_calculation_cm(7).unwrap() - moved_base * 2.).abs() < 1e-10);
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!((reopened.measured_calculation_cm(7).unwrap() - moved_base * 2.).abs() < 1e-10);

    document.selected.insert(7);
    let clipboard = document.copy_selected().unwrap();
    let pasted = document.paste(&clipboard).unwrap();
    let clone_id = *pasted.last().unwrap();
    let cloned = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!cloned.partial_view);
    assert!(matches!(&cloned.objects[clone_id - 1].kind,
        Kind::MeasuredCalculation(source, group) if *source != 6 && group.source_id() as usize == *source));
    assert!((cloned.measured_calculation_cm(clone_id).unwrap() - moved_base * 2.).abs() < 1e-10);
}

#[test]
fn decimal_parameter_calculations_update_and_reject_bad_literals() {
    let native =
        include_bytes!("../../../../../tests/fixtures/parameter_decimal_calculation_native_verified.gsp");
    let native_doc = Document::from_bytes(native, None).unwrap();
    assert!(!native_doc.partial_view);
    assert_eq!(native_doc.encode().unwrap(), native);
    assert_eq!(native_doc.parameter_calculation_value(2), Some(5.));
    let bytes = include_bytes!("../../../../../tests/fixtures/parameter_decimal_calculation_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!((doc.parameter_calculation_value(2).unwrap() - 5.125).abs() < 1e-12);
    assert!(crate::render::number_lines_for_object(&doc, 2).unwrap().1[0].ends_with("5.13"));
    assert!((doc.parameter_calculation_value(3).unwrap() - 10.25).abs() < 1e-12);
    assert!(
        doc.parameter_calculation_expression(2)
            .unwrap()
            .contains("0.125")
    );
    doc.set_parameter_value(1, 3.).unwrap();
    assert!((doc.parameter_calculation_value(3).unwrap() - 15.25).abs() < 1e-12);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!((reopened.parameter_calculation_value(3).unwrap() - 15.25).abs() < 1e-12);
    let before_invalid = doc.encode().unwrap();
    for expression in [
        "speed+.",
        "speed+1..",
        "speed+1.2.3",
        "speed+00.5",
        "speed/0.0",
    ] {
        assert!(
            doc.add_parameter_calculation(1, expression, (10, 120))
                .is_err(),
            "{expression}"
        );
        assert_eq!(doc.encode().unwrap(), before_invalid);
    }
    let fractional = doc
        .add_parameter_calculation(1, "speed^0.5", (10, 120))
        .unwrap();
    assert!((doc.parameter_calculation_value(fractional).unwrap() - 3_f64.sqrt()).abs() < 1e-12);
    let trailing = doc
        .add_parameter_calculation(1, "speed+1.", (10, 150))
        .unwrap();
    assert_eq!(doc.parameter_calculation_value(trailing), Some(4.));
    assert!(
        !doc.parameter_calculation_expression(trailing)
            .unwrap()
            .contains("1.")
    );
    doc.set_parameter_value(1, 4.).unwrap();
    assert_eq!(doc.parameter_calculation_value(trailing), Some(5.));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.parameter_calculation_value(trailing), Some(5.));
}

#[test]
fn parameter_calculation_chain_imports_and_updates() {
    let native = include_bytes!("../../../../../tests/fixtures/parameter_calculation_chain_native.gsp");
    let mut document = Document::from_bytes(native, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), native);
    for (id, expected) in (2..=6).zip([1., 1., 2., 2., 1.]) {
        assert_eq!(document.parameter_calculation_value(id), Some(expected));
        assert!(crate::render::number_lines_for_object(&document, id).is_some());
    }
    document.set_parameter_value(1, 2.).unwrap();
    for (id, expected) in (2..=6).zip([2., 4., 5., 5., 2.5]) {
        assert_eq!(document.parameter_calculation_value(id), Some(expected));
    }
    let saved = document.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.parameter_calculation_value(6), Some(2.5));
}

#[test]
fn generated_parameter_calculations_recalculate_and_remap() {
    let complex =
        include_bytes!("../../../../../tests/fixtures/parameter_calculation_generated_port.gsp");
    let mut document = Document::from_bytes(complex, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), complex);
    assert_eq!(document.parameter_calculation_value(2), Some(3.5));
    document.set_parameter_value(1, 3.).unwrap();
    assert_eq!(document.parameter_calculation_value(2), Some(6.));

    let even = include_bytes!("../../../../../tests/fixtures/parameter_calculation_even_tokens_port.gsp");
    let mut document = Document::from_bytes(even, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), even);
    assert_eq!(document.parameter_calculation_value(2), Some(24.));
    assert_eq!(document.parameter_calculation_value(3), Some(8.));
    document.set_parameter_value(1, 3.).unwrap();
    assert_eq!(document.parameter_calculation_value(2), Some(36.));
    assert_eq!(document.parameter_calculation_value(3), Some(9.));
    document.selected.insert(3);
    let clipboard = document.copy_selected().unwrap();
    let pasted = document.paste(&clipboard).unwrap();
    let clone_id = *pasted.last().unwrap();
    let cloned = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!cloned.partial_view);
    assert!(matches!(&cloned.objects[clone_id - 1].kind,
        Kind::ParameterCalculation(source, group) if *source != 1 && group.source_id() as usize == *source));
    assert_eq!(cloned.parameter_calculation_value(clone_id), Some(9.));

    let named = include_bytes!("../../../../../tests/fixtures/named_parameter_calculation_port.gsp");
    let mut document = Document::from_bytes(named, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), named);
    assert_eq!(document.parameter_calculation_value(2), Some(7.25));
    document.set_parameter_value(1, 3.).unwrap();
    assert_eq!(document.parameter_calculation_value(2), Some(10.));
    assert!(
        document
            .add_parameter_calculation(1, "speed/0", (10, 100))
            .is_err()
    );
    assert!(
        document
            .add_parameter_calculation(1, "speed+", (10, 100))
            .is_err()
    );
    assert!(
        document
            .add_parameter_calculation(1, "10+2", (10, 100))
            .is_err()
    );
    assert!(
        document
            .add_parameter_calculation(1, &format!("speed+{}", "9".repeat(4095)), (10, 100))
            .is_err()
    );
}

#[test]
fn chained_parameter_calculations_update_and_remap() {
    let bytes = include_bytes!("../../../../../tests/fixtures/parameter_calculation_chained_port.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(matches!(
        document.objects[3].kind,
        Kind::ParameterCalculation(2, _)
    ));
    assert_eq!(document.parameter_calculation_value(4), Some(30.));
    assert!(
        document
            .parameter_calculation_expression(4)
            .unwrap()
            .contains("t[1]")
    );
    document.set_parameter_value(1, 3.).unwrap();
    assert_eq!(document.parameter_calculation_value(2), Some(36.));
    assert_eq!(document.parameter_calculation_value(4), Some(42.));
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.parameter_calculation_value(4), Some(42.));

    document.selected.insert(4);
    let clipboard = document.copy_selected().unwrap();
    let pasted = document.paste(&clipboard).unwrap();
    let clone_id = *pasted.last().unwrap();
    let cloned = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!cloned.partial_view);
    assert!(matches!(&cloned.objects[clone_id - 1].kind,
        Kind::ParameterCalculation(source, group) if *source != 2 && group.source_id() as usize == *source));
    assert_eq!(cloned.parameter_calculation_value(clone_id), Some(42.));
}

#[test]
fn invalid_new_polygon_does_not_serialize() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 20., y: 0. }));
    doc.add(Kind::Polygon(vec![a, b, a]));
    assert!(doc.encode().is_err());
}
#[test]
fn ratio_matches_signed_projection_after_drag_and_reopen() {
    for (path, expected) in [
        ("wine_point_ratio", 1.2),
        ("wine_moved_ratio", 1.4),
        ("wine_opposite_ratio", -1.0),
        ("wine_offline_ratio", 1.2),
    ] {
        let path = format!("../../tests/fixtures/{path}.gsp");
        let doc = Document::open(Path::new(&path)).unwrap();
        assert_eq!(doc.objects[3].kind, Kind::PointRatio(1, 2, 3));
        assert!((doc.ratio_value(4).unwrap() - expected).abs() < 1e-9);
    }
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 100. }));
    let c = doc.add(Kind::Free(Point { x: 270., y: 100. }));
    let ratio = doc.add(Kind::PointRatio(a, b, c));
    assert!((doc.ratio_value(ratio).unwrap() - 1.2).abs() < 1e-9);
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::PointRatio(a, b, c));
    reopened.checkpoint();
    assert!(reopened.move_point(c, Point { x: 50., y: 150. }));
    assert!((reopened.ratio_value(ratio).unwrap() + 1.0).abs() < 1e-9);
    reopened.undo();
    assert!((reopened.ratio_value(ratio).unwrap() - 1.2).abs() < 1e-9);
    reopened.redo();
    assert!((reopened.ratio_value(ratio).unwrap() + 1.0).abs() < 1e-9);
    let saved = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert!((saved.ratio_value(ratio).unwrap() + 1.0).abs() < 1e-9);
}
#[test]
fn undo_redo() {
    let mut d = Document::new();
    d.add(Kind::Free(Point { x: 1., y: 2. }));
    d.undo();
    assert!(d.objects.is_empty());
    d.redo();
    assert_eq!(d.objects.len(), 1);
}
#[test]
fn segment_length_matches_original_and_updates_after_drag() {
    let original = Document::open(Path::new("../../tests/fixtures/wine_segment_length.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::SegmentLength(3));
    let (a, b, _) = original.line_points(3).unwrap();
    assert!(
        (original.segment_length_cm(4).unwrap() - a.distance(b) / 37.79527559055118).abs() < 1e-10
    );
    let moved = Document::open(Path::new("../../tests/fixtures/wine_moved_length.gsp")).unwrap();
    assert_eq!(moved.objects[3].kind, Kind::SegmentLength(3));
    assert!((moved.segment_length_cm(4).unwrap() - 5.291666666666667).abs() < 1e-10);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let segment = doc.add(Kind::Segment(a, b));
    let length = doc.add(Kind::SegmentLength(segment));
    assert!((doc.segment_length_cm(length).unwrap() - 2.6458333333333335).abs() < 1e-10);
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::SegmentLength(3));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
    assert!((reopened.segment_length_cm(length).unwrap() - 5.291666666666667).abs() < 1e-10);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .segment_length_cm(length),
        reopened.segment_length_cm(length)
    );
    reopened.selected.insert(segment);
    assert_eq!(reopened.clear_selected().unwrap(), vec![segment, length]);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        2
    );
}

#[test]
fn indexed_parameter_tables_edit_live_values_and_preserve_native_names_and_fonts() {
    for (bytes, source, name) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/numeric_parameter_table_unnamed_scalar_native_created.gsp"
            )
            .as_slice(),
            2,
            "t[4]",
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/numeric_parameter_table_all_unnamed_native_created.gsp"
            )
            .as_slice(),
            3,
            "t[6]",
        ),
    ] {
        let mut d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        d.set_parameter_value(source, 2.25).unwrap();
        let (_, lines) = crate::render::number_lines_for_object(&d, 15).unwrap();
        assert_eq!(lines[0], if name == "t[4]" { "t₄" } else { "t₆" });
        let precision = match &d.objects[source - 1].kind {
            Kind::Parameter(p) => p.precision(),
            _ => unreachable!(),
        };
        assert_eq!(lines[1], format!("{:.*}", precision, 2.25));
        let before_rows = d.encode().unwrap();
        d.change_table_rows(15, true).unwrap();
        let Kind::ValueTable(id, table) = &d.objects[14].kind else {
            panic!("table");
        };
        assert_eq!(*id, source);
        assert_eq!(table.named_label_values().unwrap().0, name);
        assert_eq!(table.rows(), 2);
        d.verify_roundtrip().unwrap();
        d.undo();
        assert_eq!(d.encode().unwrap(), before_rows);
        d.redo();
        d.change_table_rows(15, false).unwrap();
        d.move_number(15, (280, 140)).unwrap();
        let encoded = d.encode().unwrap();
        let expected = if source == 2 {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_indexed_scalar_edited_writer_native_verified.gsp").as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_indexed_all_named_edited_writer_native_verified.gsp").as_slice()
        };
        assert_eq!(encoded, expected);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.numeric_value(source), Some(2.25));
        let Kind::ValueTable(_, table) = &reopened.objects[14].kind else {
            panic!("table");
        };
        assert_eq!(table.anchor(), (280, 140));
        assert_eq!(table.named_label_values().unwrap().0, name);
        assert_eq!(reopened.encode().unwrap(), encoded);
        if std::env::var_os("SKETCHPAD_WRITE_INDEXED_TABLE_FIXTURES").is_some() {
            fs::write(
                format!("/tmp/indexed_parameter_table_{}_edited.gsp", source),
                encoded,
            )
            .unwrap();
        }
    }
}

#[test]
fn generated_parameter_table_creation_materializes_names_and_counter_atomically() {
    for (bytes, source, expected_counter, expected_names) in [
        (include_bytes!("../../../../../tests/fixtures/numeric_parameter_presentation_unlabeled_scalar_native_verified.gsp").as_slice(), 2, 5, ["t[1]", "t[4]", "t[3]"]),
        (include_bytes!("../../../../../tests/fixtures/numeric_parameter_autoname_all_unnamed_native_verified.gsp").as_slice(), 3, 7, ["t[4]", "t[5]", "t[6]"]),
    ] {
        let mut d = Document::from_bytes(bytes, None).unwrap();
        let before = d.encode().unwrap();
        let numeric = (1..=3).map(|id| d.numeric_value(id)).collect::<Vec<_>>();
        let Kind::Parameter(unnamed) = d.objects[source - 1].kind.clone() else { panic!("parameter"); };
        let table = d.tabulate_default_parameter(source, (280, 140)).unwrap();
        assert_eq!(table, 15);
        assert_eq!((1..=3).map(|id| d.numeric_value(id)).collect::<Vec<_>>(), numeric);
        for (id, name) in (1..=3).zip(expected_names) {
            let Kind::Parameter(parameter) = &d.objects[id - 1].kind else { panic!("parameter"); };
            assert_eq!(parameter.name(), name);
            assert_eq!(d.parameter_display_name(id).as_deref(), Some(name));
        }
        let expected = Document::from_bytes(if source == 2 {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_unnamed_scalar_native_created.gsp").as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_all_unnamed_native_created.gsp").as_slice()
        }, None).unwrap();
        for id in 1..=3 {
            let (Kind::Parameter(current), Kind::Parameter(native)) = (&d.objects[id - 1].kind, &expected.objects[id - 1].kind) else { panic!("parameter"); };
            assert_eq!(current.bbox(), native.bbox());
            assert_eq!(current.name(), native.name());
        }
        assert_eq!(d.text_font_state(table).unwrap().size_code, 14);
        assert_eq!(super::super::parameter_names::parameter_name_counter(&d.encode().unwrap(), 0), Some(expected_counter));
        d.verify_roundtrip().unwrap();
        let created = d.encode().unwrap();
        assert_eq!(created.as_slice(), if source == 2 {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_generated_scalar_created_writer_native_verified.gsp").as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_generated_all_named_created_writer_native_verified.gsp").as_slice()
        });
        d.undo();
        assert_eq!(d.encode().unwrap(), before);
        d.redo();
        assert_eq!(d.encode().unwrap(), created);
        let added = d.add(Kind::Parameter(unnamed));
        assert_eq!(d.parameter_display_name(added), Some(format!("t[{expected_counter}]")));
        d.undo();
        d.set_parameter_value(source, 2.5).unwrap();
        let Kind::Parameter(parameter) = &d.objects[source - 1].kind else { panic!("parameter"); };
        assert_eq!(crate::render::number_lines_for_object(&d, table).unwrap().1[1], format!("{:.*}", parameter.precision(), 2.5));
        d.undo();
        let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(&reopened.objects[table - 1].kind, Kind::ValueTable(id, _) if *id == source));
        if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
            fs::write(format!("/tmp/generated_parameter_table_{source}_created.gsp"), created).unwrap();
        }
    }
}

#[test]
fn generated_parameter_tables_match_native_multidigit_and_negative_captions() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/numeric_parameter_presentation_unlabeled_scalar_native_verified.gsp"
    );
    for negative in [false, true] {
        let mut bytes = original.to_vec();
        if !negative {
            let counter = records(&bytes)
                .unwrap()
                .into_iter()
                .find(|record| record.kind == 0x0960)
                .unwrap();
            bytes[counter.data.start + 2..counter.data.end].copy_from_slice(&12_u32.to_le_bytes());
        }
        let mut d = Document::from_bytes(&bytes, None).unwrap();
        if negative {
            d.set_parameter_value(2, -2.5).unwrap();
        }
        let before = d.encode().unwrap();
        let table = d.tabulate_default_parameter(2, (280, 140)).unwrap();
        let native = Document::from_bytes(if negative {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_unnamed_scalar_negative25_native_created.gsp").as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter12_native_created.gsp").as_slice()
        }, None).unwrap();
        assert!(!native.partial_view);
        native.verify_roundtrip().unwrap();
        let (Kind::Parameter(current), Kind::Parameter(expected)) =
            (&d.objects[1].kind, &native.objects[1].kind)
        else {
            panic!("parameter");
        };
        assert_eq!(current.name(), expected.name());
        assert_eq!(current.bbox(), expected.bbox());
        let Kind::ValueTable(_, group) = &d.objects[table - 1].kind else {
            panic!("table");
        };
        let Kind::ValueTable(_, expected) = &native.objects[14].kind else {
            panic!("table");
        };
        assert_eq!(group.named_label_values(), expected.named_label_values());
        assert_eq!(
            crate::render::number_lines_for_object(&d, table).unwrap().1[1],
            if negative { "–2.5" } else { "0" }
        );
        d.verify_roundtrip().unwrap();
        let created = d.encode().unwrap();
        assert_eq!(
            created.as_slice(),
            if negative {
                include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_generated_negative25_created_writer_native_verified.gsp").as_slice()
            } else {
                include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_generated_counter12_created_writer_native_verified.gsp").as_slice()
            }
        );
        d.undo();
        assert_eq!(d.encode().unwrap(), before);
        d.redo();
        assert_eq!(d.encode().unwrap(), created);
        if negative {
            d.change_table_rows(table, true).unwrap();
            d.verify_roundtrip().unwrap();
            let Kind::ValueTable(_, group) = &d.objects[table - 1].kind else {
                panic!("table");
            };
            assert_eq!(group.named_label_values().unwrap().1, vec!["–2.5", "–2.5"]);
            d.undo();
        }
        if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
            fs::write(
                format!(
                    "/tmp/generated_parameter_table_{}_created.gsp",
                    if negative { "negative25" } else { "counter12" }
                ),
                created,
            )
            .unwrap();
        }
    }
}

#[test]
fn generated_first_parameter_table_preserves_native_precision_and_legacy_tables() {
    let mut bytes = include_bytes!("../../../../../tests/fixtures/numeric_parameter_presentation_unlabeled_scalar_native_verified.gsp").to_vec();
    let counter = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x0960)
        .unwrap();
    bytes[counter.data.start + 2..counter.data.end].copy_from_slice(&1_u32.to_le_bytes());
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let before = d.encode().unwrap();
    let table = d.tabulate_default_parameter(2, (280, 140)).unwrap();
    let Kind::ValueTable(source, group) = &d.objects[table - 1].kind else {
        panic!("table");
    };
    assert_eq!(*source, 2);
    assert_eq!(
        group.named_label_values(),
        Some(("t[1]".into(), vec!["0".into()]))
    );
    assert_eq!(d.parameter_display_name(2).as_deref(), Some("t[1]"));
    let native = Document::from_bytes(include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter1_native_created.gsp"), None).unwrap();
    assert!(!native.partial_view);
    native.verify_roundtrip().unwrap();
    let Kind::ValueTable(native_source, native_group) = &native.objects[14].kind else {
        panic!("native table");
    };
    assert_eq!(*native_source, 2);
    assert_eq!(
        group.named_label_values(),
        native_group.named_label_values()
    );
    let (Kind::Parameter(current), Kind::Parameter(expected)) =
        (&d.objects[1].kind, &native.objects[1].kind)
    else {
        panic!("parameter");
    };
    assert_eq!(current.bbox(), expected.bbox());
    assert_eq!(
        super::super::parameter_names::parameter_name_counter(&d.encode().unwrap(), 0),
        Some(2)
    );
    assert_eq!(
        crate::render::number_lines_for_object(&d, table).unwrap().1,
        vec!["t₁", "0"]
    );
    d.verify_roundtrip().unwrap();
    let created = d.encode().unwrap();
    assert_eq!(created.as_slice(), include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_generated_counter1_created_writer_native_verified.gsp").as_slice());
    if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
        fs::write(
            "/tmp/generated_parameter_table_counter1_created.gsp",
            &created,
        )
        .unwrap();
    }
    d.change_table_rows(table, true).unwrap();
    let Kind::ValueTable(_, group) = &d.objects[table - 1].kind else {
        panic!("table");
    };
    assert_eq!(group.named_label_values().unwrap().1, vec!["0", "0"]);
    d.verify_roundtrip().unwrap();
    d.undo();
    assert_eq!(d.encode().unwrap(), created);
    d.undo();
    assert_eq!(d.encode().unwrap(), before);
    d.redo();
    assert_eq!(d.encode().unwrap(), created);

    let legacy = crate::table_records::TableGroup::for_source_value(2, (280, 140), 0.).unwrap();
    assert!(legacy.named_label_values().is_none());
    assert_eq!(legacy.default_values(), Some(vec![0.]));
}

#[test]
fn default_parameter_table_creation_materializes_other_page_parameters_atomically() {
    let mut d = Document::from_bytes(include_bytes!("../../../../../tests/fixtures/numeric_parameter_presentation_unlabeled_scalar_native_verified.gsp"), None).unwrap();
    let mut target = crate::table_records::ParameterGroup::default_value();
    target.set_anchor((200, 20)).unwrap();
    d.objects[2].kind = Kind::Parameter(target);
    d.dirty = true;
    let input = d.encode().unwrap();
    let mut d = Document::from_bytes(&input, None).unwrap();
    d.set_parameter_value(3, 2.5).unwrap();
    let before = d.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
        fs::write(
            "/tmp/default_table_with_unnamed_parameter_input.gsp",
            &before,
        )
        .unwrap();
    }
    let table = d.tabulate_default_parameter(3, (280, 140)).unwrap();
    let Kind::Parameter(other) = &d.objects[1].kind else {
        panic!("parameter");
    };
    assert_eq!(other.name(), "t[4]");
    assert_eq!(
        super::super::parameter_names::parameter_name_counter(&d.encode().unwrap(), 0),
        Some(5)
    );
    assert_eq!(d.text_font_state(table).unwrap().size_code, 14);
    d.verify_roundtrip().unwrap();
    let created = d.encode().unwrap();
    assert_eq!(created.as_slice(), include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_default_with_unnamed_created_writer_native_verified.gsp").as_slice());
    let native = Document::from_bytes(include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_default_with_unnamed_native_created.gsp"), None).unwrap();
    assert!(!native.partial_view);
    let (Kind::Parameter(current), Kind::Parameter(expected)) =
        (&d.objects[1].kind, &native.objects[1].kind)
    else {
        panic!("parameter");
    };
    assert_eq!(current.name(), expected.name());
    assert_eq!(current.bbox(), expected.bbox());
    let (Kind::ValueTable(_, current), Kind::ValueTable(_, expected)) =
        (&d.objects[14].kind, &native.objects[14].kind)
    else {
        panic!("table");
    };
    assert_eq!(
        current.default_values().unwrap(),
        expected
            .named_label_values()
            .unwrap()
            .1
            .iter()
            .map(|value| value.parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    );

    if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
        fs::write(
            "/tmp/default_table_with_unnamed_parameter_created.gsp",
            &created,
        )
        .unwrap();
    }
    d.undo();
    assert_eq!(d.encode().unwrap(), before);
    d.undo();
    assert_eq!(d.encode().unwrap(), input);
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), created);

    let mut rejected = Document::from_bytes(&input, None).unwrap();
    rejected.set_parameter_value(3, 100.).unwrap();
    let before_rejection = rejected.encode().unwrap();
    assert!(
        rejected
            .tabulate_default_parameter(3, (280, 140))
            .unwrap_err()
            .contains("99.99")
    );
    assert_eq!(rejected.encode().unwrap(), before_rejection);
    let Kind::Parameter(other) = &rejected.objects[1].kind else {
        panic!("parameter");
    };
    assert!(other.name().is_empty());
    assert_eq!(
        super::super::parameter_names::parameter_name_counter(&before_rejection, 0),
        Some(4)
    );
    rejected.undo();
    assert_eq!(rejected.encode().unwrap(), input);
}

#[test]
fn named_parameter_table_creation_and_rows_resolve_the_drawing_font_namespace() {
    let mut d = Document::new();
    let source = d.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
    let before = d.encode().unwrap();
    let table = d.tabulate_default_parameter(source, (280, 140)).unwrap();
    assert_eq!(
        d.text_font_state(table).unwrap(),
        crate::text_style_records::FontState {
            family: "Tahoma".into(),
            size_code: 14,
            bold: false,
            italic: false,
            underline: false,
        }
    );
    d.change_table_rows(table, true).unwrap();
    d.set_parameter_value(source, 3.25).unwrap();
    d.change_table_rows(table, true).unwrap();
    d.verify_roundtrip().unwrap();
    let created = d.encode().unwrap();
    assert_eq!(created.as_slice(), include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_named_font14_writer_native_verified.gsp").as_slice());
    let reopened = Document::from_bytes(&created, None).unwrap();
    assert_eq!(reopened.text_font_state(table), d.text_font_state(table));
    let Kind::ValueTable(_, group) = &reopened.objects[table - 1].kind else {
        panic!("table");
    };
    assert_eq!(
        group.named_label_values(),
        Some((
            "speed".into(),
            vec!["2.5".into(), "3.25".into(), "3.25".into()]
        ))
    );
    if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
        fs::write("/tmp/named_parameter_table_font14_writer.gsp", &created).unwrap();
    }
    for _ in 0..4 {
        d.undo();
    }
    assert_eq!(d.encode().unwrap(), before);

    let mut with_pending_font = Document::new();
    let source = with_pending_font
        .add_named_parameter("speed", 2.5, (10, 5))
        .unwrap();
    let caption = with_pending_font
        .add_text("font", Point { x: 450., y: 20. })
        .unwrap();
    with_pending_font.objects[caption - 1].text_style_override =
        Some(crate::text_style_records::FontState {
            family: "Tahoma".into(),
            size_code: 18,
            bold: false,
            italic: false,
            underline: false,
        });
    let table = with_pending_font
        .tabulate_default_parameter(source, (280, 140))
        .unwrap();
    with_pending_font.change_table_rows(table, true).unwrap();
    with_pending_font.verify_roundtrip().unwrap();
    let saved = Document::from_bytes(&with_pending_font.encode().unwrap(), None).unwrap();
    assert_eq!(saved.text_font_state(caption).unwrap().size_code, 18);
    assert_eq!(saved.text_font_state(table).unwrap().size_code, 14);
    let font_count = records(&with_pending_font.encode().unwrap())
        .unwrap()
        .iter()
        .filter(|record| record.kind == 0x04b3)
        .count();
    assert_eq!(font_count, 3);
    assert_eq!(with_pending_font.encode().unwrap().as_slice(), include_bytes!("../../../../../tests/fixtures/numeric_parameter_table_named_pending_font_writer_native_verified.gsp").as_slice());
    if std::env::var_os("SKETCHPAD_WRITE_GENERATED_TABLE_FIXTURES").is_some() {
        fs::write(
            "/tmp/named_parameter_table_pending_font_writer.gsp",
            with_pending_font.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn named_parameter_tables_support_integer_and_native_negative_rows() {
    let mut cases = Vec::new();
    for (case, value, precision, text) in [
        ("integer", 3., 0, "3"),
        ("negative", -2.5, 1, "–2.5"),
        ("negative_wide", -99.99, 2, "–99.99"),
    ] {
        let mut d = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/table_parameter_speed_original.gsp"),
            None,
        )
        .unwrap();
        d.set_parameter_value(1, value).unwrap();
        let mut input = d.encode().unwrap();
        let expression = records(&input)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x0907)
            .unwrap();
        input[expression.data.start + 28..expression.data.start + 30]
            .copy_from_slice(&(precision as u16).to_le_bytes());
        let d = Document::from_bytes(&input, None).unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_NAMED_ROW_FIXTURES").is_some() {
            fs::write(
                format!("/tmp/named_table_{case}_input.gsp"),
                d.encode().unwrap(),
            )
            .unwrap();
        }
        cases.push((case, d, text));
    }
    for (case, mut d, text) in cases {
        let before = d.encode().unwrap();
        let table = d.tabulate_default_parameter(1, (280, 140)).unwrap();
        let Kind::ValueTable(_, group) = &d.objects[table - 1].kind else {
            panic!("table");
        };
        assert_eq!(
            group.named_label_values(),
            Some(("speed".into(), vec![text.into()]))
        );
        let native_bytes = match case {
            "integer" => include_bytes!(
                "../../../../../tests/fixtures/numeric_named_table_integer_native_created.gsp"
            )
            .as_slice(),
            "negative" => include_bytes!(
                "../../../../../tests/fixtures/numeric_named_table_negative_native_created.gsp"
            )
            .as_slice(),
            _ => include_bytes!(
                "../../../../../tests/fixtures/numeric_named_table_negative_wide_native_created.gsp"
            )
            .as_slice(),
        };
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        let Kind::ValueTable(_, native_group) = &native.objects[1].kind else {
            panic!("native table");
        };
        assert_eq!(
            group.named_label_values(),
            native_group.named_label_values()
        );

        assert_eq!(
            crate::render::number_lines_for_object(&d, table).unwrap().1,
            vec!["speed", text]
        );
        d.change_table_rows(table, true).unwrap();
        d.verify_roundtrip().unwrap();
        let Kind::ValueTable(_, group) = &d.objects[table - 1].kind else {
            panic!("table");
        };
        assert_eq!(group.named_label_values().unwrap().1, vec![text, text]);
        d.change_table_rows(table, false).unwrap();
        d.verify_roundtrip().unwrap();
        let created = d.encode().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_NAMED_ROW_FIXTURES").is_some() {
            fs::write(format!("/tmp/named_table_{case}_writer.gsp"), &created).unwrap();
        }
        for _ in 0..3 {
            d.undo();
        }
        assert_eq!(d.encode().unwrap(), before);
        for _ in 0..3 {
            d.redo();
        }
        assert_eq!(d.encode().unwrap(), created);
    }
}

#[test]
fn generated_parameter_table_counter_overflow_preserves_value_history() {
    let mut bytes = include_bytes!("../../../../../tests/fixtures/numeric_parameter_presentation_unlabeled_scalar_native_verified.gsp").to_vec();
    let counter = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x0960)
        .unwrap();
    bytes[counter.data.start + 2..counter.data.end].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    assert!(!d.partial_view);
    d.set_parameter_value(3, 2.).unwrap();
    let before = d.encode().unwrap();
    assert!(
        d.tabulate_default_parameter(2, (280, 140))
            .unwrap_err()
            .contains("counter overflow")
    );
    assert_eq!(d.encode().unwrap(), before);
    d.undo();
    assert_eq!(d.numeric_value(3), Some(1.));
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn source_free_quantity_calculations_drive_transforms_and_survive_expression_edits() {
    let mut doc = Document::new();
    let length = doc.add_calculation_expression("0.3 cm", (10, 20)).unwrap();
    let area = doc
        .add_calculation_expression("2 cm*3 cm", (10, 60))
        .unwrap();
    let scalar = doc
        .add_calculation_expression("sin(pi rad/6)", (10, 100))
        .unwrap();
    let point = doc.add(Kind::Free(Point { x: 150., y: 220. }));
    let image = doc.add(Kind::MarkedDistanceTranslatedPoint(point, length, 0.));
    let bytes = doc.encode().unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.numeric_value(length), Some(0.3));
    assert_eq!(doc.numeric_value(area), Some(6.));
    assert!((doc.numeric_value(scalar).unwrap() - 0.5).abs() < 1e-12);
    assert!(
        doc.position(image).unwrap().distance(Point {
            x: 150. + 0.3 * 37.79527559055118,
            y: 220.,
        }) < 1e-9
    );
    doc.set_calculation_expression(length, "sqrt(2 cm*2 cm)")
        .unwrap();
    assert_eq!(doc.numeric_value(length), Some(2.));
    doc.verify_roundtrip().unwrap();
    assert!(
        doc.position(image).unwrap().distance(Point {
            x: 150. + 2. * 37.79527559055118,
            y: 220.,
        }) < 1e-9
    );
    let edited = doc.encode().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), edited);
    doc.set_calculation_expression(length, "sqrt(-1*(1 cm)^2)")
        .unwrap();
    assert_eq!(doc.numeric_value(length), None);
    assert_eq!(doc.position(image), None);
    doc.verify_roundtrip().unwrap();
    let undefined = doc.encode().unwrap();
    assert!(doc.set_calculation_expression(length, "2").is_err());
    assert_eq!(doc.encode().unwrap(), undefined);
}

#[test]
fn source_free_quantity_authoring_matches_native_writer_golden() {
    let mut doc = Document::new();
    let length = doc.add_calculation_expression("0.3 cm", (10, 20)).unwrap();
    doc.add_calculation_expression("2 cm*3 cm", (10, 60))
        .unwrap();
    doc.add_calculation_expression("sin(pi rad/6)", (10, 100))
        .unwrap();
    doc.add_calculation_expression("abs(-pi rad/6)", (10, 140))
        .unwrap();
    doc.add_calculation_expression("1.+2.", (10, 180)).unwrap();
    let point = doc.add(Kind::Free(Point { x: 150., y: 220. }));
    doc.add(Kind::MarkedDistanceTranslatedPoint(point, length, 0.));
    doc.verify_roundtrip().unwrap();
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/source_free_quantity_writer_native_verified.gsp")
    );
}

fn many_input_calculation_document() -> (Document, usize, usize, usize, usize) {
    let mut doc = Document::new();
    let sources = (1..=51)
        .map(|i| {
            doc.add_named_parameter(&format!("n{i}"), 1., (10, 20 + i * 5))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let expression = (1..=51)
        .map(|i| format!("v{i}"))
        .collect::<Vec<_>>()
        .join("+");
    let sum = doc
        .add_multi_calculation(&sources, &expression, (350, 20))
        .unwrap();
    let length = doc
        .add_parameter_calculation(sum, "v*(1 cm)/100", (350, 60))
        .unwrap();
    let point = doc.add(Kind::Free(Point { x: 350., y: 220. }));
    let image = doc.add(Kind::MarkedDistanceTranslatedPoint(point, length, 0.));
    (doc, sources[50], sum, length, image)
}

#[test]
fn many_input_calculations_update_geometry_and_preserve_save_history() {
    let (mut doc, last, sum, length, image) = many_input_calculation_document();
    assert_eq!(doc.numeric_value(sum), Some(51.));
    assert_eq!(doc.numeric_value(length), Some(0.51));
    let before = doc.encode().unwrap();
    let old = doc.position(image).unwrap();
    doc.set_parameter_value(last, 2.).unwrap();
    assert_eq!(doc.numeric_value(sum), Some(52.));
    assert_eq!(doc.numeric_value(length), Some(0.52));
    assert!((doc.position(image).unwrap().x - old.x - 0.01 * 37.79527559055118).abs() < 1e-10);
    doc.verify_roundtrip().unwrap();
    let after = doc.encode().unwrap();
    let reopened = Document::from_bytes(&after, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(sum), Some(52.));
    assert_eq!(reopened.position(image), doc.position(image));
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), after);
}

#[test]
fn many_input_calculation_writer_matches_native_saved_golden() {
    let (mut doc, last, _, _, _) = many_input_calculation_document();
    doc.set_parameter_value(last, 2.).unwrap();
    doc.verify_roundtrip().unwrap();
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/many_input_calculations_writer_native_verified.gsp"
        )
    );
}

#[test]
fn mixed_quantity_arithmetic_imports_native_sources_and_updates_after_edits() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/mixed_quantity_arithmetic_native_created.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    for (id, expected) in [
        (9, 91.5875),
        (10, -88.4125),
        (11, 3.5875),
        (12, -0.4125),
        (13, 97.9173043608984),
        (14, 9.504804360898404),
    ] {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        assert!((d.numeric_value(id).unwrap() - expected).abs() < 1e-12);
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
    d.set_parameter_value(6, 4.).unwrap();
    assert!((d.numeric_value(11).unwrap() - 5.5875).abs() < 1e-12);
    assert!((d.numeric_value(12).unwrap() + 2.4125).abs() < 1e-12);
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    let radius = d.numeric_value(5).unwrap();
    let angle = d.numeric_value(8).unwrap();
    assert!((d.numeric_value(9).unwrap() - radius - angle).abs() < 1e-12);
    assert!((d.numeric_value(10).unwrap() - radius + angle).abs() < 1e-12);
    assert!((d.numeric_value(11).unwrap() - radius - 4.).abs() < 1e-12);
    assert!((d.numeric_value(12).unwrap() - radius + 4.).abs() < 1e-12);
    d.verify_roundtrip().unwrap();
    let edited = d.encode().unwrap();
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn mixed_quantity_arithmetic_literal_controls_use_native_result_units() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/quantity_literal_arithmetic_native_created.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    for (id, expected) in [
        (9, 90.),
        (10, 6.917304360898404),
        (11, 0.5875),
        (12, 20414.265625),
        (13, 20412.265625),
    ] {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        assert!((d.numeric_value(id).unwrap() - expected).abs() < 1e-9);
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    let compound = (d.numeric_value(5).unwrap() * d.numeric_value(8).unwrap()).powi(2);
    assert!((d.numeric_value(12).unwrap() - compound - 1.).abs() < 1e-9);
    assert!((d.numeric_value(13).unwrap() - compound + 1.).abs() < 1e-9);
    d.verify_roundtrip().unwrap();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn mixed_quantity_arithmetic_original_projection_keeps_formula_and_dependencies() {
    let bytes = include_bytes!("../../../../../tests/fixtures/coordinate_ordinate_formatted_corpus.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(d.objects[82].kind.numeric_unit(), Some(NumericUnit::Scalar));
    let expected = 1.
        + (d.numeric_value(81).unwrap() * d.numeric_value(80).unwrap() / (15. * 3.14159265358979))
            .powi(2);
    assert!((d.numeric_value(83).unwrap() - expected).abs() < 1e-9);
    assert_eq!(d.numeric_value(82), None);
    assert_eq!(d.encode().unwrap(), bytes);
    let initial = d.numeric_value(83).unwrap();
    let mut changed = None;
    for o in &d.objects {
        let Kind::Free(p) = o.kind else {
            continue;
        };
        let mut candidate = d.clone();
        candidate.checkpoint();
        if candidate.move_point(
            o.id,
            Point {
                x: p.x + 11.,
                y: p.y - 13.,
            },
        ) && candidate
            .numeric_value(83)
            .is_some_and(|v| (v - initial).abs() > 1e-9)
        {
            changed = Some(candidate);
            break;
        }
    }
    let mut edited = changed.expect("The original projection must retain its source graph");
    let expected = 1.
        + (edited.numeric_value(81).unwrap() * edited.numeric_value(80).unwrap()
            / (15. * 3.14159265358979))
            .powi(2);
    assert!((edited.numeric_value(83).unwrap() - expected).abs() < 1e-9);
    edited.undo();
    assert_eq!(edited.encode().unwrap(), bytes);
}

fn mixed_quantity_arithmetic_writer_document() -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    for (i, sources, expression) in [
        (0, [5, 8], "v1+v2"),
        (1, [5, 8], "v1-v2"),
        (2, [5, 6], "v1+v2"),
        (3, [5, 6], "v1-v2"),
        (4, [4, 8], "v1+v2"),
        (5, [4, 5], "v1+v2"),
        (6, [5, 8], "(v1*v2)^2+1"),
        (7, [5, 8], "(v1*v2)^2-1"),
    ] {
        d.add_multi_calculation(&sources, expression, (10, 150 + 26 * i))
            .unwrap();
    }
    d.add_parameter_calculation(8, "v-0", (10, 358)).unwrap();
    d.add_parameter_calculation(5, "v-1", (10, 384)).unwrap();
    d
}

#[test]
fn mixed_quantity_arithmetic_actual_api_writer_matches_native_before_and_after_edits() {
    let mut d = mixed_quantity_arithmetic_writer_document();
    let original = include_bytes!(
        "../../../../../tests/fixtures/mixed_quantity_writer_original_native_verified.gsp"
    );
    let edited =
        include_bytes!("../../../../../tests/fixtures/mixed_quantity_writer_edited_native_verified.gsp");
    assert_eq!(d.encode().unwrap(), original);
    d.verify_roundtrip().unwrap();
    for id in 9..=18 {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
    }
    d.set_parameter_value(6, 4.).unwrap();
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    assert!((d.numeric_value(12).unwrap() + 1.61875).abs() < 1e-12);
    assert!((d.numeric_value(15).unwrap() - 45930.84765625).abs() < 1e-9);
    assert_eq!(d.encode().unwrap(), edited);
    d.verify_roundtrip().unwrap();
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), original);
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn quantity_exponents_native_sources_are_scalar_magnitudes_with_live_values() {
    let bytes = include_bytes!("../../../../../tests/fixtures/quantity_exponents_native_created.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    for (id, base, source) in [
        (9, 2., 5),
        (10, 2., 4),
        (11, 2., 8),
        (12, d.numeric_value(5).unwrap(), 8),
    ] {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        let expected = base.powf(d.numeric_value(source).unwrap());
        assert!((d.numeric_value(id).unwrap() / expected - 1.).abs() < 1e-12);
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    assert!(
        (d.numeric_value(9).unwrap() / 2_f64.powf(d.numeric_value(5).unwrap()) - 1.).abs() < 1e-12
    );
    assert!(
        (d.numeric_value(10).unwrap() / 2_f64.powf(d.numeric_value(4).unwrap()) - 1.).abs() < 1e-12
    );
    let p = d.position(7).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        7,
        Point {
            x: p.x + 60.,
            y: p.y
        }
    ));
    let angle = d.numeric_value(8).unwrap();
    assert!((angle - 45.).abs() < 1e-12);
    assert!((d.numeric_value(11).unwrap() / 2_f64.powf(angle) - 1.).abs() < 1e-12);
    assert!(
        (d.numeric_value(12).unwrap() / d.numeric_value(5).unwrap().powf(angle) - 1.).abs() < 1e-12
    );
    d.verify_roundtrip().unwrap();
    let edited = d.encode().unwrap();
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn quantity_exponents_original_trigonometry_keeps_measurement_dependencies() {
    let bytes = include_bytes!("../../../../../tests/fixtures/quantity_exponent_trigonometry_corpus.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(d.objects[70].kind.numeric_unit(), Some(NumericUnit::Scalar));
    assert_eq!(d.numeric_value(71), Some(1.));
    assert_eq!(d.encode().unwrap(), bytes);
    let (_, end, _) = d.line_points(19).unwrap();
    let mut edited = d.clone();
    edited.checkpoint();
    assert!(edited.move_point(69, end));
    assert_ne!(edited.numeric_value(71), d.numeric_value(71));
    let expected =
        (edited.numeric_value(68).unwrap() * 5_f64.powf(edited.numeric_value(70).unwrap())).trunc()
            + 1.;
    assert_eq!(edited.numeric_value(71), Some(expected));
    // This original also contains a Movement button with a constrained source,
    // whose edited serialization is outside the current supported button layout.
    edited.undo();
    assert_eq!(edited.numeric_value(71), d.numeric_value(71));
    assert_eq!(edited.position(69), d.position(69));
    edited.redo();
    assert_eq!(edited.numeric_value(71), Some(expected));
}

#[test]
fn quantity_exponents_native_literals_keep_units_and_pixel_conversion() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/quantity_literal_exponents_native_created.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    let radius = d.numeric_value(5).unwrap();
    let pixels = 2. / 37.79527559055118;
    for (id, expected) in [
        (9, radius.powi(2)),
        (10, radius.powf(pixels)),
        (11, 4.),
        (12, 2_f64.powf(pixels)),
    ] {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        assert!((d.numeric_value(id).unwrap() - expected).abs() < 1e-12);
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
    let mut edited = d.clone();
    edited.set_parameter_value(6, -2.).unwrap();
    assert_eq!(edited.numeric_value(11), Some(4.));
    assert_eq!(edited.numeric_value(12), None);
    edited.verify_roundtrip().unwrap();
    edited.undo();
    assert_eq!(edited.encode().unwrap(), bytes);
}

fn quantity_exponents_writer_document() -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    for (i, source, expression) in [
        (0, 5, "2^v"),
        (1, 4, "2^v"),
        (2, 8, "2^v"),
        (4, 5, "v^2°"),
        (5, 5, "v^2px"),
        (6, 6, "n^2°"),
        (7, 6, "n^2px"),
    ] {
        d.add_parameter_calculation(source, expression, (10, 150 + 26 * i))
            .unwrap();
    }
    d.add_multi_calculation(&[5, 8], "v1^v2", (10, 228))
        .unwrap();
    d.add_calculation_expression("2px", (10, 358)).unwrap();
    d
}

#[test]
fn quantity_exponents_actual_api_writer_matches_native_before_and_after_edits() {
    let mut d = quantity_exponents_writer_document();
    let original = include_bytes!(
        "../../../../../tests/fixtures/quantity_exponents_writer_original_native_verified.gsp"
    );
    let edited = include_bytes!(
        "../../../../../tests/fixtures/quantity_exponents_writer_edited_native_verified.gsp"
    );
    assert_eq!(d.encode().unwrap(), original);
    d.verify_roundtrip().unwrap();
    for id in 9..=16 {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
    }
    assert_eq!(d.objects[16].kind.numeric_unit(), Some(NumericUnit::Length));
    assert!((d.numeric_value(17).unwrap() - 2. / 37.79527559055118).abs() < 1e-14);
    d.set_parameter_value(6, -2.).unwrap();
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    let p = d.position(7).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        7,
        Point {
            x: p.x + 60.,
            y: p.y
        }
    ));
    assert_eq!(d.numeric_value(14), Some(4.));
    assert_eq!(d.numeric_value(15), None);
    assert_eq!(d.numeric_value(11), Some(2_f64.powf(45.)));
    assert!(
        (d.numeric_value(16).unwrap() / d.numeric_value(5).unwrap().powf(45.) - 1.).abs() < 1e-12
    );
    assert_eq!(d.encode().unwrap(), edited);
    d.verify_roundtrip().unwrap();
    d.undo();
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), original);
    d.redo();
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn function_applications_native_constant_and_quantity_literals_keep_function_dependencies() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/function_application_literal_native_created.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/function_application_length_literal_native_created.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/function_application_degree_literal_native_created.gsp"
        )
        .as_slice(),
    ] {
        let mut d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.numeric_value(12), Some(5.));
        assert_eq!(d.objects[11].kind.numeric_unit(), Some(NumericUnit::Scalar));
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        d.set_function_expression(
            9,
            crate::function_records::SimpleFunction::parse("x^3+1").unwrap(),
        )
        .unwrap();
        assert_eq!(d.numeric_value(12), Some(9.));
        d.verify_roundtrip().unwrap();
        d.undo();
        assert_eq!(d.encode().unwrap(), bytes);
    }
}

#[test]
fn function_applications_native_parameter_derivative_and_length_sources_remain_live() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/function_application_controls_native_created.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    for (id, expected) in [
        (12, 5.),
        (13, 2_f64.sqrt()),
        (14, 4.),
        (15, d.numeric_value(5).unwrap().powi(2) + 1.),
    ] {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        assert_eq!(d.numeric_value(id), Some(expected));
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
    d.set_parameter_value(6, -2.).unwrap();
    assert_eq!(d.numeric_value(12), Some(5.));
    assert_eq!(d.numeric_value(13), None);
    assert_eq!(d.numeric_value(14), Some(-4.));
    d.set_function_expression(
        9,
        crate::function_records::SimpleFunction::parse("x^3+1").unwrap(),
    )
    .unwrap();
    assert_eq!(d.numeric_value(12), Some(-7.));
    assert_eq!(d.numeric_value(14), Some(12.));
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    assert_eq!(
        d.numeric_value(15),
        Some(d.numeric_value(5).unwrap().powi(3) + 1.)
    );
    d.verify_roundtrip().unwrap();
    let edited = d.encode().unwrap();
    d.undo();
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    d.redo();
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn function_applications_original_transform_sample_uses_its_defined_function() {
    let bytes = include_bytes!("../../../../../tests/fixtures/function_application_transform_corpus.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    let x = d.numeric_value(93).unwrap();
    assert_eq!(x, -1.25);
    let expected = x.powi(2) - 4. * x + 3.;
    assert_eq!(d.function_value(1, x), Some(expected));
    assert_eq!(d.numeric_value(94), Some(expected));
    assert_eq!(
        d.parameter_calculation_expression(94).as_deref(),
        Some("f(x)")
    );
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn function_applications_nested_calls_validate_reference_roles_and_angle_mode() {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_application_base_port.gsp"),
        None,
    )
    .unwrap();
    let nested = d
        .add_multi_calculation(&[9, 10, 6], "f1(f2(v3))+f1(3)", (10, 150))
        .unwrap();
    assert!((d.numeric_value(nested).unwrap() - 13.).abs() < 1e-12);
    let reverse = d
        .add_multi_calculation(&[6, 9], "f2(v1)", (10, 180))
        .unwrap();
    assert_eq!(d.numeric_value(reverse), Some(5.));
    assert!(
        d.add_multi_calculation(&[9, 6], "v1+v2", (10, 200))
            .is_err()
    );
    assert!(
        d.add_multi_calculation(&[9, 6], "f2(v1)", (10, 200))
            .is_err()
    );
    assert!(
        d.add_multi_calculation(&[9, 6], "f1(v2)+v1", (10, 200))
            .is_err()
    );
    assert!(
        d.add_multi_calculation(&[9, 6], "f3(v2)", (10, 200))
            .is_err()
    );
    assert!(
        d.add_multi_calculation(&[9, 6], "f1(v2", (10, 200))
            .is_err()
    );
    let sine = d
        .add_function_definition(crate::function_records::SimpleFunction::parse("sin(x)").unwrap())
        .unwrap();
    let angle = d
        .add_multi_calculation(&[sine, 8], "f1(v2)", (10, 210))
        .unwrap();
    assert!((d.numeric_value(angle).unwrap() - 1.).abs() < 1e-12);
    let scalar_angle = d
        .add_multi_calculation(&[sine], "f1(90)", (10, 240))
        .unwrap();
    assert!((d.numeric_value(scalar_angle).unwrap() - 90_f64.sin()).abs() < 1e-12);
    d.set_function_expression_with_angle_unit(
        sine,
        crate::function_records::SimpleFunction::parse("sin(x)").unwrap(),
        false,
    )
    .unwrap();
    assert!((d.numeric_value(angle).unwrap() - 1.).abs() < 1e-12);
    d.verify_roundtrip().unwrap();
    d.set_parameter_value(6, -2.).unwrap();
    assert_eq!(d.numeric_value(nested), None);
    d.verify_roundtrip().unwrap();
}

fn function_applications_writer_document() -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_application_base_port.gsp"),
        None,
    )
    .unwrap();
    for (i, sources, expression) in [
        (0, vec![9], "f1(2)"),
        (1, vec![9, 6], "f1(v2)"),
        (2, vec![10, 6], "f1(v2)"),
        (3, vec![11, 6], "f1(v2)"),
        (4, vec![9, 5], "f1(v2)"),
        (5, vec![9, 10, 6], "f1(f2(v3))"),
        (6, vec![10, 9, 6], "f1(f2(v3))"),
        (7, vec![10, 11, 6], "f1(v3)+f2(v3)"),
    ] {
        d.add_multi_calculation(&sources, expression, (330, 150 + 26 * i))
            .unwrap();
    }
    let sine = d
        .add_function_definition(crate::function_records::SimpleFunction::parse("sin(x)").unwrap())
        .unwrap();
    d.objects[sine - 1].function_anchor = Some((10, 380));
    d.objects[sine - 1].function_name = Some("s".into());
    d.add_multi_calculation(&[sine, 8], "f1(v2)", (330, 358))
        .unwrap();
    d.add_multi_calculation(&[9], "f1(2°)", (330, 384)).unwrap();
    let image = d.add(Kind::PolarTranslatedPoint(
        2,
        -90.,
        37.79527559055118,
        1.5875,
    ));
    let label = d.add_point_numeric_label(image, 13).unwrap();
    d.add_point_parameter_iteration(2, 6, image, 13, label, IterationDepth::Fixed(3))
        .unwrap();
    d
}

#[test]
fn function_applications_actual_api_writer_matches_native_calls_and_iteration_edits() {
    let mut d = function_applications_writer_document();
    let original = include_bytes!(
        "../../../../../tests/fixtures/function_applications_writer_original_native_verified.gsp"
    );
    let edited = include_bytes!(
        "../../../../../tests/fixtures/function_applications_writer_edited_native_verified.gsp"
    );
    assert_eq!(d.encode().unwrap(), original);
    d.verify_roundtrip().unwrap();
    for id in (12..=19).chain([21, 22]) {
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
    }
    assert_eq!(d.numeric_value(12), Some(5.));
    assert_eq!(d.numeric_value(21), Some(1.));
    assert_eq!(d.numeric_value(22), Some(2_f64.to_radians().powi(2) + 1.));
    let labels = d.point_parameter_iteration_labels(25).unwrap();
    assert_eq!(
        labels
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["26.00", "677.00", "458330.00"]
    );
    assert_eq!(
        labels.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [
            Point { x: 220., y: 300. },
            Point { x: 220., y: 360. },
            Point { x: 220., y: 420. }
        ]
    );
    d.set_parameter_value(6, -2.).unwrap();
    d.set_function_expression(
        9,
        crate::function_records::SimpleFunction::parse("x^3+1").unwrap(),
    )
    .unwrap();
    d.set_function_expression_with_angle_unit(
        20,
        crate::function_records::SimpleFunction::parse("sin(x)").unwrap(),
        false,
    )
    .unwrap();
    let p = d.position(2).unwrap();
    d.checkpoint();
    assert!(d.move_point(
        2,
        Point {
            x: p.x + 30.,
            y: p.y
        }
    ));
    assert_eq!(d.numeric_value(12), Some(9.));
    assert_eq!(d.numeric_value(13), Some(-7.));
    assert_eq!(d.numeric_value(15), Some(12.));
    assert_eq!(d.numeric_value(22), Some(9.));
    for id in [14, 17, 18, 19] {
        assert_eq!(d.numeric_value(id), None);
    }
    let labels = d.point_parameter_iteration_labels(25).unwrap();
    assert_eq!(
        labels
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["-342.00", "-4.00 · 10⁷", "-6.40 · 10²²"]
    );
    assert_eq!(
        labels.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [
            Point { x: 250., y: 300. },
            Point { x: 250., y: 360. },
            Point { x: 250., y: 420. }
        ]
    );
    assert_eq!(d.encode().unwrap(), edited);
    d.verify_roundtrip().unwrap();
    for _ in 0..4 {
        d.undo();
    }
    assert_eq!(d.encode().unwrap(), original);
    for _ in 0..4 {
        d.redo();
    }
    assert_eq!(d.encode().unwrap(), edited);
}
