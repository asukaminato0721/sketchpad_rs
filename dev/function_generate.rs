#[path = "../src/function_records.rs"]
mod function_records;

use function_records::{SimpleFunction, encode_minimal_plot_document};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let expression = match args.next().as_deref() {
        Some("x") => SimpleFunction::Identity,
        Some("x^2") => SimpleFunction::Square,
        _ => return Err("Usage: function_generate x|x^2 OUTPUT.gsp".into()),
    };
    let output = args.next().ok_or("Missing output path")?;
    if args.next().is_some() {
        return Err("Too many arguments".into());
    }
    let bytes = encode_minimal_plot_document(expression)?;
    std::fs::write(output, bytes).map_err(|error| error.to_string())
}
