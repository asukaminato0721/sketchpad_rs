//! Generate a free-text Sketchpad file from an original blank template.
//! Compile with `rustc --edition 2024 dev/text_object_generate.rs -o /tmp/text_object_generate`.

#[path = "../src/text_object.rs"]
mod text_object;

use std::{env, fs};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("Usage: text_object_generate BLANK.gsp OUTPUT.gsp TEXT".into());
    }
    let blank = fs::read(&args[1]).map_err(|error| error.to_string())?;
    let rect = text_object::TextRect::roomy_at(91, 81, &args[3])?;
    let output = text_object::add_to_blank_document(&blank, &args[3], rect)?;
    fs::write(&args[2], output).map_err(|error| error.to_string())
}
