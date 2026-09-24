#[path = "../src/table_records.rs"]
mod table_records;

use std::{env, fs};

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 5 {
        return Err("Usage: table_record_probe SOURCE.gsp OUTPUT.gsp X Y".into());
    }
    let source = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let x = args[3].parse::<i16>().map_err(|e| e.to_string())?;
    let y = args[4].parse::<i16>().map_err(|e| e.to_string())?;
    let output = table_records::append_default_parameter_table(&source, (x, y))?;
    fs::write(&args[2], output).map_err(|e| e.to_string())
}
