use super::*;

#[test]
fn marked_angle_drives_dynamic_rotation() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
    let first = editor.doc.add(Kind::Free(Point { x: 230., y: 160. }));
    let vertex = editor.doc.add(Kind::Free(Point { x: 230., y: 220. }));
    let last = editor.doc.add(Kind::Free(Point { x: 290., y: 220. }));
    let source = editor.doc.add(Kind::Free(Point { x: 170., y: 150. }));
    let before = editor.doc.encode().unwrap();

    editor.select_only(center);
    editor.mark_center_selected();
    editor.select_only(first);
    editor.selection_order.extend([vertex, last]);
    editor.doc.selected.extend([vertex, last]);
    editor.mark_angle_selected();
    assert_eq!(
        editor.marked_angle,
        Some(MarkedAngle::Points([first, vertex, last]))
    );
    assert!(editor.selection_order.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);

    editor.select_only(source);
    editor.rotate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::AngleRotatedPoint(source, center, first, vertex, last)
    );
    assert_eq!(editor.doc.position(image), Some(Point { x: 130., y: 190. }));
    assert!(editor.doc.move_point(last, Point { x: 290., y: 190. }));
    assert_ne!(editor.doc.position(image), Some(Point { x: 130., y: 190. }));
}

#[test]
fn marked_angle_rotates_selected_segment_and_circle() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
    let first = editor.doc.add(Kind::Free(Point { x: 230., y: 160. }));
    let vertex = editor.doc.add(Kind::Free(Point { x: 230., y: 220. }));
    let last = editor.doc.add(Kind::Free(Point { x: 290., y: 220. }));
    let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
    let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    editor.marked_center = Some(center);
    editor.marked_angle = Some(MarkedAngle::Points([first, vertex, last]));
    editor.rotation_use_marked = true;
    editor.select_only(segment);
    editor.rotate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::AngleRotatedSegment(segment, center, first, vertex, last)
    );
    editor.select_only(circle);
    editor.rotate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::AngleRotatedCircle(circle, center, first, vertex, last)
    );
    assert_eq!(
        editor
            .doc
            .circle_points(editor.doc.objects.len())
            .unwrap()
            .1,
        40.
    );
}

#[test]
fn fixed_angle_rotates_selected_segment_and_circle() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
    let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
    let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    editor.marked_center = Some(center);
    editor.rotation_angle = 90.;
    editor.rotation_use_marked = false;
    editor.select_only(segment);
    editor.rotate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::RotatedSegment(segment, center, 90.)
    );
    editor.select_only(circle);
    editor.rotate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::RotatedCircle(circle, center, 90.)
    );
    assert_eq!(
        editor
            .doc
            .circle_points(editor.doc.objects.len())
            .unwrap()
            .1,
        40.
    );
}

#[test]
fn segment_reflection_and_dilation_commands() {
    let mut editor = Editor::new(Document::new());
    let center = editor.doc.add(Kind::Free(Point { x: 100., y: 120. }));
    let origin = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = editor.doc.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = editor.doc.add(Kind::Free(Point { x: 205., y: 100. }));
    let a = editor.doc.add(Kind::Free(Point { x: 140., y: 120. }));
    let b = editor.doc.add(Kind::Free(Point { x: 160., y: 120. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let mirror_a = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let mirror_b = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
    let mirror = editor.doc.add(Kind::Segment(mirror_a, mirror_b));
    editor.marked_center = Some(center);
    editor.dilation_use_marked = false;
    editor.dilation_factor = 0.5;
    editor.select_only(segment);
    editor.dilate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::ScaledSegment(segment, center, 0.5)
    );
    editor.marked_ratio = Some([origin, base, target]);
    editor.dilation_use_marked = true;
    editor.select_only(segment);
    editor.dilate_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::RatioScaledSegment(segment, center, origin, base, target)
    );
    editor.marked_mirror = Some(mirror);
    editor.select_only(segment);
    editor.reflect_selected();
    assert_eq!(
        editor.doc.objects.last().unwrap().kind,
        Kind::ReflectedSegment(segment, mirror)
    );
}

#[test]
fn fixed_transform_commands_accept_transformed_segments_and_circles() {
    for name in ["segment", "circle"] {
        let bytes = if name == "segment" {
            include_bytes!("../../../tests/fixtures/vector_rotated_segment_native.gsp").as_slice()
        } else {
            include_bytes!("../../../tests/fixtures/vector_rotated_circle_native.gsp").as_slice()
        };
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        editor.marked_center = Some(1);
        editor.rotation_use_marked = false;
        editor.rotation_angle = 45.;
        editor.select_only(6);
        editor.rotate_selected();
        assert_eq!(editor.doc.objects.len(), 7);
        assert_eq!(
            editor.doc.objects[6].kind,
            if name == "segment" {
                Kind::RotatedSegment(6, 1, 45.)
            } else {
                Kind::RotatedCircle(6, 1, 45.)
            }
        );
        editor.dilation_use_marked = false;
        editor.dilation_factor = 1.25;
        editor.dilate_selected();
        assert_eq!(editor.doc.objects.len(), 8);
        assert_eq!(
            editor.doc.objects[7].kind,
            if name == "segment" {
                Kind::ScaledSegment(7, 1, 1.25)
            } else {
                Kind::ScaledCircle(7, 1, 1.25)
            }
        );
        let mirror = editor.doc.add(Kind::Segment(1, 3));
        editor.marked_mirror = Some(mirror);
        editor.reflect_selected();
        assert_eq!(editor.doc.objects.len(), 10);
        assert_eq!(
            editor.doc.objects[9].kind,
            if name == "segment" {
                Kind::ReflectedSegment(8, mirror)
            } else {
                Kind::ReflectedCircle(8, mirror)
            }
        );
        editor.doc.verify_roundtrip().unwrap();
    }
    for ray in [false, true] {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
        let b = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let source = editor.doc.add(if ray {
            Kind::Ray(a, b)
        } else {
            Kind::Line(a, b)
        });
        editor.marked_center = Some(a);
        editor.rotation_use_marked = false;
        editor.select_only(source);
        editor.dilation_factor = 0.;
        editor.dilate_selected();
        editor.marked_mirror = Some(source);
        editor.reflect_selected();
        assert_eq!(editor.doc.objects.len(), 3);
    }
}

#[test]
fn fixed_rotated_ray_and_line_commands_pick_the_correct_extents() {
    for (name, bytes) in [
        (
            "ray",
            include_bytes!("../../../tests/fixtures/fixed_rotation_ray_original.gsp").as_slice(),
        ),
        (
            "line",
            include_bytes!("../../../tests/fixtures/fixed_rotation_line_original.gsp").as_slice(),
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        let vp = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        assert_eq!(
            editor.hit(&vp, vp.screen(Point { x: 170., y: 20. })),
            Some(5)
        );
        assert_eq!(
            editor.hit(&vp, vp.screen(Point { x: 170., y: 250. })),
            if name == "ray" { None } else { Some(5) }
        );
        editor.marked_center = Some(1);
        editor.rotation_use_marked = false;
        editor.rotation_angle = 45.;
        editor.select_only(4);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects[5].kind,
            if name == "ray" {
                Kind::RotatedRay(4, 1, 45.)
            } else {
                Kind::RotatedLine(4, 1, 45.)
            }
        );
        let mut image_only = editor.doc.clone();
        for object in &mut image_only.objects {
            object.visible = object.id == 5;
        }
        let svg = sketchpad_rs::render::svg_page(&image_only, &vp).unwrap();
        let y1 = if name == "ray" { 160 } else { 400 };
        assert!(svg.contains(&format!("<line x1=\"220\" y1=\"{y1}\" x2=\"220\" y2=\"0\"")));
        editor.doc.verify_roundtrip().unwrap();
        let count = editor.doc.objects.len();
        editor.dilation_use_marked = false;
        editor.select_only(6);
        editor.dilate_selected();
        editor.marked_mirror = Some(4);
        editor.reflect_selected();
        assert_eq!(editor.doc.objects.len(), count + 1);
        editor.select_only(4);
        editor.rotation_use_marked = true;
        editor.marked_angle = Some(MarkedAngle::Points([1, 2, 3]));
        editor.rotate_selected();
        assert_eq!(editor.doc.objects.len(), count + 1);
    }
}

#[test]
fn fixed_scaled_ray_and_line_commands_preserve_direction_and_zero_rejection() {
    for ray in [true, false] {
        let bytes = if ray {
            include_bytes!("../../../tests/fixtures/fixed_dilation_ray_original.gsp").as_slice()
        } else {
            include_bytes!("../../../tests/fixtures/fixed_dilation_line_original.gsp").as_slice()
        };
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        let vp = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        assert_eq!(
            editor.hit(&vp, vp.screen(Point { x: 450., y: 230. })),
            Some(5)
        );
        assert_eq!(
            editor.hit(&vp, vp.screen(Point { x: 180., y: 230. })),
            if ray { None } else { Some(5) }
        );
        editor.marked_center = Some(1);
        editor.dilation_use_marked = false;
        editor.dilation_factor = -2.;
        editor.select_only(4);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects[5].kind,
            if ray {
                Kind::ScaledRay(4, 1, -2.)
            } else {
                Kind::ScaledLine(4, 1, -2.)
            }
        );
        assert_eq!(
            editor.hit(&vp, vp.screen(Point { x: 20., y: 70. })),
            Some(6)
        );
        assert_eq!(
            editor.hit(&vp, vp.screen(Point { x: 100., y: 70. })),
            if ray { None } else { Some(6) }
        );
        let mut image_only = editor.doc.clone();
        for object in &mut image_only.objects {
            object.visible = object.id == 6;
        }
        let svg = sketchpad_rs::render::svg_page(&image_only, &vp).unwrap();
        assert!(svg.contains(if ray {
            "<line x1=\"100\" y1=\"120\" x2=\"0\" y2=\"120\""
        } else {
            "<line x1=\"500\" y1=\"120\" x2=\"0\" y2=\"120\""
        }));
        editor.doc.verify_roundtrip().unwrap();
        let before = editor.doc.encode().unwrap();
        editor.select_only(4);
        editor.dilation_factor = 0.;
        editor.dilate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
        editor.dilation_factor = 2.;
        editor.select_only(6);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects[6].kind,
            if ray {
                Kind::ScaledRay(6, 1, 2.)
            } else {
                Kind::ScaledLine(6, 1, 2.)
            }
        );
        editor.doc.verify_roundtrip().unwrap();
        let before = editor.doc.encode().unwrap();
        editor.select_only(4);
        editor.dilation_use_marked = true;
        editor.marked_ratio = Some([1, 2, 3]);
        editor.dilate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
    }
}

#[test]
fn fixed_ray_and_line_chain_commands_inherit_source_strokes_and_keep_marked_exclusions() {
    for (ray, bytes) in [
        (
            true,
            include_bytes!(
                "../../../tests/fixtures/fixed_straight_chain_rotation_ray_original.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            include_bytes!(
                "../../../tests/fixtures/fixed_straight_chain_rotation_line_original.gsp"
            )
            .as_slice(),
        ),
        (
            true,
            include_bytes!(
                "../../../tests/fixtures/fixed_straight_chain_dilation_ray_original.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            include_bytes!(
                "../../../tests/fixtures/fixed_straight_chain_dilation_line_original.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        editor.marked_center = Some(1);
        editor.rotation_use_marked = false;
        editor.rotation_angle = 90.;
        editor.select_only(5);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects[7].kind,
            if ray {
                Kind::RotatedRay(5, 1, 90.)
            } else {
                Kind::RotatedLine(5, 1, 90.)
            }
        );
        assert_eq!(
            editor.doc.objects[7].visual_style,
            editor.doc.objects[4].visual_style
        );
        assert_eq!(editor.doc.line_points(8), editor.doc.line_points(6));
        editor.dilation_use_marked = false;
        editor.dilation_factor = -2.;
        editor.select_only(5);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects[8].kind,
            if ray {
                Kind::ScaledRay(5, 1, -2.)
            } else {
                Kind::ScaledLine(5, 1, -2.)
            }
        );
        assert_eq!(
            editor.doc.objects[8].visual_style,
            editor.doc.objects[4].visual_style
        );
        assert_eq!(editor.doc.line_points(9), editor.doc.line_points(7));
        editor.doc.verify_roundtrip().unwrap();
        let before = editor.doc.encode().unwrap();
        editor.select_only(5);
        editor.rotation_use_marked = true;
        editor.marked_angle = Some(MarkedAngle::Points([1, 2, 3]));
        editor.rotate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
        editor.dilation_use_marked = true;
        editor.marked_ratio = Some([1, 2, 3]);
        editor.dilate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
    }
}

#[test]
fn calculated_angle_rotation_commands_keep_ray_and_line_modes() {
    for ray in [true, false] {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
        let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
        let source = editor.doc.add(if ray {
            Kind::Ray(a, b)
        } else {
            Kind::Line(a, b)
        });
        let n = editor.doc.add_named_parameter("n", 4., (50, 30)).unwrap();
        let angle = editor
            .doc
            .add_parameter_calculation(n, "360°/n", (50, 60))
            .unwrap();
        editor.marked_center = Some(center);
        editor.rotation_use_marked = true;
        editor.marked_angle = Some(MarkedAngle::Value(angle));
        editor.select_only(source);
        editor.rotate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            if ray {
                Kind::CalculatedAngleRotatedRay(source, center, angle)
            } else {
                Kind::CalculatedAngleRotatedLine(source, center, angle)
            }
        );
        assert_eq!(
            editor.doc.line_points(image).unwrap().2,
            if ray { LineMode::Ray } else { LineMode::Line }
        );
        let vp = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        let mut image_only = editor.doc.clone();
        for object in &mut image_only.objects {
            object.visible = object.id == image;
        }
        let picking = Editor::new(image_only.clone());
        assert_eq!(
            picking.hit(&vp, vp.screen(Point { x: 170., y: 20. })),
            Some(image)
        );
        assert_eq!(
            picking.hit(&vp, vp.screen(Point { x: 170., y: 250. })),
            if ray { None } else { Some(image) }
        );
        let svg = sketchpad_rs::render::svg_page(&image_only, &vp).unwrap();
        let y1 = if ray { 160 } else { 400 };
        assert!(svg.contains(&format!("<line x1=\"220\" y1=\"{y1}\" x2=\"220\" y2=\"0\"")));
        editor.doc.verify_roundtrip().unwrap();
        let before = editor.doc.encode().unwrap();
        editor.select_only(source);
        editor.marked_angle = Some(MarkedAngle::Points([a, center, b]));
        editor.rotate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
        editor.select_only(image);
        editor.rotation_use_marked = false;
        editor.rotate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
    }
}

#[test]
fn calculated_ratio_dilation_commands_keep_ray_and_line_extents_and_domains() {
    for (ray, bytes) in [
        (
            true,
            include_bytes!("../../../tests/fixtures/numeric_ratio_ray_original.gsp").as_slice(),
        ),
        (
            false,
            include_bytes!("../../../tests/fixtures/numeric_ratio_line_original.gsp").as_slice(),
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        editor.marked_center = Some(1);
        editor.dilation_use_marked = true;
        editor.marked_ratio_value = Some(7);
        editor.select_only(4);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects[8].kind,
            if ray {
                Kind::CalculatedRatioScaledRay(4, 1, 7)
            } else {
                Kind::CalculatedRatioScaledLine(4, 1, 7)
            }
        );
        assert_eq!(
            editor.doc.objects[8].visual_style,
            editor.doc.objects[3].visual_style
        );
        let vp = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        let image_only = |doc: &Document| {
            let mut doc = doc.clone();
            for object in &mut doc.objects {
                object.visible = object.id == 9;
            }
            doc
        };
        let picking = Editor::new(image_only(&editor.doc));
        assert_eq!(
            picking.hit(&vp, vp.screen(Point { x: 400., y: 230. })),
            Some(9)
        );
        assert_eq!(
            picking.hit(&vp, vp.screen(Point { x: 180., y: 230. })),
            if ray { None } else { Some(9) }
        );
        let svg = sketchpad_rs::render::svg_page(&picking.doc, &vp).unwrap();
        let x1 = if ray { 260 } else { 0 };
        assert!(svg.contains(&format!(
            "<line x1=\"{x1}\" y1=\"280\" x2=\"500\" y2=\"280\""
        )));
        editor.doc.set_parameter_value(5, -4.).unwrap();
        let picking = Editor::new(image_only(&editor.doc));
        assert_eq!(
            picking.hit(&vp, vp.screen(Point { x: 20., y: 70. })),
            Some(9)
        );
        assert_eq!(
            picking.hit(&vp, vp.screen(Point { x: 100., y: 70. })),
            if ray { None } else { Some(9) }
        );
        editor.doc.set_parameter_value(5, 0.).unwrap();
        assert!(editor.doc.line_points(9).is_none());
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.set_parameter_value(5, 4.).unwrap();
        assert!(editor.doc.line_points(9).is_some());
        let before = editor.doc.encode().unwrap();
        editor.select_only(9);
        editor.dilate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
        editor.select_only(4);
        editor.marked_ratio_value = Some(6);
        editor.dilate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
        editor.marked_ratio_value = None;
        editor.marked_ratio = Some([1, 2, 3]);
        editor.dilate_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);
    }
}
