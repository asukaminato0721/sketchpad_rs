use super::*;

#[test]
fn native_midpoint_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_midpoint_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[7].kind,
        Kind::Locus(vec![7, 3, 4, 6, 7], 500)
    );
    let points = native.locus_segments(8, 8).unwrap();
    assert_eq!(points.first().unwrap().0, Point { x: 160., y: 140. });
    assert_eq!(points.last().unwrap().1, Point { x: 250., y: 140. });

    let base = include_bytes!("../../../tests/fixtures/locus_midpoint_base.gsp");
    let mut created = Document::from_bytes(base, None).unwrap();
    assert_eq!(created.construct_locus(4, 7), Ok(8));
    assert!(created.construct_locus(1, 7).is_err());
    let bytes = created.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
    assert_eq!(reopened.locus_segments(8, 8), Some(points));

    assert!(created.move_point(1, Point { x: 100., y: 220. }));
    let edited = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!edited.partial_view);
    let moved = edited.locus_segments(8, 8).unwrap();
    assert_eq!(moved.first().unwrap().0, Point { x: 150., y: 160. });
    assert_eq!(moved.last().unwrap().1, Point { x: 250., y: 140. });
}
#[test]
fn translated_point_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_translated_point_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[5].kind, Kind::Locus(vec![5, 3, 4, 5], 500));
    let native_segments = native.locus_segments(6, 8).unwrap();
    assert!((native_segments[0].0.x - 120.).abs() < 1e-9);
    assert!((native_segments[0].0.y - (180. - 37.79527559055118)).abs() < 1e-9);
    assert!((native_segments.last().unwrap().1.x - 300.).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_translated_point_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 5), Ok(6));
    assert!(generated.construct_locus(4, 2).is_err());
    let saved = generated.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[5].kind, native.objects[5].kind);
    assert_eq!(reopened.locus_segments(6, 8), Some(native_segments));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let segments = moved.locus_segments(6, 8).unwrap();
    assert!((segments[0].0.x - 100.).abs() < 1e-9);
    assert!((segments[0].0.y - (220. - 37.79527559055118)).abs() < 1e-9);
    assert!((segments.last().unwrap().1.x - 300.).abs() < 1e-9);
}
#[test]
fn circle_driver_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_circle_driver_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[5].kind, Kind::Locus(vec![5, 3, 4, 5], 500));
    let segments = native.locus_segments(6, 4).unwrap();
    let translated_y = 180. - 37.79527559055118;
    assert!((segments[0].0.x - 260.).abs() < 1e-9);
    assert!((segments[0].0.y - translated_y).abs() < 1e-9);
    assert!((segments[0].1.x - 200.).abs() < 1e-9);
    assert!((segments[0].1.y - (translated_y - 60.)).abs() < 1e-9);
    assert!((segments.last().unwrap().1.x - 260.).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_circle_driver_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 5), Ok(6));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[5].kind, native.objects[5].kind);
    assert_eq!(reopened.locus_segments(6, 4), Some(segments));

    assert!(generated.move_point(1, Point { x: 220., y: 190. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let first = moved.locus_segments(6, 4).unwrap()[0].0;
    assert!((first.x - (220. + 1700f64.sqrt())).abs() < 1e-9);
    assert!((first.y - (190. - 37.79527559055118)).abs() < 1e-9);
}
#[test]
fn arc_driver_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_arc_driver_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 4, 5, 6], 500));
    let segments = native.locus_segments(7, 4).unwrap();
    let offset = 37.79527559055118;
    assert!((segments[0].0.x - 100.).abs() < 1e-9);
    assert!((segments[0].0.y - (200. - offset)).abs() < 1e-9);
    assert!((segments.last().unwrap().1.x - 200.).abs() < 1e-9);
    assert!((segments[1].1.y - (150. - offset)).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_arc_driver_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(5, 6), Ok(7));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
    assert_eq!(reopened.locus_segments(7, 4), Some(segments));

    assert!(generated.move_point(2, Point { x: 150., y: 130. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_segments(7, 4).unwrap();
    assert!((changed[1].1.y - (130. - offset)).abs() < 1e-9);
}
#[test]
fn two_step_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_two_translations_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[6].kind,
        Kind::Locus(vec![6, 3, 4, 5, 6], 500)
    );
    let original = native.locus_segments(7, 4).unwrap();
    let offset = 2. * 37.79527559055118;
    assert!((original[0].0.x - 120.).abs() < 1e-9);
    assert!((original[0].0.y - (180. - offset)).abs() < 1e-9);
    assert!((original.last().unwrap().1.x - 300.).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_two_translations_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 6), Ok(7));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
    assert_eq!(reopened.locus_segments(7, 4), Some(original));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_segments(7, 4).unwrap();
    assert!((changed[0].0.x - 100.).abs() < 1e-9);
    assert!((changed[0].0.y - (220. - offset)).abs() < 1e-9);
}
#[test]
fn branched_locus_and_non_right_angle_polar_cache() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_branch_midpoint_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[8].kind,
        Kind::Locus(vec![8, 3, 4, 5, 6, 7, 8], 500)
    );
    let original = native.locus_segments(9, 4).unwrap();
    let half_unit = 37.79527559055118 / 2.;
    assert!((original[0].0.x - (120. - half_unit)).abs() < 1e-9);
    assert!((original[0].0.y - (180. - half_unit)).abs() < 1e-9);
    assert!((original.last().unwrap().1.x - (300. - half_unit)).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_branch_midpoint_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    let offset = generated.objects[5].constraint_offset.unwrap();
    assert!(f64_at(base, offset + 4).abs() < 1e-12);
    assert_eq!(f64_at(base, offset + 12), -1.);
    let mut stale_cache = base.to_vec();
    stale_cache[offset + 4..offset + 20]
        .copy_from_slice(&include_bytes!("../../../assets/polar_translation_payload.bin")[4..20]);
    assert!(
        Document::from_bytes(&stale_cache, None)
            .unwrap()
            .partial_view
    );
    assert_eq!(generated.construct_locus(4, 8), Ok(9));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
    assert_eq!(reopened.locus_segments(9, 4), Some(original));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_segments(9, 4).unwrap();
    assert!((changed[0].0.x - (100. - half_unit)).abs() < 1e-9);
    assert!((changed[0].0.y - (220. - half_unit)).abs() < 1e-9);
}
#[test]
fn segment_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_segment_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[7].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
    let family = native.locus_segments(8, 384).unwrap();
    assert_eq!(family.len(), 251);
    assert_eq!(
        family.first().unwrap(),
        &(Point { x: 120., y: 180. }, Point { x: 200., y: 100. })
    );
    assert_eq!(
        family.last().unwrap(),
        &(Point { x: 300., y: 180. }, Point { x: 200., y: 100. })
    );

    let base = include_bytes!("../../../tests/fixtures/locus_midpoint_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 6), Ok(8));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
    assert_eq!(reopened.locus_segments(8, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_segments(8, 384).unwrap();
    assert_eq!(changed.first().unwrap().0, Point { x: 100., y: 220. });
    assert_eq!(changed.last().unwrap().0, Point { x: 300., y: 180. });
}
#[test]
fn circle_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_circle_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[8].kind, Kind::Locus(vec![8, 3, 4, 8], 250));
    let family = native.locus_segments(9, 384).unwrap();
    assert_eq!(family.len(), 251 * 64);
    assert_eq!(family[0].0, Point { x: 150., y: 180. });
    assert_eq!(family[250 * 64].0, Point { x: 330., y: 180. });

    let base = include_bytes!("../../../tests/fixtures/locus_circle_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 8), Ok(9));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
    assert_eq!(reopened.locus_segments(9, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_segments(9, 384).unwrap();
    assert_eq!(changed[0].0, Point { x: 130., y: 220. });
    assert_eq!(changed[250 * 64].0, Point { x: 330., y: 180. });
}
#[test]
fn line_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_line_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
    let family = native.locus_strokes(7, 384).unwrap();
    assert_eq!(family.len(), 251);
    assert_eq!(
        family.first().unwrap(),
        &(
            Point { x: 120., y: 180. },
            Point { x: 200., y: 100. },
            LineMode::Line
        )
    );
    assert_eq!(
        family.last().unwrap(),
        &(
            Point { x: 300., y: 180. },
            Point { x: 200., y: 100. },
            LineMode::Line
        )
    );

    let base = include_bytes!("../../../tests/fixtures/locus_line_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 6), Ok(7));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
    assert_eq!(reopened.locus_strokes(7, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(7, 384).unwrap();
    assert_eq!(changed.first().unwrap().0, Point { x: 100., y: 220. });
    assert_eq!(changed.last().unwrap().0, Point { x: 300., y: 180. });
    assert!(changed.iter().all(|stroke| stroke.2 == LineMode::Line));
}
#[test]
fn ray_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_ray_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
    let family = native.locus_strokes(7, 384).unwrap();
    assert_eq!(family.len(), 251);
    assert_eq!(
        family.first().unwrap(),
        &(
            Point { x: 120., y: 180. },
            Point { x: 200., y: 100. },
            LineMode::Ray
        )
    );
    assert_eq!(family.last().unwrap().0, Point { x: 300., y: 180. });
    assert!(family.iter().all(|stroke| stroke.2 == LineMode::Ray));

    let base = include_bytes!("../../../tests/fixtures/locus_ray_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 6), Ok(7));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
    assert_eq!(reopened.locus_strokes(7, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(7, 384).unwrap();
    assert_eq!(changed.first().unwrap().0, Point { x: 100., y: 220. });
    assert_eq!(changed.last().unwrap().0, Point { x: 300., y: 180. });
}
#[test]
fn arc_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_arc_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[7].kind, Kind::Locus(vec![7, 3, 4, 7], 250));
    let family = native.locus_strokes(8, 384).unwrap();
    assert_eq!(family.len(), 251 * 64);
    assert!(family[0].0.distance(Point { x: 120., y: 180. }) < 1e-9);
    assert!(family[250 * 64].0.distance(Point { x: 300., y: 180. }) < 1e-9);
    assert!(family.iter().all(|stroke| stroke.2 == LineMode::Segment));

    let base = include_bytes!("../../../tests/fixtures/locus_arc_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 7), Ok(8));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
    assert_eq!(reopened.locus_strokes(8, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(8, 384).unwrap();
    assert!(changed[0].0.distance(Point { x: 100., y: 220. }) < 1e-9);
    assert!(changed[250 * 64].0.distance(Point { x: 300., y: 180. }) < 1e-9);
}
#[test]
fn arc_on_circle_target_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_arc_on_circle_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
    let family = native.locus_strokes(7, 384).unwrap();
    assert_eq!(family.len(), 251 * 64);
    assert!(family[0].0.distance(Point { x: 330., y: 180. }) < 1e-9);
    assert!(family[250 * 64].0.distance(Point { x: 330., y: 180. }) < 1e-9);
    let first = family[0].0;

    let base = include_bytes!("../../../tests/fixtures/locus_arc_on_circle_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 6), Ok(7));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
    assert_eq!(reopened.locus_strokes(7, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 240., y: 190. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(7, 384).unwrap();
    assert!(changed[0].0.distance(moved.position(4).unwrap()) < 1e-9);
    assert!(changed[0].0.distance(first) > 1.);
    assert!(changed[250 * 64].0.distance(changed[0].0) < 1e-9);
}
#[test]
fn parallel_line_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_parallel_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[8].kind, Kind::Locus(vec![8, 3, 4, 8], 250));
    let family = native.locus_strokes(9, 384).unwrap();
    assert_eq!(family.len(), 251);
    assert_eq!(
        family[0],
        (
            Point { x: 120., y: 180. },
            Point { x: 220., y: 210. },
            LineMode::Line
        )
    );
    assert_eq!(family[250].0, Point { x: 300., y: 180. });
    assert_eq!(family[250].1, Point { x: 400., y: 210. });

    let base = include_bytes!("../../../tests/fixtures/locus_parallel_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 8), Ok(9));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
    assert_eq!(reopened.locus_strokes(9, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(9, 384).unwrap();
    assert_eq!(changed[0].0, Point { x: 100., y: 220. });
    assert_eq!(changed[0].1, Point { x: 200., y: 250. });
    assert_eq!(changed[250].0, Point { x: 300., y: 180. });
}
#[test]
fn perpendicular_line_target_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_perpendicular_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[8].kind, Kind::Locus(vec![8, 3, 4, 8], 250));
    let family = native.locus_strokes(9, 384).unwrap();
    assert_eq!(family.len(), 251);
    assert_eq!(
        family[0],
        (
            Point { x: 120., y: 180. },
            Point { x: 90., y: 280. },
            LineMode::Line
        )
    );
    assert_eq!(family[250].0, Point { x: 300., y: 180. });
    assert_eq!(family[250].1, Point { x: 270., y: 280. });

    let base = include_bytes!("../../../tests/fixtures/locus_perpendicular_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 8), Ok(9));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
    assert_eq!(reopened.locus_strokes(9, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(9, 384).unwrap();
    assert_eq!(changed[0].0, Point { x: 100., y: 220. });
    assert_eq!(changed[0].1, Point { x: 70., y: 320. });
    assert_eq!(changed[250].0, Point { x: 300., y: 180. });
}
#[test]
fn bisector_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_bisector_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[7].kind, Kind::Locus(vec![7, 3, 4, 7], 250));
    let family = native.locus_strokes(8, 384).unwrap();
    assert_eq!(family.len(), 251);
    assert_eq!(family[0].0, Point { x: 120., y: 180. });
    assert_eq!(family[250].0, Point { x: 300., y: 180. });
    assert!(family.iter().all(|stroke| stroke.2 == LineMode::Ray));
    assert!(family[0].1.y < family[0].0.y);

    let base = include_bytes!("../../../tests/fixtures/locus_bisector_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 7), Ok(8));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
    assert_eq!(reopened.locus_strokes(8, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_strokes(8, 384).unwrap();
    assert_eq!(changed[0].0, Point { x: 100., y: 220. });
    assert_eq!(changed[250].0, Point { x: 300., y: 180. });
    assert!(changed.iter().all(|stroke| stroke.2 == LineMode::Ray));
}
#[test]
fn polygon_target_locus_import_generation_and_parent_edit() {
    let native_bytes = include_bytes!("../../../tests/fixtures/locus_polygon_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[7].kind, Kind::Locus(vec![7, 3, 4, 7], 125));
    let family = native.locus_polygons(8, 384).unwrap();
    assert_eq!(family.len(), 126);
    assert_eq!(
        family[0],
        vec![
            Point { x: 120., y: 180. },
            Point { x: 150., y: 100. },
            Point { x: 250., y: 100. }
        ]
    );
    assert_eq!(family[125][0], Point { x: 300., y: 180. });
    assert_eq!(native.locus_strokes(8, 384).unwrap().len(), 126 * 3);

    let base = include_bytes!("../../../tests/fixtures/locus_polygon_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 7), Ok(8));
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
    assert_eq!(reopened.locus_polygons(8, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_polygons(8, 384).unwrap();
    assert_eq!(changed[0][0], Point { x: 100., y: 220. });
    assert_eq!(changed[125][0], Point { x: 300., y: 180. });
}
#[test]
fn new_triangle_uses_native_polygon_subtype() {
    let mut document = Document::new();
    for point in [
        Point { x: 120., y: 180. },
        Point { x: 150., y: 100. },
        Point { x: 250., y: 100. },
    ] {
        document.add(Kind::Free(point));
    }
    document.add(Kind::Polygon(vec![1, 2, 3]));
    let bytes = document.encode().unwrap();
    let polygon_header = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x07d0 && bytes[record.data.start] == 8)
        .unwrap();
    assert_eq!(
        u16::from_le_bytes(
            bytes[polygon_header.data.start + 8..polygon_header.data.start + 10]
                .try_into()
                .unwrap()
        ),
        52
    );
}
#[test]
fn vector_translated_polygon_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_vector_polygon_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[10].kind,
        Kind::Locus(vec![10, 3, 4, 7, 10], 125)
    );
    let family = native.locus_polygons(11, 384).unwrap();
    assert_eq!(family.len(), 126);
    assert_eq!(family[0][0], Point { x: 220., y: 180. });
    assert_eq!(family[125][0], Point { x: 400., y: 180. });
    assert_eq!(family[0][1], Point { x: 250., y: 100. });

    let base = include_bytes!("../../../tests/fixtures/locus_vector_polygon_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 10), Ok(11));
    let encoded = generated.encode().unwrap();
    let target_header = records(&encoded)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x07d0 && encoded[record.data.start] == 16)
        .unwrap();
    assert_eq!(
        &encoded[target_header.data.start + 18..target_header.data.start + 20],
        &[0x39, 0x20]
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[10].kind, native.objects[10].kind);
    assert_eq!(reopened.locus_polygons(11, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_polygons(11, 384).unwrap();
    assert_eq!(changed[0][0], Point { x: 200., y: 220. });
    assert_eq!(changed[125][0], Point { x: 400., y: 180. });
}
#[test]
fn polar_translated_polygon_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_polar_polygon_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[8].kind,
        Kind::Locus(vec![8, 3, 4, 7, 8], 125)
    );
    let offset = 37.79527559055118 * 2.65;
    let family = native.locus_polygons(9, 384).unwrap();
    assert_eq!(family.len(), 126);
    assert!((family[0][0].x - (120. + offset)).abs() < 1e-9);
    assert_eq!(family[0][0].y, 180.);
    assert!((family[125][0].x - (300. + offset)).abs() < 1e-9);
    assert!((family[0][1].x - (150. + offset)).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_polar_polygon_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 8), Ok(9));
    let encoded = generated.encode().unwrap();
    let locus_header = records(&encoded)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x07d0)
        .last()
        .unwrap();
    assert_eq!(
        &encoded[locus_header.data.start + 18..locus_header.data.start + 20],
        &[0xf0, 0x3f]
    );
    assert_eq!(
        &encoded[locus_header.data.start + 20..locus_header.data.start + 24],
        &2u32.to_le_bytes()
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
    assert_eq!(reopened.locus_polygons(9, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_polygons(9, 384).unwrap();
    assert!((changed[0][0].x - (100. + offset)).abs() < 1e-9);
    assert_eq!(changed[0][0].y, 220.);
    assert!((changed[125][0].x - (300. + offset)).abs() < 1e-9);
}
#[test]
fn rectangular_translated_polygon_locus_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_rectangular_polygon_target_original.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[8].kind,
        Kind::Locus(vec![8, 3, 4, 7, 8], 125)
    );
    let dx = 37.79527559055118 * 2.;
    let dy = 37.79527559055118 * 0.5;
    let family = native.locus_polygons(9, 384).unwrap();
    assert_eq!(family.len(), 126);
    assert!((family[0][0].x - (120. + dx)).abs() < 1e-9);
    assert!((family[0][0].y - (180. + dy)).abs() < 1e-9);
    assert!((family[125][0].x - (300. + dx)).abs() < 1e-9);

    let base = include_bytes!("../../../tests/fixtures/locus_rectangular_polygon_target_base.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(generated.construct_locus(4, 8), Ok(9));
    let encoded = generated.encode().unwrap();
    let locus_header = records(&encoded)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x07d0)
        .last()
        .unwrap();
    assert_eq!(
        &encoded[locus_header.data.start + 18..locus_header.data.start + 20],
        &[0, 0x40]
    );
    assert_eq!(
        &encoded[locus_header.data.start + 20..locus_header.data.start + 24],
        &2u32.to_le_bytes()
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
    assert_eq!(reopened.locus_polygons(9, 384), Some(family));

    assert!(generated.move_point(1, Point { x: 100., y: 220. }));
    let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    let changed = moved.locus_polygons(9, 384).unwrap();
    assert!((changed[0][0].x - (100. + dx)).abs() < 1e-9);
    assert!((changed[0][0].y - (220. + dy)).abs() < 1e-9);
    assert!((changed[125][0].x - (300. + dx)).abs() < 1e-9);
}

// At the midpoint the moving line has coincident endpoints. Its intersection
// is undefined for exactly one sample; the other samples remain valid.
fn locus_with_undefined_middle() -> (Document, usize, usize) {
    let mut doc = Document::new();
    let left = doc.add(Kind::Free(Point { x: 100., y: 200. }));
    let right = doc.add(Kind::Free(Point { x: 300., y: 200. }));
    let path = doc.add(Kind::Segment(left, right));
    let driver = doc.add(Kind::PointOnSegment(path, 0.25));
    let fixed = doc.add(Kind::Free(Point { x: 200., y: 200. }));
    let moving_line = doc.add(Kind::Line(driver, fixed));
    let bottom = doc.add(Kind::Free(Point { x: 250., y: 100. }));
    let top = doc.add(Kind::Free(Point { x: 250., y: 300. }));
    let vertical = doc.add(Kind::Line(bottom, top));
    let intersection = doc.add(Kind::Intersection(moving_line, vertical));
    (doc, driver, intersection)
}

#[test]
fn locus_preserves_valid_samples_without_joining_across_undefined_points() {
    let native_bytes =
        include_bytes!("../../../tests/fixtures/locus_undefined_sample_verified.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.locus_segments(11, 8).unwrap().len(), 6);
    let (mut doc, driver, target) = locus_with_undefined_middle();
    let locus = doc.construct_locus(driver, target).unwrap();
    assert_eq!(doc.locus_segments(locus, 8).unwrap().len(), 6);
    assert_eq!(doc.locus_segments(locus, 2).unwrap().len(), 0);
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.locus_segments(locus, 8),
        doc.locus_segments(locus, 8)
    );
    // No sample has a defined intersection when the fixed line is parallel.
    assert!(doc.move_point(7, Point { x: 100., y: 300. }));
    assert!(doc.move_point(8, Point { x: 300., y: 300. }));
    assert_eq!(doc.locus_segments(locus, 8), Some(Vec::new()));
    assert!(doc.encode().is_ok());
}

#[test]
fn locus_object_families_skip_undefined_members() {
    for family in 0..4 {
        let (mut doc, driver, intersection) = locus_with_undefined_middle();
        let target = doc.add(match family {
            0 => Kind::Segment(intersection, 7),
            1 => Kind::Circle(intersection, 7),
            2 => Kind::ThreePointArc(intersection, 7, 5),
            _ => Kind::Polygon(vec![intersection, 7, 5]),
        });
        let locus = doc.construct_locus(driver, target).unwrap();
        let expected = match family {
            0 => 8,
            1 | 2 => 8 * 64,
            _ => 8 * 3,
        };
        assert_eq!(
            doc.locus_strokes(locus, 8).unwrap().len(),
            expected,
            "family {family}"
        );
        if family == 3 {
            assert_eq!(doc.locus_polygons(locus, 8).unwrap().len(), 8);
            // Six adjacent valid pairs, each contributing three swept faces.
            // The two pairs bordering the missing sample must add no faces.
            assert_eq!(
                doc.locus_polygon_surfaces(locus, 8).unwrap().len(),
                8 + 6 * 3
            );
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.locus_strokes(locus, 8),
            doc.locus_strokes(locus, 8)
        );
    }
}
