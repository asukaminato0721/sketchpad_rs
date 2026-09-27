//! Read-only evaluator coordinates for the paired original arc-based mapping.
use sketchpad_rs::document::Document;
fn main() {
    let path = std::env::args().nth(1).expect("gsp");
    let bytes = std::fs::read(path).unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(d.encode().unwrap(), bytes);
    for id in [1, 2, 3, 25, 26, 29, 30, 31, 32, 33, 34, 35, 49, 50] {
        println!(
            "ID {id} {:?} pos {:?} numeric {:?}",
            d.objects[id - 1].kind,
            d.position(id),
            d.numeric_value(id)
        );
    }
    println!("arc25 {:?}", d.arc_points(25, 8));
    println!("binding50 {:?}", d.branch_segment_iteration_images(50));
}
