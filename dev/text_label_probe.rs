//! Add one canonical point label to a copied GSP file for cross-app probing.
//! Compile with `rustc --edition 2024 dev/text_label_probe.rs -o /tmp/text_label_probe`.

#[path = "../src/text_records.rs"]
mod text_records;

use std::{env, fs};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("Usage: text_label_probe INPUT.gsp OUTPUT.gsp LABEL".into());
    }
    let bytes = fs::read(&args[1]).map_err(|error| error.to_string())?;
    if bytes.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut at = 4;
    let mut insertion = None;
    while at + 8 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
        let end = at.checked_add(8 + len + len % 2).ok_or("Record overflow")?;
        if end > bytes.len() {
            return Err("Truncated record".into());
        }
        if kind == 0x07d0 && len >= 2 && bytes[at + 8..at + 10] == [0, 0] {
            insertion = Some(end);
            break;
        }
        at = end;
    }
    let insertion = insertion.ok_or("No point object found")?;
    let label = text_records::LabelAux::new_point(&args[3], true, (8, 5))?;
    let records = text_records::encode_point_label_records(&label);
    let mut result = Vec::with_capacity(bytes.len() + records.len());
    result.extend_from_slice(&bytes[..insertion]);
    result.extend_from_slice(&records);
    result.extend_from_slice(&bytes[insertion..]);
    fs::write(&args[2], result).map_err(|error| error.to_string())
}
