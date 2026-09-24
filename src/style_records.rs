//! Point-style field in a Sketchpad 5 object header (`0x07d0`).
//!
//! Changing the style of a selected free point in the original application
//! changes payload byte 11. The same field occurs in 16- and 28-byte headers.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PointStyle {
    Dot = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
}

impl PointStyle {
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Dot),
            1 => Some(Self::Small),
            2 => Some(Self::Medium),
            3 => Some(Self::Large),
            _ => None,
        }
    }
}

fn is_point_header(payload: &[u8]) -> bool {
    (payload.len() == 16 || payload.len() == 28)
        && u16::from_le_bytes([payload[0], payload[1]]) == 0
}

pub fn point_style(payload: &[u8]) -> Option<PointStyle> {
    is_point_header(payload).then(|| PointStyle::from_byte(payload[11]))?
}

pub fn set_point_style(payload: &mut [u8], style: PointStyle) -> Result<(), String> {
    if !is_point_header(payload) {
        return Err("Expected a point object header".into());
    }
    payload[11] = style as u8;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub fn point_color(payload: &[u8]) -> Option<PointColor> {
    is_point_header(payload).then_some(PointColor {
        red: payload[12],
        green: payload[13],
        blue: payload[14],
    })
}

pub fn set_point_color(payload: &mut [u8], color: PointColor) -> Result<(), String> {
    if !is_point_header(payload) {
        return Err("Expected a point object header".into());
    }
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LineWidth {
    Hairline = 0,
    Thin = 1,
    Medium = 2,
    Thick = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LinePattern {
    Dotted = 0,
    Dashed = 1,
    Solid = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentLineStyle {
    pub width: LineWidth,
    pub pattern: LinePattern,
}

fn is_segment_header(payload: &[u8]) -> bool {
    (payload.len() == 16 || payload.len() == 28)
        && u16::from_le_bytes([payload[0], payload[1]]) == 2
}

pub fn segment_line_style(payload: &[u8]) -> Option<SegmentLineStyle> {
    if !is_segment_header(payload) {
        return None;
    }
    let width = match payload[10] & 0x0f {
        0 => LineWidth::Hairline,
        1 => LineWidth::Thin,
        2 => LineWidth::Medium,
        3 => LineWidth::Thick,
        _ => return None,
    };
    let pattern = match payload[10] >> 4 {
        0 => LinePattern::Dotted,
        1 => LinePattern::Dashed,
        2 => LinePattern::Solid,
        _ => return None,
    };
    Some(SegmentLineStyle { width, pattern })
}

pub fn set_segment_line_style(
    payload: &mut [u8],
    style: SegmentLineStyle,
) -> Result<(), String> {
    if !is_segment_header(payload) {
        return Err("Expected a segment object header".into());
    }
    payload[10] = (style.pattern as u8) << 4 | style.width as u8;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_point_header(bytes: &[u8]) -> &[u8] {
        assert_eq!(&bytes[..4], b"GSP4");
        let mut offset = 4;
        while offset < bytes.len() {
            let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let payload = &bytes[offset + 8..offset + 8 + length];
            if kind == 0x07d0 && is_point_header(payload) {
                return payload;
            }
            offset += 8 + length + length % 2;
        }
        panic!("fixture has no point header");
    }

    #[test]
    fn original_menu_variants_match_header_byte() {
        let fixtures: &[(&[u8], PointStyle)] = &[
            (
                include_bytes!("../tests/fixtures/style_dot_point.gsp"),
                PointStyle::Dot,
            ),
            (
                include_bytes!("../tests/fixtures/style_small_point.gsp"),
                PointStyle::Small,
            ),
            (
                include_bytes!("../tests/fixtures/style_large_point.gsp"),
                PointStyle::Large,
            ),
        ];
        for (bytes, style) in fixtures {
            let payload = first_point_header(bytes);
            assert_eq!(point_style(payload), Some(*style));
        }
        let default = first_point_header(include_bytes!("../tests/fixtures/point.gsp"));
        assert_eq!(point_style(default), Some(PointStyle::Medium));
    }

    #[test]
    fn editing_a_style_preserves_every_other_header_byte() {
        let original = first_point_header(include_bytes!("../tests/fixtures/point.gsp"));
        let mut changed = original.to_vec();
        set_point_style(&mut changed, PointStyle::Large).unwrap();
        assert_eq!(point_style(&changed), Some(PointStyle::Large));
        changed[11] = original[11];
        assert_eq!(changed, original);
    }

    #[test]
    fn original_point_color_menu_writes_rgb_bytes() {
        let red = first_point_header(include_bytes!("../tests/fixtures/point.gsp"));
        let orange = first_point_header(include_bytes!("../tests/fixtures/style_orange_point.gsp"));
        let blue = first_point_header(include_bytes!("../tests/fixtures/style_blue_point.gsp"));
        assert_eq!(point_color(red), Some(PointColor { red: 255, green: 0, blue: 0 }));
        assert_eq!(point_color(orange), Some(PointColor { red: 255, green: 128, blue: 0 }));
        assert_eq!(point_color(blue), Some(PointColor { red: 0, green: 0, blue: 255 }));
        let mut edited = red.to_vec();
        set_point_color(&mut edited, PointColor { red: 0, green: 0, blue: 255 }).unwrap();
        assert_eq!(point_color(&edited), point_color(blue));
        edited[12..15].copy_from_slice(&red[12..15]);
        assert_eq!(edited, red);
    }

    #[test]
    fn non_point_and_unknown_styles_are_rejected() {
        assert_eq!(point_style(&[]), None);
        let mut header = first_point_header(include_bytes!("../tests/fixtures/point.gsp")).to_vec();
        header[11] = 4;
        assert_eq!(point_style(&header), None);
        header[0] = 1;
        assert!(set_point_style(&mut header, PointStyle::Dot).is_err());
    }

    #[test]
    fn original_segment_menu_variants_match_header_byte() {
        use LinePattern::{Dashed, Dotted, Solid};
        use LineWidth::{Hairline, Medium, Thick, Thin};
        let fixtures: &[(&[u8], LineWidth, LinePattern)] = &[
            (include_bytes!("../tests/fixtures/style_medium_solid_line.gsp"), Medium, Solid),
            (include_bytes!("../tests/fixtures/style_hairline_line.gsp"), Hairline, Solid),
            (include_bytes!("../tests/fixtures/style_thin_line.gsp"), Thin, Solid),
            (include_bytes!("../tests/fixtures/style_thick_line.gsp"), Thick, Solid),
            (include_bytes!("../tests/fixtures/style_dashed_line.gsp"), Medium, Dashed),
            (include_bytes!("../tests/fixtures/style_dotted_line.gsp"), Medium, Dotted),
        ];
        for (bytes, width, pattern) in fixtures {
            let payload = first_segment_header(bytes);
            assert_eq!(segment_line_style(payload), Some(SegmentLineStyle { width: *width, pattern: *pattern }));
        }
    }

    fn first_segment_header(bytes: &[u8]) -> &[u8] {
        assert_eq!(&bytes[..4], b"GSP4");
        let mut offset = 4;
        while offset < bytes.len() {
            let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let payload = &bytes[offset + 8..offset + 8 + length];
            if kind == 0x07d0 && is_segment_header(payload) {
                return payload;
            }
            offset += 8 + length + length % 2;
        }
        panic!("fixture has no segment header");
    }

    #[test]
    fn editing_segment_style_preserves_other_header_bytes() {
        let original = first_segment_header(include_bytes!("../tests/fixtures/style_medium_solid_line.gsp"));
        let mut changed = original.to_vec();
        set_segment_line_style(&mut changed, SegmentLineStyle {
            width: LineWidth::Thick,
            pattern: LinePattern::Dashed,
        }).unwrap();
        assert_eq!(changed[10], 0x13);
        changed[10] = original[10];
        assert_eq!(changed, original);
    }
}
