//! Register only for the bounded SampledMapped→mapped-collection implementation.
use super::*;
fn fixture(state: &str) -> (Vec<u8>, Document) {
    let bytes = fs::read(format!("../../tests/fixtures/sampled_locus68_{state}_native_saved.gsp")).unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    (bytes, doc)
}
const ORIGINAL_SOURCE: [(f64,f64);5]=[(725.5154,259.2123),(742.5673,190.0060),(839.1421,206.8075),(790.1980,299.3779),(725.5154,259.2123)];
const EDITED_SOURCE: [(f64,f64);5]=[(578.7420,283.2624),(586.7256,239.7456),(650.3283,245.6018),(619.9659,308.0834),(578.7420,283.2624)];
#[test]
fn native_original_sampled_collection_has_typed_source_circle_and_green_binding() {
    for (state,source_expected) in [("original",ORIGINAL_SOURCE),("edited",EDITED_SOURCE)] {
        let (bytes,doc)=fixture(state);
        assert_eq!(bytes.len(),5304);
        // The source sketch retains unrelated partial-view restrictions even
        // after the sampled/iteration/binding roles are decoded.
        assert!(doc.partial_view,"{state}");
        assert!(matches!(doc.objects[18].kind,Kind::SampledMapped(..)));
        assert!(matches!(doc.objects[20].kind,Kind::MappedPointIteration { .. }));
        assert!(matches!(doc.objects[21].kind,Kind::IterationBinding(17,21)));
        assert!(matches!(doc.objects[22].kind,Kind::IterationBinding(19,21)));
        assert_eq!(doc.numeric_value(12),Some(10.));
        assert_eq!(doc.position(9),Some(if state=="original" {Point{x:495.,y:317.}} else {Point{x:575.,y:317.}}));
        for (i,(x,y)) in source_expected.into_iter().enumerate(){
            let point=doc.sampled_mapped_point(19,i as f64/4.).unwrap();
            assert!((point.x-x).abs()<0.00006&&(point.y-y).abs()<0.00006,"source19 only {state} t{i}: {point:?}");
        }
        let source=&doc.objects[18];let binding=&doc.objects[22];
        assert!(!source.visible,"Sampled source19 is hidden; binding23 paints separately");
        assert!(binding.visible);
        assert_eq!(doc.effective_visual_style(23),Some(VisualStyle::Stroke {
            line: SegmentLineStyle { width: LineWidth::Thin, pattern: LinePattern::Solid },
            color: RgbColor { red:0, green:128, blue:0 },
        }));
        // The two native77 groups retain distinct source IDs even though their
        // stored stroke bytes agree. The Native navy pixels require a separate
        // visibility discriminator before assigning them to binding22.
        assert_eq!(doc.effective_visual_style(22),doc.effective_visual_style(23));
        let members=doc.mapped_collection_strokes(23,8).unwrap();
        assert!(!members.is_empty(),"{state}: Native paints multiple green members");
        assert!(members.iter().all(|s|s.mode==LineMode::Segment&&s.start.x.is_finite()&&s.start.y.is_finite()&&s.end.x.is_finite()&&s.end.y.is_finite()));
        assert_eq!(doc.encode().unwrap(),bytes,"{state}");
        doc.verify_roundtrip().unwrap();
    }
}
#[test]
fn native_public_edit_moves_both_collection_roles_without_rewriting_source_packets() {
    let (original,mut doc)=fixture("original");
    let (edited,expected)=fixture("edited");
    doc.checkpoint();
    assert!(doc.move_point(9,Point{x:575.,y:317.}));
    assert_eq!(doc.encode().unwrap(),edited,"public point edit reproduces exact Native-edited input/save");
    let new=doc.mapped_collection_strokes(23,8).unwrap();
    assert_eq!(new,expected.mapped_collection_strokes(23,8).unwrap());
    assert_ne!(new,Document::from_bytes(&original,None).unwrap().mapped_collection_strokes(23,8).unwrap());
    assert_eq!(doc.position(9),Some(Point{x:575.,y:317.}));
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(),original);
}
#[test]
fn public_sampled_curve_collection_authoring_preserves_outer_scope_and_exact_reopen() {
    let mut doc=Document::new();
    let center=doc.add(Kind::Free(Point{x:300.,y:300.}));
    let radius=doc.add(Kind::Free(Point{x:340.,y:300.}));
    let circle=doc.add(Kind::Circle(center,radius));
    let seed=doc.add(Kind::Free(Point{x:100.,y:180.}));
    let zero=doc.add(Kind::Free(Point{x:0.,y:0.}));
    let plus20=doc.add(Kind::Free(Point{x:20.,y:0.}));
    let output=doc.add(Kind::VectorTranslatedPoint(seed,zero,plus20));
    let source=doc.add_sampled_mapped(output,circle,seed,&[output],40).unwrap();
    let plus100=doc.add(Kind::Free(Point{x:100.,y:0.}));
    let next=doc.add(Kind::VectorTranslatedPoint(center,zero,plus100));
    let root=doc.add_mapped_point_iteration_with_bindings(
        &[center],&[vec![next]],&[circle,output,source,next],&[source],IterationDepth::Fixed(3),
    ).unwrap();
    let binding=root+1;
    let strokes=doc.mapped_collection_strokes(binding,12).unwrap();
    assert_eq!(strokes.len(),36,"three outer states, twelve chords each");
    assert_eq!(doc.position(center),Some(Point{x:300.,y:300.}));
    assert_eq!(doc.sampled_mapped_point(source,0.),Some(Point{x:360.,y:300.}));
    let bytes=doc.encode().unwrap();
    let reopened=Document::from_bytes(&bytes,None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.mapped_collection_strokes(binding,12),Some(strokes));
    assert_eq!(reopened.encode().unwrap(),bytes);
    reopened.verify_roundtrip().unwrap();
}
