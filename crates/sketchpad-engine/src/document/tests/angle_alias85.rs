use super::*;

#[test]
fn native_angle_aliases_keep_names_units_and_live_values() {
    for (bytes, id, parent, name, value) in [
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_polar115_original_native_saved.gsp").as_slice(),
            9, 8, "∠ABC", 146.33906354141996,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_polar115_moved_only_synthetic_native_saved.gsp").as_slice(),
            9, 8, "∠ABC", 207.26821914,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_vertex116_original_native_saved.gsp").as_slice(),
            28, 27, "D", 34.02077936730652,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_vertex116_show_moved_synthetic_native_saved.gsp").as_slice(),
            28, 27, "D", 320.83839940,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[id - 1].kind, Kind::NumericAlias { source: parent, unit: NumericUnit::Degrees });
        assert_eq!(doc.objects[id - 1].construction_dependencies(), vec![parent]);
        assert!((doc.numeric_value(id).unwrap() - value).abs() < 1e-6);
        assert!(doc.measurement_text(id).unwrap().starts_with(&format!("{name} = ")));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn angle_alias_rejects_unproved_measurement_packet() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/angle_values84_polar115_original_native_saved.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    let header = doc.objects[8].header_offset.unwrap();
    let packet = records(bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x0903 && record.start > header)
        .unwrap();
    let mut changed = bytes.to_vec();
    changed[packet.data.start] ^= 1;
    let guarded = Document::from_bytes(&changed, None).unwrap();
    assert_eq!(guarded.objects[8].kind, Kind::Unsupported(120));
    assert_eq!(guarded.encode().unwrap(), changed);
}

#[test]
fn native_angle_alias_precision_field_formats_live_parent() {
    for (bytes, id, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/angle_alias85_polar115_precision3_synthetic_native_saved.gsp").as_slice(),
            9, "∠ABC = 146.339°",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_alias85_vertex116_precision3_synthetic_native_saved.gsp").as_slice(),
            28, "D = 34.021°",
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[id - 1].measurement_precision, Some(3));
        assert_eq!(doc.measurement_text(id).as_deref(), Some(expected));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
