#[path = "../src/value_equation_records.rs"]
mod value_equation_records;

use std::{env, fs};
use value_equation_records::{ValueEquationKind, append_to_base};

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 6 {
        return Err("Usage: value_equation_generate point|line|circle BASE.gsp FIRST_ID SECOND_ID OUTPUT.gsp".into());
    }
    let kind = match args[1].as_str() {
        "point" => ValueEquationKind::PointValue,
        "line" => ValueEquationKind::LineEquation,
        "circle" => ValueEquationKind::CircleEquation,
        _ => return Err("Unsupported Value/Equation kind".into()),
    };
    let first: u32 = args[3].parse().map_err(|_| "Invalid first ID")?;
    let second: u32 = args[4].parse().map_err(|_| "Invalid second ID")?;
    let base = fs::read(&args[2]).map_err(|e| e.to_string())?;
    fs::write(&args[5], append_to_base(&base, kind, [first, second])?)
        .map_err(|e| e.to_string())?;
    Ok(())
}
