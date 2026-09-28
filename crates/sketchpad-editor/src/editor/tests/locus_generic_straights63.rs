use super::*;

fn isolated_outer(id: usize) -> Editor {
    // Independently authored public API control, with four presentation
    // intervals. Only the locus is visible; current endpoint markers and
    // other infinite families cannot accidentally satisfy a hit.
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/locus_generic_straights63_api_vector_input.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(doc.objects[id - 1].kind, Kind::Locus(_, 4)));
    for object in &mut doc.objects {
        object.visible = object.id == id;
    }
    Editor::new(doc)
}

fn viewport() -> Viewport {
    // A quarter-grid fan gap is about four world pixels wide. Magnification
    // separates it from the editor's seven screen-pixel picking tolerance.
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(2400., 2400.)),
        scale: 3.,
        center: Point { x: 300., y: 330. },
    }
}

#[test]
fn generic_swept_straights_pick_sample_extent_and_leave_fan_gaps_empty() {
    let vp = viewport();
    for (outer, y, positive, negative) in [
        (
            20,
            135.,
            vec![(280., 155.)],
            vec![(250., 95.), (300., 195.)],
        ),
        (
            45,
            315.,
            vec![(280., 335.), (310., 395.)],
            vec![(250., 275.)],
        ),
        (
            70,
            495.,
            vec![(280., 515.), (250., 455.), (310., 575.)],
            vec![],
        ),
    ] {
        let editor = isolated_outer(outer);
        for (x, y) in positive {
            assert_eq!(
                editor.hit(&vp, vp.screen(Point { x, y })),
                Some(outer),
                "outer {outer}: defined extent ({x},{y})"
            );
        }
        for (x, y) in negative {
            assert_eq!(
                editor.hit(&vp, vp.screen(Point { x, y })),
                None,
                "outer {outer}: excluded extent ({x},{y})"
            );
        }
        // Independent vector source: A=(230+60q,y+15), B=(250,y+55),
        // translated (+40,-15). At line fraction .4, adjacent samples are
        // nine world pixels apart. Their half-gap must not be a filled hit.
        for q in [0.125, 0.375, 0.625, 0.875] {
            let point = Point {
                x: (270. + 60. * q) * 0.6 + 290. * 0.4,
                y: y + 16.,
            };
            assert_eq!(
                editor.hit(&vp, vp.screen(point)),
                None,
                "outer {outer}: inter-sample gap q={q}"
            );
        }
    }
}
