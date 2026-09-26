use sketchpad_rs::document::{Document, Kind, records};
use sketchpad_rs::table_records::{
    CalculationGroup, NumericUnit, ParameterCalculationGroup, ParameterGroup,
};

const COOD_TRACE: &[u8] = include_bytes!("fixtures/numeric_cood_trace_native.gsp");
const CIRCLE_FORMATION: &[u8] = include_bytes!("fixtures/numeric_circle_formation_native.gsp");

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
    let mut invalid_units = raw.clone();
    invalid_units[16..18].copy_from_slice(&49_u16.to_le_bytes());
    assert!(
        ParameterCalculationGroup::parse(&invalid_units)
            .unwrap()
            .with_source_units(&[NumericUnit::Scalar])
            .is_err()
    );
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
    document.selected.insert(9);
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
    document.selected.insert(13);
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
    // A scalar variable remains a separate quantity, unlike a bare threshold.
    assert!(
        ParameterCalculationGroup::from_sources(
            &[1, 2],
            "v1-v2",
            (10, 30),
            &[NumericUnit::Length, NumericUnit::Scalar]
        )
        .is_err()
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
        Some(NumericUnit::Length)
    );
    assert_eq!(
        difference.numeric_value(15),
        Some(difference.numeric_value(6).unwrap() - 2.)
    );
}
