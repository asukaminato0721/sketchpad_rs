use super::*;
use sketchpad_rs::document::transform_model::{ElementType, NativeTransformFlavor};

fn close(a: Point, b: Point) {
    assert!(a.distance(b) < 1e-9, "{a:?} != {b:?}");
}

#[test]
fn straight_transform_commands_preserve_extent_and_live_matrix_chains() {
    for ray in [true, false] {
        let mut d = Document::new();
        d.add(Kind::Free(Point { x: 100., y: 100. }));
        d.add(Kind::Free(Point { x: 160., y: 100. }));
        let source = d.add(if ray { Kind::Ray(1, 2) } else { Kind::Line(1, 2) });
        d.add(Kind::Free(Point { x: 0., y: 0. }));
        d.add(Kind::Free(Point { x: 0., y: 100. }));
        let mirror = d.add(Kind::Line(4, 5));
        let mut e = Editor::new(d);
        e.select_only(source);
        e.marked_vector = Some((1, 2));
        e.translation_mode = TranslationMode::MarkedVector;
        e.translate_selected();
        let vector = e.doc.objects.len();
        e.marked_mirror = Some(mirror);
        e.reflect_selected();
        let reflected = e.doc.objects.len();
        e.translation_mode = TranslationMode::FixedPolar;
        e.translation_angle = 90.;
        e.translation_distance = 1.;
        e.translate_selected();
        let polar = e.doc.objects.len();
        e.marked_center = Some(4);
        e.rotation_angle = 90.;
        e.rotation_use_marked = false;
        e.rotate_selected();
        let rotated = e.doc.objects.len();
        let y = 100. - 37.79527559055118;
        for (id, flavor, start, end) in [
            (vector, NativeTransformFlavor::Vector, Point { x: 160., y: 100. }, Point { x: 220., y: 100. }),
            (reflected, NativeTransformFlavor::Reflection, Point { x: -160., y: 100. }, Point { x: -220., y: 100. }),
            (polar, NativeTransformFlavor::FixedPolar, Point { x: -160., y }, Point { x: -220., y }),
            (rotated, NativeTransformFlavor::FixedRotation, Point { x: y, y: 160. }, Point { x: y, y: 220. }),
        ] {
            let def = e.doc.objects[id - 1].kind.transform_definition().unwrap();
            assert_eq!(def.element(), if ray { ElementType::Ray } else { ElementType::Line });
            assert_eq!(def.wire_flavor(), flavor);
            let (a, b, mode) = e.doc.line_points(id).unwrap();
            assert_eq!(mode, if ray { LineMode::Ray } else { LineMode::Line });
            close(a, start);
            close(b, end);
        }
        let sequence = e.doc.transform_sequence(rotated).unwrap();
        assert_eq!(sequence.source, source);
        assert_eq!(sequence.matrices.len(), 4);
        close(sequence.apply(Point { x: 100., y: 100. }).unwrap(), Point { x: y, y: 160. });
        let saved = e.doc.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.line_points(rotated), e.doc.line_points(rotated));
        e.doc.checkpoint();
        assert!(e.doc.move_point(2, Point { x: 180., y: 100. }));
        close(e.doc.line_points(vector).unwrap().0, Point { x: 180., y: 100. });
        assert_ne!(e.doc.line_points(rotated), reopened.line_points(rotated));
        e.doc.undo();
        assert_eq!(e.doc.encode().unwrap(), saved);
    }
}

#[test]
fn straight_translation_commands_keep_unproved_modes_closed_without_partial_edits() {
    let mut d = Document::new();
    d.add(Kind::Free(Point { x: 100., y: 100. }));
    d.add(Kind::Free(Point { x: 160., y: 100. }));
    let ray = d.add(Kind::Ray(1, 2));
    let mut e = Editor::new(d);
    e.select_only(ray);
    e.translation_mode = TranslationMode::FixedRectangular;
    let before = e.doc.encode().unwrap();
    e.translate_selected();
    assert_eq!(e.doc.encode().unwrap(), before);
    assert_eq!(e.doc.objects.len(), 3);
}
