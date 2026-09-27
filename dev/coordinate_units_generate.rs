//! Reproduce Native-verified numeric coordinate units and literal points.
//! cargo build --lib
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/coordinate_units_generate.rs -o /tmp/coordinate_units_generate
//! /tmp/coordinate_units_generate /tmp/coordinate_units
use sketchpad_rs::{
    coordinate_measure_records::CoordinateMeasureKind,
    document::{Document, Kind, Point},
    plot_points_records::PlotPointsMode,
};

fn coordinate_units_document(length: f64, y_expression: &str, sources: [usize; 2]) -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../tests/fixtures/numeric_typed_parameters_native_verified.gsp"),
        None,
    )
    .unwrap();
    d.set_parameter_value(2, length).unwrap();
    d.set_parameter_value(3, 2.).unwrap();
    let origin = d.add(Kind::Free(Point { x: 280., y: 300. }));
    let _x = d
        .add_multi_calculation(&[2, 3], "v1*v2", (10, 110))
        .unwrap();
    let _y = d
        .add_calculation_expression(y_expression, (10, 140))
        .unwrap();
    let frame = d
        .add_coordinate_system_from_distances(Some(origin), &sources, Point { x: 0., y: 0. })
        .unwrap();
    let point = d
        .plot_points_at(PlotPointsMode::Cartesian, [1., 1.])
        .unwrap();
    for (kind, anchor) in [
        (CoordinateMeasureKind::Abscissa, (10, 180)),
        (CoordinateMeasureKind::Ordinate, (10, 210)),
    ] {
        let id = d.add(Kind::CoordinateMeasurement {
            kind,
            first: point,
            second: None,
            frame,
        });
        d.objects[id - 1].measurement_anchor = Some(anchor);
    }
    d
}

fn write(document: &Document, prefix: &str, state: &str) {
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
fn main() {
    let prefix = std::env::args().nth(1).expect("output prefix");
    let mut document = coordinate_units_document(1., "3 cm", [5, 6]);
    write(&document, &prefix, "original");
    document.set_parameter_value(2, 1.5).unwrap();
    document.set_calculation_expression(6, "2.5 cm").unwrap();
    write(&document, &prefix, "edited");
    document.set_calculation_expression(6, "-2.5 cm").unwrap();
    write(&document, &prefix, "negative");
    document.set_calculation_expression(6, "0 cm").unwrap();
    write(&document, &prefix, "zero");
    document.set_calculation_expression(6, "2.5 cm").unwrap();
    document.set_point_label(10, "P", true).unwrap();
    write(&document, &prefix, "restored");
    for (state, expression, sources) in [
        ("scalar", "2.5 cm", [3, 6]),
        ("degree", "2.5 cm", [1, 6]),
        ("area", "(2 cm)^2", [6, 5]),
        ("higher", "(sqrt(2)*1 cm)^4", [6, 5]),
    ] {
        write(
            &coordinate_units_document(1.5, expression, sources),
            &prefix,
            state,
        );
    }
}
