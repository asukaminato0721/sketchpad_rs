//! Reproduce the native-verified 38-object referenced-function API outputs.
//! cargo build --lib
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/function_references_generate.rs -o /tmp/function_references_generate
//! /tmp/function_references_generate /tmp/function_references
use sketchpad_rs::{
    document::{Document, IterationDepth, Kind, Point},
    function_records::SimpleFunction,
};

fn function_references_writer_document() -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    let f = d
        .add_referenced_function_definition(
            &[6, 5],
            SimpleFunction::parse_with_sources("v1*x^2+v2", 2).unwrap(),
        )
        .unwrap();
    let g = d
        .add_referenced_function_definition(
            &[f, 6],
            SimpleFunction::parse_with_sources("f1(x+v2)", 2).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[g, 6], "f1(v2)", (300, 150))
        .unwrap();
    d.plot_function(f, Point { x: 520., y: 240. }).unwrap();
    let h = d
        .add_referenced_function_definition(
            &[6, f, 5],
            SimpleFunction::parse_with_sources("v1*f2(x)-v3", 3).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[h, 6], "f1(v2)", (300, 176))
        .unwrap();
    let hp = d.plot_function(h, Point { x: 520., y: 240. }).unwrap();
    d.objects[hp - 1].visible = false;
    let a = d
        .add_referenced_function_definition(
            &[8],
            SimpleFunction::parse_with_sources("v1", 1).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[a, 6], "f1(v2)", (300, 202))
        .unwrap();
    let b = d
        .add_referenced_function_definition(
            &[8],
            SimpleFunction::parse_with_sources("sin(v1)", 1).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[b, 6], "f1(v2)", (300, 228))
        .unwrap();
    let k = d
        .add_referenced_function_definition(
            &[6],
            SimpleFunction::parse_with_sources("1/(x-v1)", 1).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[k, 6], "f1(v2+1)", (300, 254))
        .unwrap();
    let kp = d.plot_function(k, Point { x: 520., y: 240. }).unwrap();
    d.objects[kp - 1].visible = false;
    let j = d
        .add_referenced_function_definition(
            &[k],
            SimpleFunction::parse_with_sources("f1(x+1)", 1).unwrap(),
        )
        .unwrap();
    let jp = d.plot_function(j, Point { x: 520., y: 240. }).unwrap();
    d.objects[jp - 1].visible = false;
    let c = d
        .add_referenced_function_definition(
            &[6],
            SimpleFunction::parse_with_sources("v1*x+1", 1).unwrap(),
        )
        .unwrap();
    let calc = d
        .add_multi_calculation(&[c, 6], "f1(v2)", (300, 280))
        .unwrap();
    let image = d.add(Kind::PolarTranslatedPoint(
        2,
        -90.,
        37.79527559055118,
        1.5875,
    ));
    let label = d.add_point_numeric_label(image, calc).unwrap();
    d.add_point_parameter_iteration(2, 6, image, calc, label, IterationDepth::Fixed(3))
        .unwrap();
    for (i, (id, name)) in [
        (f, "f"),
        (g, "g"),
        (h, "h"),
        (a, "a"),
        (b, "b"),
        (k, "k"),
        (j, "j"),
        (c, "c"),
    ]
    .into_iter()
    .enumerate()
    {
        d.objects[id - 1].function_anchor = Some((10, 150 + 26 * i as i16));
        d.objects[id - 1].function_name = Some(name.to_owned());
    }
    d
}

fn main() {
    let prefix = std::env::args().nth(1).expect("output path prefix");
    let mut document = function_references_writer_document();
    for state in ["original", "edited"] {
        if state == "edited" {
            document.set_parameter_value(6, -1.).unwrap();
            document
                .set_function_expression(
                    9,
                    SimpleFunction::parse_with_sources("v1*x^3+v2", 2).unwrap(),
                )
                .unwrap();
            document
                .set_function_expression_with_angle_unit(
                    24,
                    SimpleFunction::parse_with_sources("sin(v1)", 1).unwrap(),
                    false,
                )
                .unwrap();
        }
        document.verify_roundtrip().unwrap();
        let bytes = document.encode().unwrap();
        let path = format!("{prefix}_{state}.gsp");
        std::fs::write(&path, &bytes).unwrap();
        println!(
            "{path}: {} objects, {} bytes",
            document.objects.len(),
            bytes.len()
        );
    }
}
