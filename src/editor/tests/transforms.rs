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
