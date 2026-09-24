//! Replace a simple literal 0x08fc text run in a copied GSP file.
//! Compile with `rustc --edition 2024 dev/text_plain_probe.rs -o /tmp/text_plain_probe`.

#[path = "../src/text_records.rs"]
mod text_records;

use std::{env, fs};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("Usage: text_plain_probe INPUT.gsp OUTPUT.gsp NEW_TEXT".into());
    }
    let bytes = fs::read(&args[1]).map_err(|error| error.to_string())?;
    if bytes.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut at = 4;
    let mut replacement = None;
    while at + 8 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
        let end = at.checked_add(8 + len + len % 2).ok_or("Record overflow")?;
        if end > bytes.len() {
            return Err("Truncated record".into());
        }
        if kind == text_records::RICH_TEXT_KIND {
            let mut text = text_records::RichTextPayload::parse(&bytes[at + 8..at + 8 + len])?;
            if text.simple_text().is_some() {
                text.replace_simple_text(&args[3])?;
                let mut record = Vec::new();
                record.extend_from_slice(&(text.payload().len() as u32).to_le_bytes());
                record.extend_from_slice(&kind.to_le_bytes());
                record.extend_from_slice(text.payload());
                if text.payload().len() % 2 != 0 {
                    record.push(0);
                }
                replacement = Some((at, end, record));
                break;
            }
        }
        at = end;
    }
    let (start, end, record) = replacement.ok_or("No simple literal text record found")?;
    let mut result = Vec::with_capacity(bytes.len() + record.len());
    result.extend_from_slice(&bytes[..start]);
    result.extend_from_slice(&record);
    result.extend_from_slice(&bytes[end..]);
    fs::write(&args[2], result).map_err(|error| error.to_string())
}
