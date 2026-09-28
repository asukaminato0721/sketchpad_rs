use super::*;
use crate::document::transform_model::{ElementType, NativeTransformFlavor};

fn arc_endpoints(
    (center, radius, start, sweep, clockwise_y): (Point, f64, f64, f64, bool),
) -> [Point; 3] {
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
fn native_fixed_rotation_of_sector_remains_analytic_and_live() {
    for (bytes, visible, expected_center_x, expected_start, expected_end) in [
        (
            include_bytes!("../../../../../tests/fixtures/sector_transform79_original_native_saved.gsp").as_slice(),
            true,
            360.,
            Point { x: 286.9075, y: 637.461 },
            Point { x: 360., y: 652. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/sector_transform79_hide46_synthetic_native_saved.gsp").as_slice(),
            false,
            360.,
            Point { x: 286.9075, y: 637.461 },
            Point { x: 360., y: 652. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/sector_transform79_angle46_90_synthetic_native_saved.gsp").as_slice(),
            true,
            360.,
            Point { x: 183.539, y: 387.9075 },
            Point { x: 169., y: 461. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/sector_transform79_angle46_90_hide46_synthetic_native_saved.gsp").as_slice(),
            false,
            360.,
            Point { x: 183.539, y: 387.9075 },
            Point { x: 169., y: 461. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/sector_transform79_center16_edited_synthetic_native_saved.gsp").as_slice(),
            true,
            390.,
            Point { x: 316.9075, y: 637.461 },
            Point { x: 390., y: 652. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/sector_transform79_center16_edited_hide46_synthetic_native_saved.gsp").as_slice(),
            false,
            390.,
            Point { x: 316.9075, y: 637.461 },
            Point { x: 390., y: 652. },
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        let definition = document.objects[45].kind.transform_definition().unwrap();
        assert_eq!(definition.element(), ElementType::Sector);
        assert_eq!(definition.wire_flavor(), NativeTransformFlavor::FixedRotation);
        assert_eq!(definition.native_references(), [45, 16]);
        assert_eq!(document.is_displayed(46), visible);
        assert!(document.objects[45].kind.is_filled_region());
        let geometry = document.sector_arc_geometry(46).unwrap();
        let [center, start, end] = arc_endpoints(geometry);
        assert!(
            center.distance(Point { x: expected_center_x, y: 461. }) < 1e-6,
            "center={center:?} expected_end={expected_end:?}"
        );
        assert!(start.distance(expected_start) < 0.01, "{start:?} vs {expected_start:?}");
        assert!(end.distance(expected_end) < 0.01, "{end:?} vs {expected_end:?}");
        assert_eq!(document.transform_sequence(46).unwrap().source, 45);
        assert!(document.filled_region_points(46).is_some());
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
