//! Exercise the contextual function record writer against native-created groups.
//! rustc --edition=2021 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/function_reference_generate.rs -o /tmp/function_reference_generate
use sketchpad_rs::{
    document::{records, Document},
    function_records::{encode_expression_record_with_sources, SimpleFunction},
};
fn main() {
    let mut bytes =
        include_bytes!("../tests/fixtures/function_reference_controls_native_created.gsp").to_vec();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let rs = records(&bytes).unwrap();
    let mut edits = Vec::new();
    for (id, expression) in [(15, "v1*x^2-1"), (17, "f1(x-2)+3")] {
        let offset = d.objects[id - 1].header_offset.unwrap();
        let start = rs
            .iter()
            .position(|r| r.kind == 0x07d0 && r.data.start == offset)
            .unwrap();
        let end = (start..rs.len()).find(|&i| rs[i].kind == 0x07d7).unwrap();
        let r = rs[start..end].iter().find(|r| r.kind == 0x0907).unwrap();
        let replacement = encode_expression_record_with_sources(
            &bytes[r.data.clone()],
            SimpleFunction::parse_with_sources(expression, 1).unwrap(),
            1,
        )
        .unwrap();
        edits.push((r.start, r.data.end + r.data.len() % 2, replacement));
    }
    for (start, end, replacement) in edits.into_iter().rev() {
        bytes.splice(start..end, replacement);
    }
    Document::from_bytes(&bytes, None)
        .unwrap()
        .verify_roundtrip()
        .unwrap();
    let output = std::env::args().nth(1).expect("output path");
    std::fs::write(output, &bytes).unwrap();
    println!("{} bytes", bytes.len());
}
