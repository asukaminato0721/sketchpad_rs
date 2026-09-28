use super::*;
use crate::document::transform_model::{ElementType, NativeTransformFlavor};

fn sector_landmarks(document: &Document, id: usize) -> [Point; 3] {
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
fn native_second_sector_rotation_replays_ordered_live_matrices() {
    for (bytes, net_angle, visible) in [
        (include_bytes!("../../../../../tests/fixtures/sector_transform81_isolate48_synthetic_native_saved.gsp").as_slice(), 60, true),
        (include_bytes!("../../../../../tests/fixtures/sector_transform81_isolate48_hide48_synthetic_native_saved.gsp").as_slice(), 60, false),
        (include_bytes!("../../../../../tests/fixtures/sector_transform81_isolate48_angle47_60_synthetic_native_saved.gsp").as_slice(), 90, true),
        (include_bytes!("../../../../../tests/fixtures/sector_transform81_isolate48_angle47_60_hide48_synthetic_native_saved.gsp").as_slice(), 90, false),
        (include_bytes!("../../../../../tests/fixtures/sector_transform81_isolate48_angle48_60_synthetic_native_saved.gsp").as_slice(), 90, true),
        (include_bytes!("../../../../../tests/fixtures/sector_transform81_isolate48_angle48_60_hide48_synthetic_native_saved.gsp").as_slice(), 90, false),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        let first = document.objects[46].kind.transform_definition().unwrap();
        let second = document.objects[47].kind.transform_definition().unwrap();
        assert_eq!(first.element(), ElementType::Sector);
        assert_eq!(first.wire_flavor(), NativeTransformFlavor::FixedRotation);
        assert_eq!(first.native_references(), [46, 1]);
        assert_eq!(second.element(), ElementType::Sector);
        assert_eq!(second.wire_flavor(), NativeTransformFlavor::FixedRotation);
        assert_eq!(second.native_references(), [47, 1]);
        assert_eq!(document.is_displayed(48), visible);
        let sequence = document.transform_sequence(48).unwrap();
        assert_eq!(sequence.source, 46);
        assert_eq!(sequence.matrices.len(), 2);
        assert_eq!(document.transform_sequence(57).unwrap().matrices.len(), 11);
        assert!(document.filled_region_points(48).is_some());
        let actual = sector_landmarks(&document, 48);
        let expected = if net_angle == 60 {
            [
                Point { x: 98.314817, y: 229.968682 },
                Point { x: 106.062992, y: 233.178082 },
                Point { x: 90., y: 228.874016 },
            ]
        } else {
            [
                Point { x: 81.685183, y: 229.968682 },
                Point { x: 90., y: 228.874016 },
                Point { x: 73.937008, y: 233.178082 },
            ]
        };
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(actual.distance(expected) < 0.01, "{actual:?} != {expected:?}");
        }
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
