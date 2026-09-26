use super::*;

#[test]
fn polygon_perimeter_matches_original_and_updates_after_drag() {
    let original = Document::open(Path::new("tests/fixtures/wine_polygon_perimeter.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::Polygon(vec![1, 2, 3]));
    assert_eq!(original.objects[4].kind, Kind::PolygonPerimeter(4));
    assert!((original.perimeter_cm(5).unwrap() - 9.525).abs() < 0.001);
    let edited = Document::open(Path::new(
        "tests/fixtures/wine_moved_generated_polygon_perimeter.gsp",
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
    let native = include_bytes!("../../../tests/fixtures/calculation_length_times_two_native.gsp");
    let imported = Document::from_bytes(native, None).unwrap();
    assert!(!imported.partial_view);
    assert_eq!(imported.encode().unwrap(), native);
    let length = imported.segment_length_cm(4).unwrap();
    assert!((imported.measured_calculation_cm(5).unwrap() - length * 2.).abs() < 1e-10);

    let mut generated = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/wine_segment_length.gsp"),
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
        include_bytes!("../../../tests/fixtures/calculation_distance_times_two_native.gsp");
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
        include_bytes!("../../../tests/fixtures/calculation_distance_times_three_port.gsp");
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
        include_bytes!("../../../tests/fixtures/calculation_perimeter_times_two_native.gsp");
    let perimeter = Document::from_bytes(perimeter_native, None).unwrap();
    assert!(!perimeter.partial_view);
    assert_eq!(perimeter.encode().unwrap(), perimeter_native);
    assert!(
        (perimeter.measured_calculation_cm(6).unwrap() - perimeter.perimeter_cm(5).unwrap() * 2.)
            .abs()
            < 1e-10
    );
    let perimeter_port =
        include_bytes!("../../../tests/fixtures/calculation_perimeter_times_three_port.gsp");
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
                "../../../tests/fixtures/calculation_circumference_times_two_native.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/fixtures/calculation_circumference_times_three_port.gsp"
            )
            .as_slice(),
            "Circumference",
            Document::circle_circumference_cm as fn(&Document, usize) -> Option<f64>,
        ),
        (
            include_bytes!("../../../tests/fixtures/calculation_radius_times_two_native.gsp")
                .as_slice(),
            include_bytes!("../../../tests/fixtures/calculation_radius_times_three_port.gsp")
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
        include_bytes!("../../../tests/fixtures/calculation_arc_length_times_two_native.gsp");
    let arc = Document::from_bytes(arc_native, None).unwrap();
    assert!(!arc.partial_view);
    assert_eq!(arc.encode().unwrap(), arc_native);
    assert!(
        (arc.measured_calculation_cm(6).unwrap() - arc.arc_length_cm(5).unwrap() * 2.).abs()
            < 1e-10
    );
    let arc_port =
        include_bytes!("../../../tests/fixtures/calculation_arc_length_times_three_port.gsp");
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
        include_bytes!("../../../tests/fixtures/calculation_marked_distance_port.gsp");
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
        include_bytes!("../../../tests/fixtures/calculation_length_divide_two_native.gsp");
    let divided = Document::from_bytes(divided_native, None).unwrap();
    assert!(!divided.partial_view);
    assert_eq!(divided.encode().unwrap(), divided_native);
    assert!(
        (divided.measured_calculation_cm(5).unwrap() - divided.segment_length_cm(4).unwrap() / 2.)
            .abs()
            < 1e-10
    );
    let divided_port =
        include_bytes!("../../../tests/fixtures/calculation_length_divide_three_port.gsp");
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
            include_bytes!("../../../tests/fixtures/calculation_length_times_twelve_native.gsp")
                .as_slice(),
            include_bytes!(
                "../../../tests/fixtures/calculation_length_times_twenty_three_port.gsp"
            )
            .as_slice(),
            23u8,
            false,
        ),
        (
            include_bytes!("../../../tests/fixtures/calculation_length_divide_twelve_native.gsp")
                .as_slice(),
            include_bytes!("../../../tests/fixtures/calculation_length_divide_twelve_port.gsp")
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
        include_bytes!("../../../tests/fixtures/calculation_two_lengths_product_ratio_port.gsp");
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
        include_bytes!("../../../tests/fixtures/calculation_two_lengths_sum_native_verified.gsp");
    let native = Document::from_bytes(native, None).unwrap();
    assert!(!native.partial_view);
    assert!(
        (native.length_pair_calculation_cm(9).unwrap() - 300. / 37.79527559055118).abs() < 1e-10
    );
    let bytes = include_bytes!("../../../tests/fixtures/calculation_two_lengths_port.gsp");
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
        include_bytes!("../../../tests/fixtures/calculation_length_chained_division_port.gsp");
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
        include_bytes!("../../../tests/fixtures/parameter_decimal_calculation_native_verified.gsp");
    let native_doc = Document::from_bytes(native, None).unwrap();
    assert!(!native_doc.partial_view);
    assert_eq!(native_doc.encode().unwrap(), native);
    assert_eq!(native_doc.parameter_calculation_value(2), Some(5.));
    let bytes = include_bytes!("../../../tests/fixtures/parameter_decimal_calculation_port.gsp");
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
    for expression in ["speed+1.", "speed+1.2.3", "speed+00.5", "speed/0.0"] {
        assert!(
            doc.add_parameter_calculation(1, expression, (10, 120))
                .is_err(),
            "{expression}"
        );
    }
    let fractional = doc
        .add_parameter_calculation(1, "speed^0.5", (10, 120))
        .unwrap();
    assert!((doc.parameter_calculation_value(fractional).unwrap() - 3_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn parameter_calculation_chain_imports_and_updates() {
    let native = include_bytes!("../../../tests/fixtures/parameter_calculation_chain_native.gsp");
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
        include_bytes!("../../../tests/fixtures/parameter_calculation_generated_port.gsp");
    let mut document = Document::from_bytes(complex, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), complex);
    assert_eq!(document.parameter_calculation_value(2), Some(3.5));
    document.set_parameter_value(1, 3.).unwrap();
    assert_eq!(document.parameter_calculation_value(2), Some(6.));

    let even = include_bytes!("../../../tests/fixtures/parameter_calculation_even_tokens_port.gsp");
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

    let named = include_bytes!("../../../tests/fixtures/named_parameter_calculation_port.gsp");
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
            .add_parameter_calculation(1, "speed+9999999999999999999999999999999", (10, 100))
            .is_err()
    );
}

#[test]
fn chained_parameter_calculations_update_and_remap() {
    let bytes = include_bytes!("../../../tests/fixtures/parameter_calculation_chained_port.gsp");
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
        let path = format!("tests/fixtures/{path}.gsp");
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
    let original = Document::open(Path::new("tests/fixtures/wine_segment_length.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::SegmentLength(3));
    let (a, b, _) = original.line_points(3).unwrap();
    assert!(
        (original.segment_length_cm(4).unwrap() - a.distance(b) / 37.79527559055118).abs() < 1e-10
    );
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_length.gsp")).unwrap();
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
