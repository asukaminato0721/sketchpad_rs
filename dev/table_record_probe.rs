#[path = "../src/table_records.rs"]
mod table_records;

use std::{env, fs};

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    let (mode, source, destination, coordinates) = match args.as_slice() {
        [_, mode, source, destination, x, y] => (mode.as_str(), source, destination, Some((x, y))),
        [_, mode, source, destination] if mode == "calculate" =>
            (mode.as_str(), source, destination, None),
        _ => return Err("Usage: table_record_probe table|parameter SOURCE.gsp OUTPUT.gsp X Y; or calculate SOURCE.gsp OUTPUT.gsp".into()),
    };
    let source = fs::read(source).map_err(|e| e.to_string())?;
    let anchor = coordinates
        .map(|(x, y)| -> Result<(i16, i16), String> {
            Ok((
                x.parse::<i16>().map_err(|e| e.to_string())?,
                y.parse::<i16>().map_err(|e| e.to_string())?,
            ))
        })
        .transpose()?;
    let output = match mode {
        "table" => table_records::append_default_parameter_table(
            &source,
            anchor.ok_or("Missing table anchor")?,
        )?,
        "parameter" => table_records::append_default_parameter(
            &source,
            anchor.ok_or("Missing parameter anchor")?,
        )?,
        "calculate" => table_records::append_literal_sum(&source)?,
        _ => return Err("Unknown mode".into()),
    };
    fs::write(destination, output).map_err(|e| e.to_string())
}
