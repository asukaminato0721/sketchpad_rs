use super::*;
fn arc59_close(actual: Option<Point>, expected: Option<Point>) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(a.distance(b) < 1e-8, "{a:?} != {b:?}"),
        (None, None) => {}
        other => panic!("{other:?}"),
    }
}
#[test]
fn native_fourteen_arc_on_circle_controls_use_start_radius_and_preserve_packets() {
    for state in [
        "baseline",
        "free_on",
        "derived_on",
        "other_circle",
        "off_start",
        "off_end",
        "both_off",
        "reverse_off",
        "start_center",
        "end_center",
        "same_coords",
        "same_angle",
        "zero_radius",
        "transformed_circle",
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/arc_circle59_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        let arc = d
            .objects
            .iter()
            .find(|o| matches!(o.kind, Kind::ArcOnCircle(..)))
            .unwrap();
        let undefined = matches!(
            state,
            "start_center" | "end_center" | "same_coords" | "same_angle" | "zero_radius"
        );
        let center = if state == "transformed_circle" {
            Point { x: 300., y: 280. }
        } else {
            Point { x: 250., y: 260. }
        };
        let radius = match state {
            "off_start" | "both_off" => 160.,
            "reverse_off" => 40.,
            _ => 80.,
        };
        let start = if state == "reverse_off" {
            std::f64::consts::FRAC_PI_2
        } else {
            0.
        };
        let sweep = if state == "reverse_off" {
            3. * std::f64::consts::FRAC_PI_2
        } else {
            std::f64::consts::FRAC_PI_2
        };
        let mut samples = 0;
        for o in &d.objects {
            if let Kind::PointOnArc(host, t) = o.kind {
                assert_eq!(host, arc.id);
                let angle = start + sweep * t;
                let expected = (!undefined).then_some(Point {
                    x: center.x + radius * angle.cos(),
                    y: center.y - radius * angle.sin(),
                });
                arc59_close(d.position(o.id), expected);
                samples += 1;
            }
            if let Kind::Segment(a, _) = o.kind {
                if matches!(d.objects[a - 1].kind, Kind::PointOnArc(..)) {
                    assert_eq!(d.line_points(o.id).is_none(), undefined, "{state}");
                }
            }
        }
        assert_eq!(samples, 3);
        assert_eq!(d.arc_geometry(arc.id).is_none(), undefined);
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
    }
}
fn arc59_author() -> (Document, usize, usize, usize, usize, usize) {
    let mut d = Document::new();
    let c = d.add(Kind::Free(Point { x: 250., y: 260. }));
    let r = d.add(Kind::Free(Point { x: 330., y: 260. }));
    let circle = d.add(Kind::Circle(c, r));
    let start = d.add(Kind::Free(Point { x: 410., y: 260. }));
    let end = d.add(Kind::Free(Point { x: 250., y: 220. }));
    let arc = d.add(Kind::ArcOnCircle(circle, start, end));
    (d, r, circle, start, end, arc)
}
#[test]
fn authored_arc_general_endpoints_update_suppress_save_recover_and_undo() {
    let (mut d, radius_point, circle, start, end, arc) = arc59_author();
    let point = d.add(Kind::PointOnArc(arc, 0.5));
    let anchor = d.add(Kind::Free(Point { x: 100., y: 400. }));
    let segment = d.add(Kind::Segment(point, anchor));
    let expected = Point {
        x: 250. + 160. / 2f64.sqrt(),
        y: 260. - 160. / 2f64.sqrt(),
    };
    arc59_close(d.position(point), Some(expected));
    let bytes = d.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&bytes, None).unwrap().objects[arc - 1].kind,
        Kind::ArcOnCircle(circle, start, end)
    );
    for edited in [end, radius_point] {
        d.checkpoint();
        assert!(d.move_point(edited, Point { x: 250., y: 260. }));
        assert!(d.position(point).is_none());
        assert!(d.line_points(segment).is_none());
        let saved = d.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(reopened.position(point).is_none());
        assert_eq!(reopened.encode().unwrap(), saved);
        d.undo();
        arc59_close(d.position(point), Some(expected));
        d.redo();
        assert!(d.position(point).is_none());
        d.undo();
    }
    d.checkpoint();
    assert!(d.move_point(start, Point { x: 330., y: 260. }));
    assert!((d.arc_geometry(arc).unwrap().1 - 80.).abs() < 1e-10);
    d.selected.insert(circle);
    let deleted = d.clear_selected().unwrap();
    assert!(deleted.contains(&arc) && deleted.contains(&point) && deleted.contains(&segment));
    d.undo();
    assert!(d.position(point).is_some());
}
#[test]
fn arc_packet_guards_retain_malformed_records_and_empty_metadata_exactly() {
    let bytes = fs::read("../../tests/fixtures/arc_circle59_free_on_native_saved.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let arc = d
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::ArcOnCircle(..)))
        .unwrap();
    let h = arc.header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    for (at, value) in [
        (h + 6, 1u16.to_le_bytes().to_vec()),
        (h + 8, 45u16.to_le_bytes().to_vec()),
        (refs.data.start + 4, 6u32.to_le_bytes().to_vec()),
        (refs.data.start + 8, 3u32.to_le_bytes().to_vec()),
        (refs.data.start + 12, (arc.id as u32).to_le_bytes().to_vec()),
    ] {
        let mut invalid = bytes.clone();
        invalid[at..at + value.len()].copy_from_slice(&value);
        let parsed = Document::from_bytes(&invalid, None).unwrap();
        assert_eq!(parsed.objects[arc.id - 1].kind, Kind::Unsupported(79));
        assert_eq!(parsed.encode().unwrap(), invalid);
    }
    let end = rs[i..].iter().find(|r| r.kind == 0x7d7).unwrap().start;
    for (kind, data, supported) in [
        (0x7d3, vec![79, 0, 0, 0], false),
        (0x232e, vec![1], false),
        (0x232e, vec![], true),
    ] {
        let mut edited = bytes.clone();
        let mut record = vec![];
        put_record(&mut record, kind, &data);
        edited.splice(end..end, record);
        let parsed = Document::from_bytes(&edited, None).unwrap();
        assert_eq!(
            matches!(parsed.objects[arc.id - 1].kind, Kind::ArcOnCircle(..)),
            supported
        );
        assert_eq!(parsed.encode().unwrap(), edited);
    }
}
#[test]
fn second_page_authored_arc_preserves_local_refs_and_rejects_cross_page_sources() {
    let bytes = fs::read("../../tests/fixtures/arc_circle59_free_on_native_saved.gsp").unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    d.add_blank_page().unwrap();
    d.move_page(0, 1).unwrap();
    d.active_page = 0;
    d.add(Kind::Free(Point { x: 10., y: 20. }));
    d.add(Kind::Free(Point { x: 20., y: 30. }));
    d.active_page = 1;
    let mut d = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    let base = d.objects.iter().filter(|o| o.page == 0).count();
    assert_eq!(base, 2);
    let (circle, start, end) = d
        .objects
        .iter()
        .find_map(|o| match o.kind {
            Kind::ArcOnCircle(c, a, b) => Some((c, a, b)),
            _ => None,
        })
        .unwrap();
    let arc = d.add(Kind::ArcOnCircle(circle, start, end));
    let point = d.add(Kind::PointOnArc(arc, 0.5));
    let bytes = d.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    arc59_close(reopened.position(point), d.position(point));
    assert_eq!(reopened.objects[arc - 1].page, 1);
    let rs = records(&bytes).unwrap();
    let h = reopened.objects[arc - 1].header_offset.unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    for (index, id) in [circle, start, end].into_iter().enumerate() {
        assert_eq!(
            u32_at(&bytes[refs.data.clone()], 4 + 4 * index),
            (id - base) as u32
        );
    }
    let external = d.objects.iter().find(|o| o.page == 0).unwrap().id;
    let bad = d.add(Kind::ArcOnCircle(circle, external, end));
    assert!(d.can_save().is_err());
    assert_eq!(d.objects[bad - 1].page, 1);
}

#[test]
fn arc_general_numeric_endpoint_preserves_definition_through_undefined_parameter() {
    let (mut d, _, circle, start, _, _) = arc59_author();
    let frame = d
        .add_default_coordinate_system(Point { x: 250., y: 260. })
        .unwrap();
    let u = d.add_named_parameter("u", 1., (10, 100)).unwrap();
    let x = d.add_parameter_calculation(u, "1/u", (10, 130)).unwrap();
    let end = d.add(Kind::NumericCoordinatePoint(x, u, frame));
    let arc = d.add(Kind::ArcOnCircle(circle, start, end));
    let point = d.add(Kind::PointOnArc(arc, 0.5));
    let anchor = d.add(Kind::Free(Point { x: 100., y: 400. }));
    let segment = d.add(Kind::Segment(point, anchor));
    assert!(d.position(point).is_some());
    d.checkpoint();
    d.set_parameter_value(u, 0.).unwrap();
    assert!(d.position(end).is_none());
    assert!(d.position(point).is_none());
    assert!(d.line_points(segment).is_none());
    let bytes = d.encode().unwrap();
    let mut reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        reopened.objects[arc - 1].kind,
        Kind::ArcOnCircle(circle, start, end)
    );
    assert!(reopened.position(point).is_none());
    reopened.set_parameter_value(u, 2.).unwrap();
    assert!(reopened.position(point).is_some());
    d.undo();
    assert!(d.position(point).is_some());
}
