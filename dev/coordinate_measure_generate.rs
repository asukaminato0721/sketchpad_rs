#[path = "../src/coordinate_measure_records.rs"]
mod coordinate_measure_records;

use coordinate_measure_records::{CoordinateMeasureKind, encode_on_original_base};
use std::{env, fs};

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        return Err("Usage: coordinate_measure_generate coordinates|abscissa|ordinate|distance POINT_ID OUTPUT.gsp".into());
    }
    let kind = match args[1].as_str() {
        "coordinates" => CoordinateMeasureKind::Coordinates,
        "abscissa" => CoordinateMeasureKind::Abscissa,
        "ordinate" => CoordinateMeasureKind::Ordinate,
        "distance" => CoordinateMeasureKind::CoordinateDistance,
        _ => return Err("Unknown coordinate measurement kind".into()),
    };
    let point: u32 = args[2].parse().map_err(|_| "Invalid point ID")?;
    let references = if kind == CoordinateMeasureKind::CoordinateDistance {
        vec![point, 4, 9]
    } else {
        vec![point, 9]
    };
    fs::write(&args[3], encode_on_original_base(kind, &references)?).map_err(|e| e.to_string())?;
    Ok(())
}
