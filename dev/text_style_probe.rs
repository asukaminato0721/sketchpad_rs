#[path = "../src/text_style_records.rs"]
mod text_style_records;

use std::{env, fs};
use text_style_records::{VerifiedHelloStyle, style_verified_hello};

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [source, output, style] = args.as_slice() else {
        return Err("Usage: text_style_probe SOURCE.gsp OUTPUT.gsp bold_off|italic|underline|larger|cjk_font".into());
    };
    let style = match style.as_str() {
        "bold_off" => VerifiedHelloStyle::BoldOff,
        "italic" => VerifiedHelloStyle::ItalicOn,
        "underline" => VerifiedHelloStyle::UnderlineOn,
        "larger" => VerifiedHelloStyle::IncreaseSize,
        "cjk_font" => VerifiedHelloStyle::NotoSansCjkJp,
        _ => return Err("Unknown verified style".into()),
    };
    let source = fs::read(source).map_err(|e| e.to_string())?;
    let encoded = style_verified_hello(&source, style)?;
    fs::write(output, encoded).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
