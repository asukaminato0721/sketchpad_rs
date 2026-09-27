//! Read actual Native saved source fields, coordinates, and numeric values.
use sketchpad_rs::document::{Document, Kind};
fn main() {
    for path in std::env::args().skip(1) {
        let d = Document::open(std::path::Path::new(&path)).unwrap();
        println!("FILE {path} partial {}", d.partial_view);
        for o in &d.objects {
            let kind = if let Kind::Parameter(g) = &o.kind {
                format!("Parameter({:?}, {:?})", g.value(), o.kind.numeric_unit())
            } else {
                format!("{:?}", o.kind)
            };
            println!(
                "{} {} p {:?} numeric {:?}",
                o.id,
                kind,
                d.position(o.id),
                d.numeric_value(o.id)
            );
        }
    }
}
