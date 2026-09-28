use super::*;

// Independent Native original/edited paint checks measured these three
// binding-18 generations. The source curve intersection (object 11) is a
// different point; picking isolates 18 to avoid accepting that source.
const NATIVE_GENERATIONS: [Point; 3] = [
    Point {
        x: 747.4633376,
        y: 430.4094488,
    },
    Point {
        x: 780.1950064,
        y: 335.9212598,
    },
    Point {
        x: 812.9266752,
        y: 203.6377953,
    },
];

fn native_bytes(state: &str) -> Vec<u8> {
    std::fs::read(format!(
        "../../tests/fixtures/function_plot_scope69_{state}_native_saved.gsp"
    ))
    .unwrap()
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(4000., 3000.)),
        scale: 2.,
        center: Point { x: 780., y: 350. },
    }
}

fn binding_only_editor(doc: &Document) -> Editor {
    let mut isolated = doc.clone();
    for object in &mut isolated.objects {
        object.visible = object.id == 18;
    }
    Editor::new(isolated)
}

fn assert_native_point_hits(doc: &Document, offset: Point) {
    let view = viewport();
    let editor = binding_only_editor(doc);
    for (generation, point) in NATIVE_GENERATIONS.iter().enumerate() {
        let at = Point {
            x: point.x + offset.x,
            y: point.y + offset.y,
        };
        assert_eq!(
            editor.hit(&view, view.screen(at)),
            Some(18),
            "Native binding18 generation {generation} at {at:?}"
        );
    }
    assert_eq!(
        editor.hit(&view, view.screen(Point { x: 950., y: 900. })),
        None,
        "point-binding exterior"
    );
}

#[test]
fn original_function_plot_mapping_picks_each_native_generation_and_exports_svg() {
    let original = native_bytes("original");
    let doc = Document::from_bytes(&original, None).unwrap();
    assert!(!doc.partial_view);
    assert!(matches!(doc.objects[7].kind, Kind::FunctionPlot(..)));
    assert!(matches!(
        doc.objects[16].kind,
        Kind::MappedPointIteration { .. }
    ));
    assert!(matches!(
        doc.objects[17].kind,
        Kind::IterationBinding(11, 17)
    ));
    assert_eq!(doc.encode().unwrap(), original);
    assert_native_point_hits(&doc, Point { x: 0., y: 0. });

    let mut isolated = doc.clone();
    for object in &mut isolated.objects {
        object.visible = (18..=22).contains(&object.id);
    }
    let svg = crate::render::svg_page(&isolated, &viewport()).unwrap();
    assert_eq!(
        svg.matches("fill=\"#ff0000\"").count(),
        6,
        "both red point bindings 18/19 have three mapped generations"
    );
    assert!(
        svg.matches("<line ").count() >= 9,
        "three mapped segment bindings"
    );
    assert!(!svg.contains("NaN"));
}

#[test]
fn public_edit_shifts_native_mapped_hits_and_undo_reopens_exactly() {
    let original = native_bytes("original");
    let edited = native_bytes("edited");
    let mut editor = Editor::new(Document::from_bytes(&original, None).unwrap());
    editor.doc.checkpoint();
    assert!(editor.doc.move_point(1, Point { x: 732., y: 526. }));
    assert_eq!(editor.doc.encode().unwrap(), edited);
    assert_native_point_hits(&editor.doc, Point { x: 50., y: 20. });
    editor.doc.verify_roundtrip().unwrap();

    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_native_point_hits(&reopened, Point { x: 50., y: 20. });
    assert_eq!(reopened.encode().unwrap(), edited);
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), original);
    assert_native_point_hits(&editor.doc, Point { x: 0., y: 0. });
}
