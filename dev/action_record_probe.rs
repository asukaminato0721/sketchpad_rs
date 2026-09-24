//! Standalone probe: append one English Hide/Show button to a three-object GSP.
//! Compile with: rustc --edition=2024 dev/action_record_probe.rs -o /tmp/action_record_probe

#[path = "../src/action_records.rs"]
mod action_records;

use action_records::HideShowButtonGroup;
use std::{env, fs};

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("Usage: action_record_probe BASELINE.gsp OUTPUT.gsp".into());
    }
    let mut bytes = fs::read(&args[1]).map_err(|error| error.to_string())?;
    if !bytes.starts_with(b"GSP4") {
        return Err("Expected GSP4 file".into());
    }
    let mut offset = 4;
    let mut page_count_offset = None;
    let mut insert_at = None;
    let mut objects = 0;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err("Truncated GSP record header".into());
        }
        let len = u32_at(&bytes, offset) as usize;
        let kind = u32_at(&bytes, offset + 4);
        let end = offset
            .checked_add(8 + len + len % 2)
            .ok_or("GSP length overflow")?;
        if end > bytes.len() {
            return Err("Truncated GSP record payload".into());
        }
        if kind == 0x03e8 {
            if page_count_offset.is_some() || len < 28 {
                return Err("Expected exactly one page".into());
            }
            page_count_offset = Some(offset + 8 + 24);
        }
        if kind == 0x07d0 {
            objects += 1;
        }
        if kind == 0x03e9 && insert_at.is_none() {
            insert_at = Some(offset);
        }
        offset = end;
    }
    let page_count_offset = page_count_offset.ok_or("No page preamble")?;
    if objects != 3 || u32_at(&bytes, page_count_offset) != 3 {
        return Err("Probe baseline must contain exactly three objects".into());
    }
    let insert_at = insert_at.ok_or("No page trailer")?;
    let group = HideShowButtonGroup::from_original_template(3, "Hide Segment", (10, 5))?;
    bytes[page_count_offset..page_count_offset + 4].copy_from_slice(&4_u32.to_le_bytes());
    bytes.splice(insert_at..insert_at, group.encode());
    fs::write(&args[2], bytes).map_err(|error| error.to_string())?;
    Ok(())
}
