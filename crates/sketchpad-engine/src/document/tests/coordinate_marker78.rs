use super::*;
use crate::coordinate_measure_records::CoordinateMeasureKind;

#[test]
fn native_marked_coordinate_abscissa_follows_point_and_preserves_empty_record() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/coordinate_measurements78_liss_original_native_saved.gsp")
                .as_slice(),
            -1.47520832,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/coordinate_measurements78_liss_point16_tedited_synthetic_native_saved.gsp")
                .as_slice(),
            -0.997795,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(document.objects[17].kind,
            Kind::CoordinateMeasurement { kind: CoordinateMeasureKind::Abscissa, first: 16, frame: 17, .. }));
        assert!((document.numeric_value(18).unwrap() - expected).abs() < 1e-5);
        assert_eq!(document.encode().unwrap(), bytes);
    }
}

#[test]
fn native_paired_abscissa_and_ordinate_keep_their_original_records() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/coordinate_measurements78_pi_original_input.gsp"
    );
    let document = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(
        document.objects[170].kind,
        Kind::CoordinateMeasurement {
            kind: CoordinateMeasureKind::Ordinate,
            first: 106,
            frame: 169,
            ..
        }
    ));
    assert!(matches!(
        document.objects[183].kind,
        Kind::CoordinateMeasurement {
            kind: CoordinateMeasureKind::Abscissa,
            first: 106,
            frame: 169,
            ..
        }
    ));
    assert!(document.numeric_value(184).unwrap().abs() < 1e-10);
    assert!((document.numeric_value(171).unwrap() - 0.2000012).abs() < 1e-6);
    assert_eq!(document.encode().unwrap(), bytes);
}
