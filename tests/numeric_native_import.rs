use sketchpad_rs::document::{Document, Kind, records};
use sketchpad_rs::table_records::{
    CalculationGroup, NumericUnit, ParameterCalculationGroup, ParameterGroup,
};

const COOD_TRACE: &[u8] = include_bytes!("fixtures/numeric_cood_trace_native.gsp");
const CIRCLE_FORMATION: &[u8] = include_bytes!("fixtures/numeric_circle_formation_native.gsp");

// These regressions copy live constructions, so select their prerequisites too.
// A lone numeric-coordinate point is copied as a free point in Native.
fn select_construction_graph(document: &mut Document, root: usize) {
    let mut pending = vec![root];
    let mut visited = std::collections::BTreeSet::new();
    while let Some(id) = pending.pop() {
        if visited.insert(id) {
            document.selected.insert(id);
            pending.extend(document.dependencies_of(id).unwrap());
        }
    }
}

fn object_group(bytes: &[u8], id: usize) -> Vec<u8> {
    let rs = records(bytes).unwrap();
    let start = rs
        .iter()
        .filter(|r| r.kind == 0x07d0)
        .nth(id - 1)
        .unwrap()
        .start;
    let end = rs
        .iter()
        .filter(|r| r.kind == 0x07d7)
        .nth(id - 1)
        .unwrap()
        .data
        .end;
    bytes[start..end].to_vec()
}

fn payload(bytes: &[u8], kind: u32) -> Vec<u8> {
    let mut document = b"GSP4".to_vec();
    document.extend_from_slice(bytes);
    let rs = records(&document).unwrap();
    let record = rs.iter().find(|r| r.kind == kind).unwrap();
    document[record.data.clone()].to_vec()
}

#[test]
fn native_cood_trace_import_evaluates_and_edits_its_parameter() {
    // The paired sibling cood-trace.htm defines t[1]=1 and Calculate(t[1]+1).
    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    assert!(matches!(document.objects[0].kind, Kind::Parameter(_)));
    assert!(matches!(
        document.objects[1].kind,
        Kind::ParameterCalculation(1, _)
    ));
    assert_eq!(document.numeric_value(1), Some(1.));
    assert_eq!(document.numeric_value(2), Some(2.));
    assert_eq!(document.encode().unwrap(), COOD_TRACE);
    assert!(!document.partial_view);
    assert!(matches!(
        document.objects[9].kind,
        Kind::ParameterLocus(_, 0., 1., 500)
    ));
    document.set_parameter_value(1, 3.5).unwrap();
    assert_eq!(document.numeric_value(2), Some(4.5));
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.numeric_value(1), Some(3.5));
    assert_eq!(reopened.numeric_value(2), Some(4.5));
    let before = object_group(COOD_TRACE, 1);
    let after = object_group(&encoded, 1);
    for kind in [0x07d0, 0x07d5, 0x07d8] {
        assert_eq!(payload(&before, kind), payload(&after, kind));
    }
    assert_eq!(object_group(&encoded, 2), object_group(COOD_TRACE, 2));
}

#[test]
fn native_font_and_named_numeric_caption_preserve_their_presentation() {
    let raw_parameter = object_group(CIRCLE_FORMATION, 5);
    let mut parameter = ParameterGroup::parse(&raw_parameter).unwrap();
    assert_eq!(parameter.name(), "t[2]");
    assert_eq!(parameter.value(), 5.);
    assert_eq!(parameter.precision(), 0);
    assert_eq!(payload(&raw_parameter, 0x07d8), [1, 0]);
    parameter.set_value(7.).unwrap();
    parameter.set_anchor((100, 200)).unwrap();
    for kind in [0x07d0, 0x07d5, 0x07d8] {
        assert_eq!(
            payload(&raw_parameter, kind),
            payload(parameter.as_bytes(), kind)
        );
    }
    assert_eq!(parameter.anchor(), (100, 200));
    assert_eq!(parameter.bbox().0, parameter.anchor());

    let raw_calculation = object_group(CIRCLE_FORMATION, 6);
    let mut calculation = ParameterCalculationGroup::parse(&raw_calculation)
        .unwrap()
        .with_source_units(&[NumericUnit::Scalar])
        .unwrap();
    assert_eq!(calculation.sources(), [5]);
    assert_eq!(calculation.value(7.), Some(7.));
    calculation.set_precision(4).unwrap();
    calculation.set_anchor((120, 220)).unwrap();
    calculation = calculation.remap_source(9).unwrap();
    assert_eq!(calculation.sources(), [9]);
    assert_eq!(calculation.value(11.), Some(11.));
    assert_eq!(calculation.anchor(), (120, 220));
    assert_eq!(calculation.precision(), 4);
    for kind in [0x07d0, 0x07d5, 0x07d8] {
        assert_eq!(
            payload(&raw_calculation, kind),
            payload(calculation.as_bytes(), kind)
        );
    }
}

#[test]
fn numeric_import_rejects_invalid_math_independently_of_presentation() {
    let raw = object_group(COOD_TRACE, 2);
    let mut invalid_prefix = raw.clone();
    invalid_prefix[60] ^= 1;
    assert!(ParameterCalculationGroup::parse(&invalid_prefix).is_err());
    let mut invalid_count = raw.clone();
    invalid_count[104..106].copy_from_slice(&4_u16.to_le_bytes());
    assert!(ParameterCalculationGroup::parse(&invalid_count).is_err());
    let mut invalid_source = raw.clone();
    invalid_source[152..154].copy_from_slice(&0x6001_u16.to_le_bytes());
    assert!(ParameterCalculationGroup::parse(&invalid_source).is_err());
    let mut retained_units = raw.clone();
    retained_units[16..18].copy_from_slice(&49_u16.to_le_bytes());
    let group = ParameterCalculationGroup::parse(&retained_units)
        .unwrap().with_source_units(&[NumericUnit::Scalar]).unwrap();
    assert_eq!(group.unit(), NumericUnit::Scalar);
    assert_eq!(group.as_bytes(), retained_units);
    let mut invalid_label = object_group(CIRCLE_FORMATION, 6);
    let label_start = 154; // One-token numeric record ends here.
    invalid_label[label_start + 8 + 22..label_start + 8 + 24]
        .copy_from_slice(&u16::MAX.to_le_bytes());
    assert!(ParameterCalculationGroup::parse(&invalid_label).is_err());
}

#[test]
fn parameter_edits_keep_a_custom_native_animation_step() {
    let mut raw = object_group(CIRCLE_FORMATION, 5);
    // Native parameters store animation step separately from display precision.
    raw[44 + 52..44 + 60].copy_from_slice(&0.5_f64.to_le_bytes());
    let mut parameter = ParameterGroup::parse(&raw).unwrap();
    parameter.set_value(7.125).unwrap();
    assert_eq!(parameter.value(), 7.125);
    assert_eq!(parameter.precision(), 3);
    assert_eq!(
        &payload(parameter.as_bytes(), 0x0907)[52..60],
        &0.5_f64.to_le_bytes()
    );
    assert_eq!(parameter.animation_domain(), (-100., 100.));
}

#[test]
fn literal_numeric_caption_accepts_font_and_label_without_losing_math() {
    let mut raw = CalculationGroup::from_expression("12/3")
        .unwrap()
        .as_bytes()
        .to_vec();
    let rs = {
        let mut document = b"GSP4".to_vec();
        document.extend_from_slice(&raw);
        records(&document).unwrap()
    };
    let font = rs.iter().find(|r| r.kind == 0x07d8).unwrap();
    let insertion = font.start - 4;
    raw[font.data.start - 4..font.data.end - 4].copy_from_slice(&1_u16.to_le_bytes());
    let label = payload(&object_group(CIRCLE_FORMATION, 6), 0x07d5);
    let mut label_record = (label.len() as u32).to_le_bytes().to_vec();
    label_record.extend_from_slice(&0x07d5_u32.to_le_bytes());
    label_record.extend_from_slice(&label);
    raw.splice(insertion..insertion, label_record);
    let mut calculation = CalculationGroup::parse(&raw).unwrap();
    assert_eq!(calculation.value(), 4.);
    calculation.set_anchor((300, 400)).unwrap();
    calculation.set_precision(5).unwrap();
    assert_eq!(calculation.value(), 4.);
    assert_eq!(calculation.anchor(), (300, 400));
    assert_eq!(calculation.precision(), 5);
    assert_eq!(payload(calculation.as_bytes(), 0x07d5), label);
    assert_eq!(payload(calculation.as_bytes(), 0x07d8), [1, 0]);
}

#[test]
fn native_coordinate_point_tracks_both_numeric_sources_and_remaps() {
    let bytes = COOD_TRACE.to_vec();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        document.objects[8].kind,
        Kind::NumericCoordinatePoint(1, 2, 8)
    ));
    assert!(!document.partial_view);
    let before = document.position(9).unwrap();
    let scale = 37.79527559055118;
    assert!((before.x - (816. + scale)).abs() < 1e-10);
    assert!((before.y - (418. - 2. * scale)).abs() < 1e-10);
    document.set_parameter_value(1, 3.5).unwrap();
    let after = document.position(9).unwrap();
    assert!((after.x - before.x - 2.5 * scale).abs() < 1e-10);
    assert!((after.y - before.y + 2.5 * scale).abs() < 1e-10);
    let encoded = document.encode().unwrap();
    assert_eq!(object_group(&encoded, 9), object_group(&bytes, 9));
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.position(9), Some(after));
    select_construction_graph(&mut document, 9);
    let clipboard = document.copy_selected().unwrap();
    let pasted = document.paste(&clipboard).unwrap();
    let copied = *pasted.last().unwrap();
    let Kind::NumericCoordinatePoint(x, y, frame) = document.objects[copied - 1].kind else {
        panic!("Expected copied numeric point")
    };
    assert!(x > 9 && y > 9 && frame > 9);
    assert_eq!(document.numeric_value(x), Some(3.5));
    assert_eq!(document.numeric_value(y), Some(4.5));
    let copied_point = document.position(copied).unwrap();
    assert!((copied_point.x - after.x - 20.).abs() < 1e-10);
    assert!((copied_point.y - after.y - 20.).abs() < 1e-10);
    let copied_bytes = document.encode().unwrap();
    let copied_reopened = Document::from_bytes(&copied_bytes, None).unwrap();
    assert_eq!(copied_reopened.position(copied), Some(copied_point));
}

#[test]
fn native_parameter_locus_keeps_its_range_live_dependencies_and_gap_breaks() {
    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    let before = document.locus_strokes(10, 500).unwrap();
    assert_eq!(before.len(), 499);
    let scale = 37.79527559055118;
    let start = before.first().unwrap().0;
    let end = before.last().unwrap().1;
    assert!((start.x - 816.).abs() < 1e-10);
    assert!((start.y - (418. - scale)).abs() < 1e-10);
    assert!((end.x - (816. + scale)).abs() < 1e-10);
    assert!((end.y - (418. - 2. * scale)).abs() < 1e-10);
    document.set_parameter_value(1, 3.5).unwrap();
    assert_eq!(document.locus_strokes(10, 500).unwrap(), before);
    assert_eq!(document.numeric_value(1), Some(3.5));
    let saved = document.encode().unwrap();
    assert_eq!(object_group(&saved, 10), object_group(COOD_TRACE, 10));
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(reopened.locus_strokes(10, 500).unwrap(), before);

    // Sampling preserves undefined slots and never joins through a pole.
    let expression =
        ParameterCalculationGroup::from_expression(1, "t[1]", "1/(t[1]-0.5)", (10, 38)).unwrap();
    document.objects[1].kind = Kind::ParameterCalculation(1, expression);
    let locus = document.construct_parameter_locus(1, 9, 0., 1., 5).unwrap();
    let segments = document.locus_strokes(locus, 5).unwrap();
    assert_eq!(segments.len(), 2);
    assert!(segments[0].1.x < 816. + 0.5 * scale);
    assert!(segments[1].0.x > 816. + 0.5 * scale);
    assert_eq!(document.numeric_value(1), Some(3.5));
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.locus_strokes(locus, 5).unwrap(), segments);
}

#[test]
fn authored_parameter_locus_matches_native_saved_file_and_copies_its_driver() {
    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    document.set_parameter_value(1, 3.5).unwrap();
    assert_eq!(
        document.encode().unwrap(),
        include_bytes!("fixtures/numeric_cood_trace_edited_native_verified.gsp")
    );
    let added = document.construct_locus(1, 9).unwrap();
    assert_eq!(added, 11);
    let bytes = document.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("fixtures/numeric_cood_trace_added_locus_native_verified.gsp")
    );
    document.selected.insert(10);
    let clipboard = document.copy_selected().unwrap();
    let pasted = document.paste(&clipboard).unwrap();
    let copied = *pasted.last().unwrap();
    let Kind::ParameterLocus(ref refs, 0., 1., 500) = document.objects[copied - 1].kind else {
        panic!("Expected copied parameter locus")
    };
    assert!(refs.iter().all(|id| *id > 11));
    assert_eq!(refs.first(), refs.last());
    let driver = refs[1];
    document.set_parameter_value(driver, 7.).unwrap();
    assert_eq!(document.numeric_value(1), Some(3.5));
    assert_eq!(document.numeric_value(driver), Some(7.));
    let saved = document.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(
        reopened.locus_strokes(copied, 500),
        document.locus_strokes(copied, 500)
    );
    // Removing the copied driver removes its numeric descendants and locus.
    document.selected.clear();
    document.selected.insert(driver);
    let removed = document.clear_selected().unwrap();
    assert!(removed.contains(&copied));
    assert!(!document.objects[0].deleted);
    let saved = document.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(reopened.locus_strokes(10, 500).is_some());
}

#[test]
fn imported_parameter_locus_range_edits_preserve_native_descriptor_mode() {
    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    document.set_plot_range(10, -1., 2., 17).unwrap();
    let segments = document.locus_strokes(10, 500).unwrap();
    assert_eq!(segments.len(), 16);
    let scale = 37.79527559055118;
    let start = segments.first().unwrap().0;
    let end = segments.last().unwrap().1;
    assert!((start.x - (816. - scale)).abs() < 1e-10);
    assert!((start.y - 418.).abs() < 1e-10);
    assert!((end.x - (816. + 2. * scale)).abs() < 1e-10);
    assert!((end.y - (418. - 3. * scale)).abs() < 1e-10);
    let encoded = document.encode().unwrap();
    let original = payload(&object_group(COOD_TRACE, 10), 0x0902);
    let changed = payload(&object_group(&encoded, 10), 0x0902);
    assert_eq!(&changed[20..], &original[20..]);
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(matches!(
        reopened.objects[9].kind,
        Kind::ParameterLocus(_, -1., 2., 17)
    ));
    assert_eq!(reopened.locus_strokes(10, 500).unwrap(), segments);
    assert!(document.set_plot_range(10, 2., -1., 17).is_err());
    assert!(document.set_plot_range(10, -1., 2., 1).is_err());
    assert_eq!(document.encode().unwrap(), encoded);
}

#[test]
fn numeric_point_and_parameter_trace_deletion_cascade_and_guard() {
    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    document.selected.insert(1);
    let removed = document.clear_selected().unwrap();
    for dependent in [1, 2, 9, 10] {
        assert!(removed.contains(&dependent));
    }
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.objects.len(), 6);
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[5].kind,
        Kind::CoordinateSystem(3, 5)
    ));

    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    document.selected.insert(10);
    let clipboard = document.copy_selected().unwrap();
    document.paste(&clipboard).unwrap();
    document.selected.clear();
    document.selected.insert(1);
    let before = document.encode().unwrap();
    let error = document.clear_selected().unwrap_err();
    assert!(error.contains("before a surviving plot"), "{error}");
    assert_eq!(document.encode().unwrap(), before);
    assert!(!document.objects[0].deleted);
}

#[test]
fn native_parameter_trace_rejects_a_function_plot_mode_and_wrong_refs() {
    let rs = records(COOD_TRACE).unwrap();
    let locus_start = rs.iter().filter(|r| r.kind == 0x07d0).nth(9).unwrap().start;
    let descriptor = rs
        .iter()
        .find(|r| r.start > locus_start && r.kind == 0x0902)
        .unwrap();
    for (offset, bytes) in [
        (20, 0x0003_0000_u32.to_le_bytes()),
        (16, 1_u32.to_le_bytes()),
    ] {
        let mut invalid = COOD_TRACE.to_vec();
        invalid[descriptor.data.start + offset..descriptor.data.start + offset + 4]
            .copy_from_slice(&bytes);
        let document = Document::from_bytes(&invalid, None).unwrap();
        assert!(matches!(document.objects[9].kind, Kind::Unsupported(97)));
        assert!(document.partial_view);
        assert_eq!(document.encode().unwrap(), invalid);
    }
    let reference = rs
        .iter()
        .find(|r| r.start > locus_start && r.kind == 0x07d2)
        .unwrap();
    let mut invalid = COOD_TRACE.to_vec();
    invalid[reference.data.start + 8..reference.data.start + 12]
        .copy_from_slice(&2_u32.to_le_bytes());
    let document = Document::from_bytes(&invalid, None).unwrap();
    assert!(matches!(document.objects[9].kind, Kind::Unsupported(97)));
}

#[test]
fn native_measured_coordinates_use_live_length_and_area_magnitudes() {
    let bytes = include_bytes!("fixtures/numeric_measured_xy_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert!(matches!(
        document.objects[12].kind,
        Kind::NumericCoordinatePoint(6, 5, 12)
    ));
    assert_eq!(
        document.objects[5].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert_eq!(
        document.objects[4].kind.numeric_unit(),
        Some(NumericUnit::Area)
    );
    assert!((document.numeric_value(6).unwrap() - 2.).abs() < 1e-12);
    assert!((document.numeric_value(5).unwrap() - 1.).abs() < 1e-12);
    let scale = 37.79527559055118;
    let initial = document.position(13).unwrap();
    assert!((initial.x - (200. + 2. * scale)).abs() < 1e-10);
    assert!((initial.y - (250. - scale)).abs() < 1e-10);
    assert!(document.move_point(
        2,
        sketchpad_rs::document::Point {
            x: 20. + 3. * scale,
            y: 200.
        }
    ));
    let moved = document.position(13).unwrap();
    assert!((moved.x - (200. + 3. * scale)).abs() < 1e-10);
    assert!((moved.y - (250. - 1.5 * scale)).abs() < 1e-10);
    let encoded = document.encode().unwrap();
    assert_eq!(object_group(&encoded, 13), object_group(bytes, 13));
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.position(13), Some(moved));
    select_construction_graph(&mut document, 13);
    let clipboard = document.copy_selected().unwrap();
    let copied = *document.paste(&clipboard).unwrap().last().unwrap();
    let position = document.position(copied).unwrap();
    assert!((position.x - moved.x - 20.).abs() < 1e-10);
    assert!((position.y - moved.y - 20.).abs() < 1e-10);
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.position(copied), Some(position));
}

#[test]
fn native_function_arguments_keep_grouped_factors_inside_the_function() {
    let bytes = include_bytes!("fixtures/numeric_sgn_grouped_argument_native.bin");
    let calculation = ParameterCalculationGroup::parse(bytes).unwrap();
    // Native Ningde2 displays Color=0.33 at x=2.51. Its following group11
    // starts (x−2), not a special function opener ending before the product.
    assert!((calculation.value(2.51).unwrap() - 1. / 3.).abs() < 1e-12);
    assert!((calculation.value(4.).unwrap() - 2. / 3.).abs() < 1e-12);
    assert!((calculation.value(7.).unwrap() - 1.).abs() < 1e-12);
    assert_eq!(calculation.value(1.), Some(0.));
    assert_eq!(calculation.as_bytes(), bytes);
    // The focused native measurement probe displays AB=2.51cm, Color=0.33.
    let measured = calculation
        .with_source_units(&[NumericUnit::Length])
        .unwrap();
    assert!(
        (measured
            .value_for_sources(&[2.51], &[NumericUnit::Length])
            .unwrap()
            - 1. / 3.)
            .abs()
            < 1e-12
    );
    assert_eq!(measured.unit(), NumericUnit::Scalar);
    let mixed = ParameterCalculationGroup::from_sources(
        &[1, 2],
        "v1-v2",
        (10, 30),
        &[NumericUnit::Length, NumericUnit::Scalar],
    )
    .unwrap();
    assert_eq!(mixed.unit(), NumericUnit::Scalar);
    assert_eq!(
        mixed.value_for_sources(&[2.51, 2.], &[NumericUnit::Length, NumericUnit::Scalar]),
        Some(2.51 - 2.)
    );
}

#[test]
fn numeric_coordinate_point_label_style_and_clipboard_remain_editable() {
    use sketchpad_rs::style_records::{PointStyle, RgbColor, VisualStyle};

    let mut document = Document::from_bytes(COOD_TRACE, None).unwrap();
    document.set_point_label(9, "P", true).unwrap();
    document.selected.insert(9);
    assert_eq!(
        document
            .set_selected_point_style(PointStyle::Large)
            .unwrap(),
        1
    );
    let color = RgbColor {
        red: 20,
        green: 80,
        blue: 180,
    };
    assert_eq!(document.set_selected_color(color).unwrap(), 1);
    let style = Some(VisualStyle::Point {
        shape: PointStyle::Large,
        color,
    });
    let position = document.position(9).unwrap();
    let encoded = document.encode().unwrap();
    let mut reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[8].kind,
        Kind::NumericCoordinatePoint(1, 2, 8)
    ));
    assert_eq!(reopened.objects[8].label, document.objects[8].label);
    assert_eq!(reopened.objects[8].visual_style, style);
    assert_eq!(reopened.position(9), Some(position));

    reopened.selected.insert(9);
    let clipboard = reopened.copy_selected().unwrap();
    let copied = *reopened.paste(&clipboard).unwrap().last().unwrap();
    let copied_position = reopened.position(copied).unwrap();
    assert!((copied_position.x - position.x - 20.).abs() < 1e-10);
    assert!((copied_position.y - position.y - 20.).abs() < 1e-10);
    let copy_label = reopened.objects[copied - 1].label.clone();
    assert!(copy_label.is_some());
    assert_eq!(reopened.objects[copied - 1].visual_style, style);
    let bytes = reopened.encode().unwrap();
    let mut saved = Document::from_bytes(&bytes, None).unwrap();
    assert!(!saved.partial_view);
    assert_eq!(saved.objects[copied - 1].label, copy_label);
    assert_eq!(saved.objects[copied - 1].visual_style, style);
    assert_eq!(saved.position(copied), Some(copied_position));
    saved.set_point_label(copied, "Copied P", true).unwrap();
    assert!(
        !Document::from_bytes(&saved.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn native_calculated_path_point_uses_live_scalar_calculation() {
    let native = include_bytes!("fixtures/numeric_calculated_path_native.bin");
    assert_eq!(
        sketchpad_rs::plot_axis_records::decode_group(native).unwrap(),
        [16, 15]
    );
    // Actual Ningde2 object17 has calculation16 and path15 as its parents.
    // Rebind only those references into the supported cood sketch to isolate
    // code95 semantics from Ningde2's other unsupported construction families.
    let bytes = include_bytes!("fixtures/numeric_calculated_path_import.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert!(matches!(
        document.objects[10].kind,
        Kind::PathValuePoint(2, 5)
    ));
    let label = document.objects[10].label.clone();
    assert!(label.is_some());
    let initial = document.position(11).unwrap();
    document.set_parameter_value(1, 3.5).unwrap();
    let updated = document.position(11).unwrap();
    assert!((updated.x - initial.x - 2.5 * 37.79527559055118).abs() < 1e-10);
    assert_eq!(updated.y, initial.y);
    let encoded = document.encode().unwrap();
    assert_eq!(object_group(&encoded, 11), object_group(bytes, 11));
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.position(11), Some(updated));
    assert_eq!(reopened.objects[10].label, label);
    let created = document.plot_parameter_on_path(2, Some(5)).unwrap();
    assert_eq!(document.position(created), Some(updated));
    // Coordinate-system clipboard groups must contain the complete frame.
    document.selected.insert(8);
    document.selected.insert(11);
    let clipboard = document.copy_selected().unwrap();
    let copied = *document.paste(&clipboard).unwrap().last().unwrap();
    assert!(document.objects[copied - 1].label.is_some());
    let saved = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!saved.partial_view);
    assert_eq!(
        saved.objects[copied - 1].label,
        document.objects[copied - 1].label
    );
}

#[test]
fn native_table_value_edits_preserve_visibility_and_observed_padding() {
    use sketchpad_rs::table_records::TableGroup;
    let native = include_bytes!("fixtures/plot_points_table_two_rows_original.gsp");
    let mut raw = object_group(native, 2);
    raw[10] |= 1;
    let mut table = TableGroup::parse(&raw).unwrap();
    assert_eq!(
        &payload(&raw, 0x090d)[payload(&raw, 0x090d).len() - 2..],
        &[b't', 0]
    );
    table.set_last_default_value(7.25).unwrap();
    assert_eq!(table.as_bytes()[10], raw[10]);
    let layout = payload(table.as_bytes(), 0x090d);
    assert_eq!(&layout[layout.len() - 2..], &[b't', 0]);
    assert_eq!(table.default_values().unwrap().last(), Some(&7.25));
    TableGroup::parse(table.as_bytes()).unwrap();
    let before = table.clone();
    assert!(table.set_last_default_value(f64::NAN).is_err());
    assert_eq!(table, before);
}

#[test]
fn native_length_threshold_expression_uses_centimeter_magnitudes() {
    let bytes = include_bytes!("fixtures/numeric_measured_literal_threshold_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        (document.numeric_value(6).unwrap() * 100.).round() / 100.,
        2.51
    );
    assert!((document.numeric_value(14).unwrap() - 1. / 3.).abs() < 1e-12);
    assert_eq!(
        document.objects[13].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert!(document.move_point(
        2,
        sketchpad_rs::document::Point {
            x: 20. + 4. * 37.79527559055118,
            y: 200.
        }
    ));
    assert!((document.numeric_value(14).unwrap() - 2. / 3.).abs() < 1e-12);
    let encoded = document.encode().unwrap();
    assert_eq!(object_group(&encoded, 14), object_group(bytes, 14));
    let saved = Document::from_bytes(&encoded, None).unwrap();
    assert!(!saved.partial_view);
    assert!((saved.numeric_value(14).unwrap() - 2. / 3.).abs() < 1e-12);
    let difference = Document::from_bytes(
        include_bytes!("fixtures/numeric_length_difference_native_verified.gsp"),
        None,
    )
    .unwrap();
    assert!(!difference.partial_view);
    assert_eq!(
        difference.objects[14].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert_eq!(
        difference.numeric_value(15),
        Some(difference.numeric_value(6).unwrap() - 2.)
    );
}

#[test]
fn native_empty_calculation_attribute_keeps_live_math_and_strict_layout() {
    let unlabeled = include_bytes!("fixtures/numeric_empty_calculation_attribute_native.bin");
    let labeled = include_bytes!("fixtures/numeric_labeled_empty_calculation_attribute_native.bin");
    for raw in [unlabeled.as_slice(), labeled.as_slice()] {
        let mut group = ParameterCalculationGroup::parse(raw).unwrap();
        if raw == unlabeled.as_slice() {
            assert_eq!(group.display_mode(), 1);
            assert!(group.caption_suppressed());
        }
        let marker = payload(raw, 0x232e);
        assert!(marker.is_empty());
        group.set_precision(4).unwrap();
        group.set_anchor((100, 150)).unwrap();
        group = group.remap_source(3).unwrap();
        assert_eq!(group.sources(), [3]);
        assert!(payload(group.as_bytes(), 0x232e).is_empty());
        assert_eq!(payload(group.as_bytes(), 0x07d8), payload(raw, 0x07d8));
    }
    let bytes =
        include_bytes!("fixtures/numeric_empty_calculation_attribute_xy_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert!(document.can_save().is_ok());
    assert!(sketchpad_rs::render::number_lines_for_object(&document, 11).is_none());
    assert!(document.numeric_value(11).unwrap().abs() < 1e-10);
    document.set_parameter_value(1, 1.25).unwrap();
    assert!((document.numeric_value(11).unwrap() - 1.).abs() < 1e-12);
    let point = document.position(12).unwrap();
    assert!((point.x - (200. + 37.79527559055118)).abs() < 1e-8);
    assert!((point.y - (250. - 2.25 * 37.79527559055118)).abs() < 1e-10);
    let saved = document.encode().unwrap();
    assert_eq!(object_group(&saved, 11), object_group(bytes, 11));
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(12), Some(point));
    let value_only = Document::from_bytes(
        include_bytes!(
            "fixtures/numeric_empty_calculation_attribute_value_only_native_verified.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(!value_only.partial_view);
    let Kind::ParameterCalculation(_, group) = &value_only.objects[10].kind else {
        panic!("expected numeric calculation");
    };
    assert_eq!(group.display_mode(), 1);
    assert!(!group.caption_suppressed());
    assert_eq!(
        sketchpad_rs::render::number_lines_for_object(&value_only, 11)
            .unwrap()
            .1,
        ["0.00"]
    );
    let color = Document::from_bytes(
        include_bytes!("fixtures/numeric_measured_literal_threshold_native_verified.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        sketchpad_rs::render::number_lines_for_object(&color, 14)
            .unwrap()
            .1,
        ["Color = 0.33"]
    );

    select_construction_graph(&mut document, 12);
    let clipboard = document.copy_selected().unwrap();
    let copied = *document.paste(&clipboard).unwrap().last().unwrap();
    let Kind::NumericCoordinatePoint(copied_calculation, _, _) = document.objects[copied - 1].kind
    else {
        panic!("expected copied numeric point");
    };
    assert!(sketchpad_rs::render::number_lines_for_object(&document, copied_calculation).is_none());
    let copied_saved = document.encode().unwrap();
    assert!(payload(&object_group(&copied_saved, copied_calculation), 0x232e).is_empty());
    let copied_reopened = Document::from_bytes(&copied_saved, None).unwrap();
    assert!(!copied_reopened.partial_view);
    assert!((copied_reopened.numeric_value(copied_calculation).unwrap() - 1.).abs() < 1e-12);
    assert_eq!(copied_reopened.position(copied), document.position(copied));

    let rs = records(bytes).unwrap();
    let auxiliary = rs.iter().find(|r| r.kind == 0x232e).unwrap();
    let object_start = rs
        .iter()
        .filter(|r| r.kind == 0x07d0)
        .nth(10)
        .unwrap()
        .start;
    let font = rs
        .iter()
        .find(|r| r.kind == 0x07d8 && r.start > object_start)
        .unwrap();
    let mut nonempty = bytes.to_vec();
    nonempty[auxiliary.start..auxiliary.start + 4].copy_from_slice(&1_u32.to_le_bytes());
    nonempty.splice(auxiliary.data.start..auxiliary.data.start, [7, 0]);
    let mut displaced = bytes.to_vec();
    let marker = displaced
        .drain(auxiliary.start..auxiliary.data.end)
        .collect::<Vec<_>>();
    displaced.splice(font.start..font.start, marker);
    let mut duplicate = bytes.to_vec();
    duplicate.splice(
        auxiliary.start..auxiliary.start,
        bytes[auxiliary.start..auxiliary.data.end].iter().copied(),
    );
    for invalid in [nonempty, displaced, duplicate] {
        let rejected = Document::from_bytes(&invalid, None).unwrap();
        assert!(rejected.partial_view);
        assert!(matches!(rejected.objects[10].kind, Kind::Unsupported(48)));
        assert_eq!(rejected.encode().unwrap(), invalid);
    }
}

#[test]
fn native_typed_parameters_preserve_unit_words_domains_and_caches() {
    let bytes = include_bytes!("fixtures/numeric_typed_parameters_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    for (id, unit, text, domain) in [
        (1, NumericUnit::Degrees, "1.00°", (0., 360.)),
        (2, NumericUnit::Length, "1.00 cm", (0., 100.)),
        (3, NumericUnit::Scalar, "1.00", (-100., 100.)),
    ] {
        let Kind::Parameter(group) = &document.objects[id - 1].kind else {
            panic!("expected typed parameter");
        };
        assert_eq!(group.unit(), unit);
        assert_eq!(document.objects[id - 1].kind.numeric_unit(), Some(unit));
        assert_eq!(group.value_text(), text);
        assert_eq!(group.value(), 1.);
        if unit != NumericUnit::Scalar {
            assert!(
                sketchpad_rs::table_records::TableGroup::named_snapshot(
                    id as u32,
                    group,
                    (10, 200)
                )
                .is_err()
            );
        }
        let range = group.animation_domain();
        assert!((range.0 - domain.0).abs() < 1e-10);
        assert!((range.1 - domain.1).abs() < 1e-10);
    }
    assert_eq!(document.unitless_numeric_value(1), None);
    assert_eq!(document.unitless_numeric_value(2), None);
    assert!(!document.objects[0].kind.is_iteration_depth_value());
    assert!(!document.objects[1].kind.is_iteration_depth_value());
    assert!(document.tabulate_default_parameter(1, (10, 200)).is_err());
    assert!(document.tabulate_default_pair((10, 200)).is_err());
    document.set_parameter_value(1, 45.).unwrap();
    document.set_parameter_value(2, 2.51).unwrap();
    let sine = document
        .add_parameter_calculation(1, "sin(t[1])", (10, 120))
        .unwrap();
    let length = document
        .add_parameter_calculation(2, "t[2]*2", (10, 150))
        .unwrap();
    assert!((document.numeric_value(sine).unwrap() - 0.5_f64.sqrt()).abs() < 1e-12);
    assert_eq!(
        document.objects[length - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert!((document.numeric_value(length).unwrap() - 5.02).abs() < 1e-12);
    let frame = document
        .add_default_coordinate_system(sketchpad_rs::document::Point { x: 200., y: 250. })
        .unwrap();
    let point = document.add(Kind::NumericCoordinatePoint(2, sine, frame));
    let degree_point = document.add(Kind::NumericCoordinatePoint(2, 1, frame));
    assert!(
        (document.position(degree_point).unwrap().y - (250. - 45. * 37.79527559055118)).abs()
            < 1e-10
    );
    assert!(document.plot_parameter_on_path(2, None).is_err());
    assert!(
        document
            .construct_parameter_locus(2, point, 0., 1., 17)
            .is_err()
    );
    let position = document.position(point).unwrap();
    assert!((position.x - (200. + 2.51 * 37.79527559055118)).abs() < 1e-10);
    let encoded = document.encode().unwrap();
    for id in [1, 2] {
        let original = object_group(bytes, id);
        let edited = object_group(&encoded, id);
        for kind in [0x07d0, 0x07d5, 0x07d8] {
            assert_eq!(payload(&edited, kind), payload(&original, kind));
        }
        // Native caches can disagree with the current value; editing the value
        // words must not rewrite the animation domain, custom step, or caches.
        let before = payload(&original, 0x0907);
        let after = payload(&edited, 0x0907);
        assert_eq!(&after[52..92], &before[52..92]);
        assert_eq!(&after[after.len() - 2..], &before[before.len() - 2..]);
    }
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(point), Some(position));
    assert_eq!(reopened.numeric_value(1), Some(45.));
    assert_eq!(reopened.numeric_value(2), Some(2.51));

    let mut degree = ParameterGroup::parse(&object_group(bytes, 1)).unwrap();
    let before = payload(degree.as_bytes(), 0x0907);
    degree.set_value(0.5123).unwrap();
    assert_eq!(degree.precision(), 4);
    assert_eq!(&payload(degree.as_bytes(), 0x0907)[52..92], &before[52..92]);
    let mut length = ParameterGroup::parse(&object_group(bytes, 2)).unwrap();
    let before = payload(length.as_bytes(), 0x0907);
    length.set_value(2.511).unwrap();
    let after = payload(length.as_bytes(), 0x0907);
    assert_eq!(length.precision(), 3);
    assert_eq!(
        f64::from_le_bytes(after[52..60].try_into().unwrap()),
        0.001 * 37.79527559055118
    );
    assert_eq!(&after[60..92], &before[60..92]);

    for (id, wrong_subtype) in [(1, 80_u16), (2, 81_u16)] {
        let mut invalid = object_group(bytes, id);
        invalid[16..18].copy_from_slice(&wrong_subtype.to_le_bytes());
        assert!(ParameterGroup::parse(&invalid).is_err());
    }
}

#[test]
fn native_typed_parameter_clipboard_preserves_units_and_live_dependencies() {
    let bytes = include_bytes!("fixtures/numeric_typed_parameters_copied_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.objects.len(), 22);
    assert_eq!(document.encode().unwrap(), bytes);
    let Kind::NumericCoordinatePoint(x, y, _) = document.objects[21].kind else {
        panic!("expected copied measured coordinate point");
    };
    assert_eq!(
        document.objects[x - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert_eq!(
        document.objects[y - 1].kind.numeric_unit(),
        Some(NumericUnit::Degrees)
    );
    assert_eq!(document.numeric_value(x), Some(2.51));
    assert_eq!(document.numeric_value(y), Some(0.5));
    let before = document.position(22).unwrap();
    document.set_parameter_value(y, 1.5).unwrap();
    let after = document.position(22).unwrap();
    assert!((after.y - before.y + 37.79527559055118).abs() < 1e-10);
    assert_eq!(document.numeric_value(1), Some(0.5));
    let encoded = document.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&encoded, None).unwrap().position(22),
        Some(after)
    );
}

#[test]
fn native_angle_suffixes_keep_degree_semantics_and_preserve_literal_encoding() {
    // Native 0102 displays 4.50°, sin=.08 and sqrt=2.12; it does not
    // denote squared degrees. Native 0100 stores radians and displays degrees.
    let alternate = include_bytes!("fixtures/numeric_alternate_degree_parameter_native.bin");
    let mut degree = ParameterGroup::parse(alternate).unwrap();
    assert_eq!(degree.unit(), NumericUnit::Degrees);
    assert_eq!(degree.value(), 30.);
    assert_eq!(degree.value_text(), "30°");
    let before = payload(alternate, 0x0907);
    degree.set_value(4.5).unwrap();
    assert_eq!(degree.value_text(), "4.5°");
    let after = payload(degree.as_bytes(), 0x0907);
    assert_eq!(&after[52..92], &before[52..92]);
    assert_eq!(&after[after.len() - 2..], &0x0102_u16.to_le_bytes());

    let bytes = include_bytes!("fixtures/numeric_alternate_degree_parameter_native_verified.gsp");
    let alternate_document = Document::from_bytes(bytes, None).unwrap();
    assert!(!alternate_document.partial_view);
    assert_eq!(alternate_document.encode().unwrap(), bytes);
    assert_eq!(alternate_document.numeric_value(1), Some(4.5));
    assert_eq!(
        alternate_document.objects[3].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert!((alternate_document.numeric_value(4).unwrap() - 4.5_f64.sqrt()).abs() < 1e-12);
    assert!(
        (alternate_document.position(13).unwrap().y - (250. - 4.5 * 37.79527559055118)).abs()
            < 1e-10
    );

    let radians = include_bytes!("fixtures/numeric_radian_parameter_native.bin");
    let mut angle = ParameterGroup::parse(radians).unwrap();
    assert!((angle.value() - 180. / 3.1415926535898).abs() < 1e-12);
    assert_eq!(angle.value_text(), "57.30°");
    let before = payload(radians, 0x0907);
    angle.set_value(3.5).unwrap();
    assert!((angle.value() - 3.5).abs() < 1e-12);
    assert_eq!(angle.precision(), 2);
    assert_eq!(angle.value_text(), "3.50°");
    assert!((angle.animation_domain().1 - 360.).abs() < 1e-10);
    let after = payload(angle.as_bytes(), 0x0907);
    assert_eq!(&after[52..92], &before[52..92]);
    assert_eq!(&after[after.len() - 2..], &0x0100_u16.to_le_bytes());

    let bytes = include_bytes!("fixtures/numeric_radian_parameter_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    assert!((document.numeric_value(4).unwrap() - 1_f64.sin()).abs() < 1e-12);
    assert!((document.numeric_value(5).unwrap() - 360. / 3.1415926535898).abs() < 1e-12);
    document.set_parameter_value(1, 3.5).unwrap();
    assert!((document.numeric_value(5).unwrap() - 7.).abs() < 1e-12);
    let expected_y = 250. - 3.5 * 37.79527559055118;
    assert!((document.position(13).unwrap().y - expected_y).abs() < 1e-10);
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(13), document.position(13));

    select_construction_graph(&mut document, 13);
    let clipboard = document.copy_selected().unwrap();
    let ids = document.paste(&clipboard).unwrap();
    let copied = *ids.last().unwrap();
    let Kind::NumericCoordinatePoint(_, angle, _) = document.objects[copied - 1].kind else {
        panic!("expected copied angle-driven coordinate point");
    };
    let Kind::Parameter(group) = &document.objects[angle - 1].kind else {
        panic!("expected copied radian parameter");
    };
    assert!((group.value() - 3.5).abs() < 1e-12);
    let value = payload(group.as_bytes(), 0x0907);
    assert_eq!(&value[value.len() - 2..], &0x0100_u16.to_le_bytes());
    document.set_parameter_value(angle, 4.5).unwrap();
    assert!((document.numeric_value(1).unwrap() - 3.5).abs() < 1e-12);
    assert!((document.numeric_value(angle).unwrap() - 4.5).abs() < 1e-12);
    let encoded = document.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&encoded, None)
            .unwrap()
            .position(copied),
        document.position(copied)
    );
}

#[test]
fn native_angle_parameter_locus_uses_radian_descriptor_and_degree_range_api() {
    let bytes = include_bytes!("fixtures/numeric_angle_parameter_locus_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    let Kind::ParameterLocus(refs, min, max, samples) = &document.objects[16].kind else {
        panic!("expected native angle parameter locus");
    };
    assert_eq!(refs, &[16, 1, 16]);
    assert_eq!((*min, *max, *samples), (0., 3.1415926535898 / 2., 17));
    let range = document.plot_range(17).unwrap();
    assert_eq!(range, (0., 90., 17));
    document
        .set_plot_range(17, range.0, range.1, range.2)
        .unwrap();
    assert_eq!(document.encode().unwrap(), bytes);
    let strokes = document.locus_strokes(17, 17).unwrap();
    assert_eq!(strokes.len(), 16);
    let start = strokes.first().unwrap().0;
    let end = strokes.last().unwrap().1;
    assert!((start.x - 320.).abs() < 1e-10 && (start.y - 250.).abs() < 1e-10);
    assert!((end.x - 250.).abs() < 1e-10 && (end.y - 180.).abs() < 1e-10);
    let before = document.position(16).unwrap();
    document.set_parameter_value(1, 6.5).unwrap();
    assert_ne!(document.position(16), Some(before));
    assert_eq!(document.locus_strokes(17, 17), Some(strokes));

    document.set_plot_range(17, -90., 180., 23).unwrap();
    assert_eq!(document.plot_range(17), Some((-90., 180., 23)));
    let encoded = document.encode().unwrap();
    let original = payload(&object_group(bytes, 17), 0x0902);
    let edited = payload(&object_group(&encoded, 17), 0x0902);
    assert_eq!(&edited[20..], &original[20..]);
    assert_eq!(
        u32::from_le_bytes(edited[32..36].try_into().unwrap()),
        0x0001_0000
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[16].kind, document.objects[16].kind);
    assert_eq!(
        reopened.locus_strokes(17, 23),
        document.locus_strokes(17, 23)
    );

    let authored = document
        .construct_parameter_locus(1, 16, 0., 90., 17)
        .unwrap();
    let encoded = document.encode().unwrap();
    let descriptor = payload(&object_group(&encoded, authored), 0x0902);
    assert_eq!(&descriptor[32..36], &0x0001_0000_u32.to_le_bytes());
    assert_eq!(document.plot_range(authored), Some((0., 90., 17)));
    document.selected.insert(17);
    let clipboard = document.copy_selected().unwrap();
    let copied = *document.paste(&clipboard).unwrap().last().unwrap();
    assert_eq!(document.plot_range(copied), Some((-90., 180., 23)));
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.locus_strokes(copied, 23),
        document.locus_strokes(copied, 23)
    );
    let verified =
        include_bytes!("fixtures/numeric_angle_parameter_locus_edited_copied_native_verified.gsp");
    assert_eq!(encoded, verified);

    let mut invalid = bytes.to_vec();
    let rs = records(&invalid).unwrap();
    let at = rs
        .iter()
        .find(|record| record.kind == 0x0902)
        .unwrap()
        .data
        .start
        + 32;
    invalid[at..at + 4].copy_from_slice(&1_u32.to_le_bytes());
    let wrong_tag = Document::from_bytes(&invalid, None).unwrap();
    assert!(matches!(wrong_tag.objects[16].kind, Kind::Unsupported(97)));
    assert!(wrong_tag.partial_view);
}

#[test]
fn native_typed_scientific_parameters_keep_literal_layout_through_edits_and_copy() {
    let exam = ParameterGroup::parse(include_bytes!(
        "fixtures/numeric_scientific_length_parameter_exam_native.bin"
    ))
    .unwrap();
    assert_eq!(exam.unit(), NumericUnit::Length);
    assert_eq!(exam.value(), 0.00001);
    for (original, raw, verified_copy, source, unit, value, edited_value, suffix) in [
        (
            include_bytes!("fixtures/numeric_scientific_length_parameter_native_verified.gsp")
                .as_slice(),
            include_bytes!("fixtures/numeric_scientific_length_parameter_native.bin").as_slice(),
            include_bytes!(
                "fixtures/numeric_scientific_length_parameter_edited_copied_native_verified.gsp"
            )
            .as_slice(),
            2,
            NumericUnit::Length,
            0.00001,
            0.000025,
            0x0201_u16,
        ),
        (
            include_bytes!("fixtures/numeric_scientific_degree_parameter_native_verified.gsp")
                .as_slice(),
            include_bytes!("fixtures/numeric_scientific_degree_parameter_native.bin").as_slice(),
            include_bytes!(
                "fixtures/numeric_scientific_degree_parameter_edited_copied_native_verified.gsp"
            )
            .as_slice(),
            1,
            NumericUnit::Degrees,
            0.0000446583259,
            0.000055,
            0x0101_u16,
        ),
    ] {
        let parameter = ParameterGroup::parse(raw).unwrap();
        assert_eq!(parameter.unit(), unit);
        assert_eq!(parameter.value(), value);
        let mut unchanged = parameter.clone();
        unchanged.set_value(parameter.value()).unwrap();
        assert_eq!(unchanged.as_bytes(), raw);
        let mut too_wide = parameter.clone();
        assert!(too_wide.set_value(1e25).is_err());
        assert_eq!(too_wide.as_bytes(), raw);
        let mut document = Document::from_bytes(original, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.objects.len(), 14);
        assert_eq!(document.encode().unwrap(), original);
        assert_eq!(document.numeric_value(source), Some(value));
        assert!((document.numeric_value(5).unwrap() - value * 100000.).abs() < 1e-12);
        assert_eq!(document.objects[4].kind.numeric_unit(), Some(unit));
        assert_eq!(document.unitless_numeric_value(source), None);
        let position = document.position(14).unwrap();
        let (x, y) = if source == 2 {
            (value * 100000., 0.5)
        } else {
            (2.51, value * 100000.)
        };
        assert!((position.x - (200. + x * 37.79527559055118)).abs() < 1e-10);
        assert!((position.y - (250. - y * 37.79527559055118)).abs() < 1e-10);

        document.set_parameter_value(source, edited_value).unwrap();
        assert_eq!(document.numeric_value(source), Some(edited_value));
        let encoded = document.encode().unwrap();
        let before = payload(&object_group(original, source), 0x0907);
        let after = payload(&object_group(&encoded, source), 0x0907);
        assert_eq!(&after[60..92], &before[60..92]);
        let words = after[92..]
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(&words[3..], &[suffix, 0x1002, 1, 0, 0x1004, 0x1001, 5]);
        let Kind::Parameter(parameter) = &document.objects[source - 1].kind else {
            panic!("expected scientific parameter");
        };
        assert_eq!(parameter.precision(), 5);
        assert_eq!(
            parameter.value_text(),
            if source == 2 {
                "0.00003 cm"
            } else {
                "0.00006°"
            }
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(14), document.position(14));
        document.selected.clear();
        select_construction_graph(&mut document, 14);
        let clipboard = document.copy_selected().unwrap();
        let copied = *document.paste(&clipboard).unwrap().last().unwrap();
        assert_eq!(copied, 24);
        let encoded = document.encode().unwrap();
        assert!(
            encoded == verified_copy,
            "scientific parameter copy differs from native fixture"
        );
        let reopened = Document::from_bytes(verified_copy, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(24), document.position(24));

        // Other integer exponents are numeric literals; conflicting units are invalid.
        let mut invalid = object_group(original, source);
        let rs = records(&[b"GSP4".as_slice(), invalid.as_slice()].concat()).unwrap();
        let record = rs.iter().find(|record| record.kind == 0x0907).unwrap();
        let last = record.data.end - 4 - 2;
        invalid[last..last + 2].copy_from_slice(&6_u16.to_le_bytes());
        let changed_exponent = ParameterGroup::parse(&invalid).unwrap();
        let original_value = ParameterGroup::parse(&object_group(original, source)).unwrap().value();
        assert!((changed_exponent.value() - original_value / 10.).abs() < 1e-12);
        let mut invalid = object_group(original, source);
        let suffix_at = record.data.start - 4 + 92 + if source == 2 { 2 } else { 20 };
        invalid[suffix_at..suffix_at + 2]
            .copy_from_slice(&(if source == 2 { 0x0101_u16 } else { 0x0201_u16 }).to_le_bytes());
        assert!(ParameterGroup::parse(&invalid).is_err());
    }
    for raw in [
        include_bytes!("fixtures/numeric_typed_zero_invalid_animation_length_native.bin")
            .as_slice(),
        include_bytes!("fixtures/numeric_typed_zero_invalid_animation_degree_native.bin")
            .as_slice(),
    ] {
        assert!(ParameterGroup::parse(raw).is_err());
    }
}

#[test]
fn native_pi_radian_parameter_retains_constant_and_updates_copied_math() {
    let raw = include_bytes!("fixtures/numeric_pi_radian_parameter_native.bin");
    let mut parameter = ParameterGroup::parse(raw).unwrap();
    assert_eq!(parameter.unit(), NumericUnit::Degrees);
    assert!((parameter.value() - 180.).abs() < 1e-12);
    assert_eq!(parameter.value_text(), "180.00°");
    let unchanged = parameter.value();
    parameter.set_value(unchanged).unwrap();
    assert_eq!(parameter.as_bytes(), raw);
    let before = payload(raw, 0x0907);
    parameter.set_value(3.5).unwrap();
    assert!((parameter.value() - 3.5).abs() < 1e-12);
    assert_eq!(parameter.value_text(), "3.50°");
    let after = payload(parameter.as_bytes(), 0x0907);
    assert_eq!(&after[52..92], &before[52..92]);
    assert_eq!(&after[after.len() - 2..], &0x0100_u16.to_le_bytes());

    let bytes = include_bytes!("fixtures/numeric_pi_radian_parameter_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(document.can_save().is_ok());
    assert!(!document.partial_view);
    assert_eq!(document.encode().unwrap(), bytes);
    assert!((document.numeric_value(1).unwrap() - 180.).abs() < 1e-12);
    assert!(document.numeric_value(4).unwrap().abs() < 1e-12);
    assert!((document.numeric_value(5).unwrap() - 360.).abs() < 1e-12);
    assert!((document.position(12).unwrap().y - 250.).abs() < 1e-10);

    select_construction_graph(&mut document, 12);
    let clipboard = document.copy_selected().unwrap();
    let copied = *document.paste(&clipboard).unwrap().last().unwrap();
    assert_eq!(copied, 23);
    let Kind::NumericCoordinatePoint(_, sine, _) = document.objects[copied - 1].kind else {
        panic!("expected copied sine coordinate point");
    };
    assert!(document.numeric_value(sine).unwrap().abs() < 1e-12);
    let Kind::ParameterCalculation(angle, _) = document.objects[sine - 1].kind else {
        panic!("expected copied live sine calculation");
    };
    let Kind::Parameter(ref copied_parameter) = document.objects[angle - 1].kind else {
        panic!("expected copied pi parameter");
    };
    let copied_words = payload(copied_parameter.as_bytes(), 0x0907);
    assert_eq!(&copied_words[92..], &[13, 0, 0, 1]);

    document.set_parameter_value(1, 3.5).unwrap();
    assert!((document.numeric_value(5).unwrap() - 7.).abs() < 1e-12);
    assert!(document.numeric_value(sine).unwrap().abs() < 1e-12);
    let expected_sine = (3.5_f64 * 3.1415926535898 / 180.).sin();
    assert!((document.numeric_value(4).unwrap() - expected_sine).abs() < 1e-12);
    assert!(
        (document.position(12).unwrap().y - (250. - expected_sine * 37.79527559055118)).abs()
            < 1e-10
    );
    document.selected.clear();
    select_construction_graph(&mut document, 12);
    let clipboard = document.copy_selected().unwrap();
    let edited_copy = *document.paste(&clipboard).unwrap().last().unwrap();
    assert_eq!(edited_copy, 33);
    assert!(
        (document.position(edited_copy).unwrap().y - (270. - expected_sine * 37.79527559055118))
            .abs()
            < 1e-10
    );
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!("fixtures/numeric_pi_radian_parameter_edited_copied_native_verified.gsp")
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(reopened.can_save().is_ok());
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.position(edited_copy),
        document.position(edited_copy)
    );

    // Changing the angle suffix changes the unit interpretation of the constant.
    let rs = records(bytes).unwrap();
    let value = rs.iter().find(|record| record.kind == 0x0907).unwrap();
    let start = rs
        .iter()
        .find(|record| record.kind == 0x07d0)
        .unwrap()
        .start;
    for suffix in [0x0101_u16, 0x0102, 0x0201] {
        let mut invalid = object_group(bytes, 1);
        let at = value.data.start - start + 94;
        invalid[at..at + 2].copy_from_slice(&suffix.to_le_bytes());
        if suffix == 0x0201 {
            assert!(ParameterGroup::parse(&invalid).is_err());
        } else {
            assert!((ParameterGroup::parse(&invalid).unwrap().value() - std::f64::consts::PI).abs() < 1e-12);
        }
    }
}

#[test]
fn native_parameter_presentation_records_are_preserved_by_group_edits() {
    for (raw, value, unnamed) in [
        (
            include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_length_native.bin")
                .as_slice(),
            0.0001,
            true,
        ),
        (
            include_bytes!(
                "fixtures/numeric_parameter_presentation_unlabeled_length_second_native.bin"
            )
            .as_slice(),
            0.0001,
            true,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_degree_native.bin")
                .as_slice(),
            35.,
            true,
        ),
        (
            include_bytes!(
                "fixtures/numeric_parameter_presentation_unlabeled_degree_second_native.bin"
            )
            .as_slice(),
            35.,
            true,
        ),
        (
            include_bytes!(
                "fixtures/numeric_parameter_presentation_unlabeled_length_zero_native.bin"
            )
            .as_slice(),
            0.,
            true,
        ),
        (
            include_bytes!(
                "fixtures/numeric_parameter_presentation_unlabeled_degree_twenty_native.bin"
            )
            .as_slice(),
            20.,
            true,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_presentation_empty_attribute_native.bin")
                .as_slice(),
            3.2,
            false,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_presentation_mode_zero_native.bin")
                .as_slice(),
            0.5,
            false,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_scalar_native.bin")
                .as_slice(),
            0.,
            true,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_presentation_scalar_attribute_native.bin")
                .as_slice(),
            27.,
            false,
        ),
    ] {
        let mut group = ParameterGroup::parse(raw).unwrap();
        assert!((group.value() - value).abs() < 1e-12);
        assert_eq!(group.name().is_empty(), unnamed);
        group.set_value(group.value()).unwrap();
        assert_eq!(group.as_bytes(), raw);
        let before = payload(raw, 0x0907);
        let mut source = b"GSP4".to_vec();
        source.extend_from_slice(raw);
        let before_kinds = records(&source)
            .unwrap()
            .iter()
            .map(|r| r.kind)
            .collect::<Vec<_>>();
        group.set_value(value + 1.25).unwrap();
        let after = payload(group.as_bytes(), 0x0907);
        assert_eq!(&after[60..92], &before[60..92]);
        assert_eq!(group.name().is_empty(), unnamed);
        let at = group.anchor();
        group.set_anchor((at.0 + 20, at.1 + 20)).unwrap();
        assert_eq!(group.anchor(), (at.0 + 20, at.1 + 20));
        assert_eq!(group.bbox().0, group.anchor());
        let mut changed = b"GSP4".to_vec();
        changed.extend_from_slice(group.as_bytes());
        assert_eq!(
            records(&changed)
                .unwrap()
                .iter()
                .map(|r| r.kind)
                .collect::<Vec<_>>(),
            before_kinds
        );
        ParameterGroup::parse(group.as_bytes()).unwrap();
    }
    let mode_zero = ParameterGroup::parse(include_bytes!(
        "fixtures/numeric_parameter_presentation_mode_zero_native.bin"
    ))
    .unwrap();
    assert_eq!(mode_zero.name(), "t[2]");
    assert_eq!(mode_zero.display_mode(), 0);
    assert_eq!(mode_zero.literal_definition(), "0.5 cm");
    let mut mode_zero_document = Document::from_bytes(
        include_bytes!("fixtures/numeric_parameter_presentation_mode_zero_native_verified.gsp"),
        None,
    )
    .unwrap();
    let squared = mode_zero_document
        .add_parameter_calculation(2, "(0.5 cm)^2", (10, 210))
        .unwrap();
    assert_eq!(mode_zero_document.numeric_value(squared), Some(0.25));
    mode_zero_document.set_parameter_value(2, 1.5).unwrap();
    assert_eq!(mode_zero_document.numeric_value(squared), Some(2.25));

    let bytes = include_bytes!(
        "fixtures/numeric_parameter_presentation_empty_attribute_native_verified.gsp"
    );
    let rs = records(bytes).unwrap();
    let attribute = rs.iter().find(|r| r.kind == 0x232e).unwrap();
    let bbox = rs.iter().filter(|r| r.kind == 0x08a3).nth(1).unwrap();
    let mut nonempty = bytes.to_vec();
    nonempty[attribute.start..attribute.start + 4].copy_from_slice(&2_u32.to_le_bytes());
    nonempty.splice(attribute.data.start..attribute.data.start, [1, 0]);
    let mut misplaced = bytes.to_vec();
    let empty = misplaced
        .drain(attribute.start..attribute.data.end)
        .collect::<Vec<_>>();
    misplaced.splice(bbox.start..bbox.start, empty);
    for invalid in [nonempty, misplaced] {
        assert!(ParameterGroup::parse(&object_group(&invalid, 2)).is_err());
        let document = Document::from_bytes(&invalid, None).unwrap();
        assert!(document.partial_view);
        assert_eq!(document.objects[1].kind, Kind::Unsupported(0));
        assert_eq!(document.encode().unwrap(), invalid);
    }
}

#[test]
fn native_parameter_names_use_page_counter_and_guard_missing_metadata() {
    let all = include_bytes!("fixtures/numeric_parameter_autoname_all_unnamed_native_verified.gsp");
    let document = Document::from_bytes(all, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.parameter_display_name(1).as_deref(), Some("t[4]"));
    assert_eq!(document.parameter_display_name(2).as_deref(), Some("t[5]"));
    assert_eq!(document.parameter_display_name(3).as_deref(), Some("t[6]"));
    let seven = Document::from_bytes(
        include_bytes!("fixtures/numeric_parameter_autoname_counter_seven_native_verified.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(seven.parameter_display_name(2).as_deref(), Some("t[7]"));
    let two = Document::from_bytes(
        include_bytes!("fixtures/numeric_parameter_autoname_two_parameters_native_verified.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(two.parameter_display_name(2).as_deref(), Some("t[4]"));
    let collision = Document::from_bytes(
        include_bytes!("fixtures/numeric_parameter_autoname_collision_native_verified.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(collision.parameter_display_name(2).as_deref(), Some("t[4]"));
    assert_eq!(collision.parameter_display_name(3).as_deref(), Some("t[4]"));
    assert_eq!(collision.numeric_value(2), Some(0.0001));
    assert_eq!(collision.numeric_value(3), Some(1.));
    assert_eq!(collision.numeric_value(5), Some(1.));
    let mut deleted = document.clone();
    deleted.selected.insert(1);
    assert_eq!(deleted.clear_selected().unwrap(), vec![1, 4, 12, 13, 14]);
    assert_eq!(deleted.parameter_display_name(2).as_deref(), Some("t[4]"));
    assert_eq!(deleted.parameter_display_name(3).as_deref(), Some("t[5]"));
    let encoded = deleted.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!("fixtures/numeric_parameter_autoname_deleted_native_verified.gsp")
    );
    let deleted = Document::from_bytes(&encoded, None).unwrap();
    assert!(!deleted.partial_view);
    assert_eq!(deleted.objects.len(), 9);
    assert_eq!(deleted.parameter_display_name(1).as_deref(), Some("t[4]"));
    assert_eq!(deleted.parameter_display_name(2).as_deref(), Some("t[5]"));
    assert_eq!(deleted.numeric_value(3), Some(1.));

    let counter = records(all)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x0960)
        .unwrap();
    let mut missing = all.to_vec();
    missing.drain(counter.start..counter.data.end);
    let mut bad_format = all.to_vec();
    bad_format[counter.data.start] = 1;
    let mut short = all.to_vec();
    short[counter.start..counter.start + 4].copy_from_slice(&4_u32.to_le_bytes());
    short.drain(counter.data.end - 2..counter.data.end);
    let mut zero = all.to_vec();
    zero[counter.data.start + 2..counter.data.end].fill(0);
    let mut duplicate = all.to_vec();
    duplicate.splice(
        counter.data.end..counter.data.end,
        all[counter.start..counter.data.end].iter().copied(),
    );
    for invalid in [missing, bad_format, short, zero, duplicate] {
        let decoded = Document::from_bytes(&invalid, None).unwrap();
        assert!(decoded.partial_view);
        assert_eq!(decoded.parameter_display_name(1), None);
        assert_eq!(decoded.objects[0].kind, Kind::Unsupported(0));
        assert_eq!(decoded.encode().unwrap(), invalid);
    }
    let mut overflow = all.to_vec();
    overflow[counter.data.start + 2..counter.data.end].copy_from_slice(&u32::MAX.to_le_bytes());
    let decoded = Document::from_bytes(&overflow, None).unwrap();
    assert!(decoded.partial_view);
    assert_eq!(decoded.parameter_display_name(2), None);
    assert_eq!(decoded.objects[1].kind, Kind::Unsupported(0));

    // Unrelated naming metadata does not prevent explicitly named parameters
    // from retaining their verified numeric meanings and captions.
    let mut named =
        include_bytes!("fixtures/numeric_typed_parameters_native_verified.gsp").to_vec();
    let counter = records(&named)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x0960)
        .unwrap();
    named[counter.data.start] = 1;
    let named = Document::from_bytes(&named, None).unwrap();
    assert!(!named.partial_view);
    assert_eq!(named.numeric_value(1), Some(1.));
    assert_eq!(named.parameter_display_name(1).as_deref(), Some("t[1]"));
}

#[test]
fn native_parameter_presentation_edits_and_copies_keep_live_numeric_points() {
    for (original, expected, parameter, value, x, y, unnamed, mode_zero) in [
        (include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_length_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_length_edited_copied_native_verified.gsp").as_slice(), 2, 0.00025, 2.5, 1., true, false),
        (include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_degree_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_degree_edited_copied_native_verified.gsp").as_slice(), 1, 45., 0.00001, (45_f64 * 3.1415926535898 / 180.).sin(), true, false),
        (include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_scalar_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_parameter_presentation_unlabeled_scalar_edited_copied_native_verified.gsp").as_slice(), 2, 2.5, 3.5, 1., true, false),
        (include_bytes!("fixtures/numeric_parameter_presentation_empty_attribute_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_parameter_presentation_empty_attribute_edited_copied_native_verified.gsp").as_slice(), 2, 2.75, 2.75, 1., false, false),
        (include_bytes!("fixtures/numeric_parameter_presentation_mode_zero_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_parameter_presentation_mode_zero_edited_copied_native_verified.gsp").as_slice(), 2, 2.25, 2.25, 1., false, true),
        (include_bytes!("fixtures/numeric_parameter_presentation_scalar_attribute_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_parameter_presentation_scalar_attribute_edited_copied_native_verified.gsp").as_slice(), 2, 32.5, 3.25, 1., false, false),
    ] {
        let mut document = Document::from_bytes(original, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), original);
        let original_name = document.parameter_display_name(parameter).unwrap();
        if unnamed { assert_eq!(original_name, "t[4]"); }
        if mode_zero { assert_eq!(original_name, "0.5 cm"); }
        document.set_parameter_value(parameter, value).unwrap();
        if parameter == 2 { document.set_parameter_value(1, 1.).unwrap(); }
        let point = document.position(14).unwrap();
        assert!((point.x - (200. + x * 37.79527559055118)).abs() < 1e-10);
        assert!((point.y - (250. - y * 37.79527559055118)).abs() < 1e-10);
        select_construction_graph(&mut document, 14);
        let clipboard = document.copy_selected().unwrap();
        assert_eq!(*document.paste(&clipboard).unwrap().last().unwrap(), 24);
        let copied_point = document.position(24).unwrap();
        assert!((copied_point.x - point.x - 20.).abs() < 1e-10);
        assert!((copied_point.y - point.y - 20.).abs() < 1e-10);
        let copied_parameter = if parameter == 1 { 15 } else { 16 };
        assert!((document.numeric_value(copied_parameter).unwrap() - value).abs() < 1e-12);
        if unnamed {
            assert_eq!(document.parameter_display_name(copied_parameter).as_deref(), Some("t[5]"));
        }
        if mode_zero {
            assert_eq!(document.parameter_display_name(parameter).as_deref(), Some("2.25 cm"));
            assert!(document.parameter_calculation_expression(5).unwrap().contains("(2.25 cm)"));
        }
        assert!(document.can_save().is_ok());
        let encoded = document.encode().unwrap();
        assert_eq!(encoded, expected);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(24), document.position(24));
        assert_eq!(reopened.parameter_display_name(copied_parameter), document.parameter_display_name(copied_parameter));

        // Resolved native names must also bind authored calculations by ID,
        // rather than turning the displayed mode-zero definition into a constant.
        let name = document.parameter_display_name(parameter).unwrap();
        let definition = if mode_zero { format!("({name})") } else { name };
        let calculation = document.add_parameter_calculation(parameter, &format!("{definition}^2"), (10, 210)).unwrap();
        assert!((document.numeric_value(calculation).unwrap() - value * value).abs() < 1e-12);
        document.set_parameter_value(parameter, value + 0.25).unwrap();
        assert!((document.numeric_value(calculation).unwrap() - (value + 0.25).powi(2)).abs() < 1e-12);
        let encoded = document.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert!((reopened.numeric_value(calculation).unwrap() - (value + 0.25).powi(2)).abs() < 1e-12);
    }
}

#[test]
fn generated_name_and_literal_parameter_tables_reject_without_mutating_history() {
    for (bytes, expected_next) in [(
        include_bytes!("fixtures/numeric_parameter_autoname_all_unnamed_native_verified.gsp")
            .as_slice(),
        "t[7]",
    )] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let original_names = (1..=3)
            .map(|id| document.parameter_display_name(id))
            .collect::<Vec<_>>();
        let Kind::Parameter(following) = document.objects[1].kind.clone() else {
            panic!("expected unnamed scalar source");
        };
        assert_eq!(following.name(), "");
        document.set_parameter_value(3, 2.).unwrap();
        let before = document.encode().unwrap();
        let count = document.objects.len();
        assert!(
            document
                .tabulate_default_parameter(2, (10, 280))
                .unwrap_err()
                .contains("supports scalar parameters")
        );
        assert!(document.tabulate_default_pair((10, 280)).is_err());
        assert_eq!(document.objects.len(), count);
        assert_eq!(document.encode().unwrap(), before);
        // The failed operation must not insert a no-op history entry or alter
        // the naming cursor. Undo still reaches the preceding value edit.
        document.undo();
        assert_eq!(document.numeric_value(3), Some(1.));
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(
            (1..=3)
                .map(|id| document.parameter_display_name(id))
                .collect::<Vec<_>>(),
            original_names
        );
        document.redo();
        assert_eq!(document.numeric_value(3), Some(2.));
        assert_eq!(document.encode().unwrap(), before);
        document.undo();
        let added = document.add(Kind::Parameter(following));
        assert_eq!(
            document.parameter_display_name(added).as_deref(),
            Some(expected_next)
        );
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.parameter_display_name(added).as_deref(),
            Some(expected_next)
        );
    }

    let bytes =
        include_bytes!("fixtures/numeric_parameter_presentation_mode_zero_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    document.set_parameter_value(2, 0.75).unwrap();
    let before = document.encode().unwrap();
    assert!(
        document
            .tabulate_default_parameter(2, (10, 280))
            .unwrap_err()
            .contains("parameter definitions")
    );
    assert_eq!(document.encode().unwrap(), before);
    assert_eq!(
        document.parameter_display_name(2).as_deref(),
        Some("0.75 cm")
    );
    document.undo();
    assert_eq!(document.numeric_value(2), Some(0.5));
    assert_eq!(
        document.parameter_display_name(2).as_deref(),
        Some("0.5 cm")
    );
    assert_eq!(document.encode().unwrap(), bytes);
}

#[test]
fn authentic_indexed_parameter_tables_import_and_literal_tables_remain_preserved() {
    use sketchpad_rs::table_records::TableGroup;
    for (bytes, expected_counter, expected_names, expected_layout_len) in [
        (
            include_bytes!("fixtures/numeric_parameter_table_unnamed_scalar_native_created.gsp")
                .as_slice(),
            5_u32,
            ["t[1]", "t[4]", "t[3]"],
            64,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_table_all_unnamed_native_created.gsp")
                .as_slice(),
            7_u32,
            ["t[4]", "t[5]", "t[6]"],
            64,
        ),
        (
            include_bytes!("fixtures/numeric_parameter_table_mode_zero_native_created.gsp")
                .as_slice(),
            4_u32,
            ["t[1]", "t[2]", "t[3]"],
            76,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.partial_view, expected_layout_len == 76);
        assert_eq!(document.objects.len(), 15);
        if expected_layout_len == 76 {
            assert_eq!(document.objects[14].kind, Kind::Unsupported(91));
        } else {
            let Kind::ValueTable(source, table) = &document.objects[14].kind else {
                panic!("indexed native table must import as a table");
            };
            let name = document.parameter_display_name(*source).unwrap();
            assert_eq!(table.named_label_values().unwrap().0, name);
            assert_eq!(table.rows(), 1);
            document.verify_roundtrip().unwrap();
            let layout = records(bytes)
                .unwrap()
                .into_iter()
                .find(|record| record.kind == 0x090d)
                .unwrap();
            let text = &bytes[layout.data.clone()];
            let at = text
                .windows(4)
                .position(|text| text.starts_with(b"T1x") && text[3].is_ascii_digit())
                .unwrap();
            let mut mismatched = bytes.to_vec();
            mismatched[layout.data.start + at + 3] += 1;
            let rejected = Document::from_bytes(&mismatched, None).unwrap();
            assert_eq!(rejected.objects[14].kind, Kind::Unsupported(91));
            assert_eq!(rejected.encode().unwrap(), mismatched);
        }
        assert_eq!(document.encode().unwrap(), bytes);
        for (object, name) in document.objects.iter().zip(expected_names) {
            let Kind::Parameter(parameter) = &object.kind else {
                panic!("native table source must remain a supported parameter");
            };
            assert_eq!(parameter.name(), name);
        }
        let counter = records(bytes)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x0960)
            .unwrap();
        assert_eq!(
            &bytes[counter.data],
            &[0, 0, expected_counter as u8, 0, 0, 0]
        );
        let group = object_group(bytes, 15);
        assert_eq!(
            TableGroup::parse(&group).is_err(),
            expected_layout_len == 76
        );
        assert_eq!(payload(&group, 0x090d).len(), expected_layout_len);
        assert_eq!(payload(&group, 0x07d8), [3, 0]);
    }
}

#[test]
fn native_calculation_radian_literals_use_full_degree_values_and_native_captions() {
    const PI: f64 = 3.1415926535898;
    for (bytes, value, literal, precision) in [
        (
            include_bytes!("fixtures/numeric_calculation_one_radian_native_verified.gsp")
                .as_slice(),
            0.01 * 180. / PI,
            "57.2958°",
            2,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_decimal_radian_native_verified.gsp")
                .as_slice(),
            0.02 * 180. / PI,
            "114.592°",
            2,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_three_radian_native_verified.gsp")
                .as_slice(),
            0.03 * 180. / PI,
            "171.887°",
            2,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_pi_radian_native_verified.gsp").as_slice(),
            3.6,
            "180°",
            2,
        ),
        (
            include_bytes!(
                "fixtures/numeric_calculation_decimal_radian_precision4_native_verified.gsp"
            )
            .as_slice(),
            0.02 * 180. / PI,
            "114.592°",
            4,
        ),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        document.can_save().unwrap();
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(
            document.objects[4].kind.numeric_unit(),
            Some(NumericUnit::Degrees)
        );
        assert!((document.numeric_value(5).unwrap() - value).abs() < 1e-12);
        let Kind::ParameterCalculation(_, group) = &document.objects[4].kind else {
            panic!("expected radian calculation");
        };
        assert_eq!(group.precision(), precision);
        assert!(
            document
                .parameter_calculation_expression(5)
                .unwrap()
                .contains(literal)
        );
        assert!(
            payload(group.as_bytes(), 0x0907)[92..]
                .chunks_exact(2)
                .any(|word| word == [0, 1])
        );
        let point = document.position(14).unwrap();
        assert!((point.x - (200. + value * 37.79527559055118)).abs() < 1e-10);
        assert!((point.y - (250. - 0.5 * 37.79527559055118)).abs() < 1e-10);
        let raw = object_group(bytes, 5);
        document
            .set_parameter_value(3, document.numeric_value(3).unwrap() * 2.)
            .unwrap();
        assert!((document.numeric_value(5).unwrap() - value / 2.).abs() < 1e-12);
        let updated = document.encode().unwrap();
        assert_eq!(object_group(&updated, 5), raw);
        let reopened = Document::from_bytes(&updated, None).unwrap();
        assert!(!reopened.partial_view);
        assert!((reopened.numeric_value(5).unwrap() - value / 2.).abs() < 1e-12);
    }
    let decimal = ParameterCalculationGroup::parse(include_bytes!(
        "fixtures/numeric_calculation_decimal_radian_original.bin"
    ))
    .unwrap()
    .with_source_units(&[NumericUnit::Length, NumericUnit::Length])
    .unwrap();
    assert_eq!(decimal.sources(), [27, 4]);
    assert_eq!(decimal.unit(), NumericUnit::Degrees);
    assert!(
        (decimal
            .value_for_sources(&[0.01, 1.], &[NumericUnit::Length, NumericUnit::Length])
            .unwrap()
            - 0.02 * 180. / PI)
            .abs()
            < 1e-12
    );
    let pi = ParameterCalculationGroup::parse(include_bytes!(
        "fixtures/numeric_calculation_pi_radian_original.bin"
    ))
    .unwrap();
    assert_eq!(pi.sources(), [5]);
    assert_eq!(pi.value(5.), Some(72.));
}

#[test]
fn authored_calculation_radians_keep_unsupported_token_layouts_guarded() {
    let bytes = include_bytes!("fixtures/numeric_calculation_decimal_radian_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    for expression in [
        "t[4]*4 rad",
        "t[4]*0.5 rad",
        "t[4]*e rad",
        "t[4]*pi°",
        "t[4]*2 radians",
    ] {
        assert!(
            document
                .add_parameter_calculation(2, expression, (170, 90))
                .is_err(),
            "{expression}"
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    assert_eq!(
        CalculationGroup::from_expression("sin(1 rad)")
            .unwrap()
            .unit(),
        NumericUnit::Scalar
    );
    for (expression, unit) in [
        (
            "pi rad^2",
            NumericUnit::Compound {
                length: 0,
                degrees: 2,
            },
        ),
        ("pi rad+1", NumericUnit::Scalar),
    ] {
        assert_eq!(
            CalculationGroup::from_expression(expression)
                .unwrap()
                .unit(),
            unit
        );
    }
    let mixed = CalculationGroup::from_expression("pi rad+1 cm").unwrap();
    assert_eq!(mixed.unit(), NumericUnit::Scalar);
    assert_eq!(mixed.evaluated_value(), Some(181.));
    let calc = document
        .add_parameter_calculation(2, "t[4]*2 rad", (170, 90))
        .unwrap();
    assert_eq!(
        document.objects[calc - 1].kind.numeric_unit(),
        Some(NumericUnit::Degrees)
    );
    assert!(
        document
            .parameter_calculation_expression(calc)
            .unwrap()
            .contains("114.592°")
    );
    let Kind::ParameterCalculation(_, group) = &document.objects[calc - 1].kind else {
        panic!("expected authored calculation");
    };
    assert_eq!(
        &payload(group.as_bytes(), 0x0907)[92..],
        &[0, 96, 2, 16, 2, 0, 0, 1]
    );
    document.set_parameter_value(2, 0.03).unwrap();
    assert!((document.numeric_value(calc).unwrap() - 0.06 * 180. / 3.1415926535898).abs() < 1e-12);
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(calc), document.numeric_value(calc));
}

#[test]
fn radian_calculation_edits_authoring_and_clipboard_match_native_golden() {
    let original =
        include_bytes!("fixtures/numeric_calculation_decimal_radian_native_verified.gsp");
    let mut document = Document::from_bytes(original, None).unwrap();
    document.set_parameter_value(1, 1.5).unwrap();
    document.set_parameter_value(2, 0.03).unwrap();
    document.set_parameter_value(3, 200.).unwrap();
    let rad = document
        .add_parameter_calculation(2, "t[4]*2 rad", (170, 90))
        .unwrap();
    let pi = document
        .add_parameter_calculation(3, "2*pi rad/t[3]", (170, 140))
        .unwrap();
    let rp = document.add(Kind::NumericCoordinatePoint(rad, 1, 11));
    let pp = document.add(Kind::NumericCoordinatePoint(pi, 1, 11));
    let positions = [
        document.position(rp).unwrap(),
        document.position(pp).unwrap(),
    ];
    let original_count = document.objects.len();
    document.selected.insert(1); // Include the shared coordinate input to retain live constructions.
    document.selected.insert(rp);
    document.selected.insert(pp);
    let clipboard = document.copy_selected().unwrap();
    document.paste(&clipboard).unwrap();
    document.can_save().unwrap();
    assert_eq!(document.objects.len(), 31);
    assert!((document.numeric_value(rad).unwrap() - 3.4377467707849316).abs() < 1e-12);
    assert_eq!(document.numeric_value(pi), Some(1.8));
    let copied = document
        .objects
        .iter()
        .skip(original_count)
        .filter(|object| matches!(object.kind, Kind::NumericCoordinatePoint(_, _, _)))
        .map(|object| object.id)
        .collect::<Vec<_>>();
    assert_eq!(copied.len(), 2);
    for (&id, expected) in copied.iter().zip(positions) {
        let actual = document.position(id).unwrap();
        assert!((actual.x - expected.x - 20.).abs() < 1e-10);
        assert!((actual.y - expected.y - 20.).abs() < 1e-10);
    }
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!(
            "fixtures/numeric_calculation_radian_edited_authored_copied_native_verified.gsp"
        )
    );
    assert_eq!(object_group(&encoded, 5), object_group(original, 5));
    let mut reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    for &id in &copied {
        assert_eq!(reopened.position(id), document.position(id));
    }
    let Kind::NumericCoordinatePoint(copied_rad, _, _) = reopened.objects[copied[0] - 1].kind
    else {
        panic!("expected copied numeric point");
    };
    let Kind::ParameterCalculation(copied_source, _) = reopened.objects[copied_rad - 1].kind else {
        panic!("expected copied radian calculation");
    };
    reopened.set_parameter_value(copied_source, 0.04).unwrap();
    assert!(
        (reopened.numeric_value(copied_rad).unwrap() - 0.08 * 180. / 3.1415926535898).abs() < 1e-12
    );
    assert_eq!(reopened.numeric_value(rad), document.numeric_value(rad));
    let rereopened = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert!(!rereopened.partial_view);
    assert_eq!(
        rereopened.numeric_value(copied_rad),
        reopened.numeric_value(copied_rad)
    );
}

#[test]
fn authentic_sourcefree_radian_angles_preserve_native_numeric_and_presentation_records() {
    for (raw, name, mode, precision, value) in [
        (
            include_bytes!("fixtures/numeric_sourcefree_radian_named_original.bin").as_slice(),
            Some("x轴首刻度"),
            3,
            2,
            15.,
        ),
        (
            include_bytes!("fixtures/numeric_sourcefree_radian_unnamed_original.bin").as_slice(),
            None,
            0,
            5,
            100. / 7.,
        ),
    ] {
        let mut group = CalculationGroup::parse(raw).unwrap();
        assert_eq!(group.as_bytes(), raw);
        assert!(group.is_native_radian_angle());
        assert_eq!(group.name(), name);
        assert_eq!(group.display_mode(), mode);
        assert_eq!(group.precision(), precision);
        assert_eq!(group.unit(), NumericUnit::Degrees);
        assert!((group.value() - value).abs() < 1e-12);
        assert!(group.expression().contains("180°"));
        group.set_expression("pi rad/120").unwrap();
        assert_eq!(group.value(), 1.5);
        for kind in [0x07d0, 0x07d8, 0x08a3] {
            assert_eq!(payload(group.as_bytes(), kind), payload(raw, kind));
        }
        if name.is_some() {
            assert_eq!(payload(group.as_bytes(), 0x07d5), payload(raw, 0x07d5));
        }
        let before = payload(raw, 0x0907);
        let after = payload(group.as_bytes(), 0x0907);
        for range in [0..36, 40..44, 46..92] {
            assert_eq!(after[range.clone()], before[range]);
        }
        for invalid in ["2+3", "sin(1 rad)", "pi rad^2", "4 rad"] {
            let previous = group.clone();
            assert!(group.set_expression(invalid).is_err(), "{invalid}");
            assert_eq!(group, previous);
        }
        let mut wrong_subtype = raw.to_vec();
        wrong_subtype[16..18].copy_from_slice(&48_u16.to_le_bytes());
        assert!(CalculationGroup::parse(&wrong_subtype).is_err());
        let mut unproven_mode = raw.to_vec();
        let numeric = records(&[b"GSP4".as_slice(), raw].concat())
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x0907)
            .unwrap()
            .data
            .start
            - 4;
        unproven_mode[numeric + 26..numeric + 28].copy_from_slice(&2_u16.to_le_bytes());
        assert!(CalculationGroup::parse(&unproven_mode).is_err());
    }
}

#[test]
fn native_sourcefree_radian_import_live_expression_edits_and_failed_edit_history() {
    for (bytes, initial, name) in [
        (
            include_bytes!("fixtures/numeric_sourcefree_radian_named_native_verified.gsp")
                .as_slice(),
            15.,
            Some("x轴首刻度"),
        ),
        (
            include_bytes!("fixtures/numeric_sourcefree_radian_unnamed_native_verified.gsp")
                .as_slice(),
            100. / 7.,
            None,
        ),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        document.can_save().unwrap();
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(
            document.objects[4].kind.numeric_unit(),
            Some(NumericUnit::Degrees)
        );
        assert!((document.numeric_value(5).unwrap() - initial).abs() < 1e-12);
        assert!((document.numeric_value(15).unwrap() - initial / 10.).abs() < 1e-12);
        let position = document.position(16).unwrap();
        assert!((position.x - (200. + initial / 10. * 37.79527559055118)).abs() < 1e-10);
        assert!((position.y - (250. - 0.5 * 37.79527559055118)).abs() < 1e-10);
        if let Some(name) = name {
            assert!(
                document
                    .parameter_calculation_expression(15)
                    .unwrap()
                    .contains(name)
            );
        }
        document
            .set_calculation_expression(5, "pi rad/120")
            .unwrap();
        assert_eq!(document.numeric_value(5), Some(1.5));
        assert_eq!(document.numeric_value(15), Some(0.15));
        let before = document.encode().unwrap();
        for invalid in ["2+3", "sin(1 rad)", "pi rad^2", "4 rad"] {
            assert!(document.set_calculation_expression(5, invalid).is_err());
            assert_eq!(document.encode().unwrap(), before);
        }
        document.undo();
        assert!((document.numeric_value(5).unwrap() - initial).abs() < 1e-12);
        assert_eq!(document.encode().unwrap(), bytes);
        document.redo();
        assert_eq!(document.encode().unwrap(), before);
        let reopened = Document::from_bytes(&before, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.numeric_value(5), Some(1.5));
        assert_eq!(reopened.numeric_value(15), Some(0.15));
    }
}

#[test]
fn sourcefree_radian_edits_authoring_and_copies_match_native_writer_goldens() {
    for (original, golden) in [
        (
            include_bytes!("fixtures/numeric_sourcefree_radian_named_native_verified.gsp").as_slice(),
            include_bytes!("fixtures/numeric_sourcefree_radian_named_edited_authored_copied_native_verified.gsp").as_slice(),
        ),
        (
            include_bytes!("fixtures/numeric_sourcefree_radian_unnamed_native_verified.gsp").as_slice(),
            include_bytes!("fixtures/numeric_sourcefree_radian_unnamed_edited_authored_copied_native_verified.gsp").as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(original, None).unwrap();
        document.set_parameter_value(1, 1.5).unwrap();
        document.set_calculation_expression(5, "pi rad/120").unwrap();
        document.set_numeric_precision(&[5], 4).unwrap();
        document.move_number(5, (170, 90)).unwrap();
        let literal = document.add_calculation_expression("pi rad/120", (10, 170)).unwrap();
        let point = document.add(Kind::NumericCoordinatePoint(literal, 1, 11));
        let original_count = document.objects.len();
        let original_position = document.position(16).unwrap();
        document.selected.insert(1); // Copy the live coordinate constructions with their shared input.
        document.selected.insert(16);
        document.selected.insert(point);
        let clipboard = document.copy_selected().unwrap();
        document.paste(&clipboard).unwrap();
        let copied_point = document.objects.iter().skip(original_count)
            .find(|object| matches!(object.kind, Kind::NumericCoordinatePoint(_, _, _)))
            .unwrap().id;
        let Kind::NumericCoordinatePoint(calculation, _, _) = document.objects[copied_point - 1].kind else {
            panic!("expected copied coordinate point");
        };
        let Kind::ParameterCalculation(source, _) = document.objects[calculation - 1].kind else {
            panic!("expected copied source calculation");
        };
        let copied_position = document.position(copied_point).unwrap();
        assert!((copied_position.x - original_position.x - 20.).abs() < 1e-10);
        assert!((copied_position.y - original_position.y - 20.).abs() < 1e-10);
        document.set_calculation_expression(source, "pi rad/240").unwrap();
        assert_eq!(document.numeric_value(5), Some(1.5));
        assert_eq!(document.numeric_value(source), Some(0.75));
        assert_eq!(document.numeric_value(calculation), Some(0.075));
        document.can_save().unwrap();
        let encoded = document.encode().unwrap();
        assert_eq!(encoded, golden);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.numeric_value(source), Some(0.75));
        assert_eq!(reopened.position(copied_point), document.position(copied_point));
    }
}

#[test]
fn sourcefree_radian_literal_and_scalar_rounding_controls_match_native_writer_goldens() {
    let original = include_bytes!("fixtures/numeric_sourcefree_radian_named_native_verified.gsp");
    let mut document = Document::from_bytes(original, None).unwrap();
    let literal = document
        .add_calculation_expression("1 rad", (10, 170))
        .unwrap();
    let derived = document
        .add_parameter_calculation(literal, "v/100", (170, 170))
        .unwrap();
    let point = document.add(Kind::NumericCoordinatePoint(derived, 1, 11));
    assert!((document.numeric_value(literal).unwrap() - 180. / 3.1415926535898).abs() < 1e-12);
    assert!(document.position(point).unwrap().x > 200.);
    document.can_save().unwrap();
    assert_eq!(
        document.encode().unwrap(),
        include_bytes!("fixtures/numeric_sourcefree_radian_one_literal_native_verified.gsp")
    );

    for (expression, value, golden) in [
        (
            "3/40",
            0.075,
            include_bytes!("fixtures/numeric_scalar_fraction_precision_native_verified.gsp")
                .as_slice(),
        ),
        (
            "1/8",
            0.125,
            include_bytes!("fixtures/numeric_scalar_true_half_precision_native_verified.gsp")
                .as_slice(),
        ),
        (
            "-1/8",
            -0.125,
            include_bytes!("fixtures/numeric_scalar_negative_half_precision_native_verified.gsp")
                .as_slice(),
        ),
    ] {
        let mut control = Document::from_bytes(original, None).unwrap();
        let calculation = control
            .add_calculation_expression(expression, (170, 90))
            .unwrap();
        let point = control.add(Kind::NumericCoordinatePoint(calculation, 1, 11));
        assert_eq!(control.numeric_value(calculation), Some(value));
        assert!(
            (control.position(point).unwrap().x - (200. + value * 37.79527559055118)).abs() < 1e-10
        );
        control.can_save().unwrap();
        assert_eq!(control.encode().unwrap(), golden);
        assert!(!Document::from_bytes(golden, None).unwrap().partial_view);
    }
}

#[test]
fn authentic_centimeter_literals_keep_quantity_units_separate_from_presentation() {
    for (raw, source_unit, source_value, value, mode) in [
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_integer_original.bin")
                .as_slice(),
            NumericUnit::Scalar,
            -1.5,
            1.5,
            0,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_decimal_original.bin")
                .as_slice(),
            NumericUnit::Scalar,
            -1.5,
            1.2,
            0,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_empty_original.bin").as_slice(),
            NumericUnit::Scalar,
            3.,
            1.5,
            1,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_named_original.bin").as_slice(),
            NumericUnit::Length,
            0.5,
            1.,
            3,
        ),
    ] {
        let group = ParameterCalculationGroup::parse(raw)
            .unwrap()
            .with_source_units(&[source_unit])
            .unwrap();
        assert_eq!(group.as_bytes(), raw);
        assert_eq!(group.display_mode(), mode);
        assert!(group.expression("v").contains(" cm"));
        assert!(
            (group
                .value_with_source_unit(source_value, source_unit)
                .unwrap()
                - value)
                .abs()
                < 1e-12
        );
        assert_eq!(
            group.unit(),
            if mode == 3 {
                NumericUnit::Scalar
            } else {
                NumericUnit::Length
            }
        );
        let mut edited = group.clone();
        edited.set_precision(4).unwrap();
        edited.set_anchor((10, 170)).unwrap();
        assert_eq!(edited.expression("v"), group.expression("v"));
        assert_eq!(edited.unit(), group.unit());
        assert_eq!(edited.caption_suppressed(), mode == 1);
        for kind in [0x07d0, 0x07d2, 0x07d8] {
            assert_eq!(payload(edited.as_bytes(), kind), payload(raw, kind));
        }
        let before = payload(raw, 0x0907);
        let after = payload(edited.as_bytes(), 0x0907);
        for range in [0..12, 16..28, 30..before.len()] {
            assert_eq!(after[range.clone()], before[range]);
        }
        let mut invalid_suffix = raw.to_vec();
        let records = records(&[b"GSP4".as_slice(), raw].concat()).unwrap();
        let numeric = records
            .iter()
            .find(|record| record.kind == 0x0907)
            .unwrap()
            .data
            .start
            - 4;
        let at = before[92..]
            .chunks_exact(2)
            .position(|word| word == [1, 2])
            .unwrap();
        invalid_suffix[numeric + 92 + at * 2..numeric + 94 + at * 2]
            .copy_from_slice(&0x0202_u16.to_le_bytes());
        assert!(ParameterCalculationGroup::parse(&invalid_suffix).is_err());
    }
    for expression in ["pi cm", "e cm", "1 cm cm", "1 rad cm"] {
        assert!(
            CalculationGroup::from_expression(expression).is_err(),
            "{expression}"
        );
    }
    let mixed = ParameterCalculationGroup::from_expression_with_source_unit(
        1,
        "v",
        "1 cm+v",
        (10, 10),
        NumericUnit::Degrees,
    )
    .unwrap();
    assert_eq!(mixed.unit(), NumericUnit::Scalar);
    assert_eq!(
        mixed.value_with_source_unit(90., NumericUnit::Degrees),
        Some(91.)
    );
    let division = ParameterCalculationGroup::from_expression_with_source_unit(
        1,
        "v",
        "1 cm/v+1",
        (10, 10),
        NumericUnit::Length,
    )
    .unwrap();
    assert_eq!(division.unit(), NumericUnit::Scalar);
    assert_eq!(
        division.value_with_source_unit(0.5, NumericUnit::Length),
        Some(3.)
    );
}

#[test]
fn native_centimeter_calculation_layouts_import_live_values_and_exact_records() {
    for (bytes, value, unit, suppressed) in [
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_integer_native_verified.gsp")
                .as_slice(),
            1.5,
            NumericUnit::Length,
            false,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_decimal_native_verified.gsp")
                .as_slice(),
            1.2,
            NumericUnit::Length,
            false,
        ),
        (
            include_bytes!(
                "fixtures/numeric_calculation_centimeter_precision4_native_verified.gsp"
            )
            .as_slice(),
            1.2,
            NumericUnit::Length,
            false,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_empty_native_verified.gsp")
                .as_slice(),
            1.5,
            NumericUnit::Length,
            true,
        ),
        (
            include_bytes!(
                "fixtures/numeric_calculation_centimeter_empty_visible_native_verified.gsp"
            )
            .as_slice(),
            1.5,
            NumericUnit::Length,
            false,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_named_native_verified.gsp")
                .as_slice(),
            1.,
            NumericUnit::Scalar,
            false,
        ),
        (
            include_bytes!(
                "fixtures/numeric_calculation_centimeter_named_ratio_native_verified.gsp"
            )
            .as_slice(),
            3.,
            NumericUnit::Scalar,
            false,
        ),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        document.can_save().unwrap();
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(document.objects[4].kind.numeric_unit(), Some(unit));
        assert!((document.numeric_value(5).unwrap() - value).abs() < 1e-12);
        let position = document.position(14).unwrap();
        assert!((position.x - (200. + value * 37.79527559055118)).abs() < 1e-10);
        let Kind::ParameterCalculation(_, group) = &document.objects[4].kind else {
            panic!("expected native calculation");
        };
        assert_eq!(group.caption_suppressed(), suppressed);
        let original_group = object_group(bytes, 5);
        document.set_parameter_value(2, 2.).unwrap();
        let live = document.numeric_value(5).unwrap();
        assert_eq!(object_group(&document.encode().unwrap(), 5), original_group);
        if (live - value).abs() > 1e-12 {
            assert_ne!(document.position(14), Some(position));
        } else {
            assert_eq!(document.position(14), Some(position));
        }
        let changed = document.encode().unwrap();
        assert_eq!(
            Document::from_bytes(&changed, None)
                .unwrap()
                .numeric_value(5),
            Some(live)
        );
        document.undo();
        assert_eq!(document.encode().unwrap(), bytes);
        let before = document.encode().unwrap();
        let length = document
            .add_calculation_expression("1 cm", (10, 10))
            .unwrap();
        assert_eq!(document.numeric_value(length), Some(1.));
        document.verify_roundtrip().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), before);
    }
}

#[test]
fn centimeter_edits_authoring_and_live_copies_match_native_writer_goldens() {
    for (original, golden, source_value, expression, copied_value, result) in [
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_decimal_native_verified.gsp").as_slice(),
            include_bytes!("fixtures/numeric_calculation_centimeter_decimal_edited_authored_copied_native_verified.gsp").as_slice(),
            -2.5, "t[4]*(-0.5 cm)", -0.75, 2.,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_empty_visible_native_verified.gsp").as_slice(),
            include_bytes!("fixtures/numeric_calculation_centimeter_empty_visible_edited_authored_copied_native_verified.gsp").as_slice(),
            4., "0.5 cm*t[4]", 2., 2.,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_centimeter_named_ratio_native_verified.gsp").as_slice(),
            include_bytes!("fixtures/numeric_calculation_centimeter_named_ratio_edited_authored_copied_native_verified.gsp").as_slice(),
            1.25, "(1 cm/t[2])+1", 2.5, 1.8,
        ),
    ] {
        let mut document = Document::from_bytes(original, None).unwrap();
        document.set_parameter_value(2, source_value).unwrap();
        document.set_numeric_precision(&[5], 4).unwrap();
        document.move_number(5, (10, 170)).unwrap();
        let authored = document.add_parameter_calculation(2, expression, (170, 90)).unwrap();
        let point = document.add(Kind::NumericCoordinatePoint(authored, 1, 11));
        let original_count = document.objects.len();
        let original_position = document.position(14).unwrap();
        select_construction_graph(&mut document, 14);
        document.selected.insert(point);
        let clipboard = document.copy_selected().unwrap();
        document.paste(&clipboard).unwrap();
        let copied_point = document.objects.iter().skip(original_count)
            .find(|object| matches!(object.kind, Kind::NumericCoordinatePoint(_, _, _))).unwrap().id;
        let Kind::NumericCoordinatePoint(calculation, _, _) = document.objects[copied_point - 1].kind else { panic!("expected copied point"); };
        let Kind::ParameterCalculation(source, _) = document.objects[calculation - 1].kind else { panic!("expected copied calculation"); };
        let copied_position = document.position(copied_point).unwrap();
        assert!((copied_position.x - original_position.x - 20.).abs() < 1e-10);
        assert!((copied_position.y - original_position.y - 20.).abs() < 1e-10);
        document.set_parameter_value(source, copied_value).unwrap();
        assert!((document.numeric_value(5).unwrap() - result).abs() < 1e-12);
        assert_ne!(document.numeric_value(calculation), document.numeric_value(5));
        assert_eq!(document.numeric_value(2), Some(source_value));
        document.can_save().unwrap();
        let encoded = document.encode().unwrap();
        assert_eq!(document.objects.len(), 28);
        assert_eq!(encoded, golden);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(copied_point), document.position(copied_point));
        assert_eq!(reopened.numeric_value(authored), document.numeric_value(authored));
        assert_eq!(reopened.numeric_value(source), Some(copied_value));
    }
}

#[test]
fn native_trailing_decimal_words_normalize_captions_without_rewriting_math() {
    for (raw, units, values, expected) in [
        (
            include_bytes!("fixtures/numeric_calculation_trailing_decimal_pursuit_original.bin")
                .as_slice(),
            vec![NumericUnit::Length, NumericUnit::Length],
            vec![0.01, 1.],
            2.4,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube84_original.bin")
                .as_slice(),
            vec![NumericUnit::Scalar],
            vec![3.],
            2.,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube86_original.bin")
                .as_slice(),
            vec![NumericUnit::Scalar],
            vec![4.],
            3.,
        ),
        (
            include_bytes!("fixtures/numeric_calculation_trailing_decimal_mechanical_original.bin")
                .as_slice(),
            vec![
                NumericUnit::Length,
                NumericUnit::Scalar,
                NumericUnit::Scalar,
                NumericUnit::Scalar,
            ],
            vec![1., 1., 3., 0.05],
            std::f64::consts::FRAC_1_SQRT_2,
        ),
    ] {
        let group = ParameterCalculationGroup::parse(raw)
            .unwrap()
            .with_source_units(&units)
            .unwrap();
        assert_eq!(group.as_bytes(), raw);
        assert!((group.value_for_sources(&values, &units).unwrap() - expected).abs() < 1e-12);
        let expression = group.expression_for_sources(&["a", "b", "c", "d"]);
        assert!(!expression.contains("240."));
        assert!(!expression.contains("1."));
        assert!(!expression.contains("0."));
        let mut edited = group.clone();
        edited.set_precision(4).unwrap();
        edited.set_anchor((10, 170)).unwrap();
        let before = payload(raw, 0x0907);
        let after = payload(edited.as_bytes(), 0x0907);
        assert_eq!(after[92..], before[92..]);
        assert_eq!(after[48..92], before[48..92]);
    }
    let group = ParameterCalculationGroup::from_expression(1, "v", "240.*v/10", (10, 10)).unwrap();
    assert!((group.value(0.1).unwrap() - 2.4).abs() < 1e-12);
    assert!(group.expression("v").contains("240"));
    assert!(!group.expression("v").contains("240."));
    for invalid in [".+v", "1..+v", "1.rad*v", "v*1.°", "7000(v)"] {
        assert!(
            ParameterCalculationGroup::from_expression(1, "v", invalid, (10, 10)).is_err(),
            "{invalid}"
        );
    }
    for (expression, value) in [("1.", 1.), ("1.+2", 3.)] {
        assert_eq!(
            CalculationGroup::from_expression(expression)
                .unwrap()
                .evaluated_value(),
            Some(value)
        );
    }
    let promoted = ParameterCalculationGroup::parse(include_bytes!(
        "fixtures/numeric_calculation_trailing_decimal_sqrt2_original.bin"
    ))
    .unwrap()
    .with_source_units(&[NumericUnit::Scalar])
    .unwrap();
    assert_eq!(promoted.unit(), NumericUnit::Scalar);
    assert_eq!(promoted.value(2.), Some(3.));
    let bytes =
        include_bytes!("fixtures/numeric_calculation_trailing_decimal_sqrt2_native_verified.gsp");
    let document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[4].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert_eq!(document.numeric_value(5), Some(3.));
    assert_eq!(document.encode().unwrap(), bytes);
}

#[test]
fn native_trailing_decimal_imports_keep_live_sources_and_original_records() {
    for (bytes, expected, suppressed) in [
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_pursuit_native_verified.gsp").as_slice(), 2.4, false),
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_pursuit_formula_native_verified.gsp").as_slice(), 2.4, false),
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube84_native_verified.gsp").as_slice(), 2., true),
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube86_native_verified.gsp").as_slice(), 3., true),
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube_formula_native_verified.gsp").as_slice(), 2., true),
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube_formula_visible_native_verified.gsp").as_slice(), 2., false),
        (include_bytes!("fixtures/numeric_calculation_trailing_decimal_mechanical_native_verified.gsp").as_slice(), std::f64::consts::FRAC_1_SQRT_2, false),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        document.can_save().unwrap();
        assert_eq!(document.encode().unwrap(), bytes);
        assert!((document.numeric_value(5).unwrap() - expected).abs() < 1e-12);
        assert!((document.position(14).unwrap().x - (200. + expected * 37.79527559055118)).abs() < 1e-10);
        let Kind::ParameterCalculation(_, group) = &document.objects[4].kind else { panic!("expected source calculation"); };
        assert_eq!(group.caption_suppressed(), suppressed);
        document.set_parameter_value(2, 1.5).unwrap();
        assert_ne!(document.numeric_value(5), Some(expected));
        assert_eq!(object_group(&document.encode().unwrap(), 5), object_group(bytes, 5));
        document.undo();
        assert_eq!(document.encode().unwrap(), bytes);
    }
}

#[test]
fn trailing_decimal_edits_authored_formulas_and_copies_match_native_writer_goldens() {
    for (case, original, golden, count) in [
        ("pursuit", include_bytes!("fixtures/numeric_calculation_trailing_decimal_pursuit_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_calculation_trailing_decimal_pursuit_edited_authored_copied_native_verified.gsp").as_slice(), 29),
        ("cube84", include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube84_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_calculation_trailing_decimal_cube84_edited_authored_copied_suppression_writer_native_verified.gsp").as_slice(), 28),
        ("mechanical", include_bytes!("fixtures/numeric_calculation_trailing_decimal_mechanical_native_verified.gsp").as_slice(), include_bytes!("fixtures/numeric_calculation_trailing_decimal_mechanical_edited_authored_copied_native_verified.gsp").as_slice(), 30),
    ] {
        let mut document = Document::from_bytes(original, None).unwrap();
        let source_value = match case { "pursuit" => 0.02, "cube84" => 4., _ => 1.5 };
        document.set_parameter_value(2, source_value).unwrap();
        if case == "pursuit" {
            document.set_parameter_value(3, 4.).unwrap();
        }
        document.set_numeric_precision(&[5], 4).unwrap();
        let authored = match case {
            "pursuit" => document.add_multi_calculation(&[2, 3], "240.*v1/v2", (170, 90)).unwrap(),
            "cube84" => document.add_parameter_calculation(2, "-1.+t[4]", (170, 90)).unwrap(),
            _ => document.add_multi_calculation(&[2, 1], "v1*sgn(v2-0.)", (170, 90)).unwrap(),
        };
        let point = document.add(Kind::NumericCoordinatePoint(authored, 1, 11));
        let before_copy = document.objects.len();
        let original_value = document.numeric_value(5);
        let original_position = document.position(14).unwrap();
        select_construction_graph(&mut document, 14);
        document.selected.insert(point);
        let clipboard = document.copy_selected().unwrap();
        document.paste(&clipboard).unwrap();
        let copied_point = document.objects.iter().skip(before_copy)
            .find(|object| matches!(object.kind, Kind::NumericCoordinatePoint(_, _, _))).unwrap().id;
        let Kind::NumericCoordinatePoint(calculation, _, _) = document.objects[copied_point - 1].kind else { panic!("expected copied coordinate point"); };
        let Kind::ParameterCalculation(source, _) = document.objects[calculation - 1].kind else { panic!("expected copied calculation"); };
        let position = document.position(copied_point).unwrap();
        assert!((position.x - original_position.x - 20.).abs() < 1e-10);
        assert!((position.y - original_position.y - 20.).abs() < 1e-10);
        let copied_value = match case { "pursuit" => 0.03, "cube84" => 2.5, _ => 0.75 };
        document.set_parameter_value(source, copied_value).unwrap();
        assert_eq!(document.numeric_value(5), original_value);
        assert_eq!(document.numeric_value(2), Some(source_value));
        assert_ne!(document.numeric_value(calculation), original_value);
        document.can_save().unwrap();
        let encoded = document.encode().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_TRAILING_DECIMAL_FIXTURE").is_some() {
            std::fs::write(format!("/tmp/numeric_trailing_decimal_{case}_clipboard_port.gsp"), &encoded).unwrap();
        }
        assert_eq!(encoded, golden);
        assert_eq!(document.objects.len(), count);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.numeric_value(calculation), document.numeric_value(calculation));
        assert_eq!(reopened.numeric_value(authored), document.numeric_value(authored));
        assert_eq!(reopened.position(copied_point), document.position(copied_point));
    }
}

#[test]
fn source_free_quantity_corpus_controls_retain_native_records() {
    for (bytes, id, value, unit) in [
        (
            include_bytes!("fixtures/source_free_length_calculation_corpus.gsp").as_slice(),
            3,
            3. * 3_f64.sqrt(),
            NumericUnit::Length,
        ),
        (
            include_bytes!("fixtures/source_free_inverse_angle_calculation_corpus.gsp").as_slice(),
            101,
            6_f64.atan().to_degrees(),
            NumericUnit::Degrees,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[id - 1].kind.numeric_unit(), Some(unit));
        assert!((doc.numeric_value(id).unwrap() - value).abs() < 1e-10);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn authentic_twenty_input_calculation_evaluates_all_native_indexes() {
    let bytes = include_bytes!("fixtures/polar_fractional_distance_corpus.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    let Kind::ParameterCalculation(_, group) = &doc.objects[25].kind else {
        panic!("expected original 20-input calculation");
    };
    let sources = group.sources();
    assert_eq!(sources.len(), 20);
    let values = sources
        .iter()
        .map(|id| doc.numeric_value(*id as usize).unwrap())
        .collect::<Vec<_>>();
    // Original formula is ten pairs of products summed from the first input to the last.
    let expected = values.chunks_exact(2).map(|v| v[0] * v[1]).sum::<f64>();
    assert_eq!(doc.numeric_value(26), Some(expected));
    assert!((expected - 2.718281828).abs() < 1e-12);
    assert_eq!(doc.encode().unwrap(), bytes);
    let isolated = ParameterCalculationGroup::parse(&object_group(bytes, 26)).unwrap();
    assert_eq!(isolated.sources(), sources);
    assert_eq!(
        isolated.value_for_sources(&values, &[NumericUnit::Scalar; 20]),
        Some(expected)
    );
}
