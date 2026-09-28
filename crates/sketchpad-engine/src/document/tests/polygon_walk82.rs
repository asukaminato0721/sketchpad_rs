use super::*;

fn near(actual: Point, expected: Point) {
    assert!(
        actual.distance(expected) < 1e-8,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn imported_eight_step_polygon_point_uses_stored_edges_and_live_vertices() {
    let fraction = 0.18536499036601023;
    for (bytes, t, start, visible) in [
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_original_synthetic_native_saved.gsp").as_slice(), fraction, Point { x: 548., y: 413. }, true),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_hide10_synthetic_native_saved.gsp").as_slice(), fraction, Point { x: 548., y: 413. }, false),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_half_synthetic_native_saved.gsp").as_slice(), 0.5, Point { x: 548., y: 413. }, true),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_half_hide10_synthetic_native_saved.gsp").as_slice(), 0.5, Point { x: 548., y: 413. }, false),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_vertex4_608_393_synthetic_native_saved.gsp").as_slice(), fraction, Point { x: 608., y: 393. }, true),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_vertex4_608_393_hide10_synthetic_native_saved.gsp").as_slice(), fraction, Point { x: 608., y: 393. }, false),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&doc.objects[8].kind, Kind::Polygon(vertices) if vertices.len() == 8));
        assert_eq!(doc.objects[9].kind, Kind::PointOnPolygon(9, 3, t));
        assert_eq!(doc.is_displayed(10), visible);
        near(doc.position(10).unwrap(), Point {
            x: start.x + (598. - start.x) * t,
            y: start.y + (240. - start.y) * t,
        });
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn imported_repeated_polygon_point_uses_twelve_stored_slots() {
    for (bytes, t, visible) in [
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_repeat12_original_native_saved.gsp").as_slice(), 0.32244704910574673, true),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_repeat12_hide187_synthetic_native_saved.gsp").as_slice(), 0.32244704910574673, false),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_repeat12_threequarters_synthetic_native_saved.gsp").as_slice(), 0.75, true),
        (include_bytes!("../../../../../tests/fixtures/polygon_walk82_repeat12_threequarters_hide187_synthetic_native_saved.gsp").as_slice(), 0.75, false),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&doc.objects[185].kind, Kind::Polygon(vertices) if vertices.len() == 12));
        assert_eq!(doc.objects[186].kind, Kind::PointOnPolygon(186, 2, t));
        assert_eq!(doc.is_displayed(187), visible);
        near(doc.position(187).unwrap(), Point {
            x: 211.461942257218 + (258.923884514436 - 211.461942257218) * t,
            y: 336.538057742782,
        });
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn imported_high_word_one_polygon_uses_large_stored_edge_index() {
    for (bytes, t, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/polygon_walk82_long266_isolate468_shift_hidden_synthetic_native_saved.gsp").as_slice(),
            7. / 9.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/polygon_walk82_long266_isolate468_shift_show_synthetic_native_saved.gsp").as_slice(),
            7. / 9.,
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/polygon_walk82_long266_isolate468_shift_half_hidden_synthetic_native_saved.gsp").as_slice(),
            0.5,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/polygon_walk82_long266_isolate468_shift_half_show_synthetic_native_saved.gsp").as_slice(),
            0.5,
            true,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&doc.objects[466].kind, Kind::Polygon(vertices) if vertices.len() == 266));
        assert_eq!(doc.objects[467].kind, Kind::PointOnPolygon(467, 70, t));
        assert_eq!(doc.is_displayed(468), visible);
        near(
            doc.position(468).unwrap(),
            Point {
                x: 900. - 300. * t,
                y: 350.,
            },
        );
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn imported_polygon_point_rejects_unproved_marker_layout() {
    let original = include_bytes!("../../../../../tests/fixtures/polygon_walk82_small_isolate10_original_synthetic_native_saved.gsp");
    let doc = Document::from_bytes(original, None).unwrap();
    let constraint = doc.objects[9].constraint_offset.unwrap();
    let end = records(original)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x07d7 && record.start > constraint)
        .unwrap()
        .start;
    let mut marked = original.to_vec();
    marked.splice(end..end, [0, 0, 0, 0, 0x2e, 0x23, 0, 0]);
    let guarded = Document::from_bytes(&marked, None).unwrap();
    assert_eq!(guarded.objects[9].kind, Kind::Unsupported(15));
    assert_eq!(guarded.encode().unwrap(), marked);
}
