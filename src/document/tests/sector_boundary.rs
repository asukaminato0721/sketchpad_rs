use super::*;
const SECTOR_UNIT: f64 = 37.79527559055118;
fn native(state: &str) -> Vec<u8> {
    fs::read(format!(
        "tests/fixtures/sector_boundary59_{state}_native_saved.gsp"
    ))
    .unwrap()
}
fn close(a: Option<Point>, b: Point) {
    assert!(a.unwrap().distance(b) < 1e-9, "{a:?} vs {b:?}");
}
#[test]
fn native_sector_boundary_has_closed_perimeter_points_and_distinct_analytic_measurements() {
    for (state, center, radius, major, mid) in [
        (
            "center_quarter",
            Point { x: 240., y: 280. },
            120.,
            false,
            Point {
                x: 324.8528137423857,
                y: 195.1471862576143,
            },
        ),
        (
            "center_reverse",
            Point { x: 240., y: 280. },
            120.,
            true,
            Point {
                x: 155.1471862576143,
                y: 364.8528137423857,
            },
        ),
        (
            "threepoint_minor",
            Point { x: 240., y: 280. },
            120.,
            false,
            Point {
                x: 324.8528137423857,
                y: 195.1471862576143,
            },
        ),
        (
            "threepoint_major",
            Point { x: 240., y: 280. },
            120.,
            true,
            Point {
                x: 155.1471862576143,
                y: 364.8528137423857,
            },
        ),
        (
            "reflected",
            Point { x: 560., y: 280. },
            120.,
            false,
            Point {
                x: 475.1471862576143,
                y: 195.1471862576143,
            },
        ),
        (
            "edited",
            Point { x: 260., y: 280. },
            140.,
            false,
            Point {
                x: 358.99494936611666,
                y: 181.00505063388334,
            },
        ),
    ] {
        let bytes = native(state);
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert_eq!(
            d.objects[9].kind,
            Kind::SectorBoundary(if state == "reflected" { 9 } else { 5 })
        );
        close(d.position(11), center);
        close(d.position(17), center);
        close(d.position(14), mid);
        close(
            d.position(22),
            Point {
                x: (100. + mid.x) / 2.,
                y: (450. + mid.y) / 2.,
            },
        );
        for id in [18, 19] {
            assert!(matches!(d.objects[id-1].kind,Kind::PointOnSector(10,t) if t.is_finite()));
            assert_eq!(d.position(id), None);
        }
        let sweep = std::f64::consts::FRAC_PI_2 * if major { 3. } else { 1. };
        assert!((d.numeric_value(25).unwrap() - radius * (2. + sweep) / SECTOR_UNIT).abs() < 1e-10);
        assert!((d.numeric_value(26).unwrap() - radius * sweep / SECTOR_UNIT).abs() < 1e-10);
        assert!(
            (d.numeric_value(28).unwrap() - 0.5 * radius * radius * sweep / SECTOR_UNIT.powi(2))
                .abs()
                < 1e-10
        );
        assert!(
            (d.point_distance_cm(23).unwrap()
                - Point { x: 100., y: 450. }.distance(mid) / SECTOR_UNIT)
                .abs()
                < 1e-10
        );
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
    }
    let d = Document::from_bytes(&native("center_quarter"), None).unwrap();
    close(
        d.position(12),
        Point {
            x: 282.8495559215388,
            y: 280.,
        },
    );
    close(
        d.position(13),
        Point {
            x: 347.1238898038469,
            y: 280.,
        },
    );
    close(
        d.position(15),
        Point {
            x: 240.,
            y: 172.8761101961531,
        },
    );
    close(
        d.position(16),
        Point {
            x: 240.,
            y: 237.1504440784612,
        },
    );
}
#[test]
fn sector_projection_moves_across_radial_and_arc_parts_without_polyline_error() {
    let bytes = native("center_quarter");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let perimeter = 120. * (2. + std::f64::consts::FRAC_PI_2);
    for (query, expected, t) in [
        (
            Point { x: 300., y: 290. },
            Point { x: 300., y: 280. },
            60. / perimeter,
        ),
        (
            Point { x: 380., y: 140. },
            Point {
                x: 324.8528137423857,
                y: 195.1471862576143,
            },
            0.5,
        ),
        (
            Point { x: 230., y: 220. },
            Point { x: 240., y: 220. },
            1. - 60. / perimeter,
        ),
    ] {
        let projected = d.sector_boundary_projection(10, query).unwrap();
        assert!((projected - t).abs() < 1e-12);
        close(d.point_on_sector(10, projected), expected);
        assert!(d.move_point(14, query));
        close(d.position(14), expected);
        assert!(
            matches!(d.objects[13].kind,Kind::PointOnSector(10,actual) if (actual-t).abs()<1e-12)
        );
        d.verify_roundtrip().unwrap();
    }
    assert!(d.sector_contains(10, Point { x: 260., y: 260. }));
    assert!(!d.sector_contains(10, Point { x: 220., y: 260. }));
    assert!(!d.sector_contains(10, Point { x: 400., y: 280. }));
    let points = d.filled_region_points(10).unwrap();
    assert_ne!(points.first(), points.last());
    assert_eq!(points[0], Point { x: 240., y: 280. });
    assert!((d.sector_area_pixels2(10).unwrap() - 3600. * std::f64::consts::PI).abs() < 1e-9);
}
#[test]
fn sector_undefined_and_linear_sources_preserve_roles_and_recover_live() {
    for state in ["linear", "undefined"] {
        let bytes = native(state);
        let mut d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view);
        assert!(matches!(d.objects[9].kind, Kind::SectorBoundary(5)));
        assert!(d.undefined_sector(10));
        assert_eq!(d.sector_boundary_points(10, 16), None);
        for id in 11..=19 {
            assert!(matches!(d.objects[id - 1].kind, Kind::PointOnSector(10, _)));
            assert_eq!(d.position(id), None);
        }
        for id in [23, 25, 26, 28] {
            assert_eq!(d.numeric_value(id), None);
        }
        assert_eq!(d.position(20), Some(Point { x: 100., y: 450. }));
        if state == "linear" {
            assert!((d.numeric_value(24).unwrap() - 120. / SECTOR_UNIT).abs() < 1e-12);
        } else {
            assert_eq!(d.numeric_value(24), None);
        }
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        d.checkpoint();
        if state == "linear" {
            assert!(d.move_point(3, Point { x: 240., y: 160. }));
            assert!(d.move_point(
                4,
                Point {
                    x: 324.8528137423857,
                    y: 195.1471862576143
                }
            ));
        } else {
            assert!(d.move_point(2, Point { x: 360., y: 280. }));
        }
        assert!(d.sector_boundary_points(10, 8).is_some());
        assert!(d.position(14).is_some());
        d.verify_roundtrip().unwrap();
        d.undo();
        assert_eq!(d.encode().unwrap(), bytes);
        assert!(d.undefined_sector(10));
        d.redo();
        assert!(d.position(14).is_some());
        d.verify_roundtrip().unwrap();
    }
}
#[test]
fn sector_authoring_scoped_freezing_and_page_local_writing_keep_live_arc_dependencies() {
    let base = Document::from_bytes(&native("center_quarter"), None).unwrap();
    let original = base.position(14).unwrap();
    let mut live = base.clone();
    live.freeze_scoped_evaluation_inputs(&base, &[1, 3])
        .unwrap();
    live.set_evaluation_point(2, Point { x: 400., y: 280. })
        .unwrap();
    close(
        live.position(14),
        Point {
            x: 353.1370849898476,
            y: 166.8629150101524,
        },
    );
    let mut frozen = live.clone();
    frozen
        .freeze_scoped_evaluation_inputs(&live, &[10])
        .unwrap();
    frozen
        .set_evaluation_point(2, Point { x: 480., y: 280. })
        .unwrap();
    close(frozen.position(14), live.position(14).unwrap());
    assert!(frozen.position(14).unwrap().distance(original) > 20.);
    for multi in [false, true] {
        let mut d = if multi {
            Document::from_bytes(
                include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp"),
                None,
            )
            .unwrap()
        } else {
            base.clone()
        };
        let other_sector = if multi {
            d.active_page = 0;
            let c = d.add(Kind::Free(Point { x: 100., y: 100. }));
            let a = d.add(Kind::Free(Point { x: 160., y: 100. }));
            let b = d.add(Kind::Free(Point { x: 100., y: 40. }));
            let arc = d.add(Kind::CenterArc(c, a, b));
            let sector = d.add_sector_boundary(arc).unwrap();
            d.active_page = 1;
            Some(sector)
        } else {
            None
        };
        let page = d.active_page;
        let c = d.add(Kind::Free(Point { x: 200., y: 200. }));
        let a = d.add(Kind::Free(Point { x: 260., y: 200. }));
        let b = d.add(Kind::Free(Point { x: 200., y: 140. }));
        let arc = d.add(Kind::CenterArc(c, a, b));
        let region = d.add_sector_boundary(arc).unwrap();
        let point = d.add_point_on_sector(region, 0.5).unwrap();
        let outside = d.add_point_on_sector(region, 1.1).unwrap();
        assert_eq!(d.position(outside), None);
        assert!(d.add_point_on_sector(region, f64::NAN).is_err());
        assert!(d.add_sector_boundary(c).is_err());
        if let Some(other_sector) = other_sector {
            assert!(d.add_point_on_sector(other_sector, 0.5).is_err());
        }
        let saved = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert!(saved
            .objects
            .iter()
            .all(|o| !matches!(o.kind, Kind::Unsupported(_))));
        assert_eq!(saved.objects[region - 1].kind, Kind::SectorBoundary(arc));
        assert_eq!(saved.objects[point - 1].page, page);
        close(
            saved.position(point),
            Point {
                x: 242.42640687119285,
                y: 157.57359312880715,
            },
        );
        d.verify_roundtrip().unwrap();
        let before = d.encode().unwrap();
        // Reopen the authored pages before applying the existing-page deletion policy.
        d = Document::from_bytes(&before, None).unwrap();
        d.active_page = page;
        d.selected = [region].into_iter().collect();
        let deleted = d.clear_selected().unwrap();
        assert!(deleted.contains(&point));
        assert!(deleted.contains(&outside));
        d.verify_roundtrip().unwrap();
        d.undo();
        assert_eq!(d.encode().unwrap(), before);
    }
}
#[test]
fn malformed_sector_and_point_records_stay_opaque_without_source_role_widening() {
    let bytes = native("center_quarter");
    let d = Document::from_bytes(&bytes, None).unwrap();
    let sector = d.objects[9].header_offset.unwrap();
    let point = d.objects[13].header_offset.unwrap();
    let c = d.objects[13].constraint_offset.unwrap();
    let mut mutations = vec![];
    for subtype in [67u16, 69] {
        let mut b = bytes.clone();
        b[sector + 8..sector + 10].copy_from_slice(&subtype.to_le_bytes());
        mutations.push((b, 10, 82));
    }
    let rs = records(&bytes).unwrap();
    let i = rs.iter().position(|r| r.data.start == sector).unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap().data.start;
    for host in [1u32, 10, 20] {
        let mut b = bytes.clone();
        b[refs + 4..refs + 8].copy_from_slice(&host.to_le_bytes());
        mutations.push((b, 10, 82));
    }
    let mut nan = bytes.clone();
    nan[c + 4..c + 12].copy_from_slice(&f64::NAN.to_le_bytes());
    mutations.push((nan, 14, 15));
    let mut wrong = bytes.clone();
    wrong[point + 6..point + 8].copy_from_slice(&2u16.to_le_bytes());
    mutations.push((wrong, 14, 15));
    let rec = rs.iter().find(|r| r.data.start == c).unwrap();
    let extra = bytes[rec.start..rec.data.end].to_vec();
    let mut duplicate = bytes.clone();
    duplicate.splice(rec.start..rec.start, extra);
    mutations.push((duplicate, 14, 15));
    for (b, id, code) in mutations {
        let parsed = Document::from_bytes(&b, None).unwrap();
        assert_eq!(parsed.objects[id - 1].kind, Kind::Unsupported(code));
        assert_eq!(parsed.encode().unwrap(), b);
    }
}
