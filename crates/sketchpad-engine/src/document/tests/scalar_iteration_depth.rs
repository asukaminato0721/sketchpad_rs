use super::*;
#[test]
fn native_measured_iteration_depths_are_live_scalar_values_with_truncation() {
    for (name,iteration,binding,depth) in [("ratio",10,11,4),("path_value",11,12,5)] {
        let source=fs::read(format!("../../tests/fixtures/scalar_iteration_depth50_{name}_native_verified.gsp")).unwrap();
        let mut doc=Document::from_bytes(&source,None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(doc.objects[iteration-1].kind,Kind::MappedPointIteration{depth:IterationDepth::Value(id),..} | Kind::PointIteration(_,_,IterationDepth::Value(id)) if id==depth));
        assert_eq!(doc.numeric_value(depth),Some(3.5));
        let images=doc.mapped_point_binding_images(binding).unwrap();
        assert_eq!(images,vec![Point{x:240.,y:180.},Point{x:260.,y:180.},Point{x:280.,y:180.}]);
        assert_eq!(doc.encode().unwrap(),source);
        doc.checkpoint();
        assert!(doc.move_point(3,Point{x:360.,y:100.}));
        assert_eq!(doc.numeric_value(depth),Some(2.6));
        assert_eq!(doc.mapped_point_binding_images(binding).unwrap().len(),2);
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(),source);
        doc.redo();
        assert_eq!(doc.mapped_point_binding_images(binding).unwrap().len(),2);
        doc.checkpoint();
        assert!(doc.move_point(3,Point{x:100.,y:100.}));
        assert!(doc.mapped_point_binding_images(binding).is_none());
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.mapped_point_binding_images(binding).unwrap().len(),2);
    }
}

#[test]
fn measured_scalar_depth_authoring_preserves_units_and_dependency_remapping() {
    let mut doc=Document::new();
    let unrelated=doc.add(Kind::Free(Point{x:10.,y:10.}));
    let a=doc.add(Kind::Free(Point{x:100.,y:100.}));
    let b=doc.add(Kind::Free(Point{x:200.,y:100.}));
    let target=doc.add(Kind::Free(Point{x:450.,y:100.}));
    let ratio=doc.add(Kind::PointRatio(a,b,target));
    let length=doc.add(Kind::PointDistance(a,target));
    let image=doc.add(Kind::VectorTranslatedPoint(a,a,b));
    let iteration=doc.add_mapped_point_iteration_with_bindings(&[a],&[vec![image]],&[image],&[image],IterationDepth::Value(ratio)).unwrap();
    assert_eq!(doc.mapped_point_binding_images(iteration+1).unwrap().len(),3);
    let count=doc.objects.len();
    assert!(doc.add_mapped_point_iteration_with_bindings(&[a],&[vec![image]],&[image],&[image],IterationDepth::Value(length)).is_err());
    assert_eq!(doc.objects.len(),count);
    doc.verify_roundtrip().unwrap();
    doc.selected.insert(unrelated);
    doc.clear_selected().unwrap();
    doc.verify_roundtrip().unwrap();
    let bytes=doc.encode().unwrap();
    let reopened=Document::from_bytes(&bytes,None).unwrap();
    assert!(!reopened.partial_view);
    let binding=reopened.objects.last().unwrap().id;
    assert_eq!(reopened.mapped_point_binding_images(binding).unwrap().len(),3);
    doc.selected.clear();
    doc.selected.insert(ratio);
    doc.clear_selected().unwrap();
    assert!(doc.objects[iteration-1].deleted);
    assert!(doc.objects[iteration].deleted);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(),bytes);
}
