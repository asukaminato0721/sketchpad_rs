use super::*;
use crate::document::transform_model::{ElementType, NativeTransformFlavor};

fn endpoints(document: &Document, id: usize) -> [Point; 3] {
    let (center, radius, start, sweep, clockwise_y) = document.sector_arc_geometry(id).unwrap();
    let at = |angle: f64| Point {
        x: center.x + radius * angle.cos(),
        y: center.y
            + if clockwise_y {
                -radius * angle.sin()
            } else {
                radius * angle.sin()
            },
    };
    [center, at(start), at(start + sweep)]
}

#[test]
fn native_reflected_and_chained_sector_keep_analytic_geometry() {
    for (bytes, state) in [
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_original_native_saved.gsp").as_slice(), "original"),
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_hide47_synthetic_native_saved.gsp").as_slice(), "hide47"),
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_hide48_synthetic_native_saved.gsp").as_slice(), "hide48"),
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_mirror25_angle60_synthetic_native_saved.gsp").as_slice(), "mirror60"),
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_mirror25_angle60_hide47_synthetic_native_saved.gsp").as_slice(), "mirror60_hide47"),
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_angle48_90_synthetic_native_saved.gsp").as_slice(), "angle90"),
        (include_bytes!("../../../../../tests/fixtures/sector_transform80_angle48_90_hide48_synthetic_native_saved.gsp").as_slice(), "angle90_hide48"),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        let reflected = document.objects[46].kind.transform_definition().unwrap();
        let chained = document.objects[47].kind.transform_definition().unwrap();
        assert_eq!(reflected.element(), ElementType::Sector);
        assert_eq!(reflected.wire_flavor(), NativeTransformFlavor::Reflection);
        assert_eq!(reflected.native_references(), [45, 26]);
        assert_eq!(chained.element(), ElementType::Sector);
        assert_eq!(chained.wire_flavor(), NativeTransformFlavor::FixedRotation);
        assert_eq!(chained.native_references(), [47, 16]);
        assert_eq!(document.is_displayed(47), state != "hide47" && state != "mirror60_hide47");
        assert_eq!(document.is_displayed(48), state != "hide48" && state != "angle90_hide48");
        let sequence = document.transform_sequence(48).unwrap();
        assert_eq!(sequence.source, 45);
        assert_eq!(sequence.element, ElementType::Sector);
        assert_eq!(sequence.matrices.len(), 2);
        assert!(document.filled_region_points(47).is_some());
        assert!(document.filled_region_points(48).is_some());
        let [c47, s47, e47] = endpoints(&document, 47);
        let [c48, s48, e48] = endpoints(&document, 48);
        if state.starts_with("mirror60") {
            assert!(c47.distance(Point { x: 194.589, y: 365.5 }) < 0.1, "{c47:?}");
            assert!(s47.distance(Point { x: 310.863, y: 213.97 }) < 0.1, "{s47:?}");
            assert!(e47.distance(Point { x: 360., y: 270. }) < 0.1, "{e47:?}");
        } else {
            assert!(c47.distance(Point { x: 360., y: 461. }) < 1e-6, "{c47:?}");
            assert!(s47.distance(Point { x: 286.9075, y: 284.539 }) < 0.01, "{s47:?}");
            assert!(e47.distance(Point { x: 360., y: 270. }) < 0.01, "{e47:?}");
            if state.starts_with("angle90") {
                assert!(c48.distance(Point { x: 360., y: 461. }) < 1e-6);
                assert!(s48.distance(Point { x: 183.539, y: 534.0925 }) < 0.01, "{s48:?}");
                assert!(e48.distance(Point { x: 169., y: 461. }) < 0.01, "{e48:?}");
            } else {
                assert!(c48.distance(Point { x: 360., y: 461. }) < 1e-6);
                assert!(s48.distance(Point { x: 433.0925, y: 637.461 }) < 0.01, "{s48:?}");
                assert!(e48.distance(Point { x: 360., y: 652. }) < 0.01, "{e48:?}");
            }
        }
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
