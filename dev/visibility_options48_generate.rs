//! Ordinary point dependencies for isolated original visibility-button controls.
use sketchpad_rs::document::{Document, Kind, Point};
fn main() {
    let mut document = Document::new();
    document.add(Kind::Free(Point { x: 140., y: 200. }));
    let hidden = document.add(Kind::Free(Point { x: 240., y: 240. }));
    document.objects[hidden - 1].visible = false;
    document.add(Kind::Free(Point { x: 340., y: 200. }));
    let bytes = document.encode().unwrap();
    document.verify_roundtrip().unwrap();
    std::fs::write("/tmp/visibility48_base.gsp", bytes).unwrap();
}
