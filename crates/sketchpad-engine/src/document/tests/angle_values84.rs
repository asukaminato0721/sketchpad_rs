use super::*;
use crate::table_records::NumericUnit;

#[test]
fn native_polar_and_vertex_angle_values_keep_opposite_live_orientation() {
    for (bytes, id, expected_kind, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_polar115_original_native_saved.gsp").as_slice(),
            8,
            Kind::PolarAngleValue(1, 4, 2),
            146.33906354141996,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_polar115_moved_only_synthetic_native_saved.gsp").as_slice(),
            8,
            Kind::PolarAngleValue(1, 4, 2),
            207.26821914,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_vertex116_original_native_saved.gsp").as_slice(),
            27,
            Kind::VertexAngleValue(2, 3, 1),
            34.02077936730652,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_values84_vertex116_show_moved_synthetic_native_saved.gsp").as_slice(),
            27,
            Kind::VertexAngleValue(2, 3, 1),
            320.83839940,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[id - 1].kind, expected_kind);
        assert_eq!(doc.objects[id - 1].kind.numeric_unit(), Some(NumericUnit::Degrees));
        assert!(!doc.objects[id - 1].kind.is_measurement());
        assert!((doc.numeric_value(id).unwrap() - expected).abs() < 1e-6);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn angle_marker_arrow_flag_preserves_the_directed_value() {
    for (bytes, id) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/angle_values84_polar115_original_native_saved.gsp"
            )
            .as_slice(),
            8,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/angle_values84_vertex116_original_native_saved.gsp"
            )
            .as_slice(),
            27,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let header = doc.objects[id - 1].header_offset.unwrap();
        let marker = records(bytes)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x090e && record.start > header)
            .unwrap();
        let mut changed = bytes.to_vec();
        changed[marker.data.start + 2] = 0;
        let guarded = Document::from_bytes(&changed, None).unwrap();
        assert_eq!(guarded.objects[id - 1].kind, doc.objects[id - 1].kind);
        assert_eq!(guarded.numeric_value(id), doc.numeric_value(id));
        assert_eq!(doc.angle_marker_geometry(id).unwrap().arrows.len(), 1);
        assert!(guarded.angle_marker_geometry(id).unwrap().arrows.is_empty());
        assert_eq!(guarded.encode().unwrap(), changed);
    }
}
