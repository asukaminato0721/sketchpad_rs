//! Point-style field in a Sketchpad 5 object header (`0x07d0`).
//!
//! Changing the style of a selected free point in the original application
//! changes payload byte 11 in modern headers. The observed legacy 16-byte
//! free-point layout has implicit Small size and no editable size field.

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
    if payload.len() != 16 && payload.len() != 28 {
        return false;
    }
    let code = u16::from_le_bytes([payload[0], payload[1]]);
    let subtype = u16::from_le_bytes([payload[8], payload[9]]);
    matches!(
        (code, subtype),
        (
            0 | 75 | 88 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34,
            4
        ) | (0, 41)
            | (1, 11)
            | (15, 10)
            | (101, 89)
            | (9 | 11 | 12 | 13 | 14 | 93 | 98, 45)
            | (67 | 68 | 69 | 95 | 109 | 110 | 123, 56)
    ) || (payload.len() == 28 && matches!(code, 16 | 33) && subtype == 89 && payload[6..8] == [1, 0])
}

fn is_legacy_free_point_header(payload: &[u8]) -> bool {
    payload.len() == 16
        && payload[..2] == [0, 0]
        && payload[6..10] == [1, 0, 4, 0]
        && payload[10..12] == [1, 0]
}

pub fn point_style(payload: &[u8]) -> Option<PointStyle> {
    if is_legacy_free_point_header(payload) {
        return Some(PointStyle::Small);
    }
    is_point_header(payload).then(|| PointStyle::from_byte(payload[11]))?
}

pub fn set_point_style(payload: &mut [u8], style: PointStyle) -> Result<(), String> {
    if !is_point_header(payload) {
        return Err("Expected a point object header".into());
    }
    if is_legacy_free_point_header(payload) {
        return if style == PointStyle::Small {
            Ok(())
        } else {
            Err("Legacy free-point headers do not store point size".into())
        };
    }
    payload[11] = style as u8;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub fn point_color(payload: &[u8]) -> Option<RgbColor> {
    is_point_header(payload).then(|| RgbColor {
        red: payload[12],
        green: payload[13],
        blue: payload[14],
    })
}

pub fn set_point_color(payload: &mut [u8], color: RgbColor) -> Result<(), String> {
    if !is_point_header(payload) {
        return Err("Expected a point object header".into());
    }
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
}

pub fn segment_color(payload: &[u8]) -> Option<RgbColor> {
    is_segment_header(payload).then(|| RgbColor {
        red: payload[12],
        green: payload[13],
        blue: payload[14],
    })
}

pub fn set_segment_color(payload: &mut [u8], color: RgbColor) -> Result<(), String> {
    if !is_segment_header(payload) {
        return Err("Expected a segment object header".into());
    }
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
}

fn is_circle_header(payload: &[u8]) -> bool {
    if payload.len() != 16 && payload.len() != 28 {
        return false;
    }
    let code = u16::from_le_bytes([payload[0], payload[1]]);
    let subtype = u16::from_le_bytes([payload[8], payload[9]]);
    matches!(
        (code, subtype),
        (
            3 | 4 | 75 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34,
            28
        )
    )
}

pub fn circle_color(payload: &[u8]) -> Option<RgbColor> {
    is_circle_header(payload).then(|| RgbColor {
        red: payload[12],
        green: payload[13],
        blue: payload[14],
    })
}

pub fn set_circle_color(payload: &mut [u8], color: RgbColor) -> Result<(), String> {
    if !is_circle_header(payload) {
        return Err("Expected a circle object header".into());
    }
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
}

fn is_polygon_header(payload: &[u8]) -> bool {
    if payload.len() != 16 && payload.len() != 28 {
        return false;
    }
    let code = u16::from_le_bytes([payload[0], payload[1]]);
    let subtype = u16::from_le_bytes([payload[8], payload[9]]);
    code == 8
        || (code == 82 && payload.len() == 28 && payload[6..10] == [13, 0, 68, 0])
        || (matches!(
            code,
            75 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34
        ) && subtype >= 52
            && payload[6..8] == [4, 0])
}

pub fn polygon_color(payload: &[u8]) -> Option<RgbColor> {
    is_polygon_header(payload).then(|| RgbColor {
        red: payload[12],
        green: payload[13],
        blue: payload[14],
    })
}

pub fn set_polygon_color(payload: &mut [u8], color: RgbColor) -> Result<(), String> {
    if !is_polygon_header(payload) {
        return Err("Expected a polygon interior object header".into());
    }
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
}

// Native GSP4 class-4 polygons have no opacity field. Sketchpad renders their
// implicit 75% fill and writes 0xbfff when a native clipboard operation upgrades
// the short header to the 28-byte layout.
const LEGACY_POLYGON_OPACITY: u16 = 0xbfff;

fn is_legacy_polygon_header(payload: &[u8]) -> bool {
    payload.len() == 16
        && payload[..2] == [8, 0]
        && payload[6..8] == [4, 0]
        && u16::from_le_bytes([payload[8], payload[9]]) >= 52
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

pub type CircleLineStyle = SegmentLineStyle;

fn is_segment_header(payload: &[u8]) -> bool {
    (payload.len() == 16 || payload.len() == 28)
        && matches!(
            (
                u16::from_le_bytes([payload[0], payload[1]]),
                u16::from_le_bytes([payload[8], payload[9]])
            ),
            (
                2 | 75 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34,
                14
            )
        )
}

// Native GSP4 stroke headers use a small pen-style enum, rather than the
// modern packed width/pattern byte. Restrict this to independently proved
// primitive families and enum values; keep other short layouts unchanged.
fn is_legacy_stroke_header(payload: &[u8]) -> bool {
    payload.len() == 16
        && payload[11] == 0
        && payload[10] <= 2
        && matches!(
            (
                u16::from_le_bytes([payload[0], payload[1]]),
                u16::from_le_bytes([payload[6], payload[7]]),
                u16::from_le_bytes([payload[8], payload[9]])
            ),
            (2, 2, 14) | (3, 3, 28) | (64, 2, 13) | (5, 2, 47)
        )
}

fn stroke_header_style(payload: &[u8]) -> Option<SegmentLineStyle> {
    if is_legacy_stroke_header(payload) {
        return Some(match payload[10] {
            0 => SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Dashed,
            },
            1 => SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Solid,
            },
            2 => SegmentLineStyle {
                width: LineWidth::Medium,
                pattern: LinePattern::Solid,
            },
            _ => unreachable!(),
        });
    }
    decode_line_style(payload[10])
}

fn set_stroke_header_style(payload: &mut [u8], style: SegmentLineStyle) -> Result<(), String> {
    if is_legacy_stroke_header(payload) {
        let value = match (style.width, style.pattern) {
            (LineWidth::Thin, LinePattern::Dashed) => 0,
            (LineWidth::Thin, LinePattern::Solid) => 1,
            (LineWidth::Medium, LinePattern::Solid) => 2,
            _ => return Err("That stroke style requires a modern object header".into()),
        };
        payload[10] = value;
    } else {
        payload[10] = (style.pattern as u8) << 4 | style.width as u8;
    }
    Ok(())
}

pub fn segment_line_style(payload: &[u8]) -> Option<SegmentLineStyle> {
    if !is_segment_header(payload) {
        return None;
    }
    stroke_header_style(payload)
}

pub fn circle_line_style(payload: &[u8]) -> Option<CircleLineStyle> {
    if !is_circle_header(payload) {
        return None;
    }
    stroke_header_style(payload)
}

fn is_straight_or_arc_header(payload: &[u8]) -> bool {
    if payload.len() != 16 && payload.len() != 28 {
        return false;
    }
    let code = u16::from_le_bytes([payload[0], payload[1]]);
    (payload.len() == 28 && code == 75
        && (payload[6..10] == [12, 0, 67, 0]
            || (payload[6..8] == [2,0] && matches!(u16::from_le_bytes([payload[8],payload[9]]),12|13))))
        || matches!(code, 5 | 6 | 7 | 63 | 64 | 79 | 80 | 81)
        || (payload.len() == 28
            && payload[6..10] == [12, 0, 67, 0]
            && matches!(code, 16..=21 | 24 | 27..=30 | 32..=34))
        || (payload.len() == 28
            && matches!(code, 16 | 21 | 27 | 29 | 30 | 33 | 34)
            && payload[6..8] == [2, 0]
            && matches!(u16::from_le_bytes([payload[8], payload[9]]), 12 | 13))
        || (payload[6..8] == [6, 0]
            && matches!(
                (code, u16::from_le_bytes([payload[8], payload[9]])),
                (35 | 97, 42) | (72, 58) | (126 | 127, 99)
            ))
}

pub fn straight_or_arc_line_style(payload: &[u8]) -> Option<SegmentLineStyle> {
    is_straight_or_arc_header(payload).then(|| stroke_header_style(payload))?
}

fn decode_line_style(value: u8) -> Option<SegmentLineStyle> {
    let width = match value & 0x0f {
        0 => LineWidth::Hairline,
        1 => LineWidth::Thin,
        2 => LineWidth::Medium,
        3 => LineWidth::Thick,
        _ => return None,
    };
    let pattern = match value >> 4 {
        0 => LinePattern::Dotted,
        1 => LinePattern::Dashed,
        2 => LinePattern::Solid,
        _ => return None,
    };
    Some(SegmentLineStyle { width, pattern })
}

pub fn set_segment_line_style(payload: &mut [u8], style: SegmentLineStyle) -> Result<(), String> {
    if !is_segment_header(payload) {
        return Err("Expected a segment object header".into());
    }
    set_stroke_header_style(payload, style)
}

pub fn set_circle_line_style(payload: &mut [u8], style: CircleLineStyle) -> Result<(), String> {
    if !is_circle_header(payload) {
        return Err("Expected a circle object header".into());
    }
    set_stroke_header_style(payload, style)
}

pub fn set_straight_or_arc_line_style(
    payload: &mut [u8],
    style: SegmentLineStyle,
) -> Result<(), String> {
    if !is_straight_or_arc_header(payload) {
        return Err("Expected a line, ray, arc, or point-curve object header".into());
    }
    set_stroke_header_style(payload, style)
}

pub fn straight_or_arc_color(payload: &[u8]) -> Option<RgbColor> {
    is_straight_or_arc_header(payload).then(|| RgbColor {
        red: payload[12],
        green: payload[13],
        blue: payload[14],
    })
}

pub fn set_straight_or_arc_color(payload: &mut [u8], color: RgbColor) -> Result<(), String> {
    if !is_straight_or_arc_header(payload) {
        return Err("Expected a line, ray, arc, or point-curve object header".into());
    }
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualStyle {
    Point {
        shape: PointStyle,
        color: RgbColor,
    },
    Stroke {
        line: SegmentLineStyle,
        color: RgbColor,
    },
    Polygon {
        color: RgbColor,
        opacity: u16,
    },
}

pub fn visual_style(payload: &[u8]) -> Option<VisualStyle> {
    if let (Some(shape), Some(color)) = (point_style(payload), point_color(payload)) {
        return Some(VisualStyle::Point { shape, color });
    }
    let line = segment_line_style(payload)
        .or_else(|| circle_line_style(payload))
        .or_else(|| straight_or_arc_line_style(payload));
    let color = segment_color(payload)
        .or_else(|| circle_color(payload))
        .or_else(|| straight_or_arc_color(payload));
    if let (Some(line), Some(color)) = (line, color) {
        return Some(VisualStyle::Stroke { line, color });
    }
    polygon_color(payload).map(|color| VisualStyle::Polygon {
        color,
        opacity: if payload.len() >= 18 {
            u16::from_le_bytes([payload[16], payload[17]])
        } else if is_legacy_polygon_header(payload) {
            LEGACY_POLYGON_OPACITY
        } else {
            32767
        },
    })
}

pub fn set_visual_style(payload: &mut [u8], style: VisualStyle) -> Result<(), String> {
    match style {
        VisualStyle::Point { shape, color } if point_style(payload).is_some() => {
            set_point_style(payload, shape)?;
            set_point_color(payload, color)
        }
        VisualStyle::Stroke { line, color } if segment_line_style(payload).is_some() => {
            set_segment_line_style(payload, line)?;
            set_segment_color(payload, color)
        }
        VisualStyle::Stroke { line, color } if circle_line_style(payload).is_some() => {
            set_circle_line_style(payload, line)?;
            set_circle_color(payload, color)
        }
        VisualStyle::Stroke { line, color } if straight_or_arc_line_style(payload).is_some() => {
            set_straight_or_arc_line_style(payload, line)?;
            set_straight_or_arc_color(payload, color)
        }
        VisualStyle::Polygon { color, opacity } if polygon_color(payload).is_some() => {
            if is_legacy_polygon_header(payload) && opacity != LEGACY_POLYGON_OPACITY {
                return Err("Legacy polygon headers cannot store a different opacity".into());
            }
            set_polygon_color(payload, color)?;
            if payload.len() >= 18 {
                payload[16..18].copy_from_slice(&opacity.to_le_bytes());
            }
            Ok(())
        }
        _ => Err("Style does not match object header".into()),
    }
}

/// Binding headers keep their own appearance while carrying a polygon or segment.
pub fn iteration_binding_visual_style(payload: &[u8], polygon: bool) -> Option<VisualStyle> {
    if payload.len() != 28 || payload[..2] != 77u16.to_le_bytes() {
        return None;
    }
    let mut header = payload.to_vec();
    header[..2].copy_from_slice(&(if polygon { 8u16 } else { 2u16 }).to_le_bytes());
    if !polygon {
        header[8..10].copy_from_slice(&14u16.to_le_bytes());
    }
    visual_style(&header)
}

/// Point bindings use byte 11 and RGB while retaining their class-15 descriptor.
pub fn iteration_point_binding_visual_style(payload: &[u8]) -> Option<VisualStyle> {
    if payload.len() != 28
        || payload[..2] != 77u16.to_le_bytes()
        || payload[6..10] != [15, 0, 65, 0]
    {
        return None;
    }
    let mut header = payload.to_vec();
    header[..2].copy_from_slice(&0u16.to_le_bytes());
    header[6..10].copy_from_slice(&[1, 0, 4, 0]);
    visual_style(&header)
}

pub fn set_iteration_binding_visual_style(
    payload: &mut [u8],
    style: VisualStyle,
) -> Result<(), String> {
    if payload.len() != 28 || payload[..2] != 77u16.to_le_bytes() {
        return Err("Expected an iteration binding header".into());
    }
    let mut header = payload.to_vec();
    match style {
        VisualStyle::Polygon { .. } => header[..2].copy_from_slice(&8u16.to_le_bytes()),
        VisualStyle::Stroke { .. } => {
            header[..2].copy_from_slice(&2u16.to_le_bytes());
            header[8..10].copy_from_slice(&14u16.to_le_bytes());
        }
        VisualStyle::Point { .. } => {
            header[..2].copy_from_slice(&0u16.to_le_bytes());
            header[6..10].copy_from_slice(&[1, 0, 4, 0]);
        }
    }
    set_visual_style(&mut header, style)?;
    payload[10..18].copy_from_slice(&header[10..18]);
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
        assert_eq!(
            point_color(red),
            Some(RgbColor {
                red: 255,
                green: 0,
                blue: 0
            })
        );
        assert_eq!(
            point_color(orange),
            Some(RgbColor {
                red: 255,
                green: 128,
                blue: 0
            })
        );
        assert_eq!(
            point_color(blue),
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
        );
        let mut edited = red.to_vec();
        set_point_color(
            &mut edited,
            RgbColor {
                red: 0,
                green: 0,
                blue: 255,
            },
        )
        .unwrap();
        assert_eq!(point_color(&edited), point_color(blue));
        edited[12..15].copy_from_slice(&red[12..15]);
        assert_eq!(edited, red);
    }

    #[test]
    fn supported_derived_point_headers_share_style_bytes() {
        let cases: &[(&[u8], u16)] = &[
            (include_bytes!("../tests/fixtures/middle_point.gsp"), 1),
            (include_bytes!("../tests/fixtures/point_on_segment.gsp"), 15),
            (
                include_bytes!("../tests/fixtures/point_translation.gsp"),
                21,
            ),
            (include_bytes!("../tests/fixtures/rotation.gsp"), 27),
            (
                include_bytes!("../tests/fixtures/wine_moved_dilation.gsp"),
                30,
            ),
            (include_bytes!("../tests/fixtures/reflection.gsp"), 34),
            (include_bytes!("../tests/fixtures/line_intersection.gsp"), 9),
            (
                include_bytes!("../tests/fixtures/line_circle_intersection.gsp"),
                12,
            ),
            (
                include_bytes!("../tests/fixtures/circle_circle_intersection.gsp"),
                14,
            ),
        ];
        for (bytes, code) in cases {
            let header = records_headers(bytes)
                .into_iter()
                .find(|header| {
                    u16::from_le_bytes([header[0], header[1]]) == *code && is_point_header(header)
                })
                .unwrap();
            assert_eq!(point_style(&header), Some(PointStyle::Medium));
            assert_eq!(
                point_color(&header),
                Some(RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0
                })
            );
        }
        let saved = records_headers(include_bytes!(
            "../tests/fixtures/style_generated_large_blue_midpoint.gsp"
        ))
        .into_iter()
        .find(|header| u16::from_le_bytes([header[0], header[1]]) == 1)
        .unwrap();
        assert_eq!(point_style(&saved), Some(PointStyle::Large));
        assert_eq!(
            point_color(&saved),
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
        );
    }

    #[test]
    fn non_point_and_unknown_styles_are_rejected() {
        assert_eq!(point_style(&[]), None);
        let mut header = first_point_header(include_bytes!("../tests/fixtures/point.gsp")).to_vec();
        header[11] = 4;
        assert_eq!(point_style(&header), None);
        header[0] = 1;
        assert!(set_point_style(&mut header, PointStyle::Dot).is_err());
        let text = include_bytes!("../tests/fixtures/text_on_point.gsp");
        let mut text_header = first_point_header(text).to_vec();
        text_header[8..10].copy_from_slice(&19_u16.to_le_bytes());
        assert_eq!(point_style(&text_header), None);
        assert_eq!(point_color(&text_header), None);
    }

    #[test]
    fn original_segment_menu_variants_match_header_byte() {
        use LinePattern::{Dashed, Dotted, Solid};
        use LineWidth::{Hairline, Medium, Thick, Thin};
        let fixtures: &[(&[u8], LineWidth, LinePattern)] = &[
            (
                include_bytes!("../tests/fixtures/style_medium_solid_line.gsp"),
                Medium,
                Solid,
            ),
            (
                include_bytes!("../tests/fixtures/style_hairline_line.gsp"),
                Hairline,
                Solid,
            ),
            (
                include_bytes!("../tests/fixtures/style_thin_line.gsp"),
                Thin,
                Solid,
            ),
            (
                include_bytes!("../tests/fixtures/style_thick_line.gsp"),
                Thick,
                Solid,
            ),
            (
                include_bytes!("../tests/fixtures/style_dashed_line.gsp"),
                Medium,
                Dashed,
            ),
            (
                include_bytes!("../tests/fixtures/style_dotted_line.gsp"),
                Medium,
                Dotted,
            ),
        ];
        for (bytes, width, pattern) in fixtures {
            let payload = first_segment_header(bytes);
            assert_eq!(
                segment_line_style(payload),
                Some(SegmentLineStyle {
                    width: *width,
                    pattern: *pattern
                })
            );
        }
    }

    #[test]
    fn original_segment_color_menu_writes_rgb_bytes() {
        let navy = first_segment_header(include_bytes!(
            "../tests/fixtures/style_medium_solid_line.gsp"
        ));
        let blue = first_segment_header(include_bytes!("../tests/fixtures/style_blue_line.gsp"));
        assert_eq!(
            segment_color(navy),
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 128
            })
        );
        assert_eq!(
            segment_color(blue),
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
        );
        let mut edited = navy.to_vec();
        set_segment_color(
            &mut edited,
            RgbColor {
                red: 0,
                green: 0,
                blue: 255,
            },
        )
        .unwrap();
        assert_eq!(segment_color(&edited), segment_color(blue));
        edited[12..15].copy_from_slice(&navy[12..15]);
        assert_eq!(edited, navy);
    }

    #[test]
    fn original_circle_color_menu_writes_rgb_bytes() {
        let green = first_circle_header(include_bytes!("../tests/fixtures/style_green_circle.gsp"));
        let blue = first_circle_header(include_bytes!("../tests/fixtures/style_blue_circle.gsp"));
        assert_eq!(
            circle_color(green),
            Some(RgbColor {
                red: 0,
                green: 128,
                blue: 0
            })
        );
        assert_eq!(
            circle_color(blue),
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
        );
        let mut edited = green.to_vec();
        set_circle_color(
            &mut edited,
            RgbColor {
                red: 0,
                green: 0,
                blue: 255,
            },
        )
        .unwrap();
        assert_eq!(circle_color(&edited), circle_color(blue));
        edited[12..15].copy_from_slice(&green[12..15]);
        assert_eq!(edited, green);
    }

    #[test]
    fn legacy_polygon_rgb_and_implicit_opacity_match_native_copy_conversion() {
        let header = |bytes: &[u8]| {
            crate::document::records(bytes)
                .unwrap()
                .into_iter()
                .filter(|r| r.kind == 0x07d0)
                .nth(4)
                .map(|r| bytes[r.data].to_vec())
                .unwrap()
        };
        let original = header(include_bytes!(
            "../tests/fixtures/legacy_polygon_rank_original.gsp"
        ));
        let blue = header(include_bytes!(
            "../tests/fixtures/legacy_polygon_blue_native_verified.gsp"
        ));
        let converted = header(include_bytes!(
            "../tests/fixtures/legacy_polygon_copy_native_verified.gsp"
        ));
        let color = RgbColor {
            red: 0,
            green: 128,
            blue: 255,
        };
        let style = VisualStyle::Polygon {
            color,
            opacity: 49151,
        };
        assert_eq!(original.len(), 16);
        assert_eq!(
            visual_style(&original),
            Some(VisualStyle::Polygon {
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0
                },
                opacity: 49151
            })
        );
        assert_eq!(visual_style(&blue), Some(style));
        assert_eq!(converted.len(), 28);
        assert_eq!(&converted[16..18], &[0xff, 0xbf]);
        assert_eq!(visual_style(&converted), Some(style));
        let mut edited = original.clone();
        set_visual_style(&mut edited, style).unwrap();
        assert_eq!(edited, blue);
        for opacity in [0, 32767, 65535] {
            let mut rejected = original.clone();
            assert!(
                set_visual_style(&mut rejected, VisualStyle::Polygon { color, opacity }).is_err()
            );
            assert_eq!(rejected, original);
        }
    }

    #[test]
    fn polygon_interior_header_has_orange_rgb() {
        let header = records_headers(include_bytes!(
            "../tests/fixtures/wine_polygon_perimeter.gsp"
        ))
        .into_iter()
        .find(|header| is_polygon_header(header))
        .unwrap();
        assert_eq!(
            polygon_color(&header),
            Some(RgbColor {
                red: 255,
                green: 128,
                blue: 0
            })
        );
        let mut edited = header.clone();
        set_polygon_color(
            &mut edited,
            RgbColor {
                red: 0,
                green: 0,
                blue: 255,
            },
        )
        .unwrap();
        assert_eq!(
            polygon_color(&edited),
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 255
            })
        );
        edited[12..15].copy_from_slice(&header[12..15]);
        assert_eq!(edited, header);
    }

    #[test]
    fn visual_style_round_trip_preserves_unrelated_header_bytes() {
        assert_eq!(visual_style(&[]), None);
        for bytes in [
            include_bytes!("../tests/fixtures/point.gsp").as_slice(),
            include_bytes!("../tests/fixtures/style_medium_solid_line.gsp").as_slice(),
            include_bytes!("../tests/fixtures/style_green_circle.gsp").as_slice(),
            include_bytes!("../tests/fixtures/line.gsp").as_slice(),
            include_bytes!("../tests/fixtures/three_point_arc.gsp").as_slice(),
            include_bytes!("../tests/fixtures/wine_polygon_perimeter.gsp").as_slice(),
        ] {
            for header in records_headers(bytes) {
                if let Some(style) = visual_style(&header) {
                    let mut edited = header.clone();
                    set_visual_style(&mut edited, style).unwrap();
                    assert_eq!(edited, header);
                }
            }
        }
        let mut point = first_point_header(include_bytes!("../tests/fixtures/point.gsp")).to_vec();
        assert!(
            set_visual_style(
                &mut point,
                VisualStyle::Polygon {
                    opacity: 32767,
                    color: RgbColor {
                        red: 1,
                        green: 2,
                        blue: 3
                    }
                }
            )
            .is_err()
        );
    }

    #[test]
    fn original_circle_line_menu_writes_the_same_nibbles_as_segments() {
        let medium =
            first_circle_header(include_bytes!("../tests/fixtures/style_green_circle.gsp"));
        let thick = first_circle_header(include_bytes!("../tests/fixtures/style_thick_circle.gsp"));
        let dashed =
            first_circle_header(include_bytes!("../tests/fixtures/style_dashed_circle.gsp"));
        assert_eq!(
            circle_line_style(medium),
            Some(CircleLineStyle {
                width: LineWidth::Medium,
                pattern: LinePattern::Solid
            })
        );
        assert_eq!(
            circle_line_style(thick),
            Some(CircleLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Solid
            })
        );
        assert_eq!(
            circle_line_style(dashed),
            Some(CircleLineStyle {
                width: LineWidth::Medium,
                pattern: LinePattern::Dashed
            })
        );
        let mut edited = medium.to_vec();
        set_circle_line_style(
            &mut edited,
            CircleLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Solid,
            },
        )
        .unwrap();
        assert_eq!(edited[10], thick[10]);
        edited[10] = medium[10];
        assert_eq!(edited, medium);
        assert_eq!(segment_line_style(medium), None);
        assert_eq!(
            circle_line_style(first_segment_header(include_bytes!(
                "../tests/fixtures/style_medium_solid_line.gsp"
            ))),
            None
        );
    }

    #[test]
    fn point_curve_headers_use_native_strokes_without_casting_swept_loci() {
        for bytes in [
            include_bytes!("../tests/fixtures/locus_midpoint_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/function_plot_identity.gsp").as_slice(),
            include_bytes!("../tests/fixtures/parametric_rectangular_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/parametric_polar_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/numeric_cood_trace_added_locus_native_verified.gsp")
                .as_slice(),
        ] {
            let header = records_headers(bytes)
                .into_iter()
                .find(|h| matches!(u16::from_le_bytes([h[0], h[1]]), 35 | 72 | 97 | 126 | 127))
                .unwrap();
            assert!(matches!(
                visual_style(&header),
                Some(VisualStyle::Stroke { .. })
            ));
            let mut changed = header.clone();
            set_visual_style(
                &mut changed,
                VisualStyle::Stroke {
                    line: SegmentLineStyle {
                        width: LineWidth::Thick,
                        pattern: LinePattern::Dashed,
                    },
                    color: RgbColor {
                        red: 14,
                        green: 150,
                        blue: 210,
                    },
                },
            )
            .unwrap();
            for index in 0..header.len() {
                if ![10, 12, 13, 14].contains(&index) {
                    assert_eq!(changed[index], header[index]);
                }
            }
        }
        for bytes in [
            include_bytes!("../tests/fixtures/locus_segment_target_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/locus_circle_target_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/locus_vector_polygon_target_original.gsp").as_slice(),
        ] {
            let mut header = records_headers(bytes)
                .into_iter()
                .find(|h| u16::from_le_bytes([h[0], h[1]]) == 35)
                .unwrap();
            assert_eq!(header[8], 43);
            assert_eq!(visual_style(&header), None);
            assert!(
                set_straight_or_arc_color(
                    &mut header,
                    RgbColor {
                        red: 14,
                        green: 150,
                        blue: 210
                    }
                )
                .is_err()
            );
        }
    }

    #[test]
    fn line_ray_and_arc_fixtures_start_with_the_same_stroke_style() {
        for bytes in [
            include_bytes!("../tests/fixtures/line.gsp").as_slice(),
            include_bytes!("../tests/fixtures/ray.gsp").as_slice(),
            include_bytes!("../tests/fixtures/three_point_arc.gsp").as_slice(),
            include_bytes!("../tests/fixtures/arc_on_circle.gsp").as_slice(),
        ] {
            let mut headers = records_headers(bytes);
            let header = headers
                .iter_mut()
                .find(|header| is_straight_or_arc_header(header))
                .unwrap();
            assert_eq!(
                straight_or_arc_line_style(header),
                Some(SegmentLineStyle {
                    width: LineWidth::Medium,
                    pattern: LinePattern::Solid
                })
            );
            set_straight_or_arc_line_style(
                header,
                SegmentLineStyle {
                    width: LineWidth::Thick,
                    pattern: LinePattern::Dashed,
                },
            )
            .unwrap();
            assert_eq!(
                straight_or_arc_line_style(header),
                Some(SegmentLineStyle {
                    width: LineWidth::Thick,
                    pattern: LinePattern::Dashed
                })
            );
        }
    }

    #[test]
    fn line_ray_and_arc_headers_carry_rgb_bytes() {
        let fixtures: &[(&[u8], RgbColor)] = &[
            (
                include_bytes!("../tests/fixtures/line.gsp"),
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 128,
                },
            ),
            (
                include_bytes!("../tests/fixtures/ray.gsp"),
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 128,
                },
            ),
            (
                include_bytes!("../tests/fixtures/three_point_arc.gsp"),
                RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0,
                },
            ),
            (
                include_bytes!("../tests/fixtures/arc_on_circle.gsp"),
                RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0,
                },
            ),
        ];
        for (bytes, expected) in fixtures {
            let mut header = records_headers(bytes)
                .into_iter()
                .find(|header| is_straight_or_arc_header(header))
                .unwrap();
            assert_eq!(straight_or_arc_color(&header), Some(*expected));
            set_straight_or_arc_color(
                &mut header,
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255,
                },
            )
            .unwrap();
            assert_eq!(
                straight_or_arc_color(&header),
                Some(RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255
                })
            );
        }
    }

    fn records_headers(bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut headers = Vec::new();
        let mut offset = 4;
        while offset + 8 <= bytes.len() {
            let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            if u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) == 0x07d0 {
                headers.push(bytes[offset + 8..offset + 8 + length].to_vec());
            }
            offset += 8 + length + length % 2;
        }
        headers
    }

    fn first_circle_header(bytes: &[u8]) -> &[u8] {
        assert_eq!(&bytes[..4], b"GSP4");
        let mut offset = 4;
        while offset < bytes.len() {
            let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let payload = &bytes[offset + 8..offset + 8 + length];
            if kind == 0x07d0 && is_circle_header(payload) {
                return payload;
            }
            offset += 8 + length + length % 2;
        }
        panic!("fixture has no circle header");
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
        let original = first_segment_header(include_bytes!(
            "../tests/fixtures/style_medium_solid_line.gsp"
        ));
        let mut changed = original.to_vec();
        set_segment_line_style(
            &mut changed,
            SegmentLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Dashed,
            },
        )
        .unwrap();
        assert_eq!(changed[10], 0x13);
        changed[10] = original[10];
        assert_eq!(changed, original);
    }

    #[test]
    fn constructed_circle_and_line_headers_share_stroke_fields() {
        let cases: &[(&[u8], u16, RgbColor)] = &[
            (
                include_bytes!("../tests/fixtures/circle_center_radius.gsp"),
                4,
                RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0,
                },
            ),
            (
                include_bytes!("../tests/fixtures/wine_moved_dilation.gsp"),
                30,
                RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0,
                },
            ),
            (
                include_bytes!("../tests/fixtures/wine_moved_reflected_circle.gsp"),
                34,
                RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0,
                },
            ),
            (
                include_bytes!("../tests/fixtures/perpendicular.gsp"),
                5,
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 128,
                },
            ),
            (
                include_bytes!("../tests/fixtures/parallel.gsp"),
                6,
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 128,
                },
            ),
            (
                include_bytes!("../tests/fixtures/bisector.gsp"),
                7,
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 128,
                },
            ),
        ];
        for (file, code, color) in cases {
            let mut header = records_headers(file)
                .into_iter()
                .find(|payload| {
                    u16::from_le_bytes(payload[..2].try_into().unwrap()) == *code
                        && matches!(visual_style(payload), Some(VisualStyle::Stroke { .. }))
                })
                .unwrap();
            assert_eq!(
                visual_style(&header),
                Some(VisualStyle::Stroke {
                    line: SegmentLineStyle {
                        width: LineWidth::Medium,
                        pattern: LinePattern::Solid
                    },
                    color: *color,
                })
            );
            let original = header.clone();
            set_visual_style(
                &mut header,
                VisualStyle::Stroke {
                    line: SegmentLineStyle {
                        width: LineWidth::Thick,
                        pattern: LinePattern::Dashed,
                    },
                    color: RgbColor {
                        red: 0,
                        green: 0,
                        blue: 255,
                    },
                },
            )
            .unwrap();
            assert_eq!(header[10], 0x13);
            assert_eq!(&header[12..15], &[0, 0, 255]);
            header[10] = original[10];
            header[12..15].copy_from_slice(&original[12..15]);
            assert_eq!(header, original);
        }
    }

    #[test]
    fn native_legacy_free_points_have_implicit_small_size_independent_of_opaque_byte() {
        let source =
            include_bytes!("../tests/fixtures/legacy_free_point_style_opaque_original.gsp");
        let copied =
            include_bytes!("../tests/fixtures/legacy_free_point_style_opaque_copied_native.gsp");
        let old = records_headers(source);
        let modern = records_headers(copied);
        assert_eq!(old.len(), 5);
        assert_eq!(modern.len(), 10);
        for (index, opaque) in [0x00, 0x20, 0x40, 0x60].into_iter().enumerate() {
            let header = &old[index];
            assert_eq!(header.len(), 16);
            assert_eq!(&header[10..12], &[1, 0]);
            assert_eq!(header[15], opaque);
            assert_eq!(point_style(header), Some(PointStyle::Small));
            for id in [index, index + 5] {
                assert_eq!(modern[id].len(), 28);
                assert_eq!(modern[id][11], 1);
                assert_eq!(point_style(&modern[id]), Some(PointStyle::Small));
                assert_eq!(modern[id][15], 0);
                assert_eq!(point_color(header), point_color(&modern[id]));
            }
        }
    }

    #[test]
    fn legacy_free_point_size_setter_preserves_fields_or_rejects_atomically() {
        let headers = records_headers(include_bytes!(
            "../tests/fixtures/legacy_free_point_style_opaque_original.gsp"
        ));
        for original in headers.into_iter().take(4) {
            let mut changed = original.clone();
            set_point_style(&mut changed, PointStyle::Small).unwrap();
            assert_eq!(changed, original);
            for style in [PointStyle::Dot, PointStyle::Medium, PointStyle::Large] {
                assert!(set_point_style(&mut changed, style).is_err());
                assert_eq!(changed, original);
            }
            set_point_color(
                &mut changed,
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255,
                },
            )
            .unwrap();
            assert_eq!(point_style(&changed), Some(PointStyle::Small));
            assert_eq!(changed[15], original[15]);
            changed[12..15].copy_from_slice(&original[12..15]);
            assert_eq!(changed, original);
        }
    }

    #[test]
    fn modern_short_free_point_headers_keep_the_explicit_style_byte() {
        let native = first_point_header(include_bytes!("../tests/fixtures/point.gsp"));
        for style in [
            PointStyle::Dot,
            PointStyle::Small,
            PointStyle::Medium,
            PointStyle::Large,
        ] {
            let mut short = native[..16].to_vec();
            assert_eq!(short[10], 0x21);
            short[11] = style as u8;
            assert_eq!(point_style(&short), Some(style));
            let original = short.clone();
            set_point_style(&mut short, PointStyle::Large).unwrap();
            assert_eq!(point_style(&short), Some(PointStyle::Large));
            short[11] = original[11];
            assert_eq!(short, original);
        }
    }

    #[test]
    fn legacy_stroke_enums_match_native_converted_segment_circle_ray_and_line_styles() {
        for (source, copied) in [
            (
                include_bytes!("../tests/fixtures/legacy_stroke_styles_original.gsp").as_slice(),
                include_bytes!("../tests/fixtures/legacy_stroke_styles_copied_native.gsp")
                    .as_slice(),
            ),
            (
                include_bytes!("../tests/fixtures/legacy_stroke_dashed_original.gsp").as_slice(),
                include_bytes!("../tests/fixtures/legacy_stroke_dashed_copied_native.gsp")
                    .as_slice(),
            ),
            (
                include_bytes!("../tests/fixtures/legacy_stroke_medium_paths_original.gsp")
                    .as_slice(),
                include_bytes!("../tests/fixtures/legacy_stroke_medium_paths_copied_native.gsp")
                    .as_slice(),
            ),
        ] {
            let old = records_headers(source);
            let modern = records_headers(copied);
            assert_eq!(old.len(), 10);
            assert_eq!(modern.len(), 20);
            for index in 4..10 {
                assert_eq!(old[index].len(), 16);
                let expected = match old[index][10] {
                    0 => SegmentLineStyle {
                        width: LineWidth::Thin,
                        pattern: LinePattern::Dashed,
                    },
                    1 => SegmentLineStyle {
                        width: LineWidth::Thin,
                        pattern: LinePattern::Solid,
                    },
                    2 => SegmentLineStyle {
                        width: LineWidth::Medium,
                        pattern: LinePattern::Solid,
                    },
                    value => panic!("Unexpected native pen enum {value}"),
                };
                let style = visual_style(&old[index]).unwrap();
                assert!(matches!(style, VisualStyle::Stroke { line, .. } if line == expected));
                for copied_index in [index, index + 10] {
                    assert_eq!(modern[copied_index].len(), 28);
                    assert_eq!(visual_style(&modern[copied_index]), Some(style));
                    assert_eq!(&modern[copied_index][12..15], &old[index][12..15]);
                }
            }
        }
    }

    #[test]
    fn legacy_stroke_setters_preserve_opaque_fields_and_reject_unrepresented_styles_atomically() {
        let headers = records_headers(include_bytes!(
            "../tests/fixtures/legacy_stroke_styles_original.gsp"
        ));
        for original in headers.into_iter().skip(4) {
            let VisualStyle::Stroke { color, .. } = visual_style(&original).unwrap() else {
                panic!("Expected stroke");
            };
            for (word, width, pattern) in [
                (0, LineWidth::Thin, LinePattern::Dashed),
                (1, LineWidth::Thin, LinePattern::Solid),
                (2, LineWidth::Medium, LinePattern::Solid),
            ] {
                let mut changed = original.clone();
                let style = VisualStyle::Stroke {
                    color,
                    line: SegmentLineStyle { width, pattern },
                };
                set_visual_style(&mut changed, style).unwrap();
                assert_eq!(changed[10], word);
                assert_eq!(visual_style(&changed), Some(style));
                changed[10] = original[10];
                assert_eq!(changed, original);
            }
            for line in [
                SegmentLineStyle {
                    width: LineWidth::Hairline,
                    pattern: LinePattern::Solid,
                },
                SegmentLineStyle {
                    width: LineWidth::Thick,
                    pattern: LinePattern::Solid,
                },
                SegmentLineStyle {
                    width: LineWidth::Thin,
                    pattern: LinePattern::Dotted,
                },
                SegmentLineStyle {
                    width: LineWidth::Medium,
                    pattern: LinePattern::Dashed,
                },
            ] {
                let mut changed = original.clone();
                assert!(
                    set_visual_style(&mut changed, VisualStyle::Stroke { color, line }).is_err()
                );
                assert_eq!(changed, original);
            }
        }
    }

    #[test]
    fn modern_short_stroke_headers_keep_packed_pattern_and_width_semantics() {
        let headers = records_headers(include_bytes!(
            "../tests/fixtures/legacy_stroke_styles_copied_native.gsp"
        ));
        for modern in headers.into_iter().skip(4).take(6) {
            let mut short = modern[..16].to_vec();
            assert_eq!(short[11], 1);
            let VisualStyle::Stroke { color, .. } = visual_style(&short).unwrap() else {
                panic!("Expected stroke");
            };
            for line in [
                SegmentLineStyle {
                    width: LineWidth::Thin,
                    pattern: LinePattern::Dotted,
                },
                SegmentLineStyle {
                    width: LineWidth::Medium,
                    pattern: LinePattern::Dashed,
                },
                SegmentLineStyle {
                    width: LineWidth::Thick,
                    pattern: LinePattern::Solid,
                },
            ] {
                let style = VisualStyle::Stroke { color, line };
                set_visual_style(&mut short, style).unwrap();
                assert_eq!(short[10], (line.pattern as u8) << 4 | line.width as u8);
                assert_eq!(visual_style(&short), Some(style));
                assert_eq!(short[11], 1);
            }
        }
    }

    #[test]
    fn fitting_legacy_stroke_api_edits_match_native_saved_short_headers() {
        let source = include_bytes!("../tests/fixtures/legacy_stroke_styles_original.gsp");
        let native =
            include_bytes!("../tests/fixtures/legacy_stroke_styles_edited_native_verified.gsp");
        let mut document = crate::document::Document::from_bytes(source, None).unwrap();
        for (id, width) in [
            (5, LineWidth::Thin),
            (7, LineWidth::Thin),
            (10, LineWidth::Medium),
        ] {
            document.selected = [id].into_iter().collect();
            assert_eq!(document.set_selected_line_width(width).unwrap(), 1);
        }
        for id in [6, 8, 9] {
            document.selected = [id].into_iter().collect();
            assert_eq!(
                document
                    .set_selected_line_pattern(LinePattern::Dashed)
                    .unwrap(),
                1
            );
        }
        assert_eq!(document.encode().unwrap(), native);
        let headers = records_headers(native);
        assert!(headers.iter().all(|header| header.len() == 16));
        assert_eq!(
            headers[4..]
                .iter()
                .map(|header| header[10])
                .collect::<Vec<_>>(),
            [1, 0, 1, 0, 0, 2]
        );
        document.verify_roundtrip().unwrap();
    }
}
