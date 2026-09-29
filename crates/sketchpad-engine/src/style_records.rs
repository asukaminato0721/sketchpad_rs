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
            0 | 75 | 88 | 119 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34,
            4
        ) | (0, 41)
            | (1, 11)
            | (15, 10)
            | (101, 89)
            | (9 | 11 | 12 | 13 | 14 | 93 | 98, 45)
            | (67 | 68 | 69 | 95 | 109 | 110 | 123, 56)
    ) || (payload.len() == 28
        && matches!(code, 16 | 33)
        && subtype == 89
        && payload[6..8] == [1, 0])
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
        || (matches!(code, 10 | 75) && payload.len() == 28 && payload[6..10] == [5, 0, 16, 0])
        || (code == 75 && payload.len() == 28 && payload[6..10] == [4, 0, 15, 0])
        || (code == 82 && payload.len() == 28 && payload[6..10] == [13, 0, 68, 0])
        || (code == 83 && payload.len() == 28 && payload[6..10] == [13, 0, 69, 0])
        || (matches!(
            code,
            75 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 29 | 30 | 32 | 33 | 34
        ) && ((subtype >= 52 && payload[6..8] == [4, 0])
            || (payload.len() == 28 && matches!(payload[6..10], [5, 0, 16, 0] | [13, 0, 68, 0]))))
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
    payload.len() == 28 && payload[..2] == 121u16.to_le_bytes() && payload[6..10] == [19, 0, 96, 0]
        || (payload.len() == 16 || payload.len() == 28)
            && matches!(
                (
                    u16::from_le_bytes([payload[0], payload[1]]),
                    u16::from_le_bytes([payload[8], payload[9]])
                ),
                (
                    2 | 75
                        | 119
                        | 16
                        | 17
                        | 18
                        | 19
                        | 20
                        | 21
                        | 24
                        | 27
                        | 28
                        | 29
                        | 30
                        | 32
                        | 33
                        | 34,
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
    (payload.len() == 28 && matches!(code, 113 | 115 | 116) && payload[6..10] == [18, 0, 95, 0])
        || (payload.len() == 28
            && code == 75
            && (payload[6..10] == [12, 0, 67, 0]
                || (payload[6..8] == [2, 0]
                    && matches!(u16::from_le_bytes([payload[8], payload[9]]), 12 | 13))))
        || matches!(code, 5 | 6 | 7 | 63 | 64 | 79 | 80 | 81)
        || (payload.len() == 28
            && payload[6..10] == [12, 0, 67, 0]
            && matches!(code, 16..=21 | 24 | 27..=30 | 32..=34 | 119))
        || (payload.len() == 28
            && matches!(code, 16 | 21 | 27 | 29 | 30 | 33 | 34)
            && payload[6..8] == [2, 0]
            && matches!(u16::from_le_bytes([payload[8], payload[9]]), 12 | 13))
        || (payload[6..8] == [6, 0]
            && (code != 102 || payload.len() == 28)
            && matches!(
                (code, u16::from_le_bytes([payload[8], payload[9]])),
                (35 | 97, 42) | (102, 90) | (72, 58) | (126 | 127, 99)
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

/// Class43 identifies several sweep targets. The document's resolved target
/// supplies the straight-family context; polygon sweeps retain polygon style.
pub fn locus_straight_visual_style(payload: &[u8]) -> Option<VisualStyle> {
    if payload.len() != 28
        || !matches!(u16::from_le_bytes([payload[0], payload[1]]), 35 | 97)
        || payload[6..10] != [6, 0, 43, 0]
    {
        return None;
    }
    Some(VisualStyle::Stroke {
        line: stroke_header_style(payload)?,
        color: RgbColor {
            red: payload[12],
            green: payload[13],
            blue: payload[14],
        },
    })
}

pub fn set_locus_straight_visual_style(
    payload: &mut [u8],
    style: VisualStyle,
) -> Result<(), String> {
    if locus_straight_visual_style(payload).is_none() {
        return Err("Expected a supported straight locus header".into());
    }
    let VisualStyle::Stroke { line, color } = style else {
        return Err("Expected a straight locus stroke style".into());
    };
    set_stroke_header_style(payload, line)?;
    payload[12..15].copy_from_slice(&[color.red, color.green, color.blue]);
    Ok(())
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
