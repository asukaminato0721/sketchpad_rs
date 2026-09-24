//! Patch one observed style field in a GSP4 copy for original-app probes.
//!
//! Usage:
//!   style_record_probe point-rgb INPUT.gsp OUTPUT.gsp R G B
//!   style_record_probe circle-rgb INPUT.gsp OUTPUT.gsp R G B
//!   style_record_probe segment-line INPUT.gsp OUTPUT.gsp WIDTH PATTERN
//!   style_record_probe circle-line INPUT.gsp OUTPUT.gsp WIDTH PATTERN

#[path = "../src/style_records.rs"]
mod style_records;

use style_records::{
    LinePattern, LineWidth, RgbColor, SegmentLineStyle, circle_color, circle_line_style,
    point_color, segment_line_style, set_circle_color, set_circle_line_style,
    set_point_color, set_segment_line_style,
};

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 6 {
        return Err("Expected command, input, output, and style fields".into());
    }
    let mut bytes = std::fs::read(&args[2]).map_err(|e| e.to_string())?;
    if bytes.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 file".into());
    }
    let mut offset = 4;
    let mut changed = false;
    while offset + 8 <= bytes.len() {
        let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
        let start = offset + 8;
        let end = start.checked_add(length).ok_or("Record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated record".into());
        }
        if kind == 0x07d0 && !changed {
            let payload = &mut bytes[start..end];
            match args[1].as_str() {
                "point-rgb" if point_color(payload).is_some() && args.len() == 7 => {
                    let channel = |index: usize| args[index].parse::<u8>().map_err(|e| e.to_string());
                    set_point_color(payload, RgbColor {
                        red: channel(4)?,
                        green: channel(5)?,
                        blue: channel(6)?,
                    })?;
                    changed = true;
                }
                "circle-rgb" if circle_color(payload).is_some() && args.len() == 7 => {
                    let channel = |index: usize| args[index].parse::<u8>().map_err(|e| e.to_string());
                    set_circle_color(payload, RgbColor {
                        red: channel(4)?,
                        green: channel(5)?,
                        blue: channel(6)?,
                    })?;
                    changed = true;
                }
                "segment-line" | "circle-line"
                    if args.len() == 6
                        && ((args[1] == "segment-line" && segment_line_style(payload).is_some())
                            || (args[1] == "circle-line" && circle_line_style(payload).is_some())) => {
                    let width = match args[4].as_str() {
                        "hairline" => LineWidth::Hairline,
                        "thin" => LineWidth::Thin,
                        "medium" => LineWidth::Medium,
                        "thick" => LineWidth::Thick,
                        _ => return Err("Unknown line width".into()),
                    };
                    let pattern = match args[5].as_str() {
                        "solid" => LinePattern::Solid,
                        "dashed" => LinePattern::Dashed,
                        "dotted" => LinePattern::Dotted,
                        _ => return Err("Unknown line pattern".into()),
                    };
                    if args[1] == "segment-line" && segment_line_style(payload).is_some() {
                        set_segment_line_style(payload, SegmentLineStyle { width, pattern })?;
                    } else if args[1] == "circle-line" && circle_line_style(payload).is_some() {
                        set_circle_line_style(payload, SegmentLineStyle { width, pattern })?;
                    }
                    changed = true;
                }
                _ => {}
            }
        }
        offset = end + length % 2;
    }
    if !changed {
        return Err("No matching object header or invalid arguments".into());
    }
    std::fs::write(&args[3], bytes).map_err(|e| e.to_string())?;
    Ok(())
}
