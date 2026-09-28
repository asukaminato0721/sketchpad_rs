use super::*;
fn native(state: &str) -> Vec<u8> {
    fs::read(format!(
        "../../tests/fixtures/circle_interior60_{state}_native_saved.gsp"
    ))
    .unwrap()
}
fn interiors(d: &Document) -> Vec<usize> {
    d.objects
        .iter()
        .filter(|o| o.kind.is_circle_interior())
        .map(|o| o.id)
        .collect()
}
fn disk(d: &Document, id: usize, center: Point, radius: f64) {
    let (c, r) = d.circle_interior_geometry(id).unwrap();
    assert!(c.distance(center) < 1e-9, "{c:?} vs {center:?}");
    assert!((r - radius).abs() < 1e-9, "{r} vs {radius}");
}
#[test]
fn native_modern_circle_interiors_keep_independent_geometry_style_visibility_and_order() {
    for state in [
        "modern",
        "by_segment",
        "transformed_circle",
        "two_fill",
        "two_fill_reverse",
        "hidden",
        "edited",
        "zero",
    ] {
        let bytes = native(state);
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
        d.verify_roundtrip().unwrap();
        let ids = interiors(&d);
        assert_eq!(ids.len(), if state.starts_with("two_fill") { 2 } else { 1 });
        for &id in &ids {
            let Kind::CircleInterior(source) = d.objects[id - 1].kind else {
                unreachable!()
            };
            assert!(
                !d.objects[source - 1].visible,
                "source must remain independently hidden"
            );
            assert!(!d.objects[id - 1].kind.is_circle());
            assert!(!d.objects[id - 1].kind.is_polygon());
            if state == "zero" {
                assert_eq!(d.circle_interior_geometry(id), None);
                assert_eq!(d.filled_region_points(id), None);
                continue;
            }
            let c = if state == "transformed_circle" {
                Point { x: 310., y: 280. }
            } else {
                Point { x: 250., y: 260. }
            };
            disk(&d, id, c, if state == "edited" { 120. } else { 80. });
            assert!(d.circle_interior_contains(id, c));
            assert!(!d.circle_interior_contains(
                id,
                Point {
                    x: c.x + 121.,
                    y: c.y
                }
            ));
            assert!(!d.circle_interior_contains(
                id,
                Point {
                    x: f64::NAN,
                    y: c.y
                }
            ));
            let outline = d.filled_region_points(id).unwrap();
            assert_eq!(outline.len(), 192);
            assert_ne!(outline.first(), outline.last());
            assert_eq!(d.objects[id - 1].visible, state != "hidden");
        }
        let VisualStyle::Polygon { color, opacity } = d.objects[ids[0] - 1].visual_style.unwrap()
        else {
            panic!("{state}")
        };
        assert_eq!(
            color,
            RgbColor {
                red: 0,
                green: 128,
                blue: 0
            }
        );
        assert_eq!(
            opacity,
            if state.starts_with("two_fill") {
                65535
            } else {
                32767
            }
        );
        if ids.len() == 2 {
            let VisualStyle::Polygon { color, opacity } =
                d.objects[ids[1] - 1].visual_style.unwrap()
            else {
                panic!()
            };
            assert_eq!(
                color,
                RgbColor {
                    red: 255,
                    green: 0,
                    blue: 255
                }
            );
            assert_eq!(opacity, 32767);
            assert_eq!(
                (
                    d.objects[ids[0] - 1].interior_order,
                    d.objects[ids[1] - 1].interior_order
                ),
                if state == "two_fill_reverse" {
                    (Some(3), Some(0))
                } else {
                    (Some(0), Some(3))
                }
            );
        }
    }
}
#[test]
fn circle_interior_live_source_edits_zero_recovery_and_scoped_snapshots_keep_identity() {
    let bytes = native("modern");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let id = interiors(&d)[0];
    let circle_style = d.objects[2].visual_style;
    d.checkpoint();
    assert!(d.move_point(2, Point { x: 410., y: 260. }));
    disk(&d, id, Point { x: 250., y: 260. }, 160.);
    assert_eq!(d.objects[2].visual_style, circle_style);
    d.verify_roundtrip().unwrap();
    let saved = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    disk(&saved, id, Point { x: 250., y: 260. }, 160.);
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    d.redo();
    disk(&d, id, Point { x: 250., y: 260. }, 160.);
    d.checkpoint();
    assert!(d.move_point(2, Point { x: 250., y: 260. }));
    assert_eq!(d.circle_interior_geometry(id), None);
    d.verify_roundtrip().unwrap();
    d.undo();
    disk(&d, id, Point { x: 250., y: 260. }, 160.);
    let base = Document::from_bytes(&bytes, None).unwrap();
    let mut caller = base.clone();
    caller.freeze_scoped_evaluation_inputs(&base, &[]).unwrap();
    caller
        .set_evaluation_point(2, Point { x: 410., y: 260. })
        .unwrap();
    disk(&caller, id, Point { x: 250., y: 260. }, 160.);
    for boundary in [id, 3] {
        let mut frozen = caller.clone();
        frozen
            .freeze_scoped_evaluation_inputs(&caller, &[boundary])
            .unwrap();
        frozen
            .set_evaluation_point(2, Point { x: 490., y: 260. })
            .unwrap();
        disk(&frozen, id, Point { x: 250., y: 260. }, 160.);
    }
    let mut live = caller.clone();
    live.freeze_scoped_evaluation_inputs(&caller, &[]).unwrap();
    live.set_evaluation_point(2, Point { x: 490., y: 260. })
        .unwrap();
    disk(&live, id, Point { x: 250., y: 260. }, 240.);
}
#[test]
fn authored_circle_interiors_preserve_page_local_sources_delete_and_undo() {
    for multi in [false, true] {
        let mut d = if multi {
            Document::from_bytes(
                include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
                None,
            )
            .unwrap()
        } else {
            Document::new()
        };
        let first = if multi {
            d.active_page = 0;
            let a = d.add(Kind::Free(Point { x: 100., y: 100. }));
            let b = d.add(Kind::Free(Point { x: 150., y: 100. }));
            let c = d.add(Kind::Circle(a, b));
            d.add_circle_interior(c).unwrap();
            d.active_page = 1;
            Some(c)
        } else {
            None
        };
        let page = d.active_page;
        let a = d.add(Kind::Free(Point { x: 200., y: 200. }));
        let b = d.add(Kind::Free(Point { x: 270., y: 200. }));
        let c = d.add(Kind::Circle(a, b));
        let fill = d.add_circle_interior(c).unwrap();
        assert!(d.add_circle_interior(a).is_err());
        if let Some(first) = first {
            assert!(d.add_circle_interior(first).is_err());
        }
        let before = d.encode().unwrap();
        let mut saved = Document::from_bytes(&before, None).unwrap();
        assert!(saved
            .objects
            .iter()
            .all(|o| !matches!(o.kind, Kind::Unsupported(_))));
        assert_eq!(saved.objects[fill - 1].kind, Kind::CircleInterior(c));
        assert_eq!(saved.objects[fill - 1].page, page);
        disk(&saved, fill, Point { x: 200., y: 200. }, 70.);
        saved.verify_roundtrip().unwrap();
        saved.active_page = page;
        saved.selected = [c].into_iter().collect();
        assert!(saved.clear_selected().unwrap().contains(&fill));
        saved.verify_roundtrip().unwrap();
        saved.undo();
        assert_eq!(saved.encode().unwrap(), before);
    }
}
#[test]
fn unproved_legacy_and_malformed_modern_circle_interiors_remain_byte_exact_opaque() {
    let legacy = fs::read("../../tests/fixtures/circle_interior60_legacy_input.gsp").unwrap();
    let ld = Document::from_bytes(&legacy, None).unwrap();
    assert!(ld.objects.iter().any(|o| o.kind == Kind::Unsupported(10)));
    assert_eq!(ld.encode().unwrap(), legacy);
    let bytes = native("modern");
    let d = Document::from_bytes(&bytes, None).unwrap();
    let id = interiors(&d)[0];
    let at = d.objects[id - 1].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs.iter().position(|r| r.data.start == at).unwrap();
    let end = (i..rs.len()).find(|&j| rs[j].kind == 0x7d7).unwrap();
    let refs = rs[i..=end].iter().find(|r| r.kind == 0x7d2).unwrap();
    let mut cases = vec![];
    for host in [1u32, id as u32, (id + 1) as u32] {
        let mut b = bytes.clone();
        b[refs.data.start + 4..refs.data.start + 8].copy_from_slice(&host.to_le_bytes());
        cases.push(b);
    }
    let mut class = bytes.clone();
    class[at + 6..at + 8].copy_from_slice(&4u16.to_le_bytes());
    cases.push(class);
    let mut extra = bytes.clone();
    extra.splice(
        rs[end].start..rs[end].start,
        [0, 0, 0, 0, 0x2e, 0x23, 0, 0, 0, 0, 0, 0, 0x2e, 0x23, 0, 0],
    );
    cases.push(extra);
    let mut duplicate = bytes.clone();
    duplicate.splice(
        refs.start..refs.start,
        bytes[refs.start..refs.data.end].iter().copied(),
    );
    cases.push(duplicate);
    for b in cases {
        let p = Document::from_bytes(&b, None).unwrap();
        assert_eq!(p.objects[id - 1].kind, Kind::Unsupported(10));
        assert_eq!(p.encode().unwrap(), b);
    }
}
