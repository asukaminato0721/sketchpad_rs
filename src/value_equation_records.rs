//! Conservative record templates and evaluation for Measure > Value of Point
//! and Cartesian line/circle equations in Sketchpad 5.

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueEquationKind {
    PointValue,
    LineEquation,
    CircleEquation,
}

impl ValueEquationKind {
    pub fn code(self) -> u16 {
        match self {
            Self::PointValue => 94,
            Self::LineEquation => 50,
            Self::CircleEquation => 51,
        }
    }
    fn from_code(code: u16) -> Option<Self> {
        match code {
            94 => Some(Self::PointValue),
            50 => Some(Self::LineEquation),
            51 => Some(Self::CircleEquation),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CartesianFrame {
    pub origin: ScreenPoint,
    pub x_unit: ScreenPoint,
    pub y_unit: ScreenPoint,
}

impl CartesianFrame {
    pub fn coordinates(self, point: ScreenPoint) -> Option<ScreenPoint> {
        let xx = self.x_unit.x - self.origin.x;
        let xy = self.x_unit.y - self.origin.y;
        let yx = self.y_unit.x - self.origin.x;
        let yy = self.y_unit.y - self.origin.y;
        let dx = point.x - self.origin.x;
        let dy = point.y - self.origin.y;
        let determinant = xx.mul_add(yy, -xy * yx);
        let norm = xx.hypot(xy) * yx.hypot(yy);
        if !norm.is_finite()
            || norm == 0.
            || !determinant.is_finite()
            || determinant.abs() <= 1e-12 * norm
        {
            return None;
        }
        let x = dx.mul_add(yy, -dy * yx) / determinant;
        let y = xx.mul_add(dy, -xy * dx) / determinant;
        (x.is_finite() && y.is_finite()).then_some(ScreenPoint { x, y })
    }
    fn square_unit_scale(self) -> Option<f64> {
        let dx = ScreenPoint {
            x: self.x_unit.x - self.origin.x,
            y: self.x_unit.y - self.origin.y,
        };
        let dy = ScreenPoint {
            x: self.y_unit.x - self.origin.x,
            y: self.y_unit.y - self.origin.y,
        };
        let lx = dx.x.hypot(dx.y);
        let ly = dy.x.hypot(dy.y);
        if !lx.is_finite()
            || lx == 0.
            || !ly.is_finite()
            || (lx - ly).abs() > 1e-9 * lx.max(ly)
            || (dx.x * dy.x + dx.y * dy.y).abs() > 1e-9 * lx * ly
        {
            return None;
        }
        Some(lx)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EquationValue {
    /// Constraint parameter for a point on a segment or normalized turn on a circle.
    PointValue(f64),
    Line {
        slope: f64,
        intercept: f64,
    },
    VerticalLine {
        x: f64,
    },
    Circle {
        center: ScreenPoint,
        radius: f64,
    },
}

pub fn segment_point_value(
    point: ScreenPoint,
    a: ScreenPoint,
    b: ScreenPoint,
) -> Option<EquationValue> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length_sq = dx.mul_add(dx, dy * dy);
    if !length_sq.is_finite() || length_sq == 0. {
        return None;
    }
    let t = ((point.x - a.x) * dx + (point.y - a.y) * dy) / length_sq;
    t.is_finite().then_some(EquationValue::PointValue(t))
}

pub fn circle_point_value(unit_x: f64, unit_y: f64) -> Option<EquationValue> {
    if !unit_x.is_finite()
        || !unit_y.is_finite()
        || (unit_x * unit_x + unit_y * unit_y - 1.).abs() > 1e-6
    {
        return None;
    }
    Some(EquationValue::PointValue(
        unit_y.atan2(unit_x).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU,
    ))
}

pub fn line_equation(
    frame: CartesianFrame,
    a: ScreenPoint,
    b: ScreenPoint,
) -> Option<EquationValue> {
    let a = frame.coordinates(a)?;
    let b = frame.coordinates(b)?;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    if dx.abs() <= 1e-12 * a.x.abs().max(b.x.abs()).max(1.) {
        return (dy.abs() > 1e-12).then_some(EquationValue::VerticalLine {
            x: (a.x + b.x) / 2.,
        });
    }
    let slope = dy / dx;
    let intercept = a.y - slope * a.x;
    (slope.is_finite() && intercept.is_finite()).then_some(EquationValue::Line { slope, intercept })
}

pub fn circle_equation(
    frame: CartesianFrame,
    center: ScreenPoint,
    edge: ScreenPoint,
) -> Option<EquationValue> {
    let scale = frame.square_unit_scale()?;
    let radius = (edge.x - center.x).hypot(edge.y - center.y) / scale;
    if !radius.is_finite() || radius <= 0. {
        return None;
    }
    Some(EquationValue::Circle {
        center: frame.coordinates(center)?,
        radius,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueEquationGroup {
    pub kind: ValueEquationKind,
    /// Point, host for PointValue; line/circle, frame for equations.
    pub references: [u32; 2],
    pub visible: bool,
}

#[derive(Clone, Debug)]
struct Record {
    kind: u32,
    start: usize,
    payload: Range<usize>,
    end: usize,
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn records(b: &[u8], start: usize) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    let mut at = start;
    while at < b.len() {
        if b.len() - at < 8 {
            return Err("Truncated GSP record".into());
        }
        let len = u32_at(b, at) as usize;
        let begin = at + 8;
        let end_payload = begin.checked_add(len).ok_or("Record length overflow")?;
        let end = end_payload
            .checked_add(len & 1)
            .ok_or("Record length overflow")?;
        if end > b.len() {
            return Err("Truncated GSP payload".into());
        }
        out.push(Record {
            kind: u32_at(b, at + 4),
            start: at,
            payload: begin..end_payload,
            end,
        });
        at = end;
    }
    Ok(out)
}

pub fn decode_group(group: &[u8]) -> Result<ValueEquationGroup, String> {
    let rs = records(group, 0)?;
    if rs.len() != 5
        || rs[0].kind != 0x07d0
        || rs[0].payload.len() != 28
        || rs[1].kind != 0x07d2
        || rs[3].kind != 0x07d8
        || rs[4].kind != 0x07d7
        || rs[4].end != group.len()
    {
        return Err("Unsupported Value/Equation group layout".into());
    }
    let header = &group[rs[0].payload.clone()];
    let kind = ValueEquationKind::from_code(u16::from_le_bytes(header[..2].try_into().unwrap()))
        .ok_or("Unsupported Value/Equation code")?;
    let refs = &group[rs[1].payload.clone()];
    if refs.len() != 12 || u32_at(refs, 0) != 2 || u32_at(refs, 4) == 0 || u32_at(refs, 8) == 0 {
        return Err("Invalid Value/Equation references".into());
    }
    let aux = &group[rs[2].payload.clone()];
    match kind {
        ValueEquationKind::PointValue => {
            if rs[2].kind != 0x0903
                || aux.len() != 32
                || aux[..12] != [0xfc, 0x08, 0, 0, 0x16, 0, 0, 0, 4, 0, 0, 0]
                || !matches!(aux[12..14], [0x0a, 0] | [0x4a, 0])
                || aux[18..22] != [0x3c, 0x30, 0x3e, 0]
                || u32_at(aux, 24) != 6
                || u16::from_le_bytes(aux[28..30].try_into().unwrap()) > 5
            {
                return Err("Unsupported Value of Point format".into());
            }
        }
        ValueEquationKind::LineEquation | ValueEquationKind::CircleEquation => {
            if rs[2].kind != 0x08fc
                || aux.len() < 20
                || u32_at(aux, 0) != 0x08fc
                || !matches!(aux.len().checked_sub(u32_at(aux, 4) as usize), Some(0 | 2))
                || u32_at(aux, 8) != 4
            {
                return Err("Unsupported equation text format".into());
            }
        }
    }
    let style = &group[rs[3].payload.clone()];
    if style.len() != 2 || !matches!(style, [1 | 2 | 3, 0]) {
        return Err("Unsupported measurement style".into());
    }
    Ok(ValueEquationGroup {
        kind,
        references: [u32_at(refs, 4), u32_at(refs, 8)],
        visible: u32_at(header, 0) & 0x10000 == 0,
    })
}

fn template(kind: ValueEquationKind) -> &'static [u8] {
    match kind {
        ValueEquationKind::PointValue => {
            include_bytes!("../tests/fixtures/value_equation_point_value_original.gsp")
        }
        ValueEquationKind::LineEquation => {
            include_bytes!("../tests/fixtures/value_equation_line_original.gsp")
        }
        ValueEquationKind::CircleEquation => {
            include_bytes!("../tests/fixtures/value_equation_circle_original.gsp")
        }
    }
}
fn last_group(bytes: &[u8]) -> Result<Range<usize>, String> {
    if !bytes.starts_with(b"GSP4") {
        return Err("Expected GSP4 document".into());
    }
    let rs = records(bytes, 4)?;
    let last = rs
        .iter()
        .rposition(|r| r.kind == 0x07d7)
        .ok_or("No object group")?;
    let first = rs[..last]
        .iter()
        .rposition(|r| r.kind == 0x07d0)
        .ok_or("No group header")?;
    Ok(rs[first].start..rs[last].end)
}

/// Remap the two object references in the original observed group, preserving
/// opaque presentation records. The original recalculates displayed numbers.
pub fn encode_group(kind: ValueEquationKind, references: [u32; 2]) -> Result<Vec<u8>, String> {
    encode_group_from_template(template(kind), references)
}

fn encode_group_from_template(source: &[u8], references: [u32; 2]) -> Result<Vec<u8>, String> {
    if references.contains(&0) || references[0] == references[1] {
        return Err("Invalid Value/Equation references".into());
    }
    let range = last_group(source)?;
    let mut group = source[range].to_vec();
    let rs = records(&group, 0)?;
    let begin = rs[1].payload.start + 4;
    group[begin..begin + 4].copy_from_slice(&references[0].to_le_bytes());
    group[begin + 4..begin + 8].copy_from_slice(&references[1].to_le_bytes());
    decode_group(&group)?;
    Ok(group)
}

pub fn encode_arc_point_value_group(
    references: [u32; 2],
    through_three_points: bool,
    parameter: f64,
) -> Result<Vec<u8>, String> {
    if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
        return Err("Invalid point-on-arc parameter".into());
    }
    let source: &[u8] = if through_three_points {
        include_bytes!("../tests/fixtures/point_on_arc_value_native.gsp")
    } else {
        include_bytes!("../tests/fixtures/point_on_circle_arc_value_native.gsp")
    };
    let mut group = encode_group_from_template(source, references)?;
    if through_three_points {
        let rs = records(&group, 0)?;
        let aux = rs[2].payload.start;
        group[aux + 30..aux + 32].copy_from_slice(&parameter.to_le_bytes()[6..8]);
    }
    decode_group(&group)?;
    Ok(group)
}

/// Native Value of Point uses zeroed path-format fields for rays and lines.
pub fn encode_straight_point_value_group(references: [u32; 2]) -> Result<Vec<u8>, String> {
    let mut group = encode_group(ValueEquationKind::PointValue, references)?;
    let rs = records(&group, 0)?;
    let aux = rs[2].payload.start;
    group[aux + 22..aux + 24].fill(0);
    group[aux + 30..aux + 32].fill(0);
    decode_group(&group)?;
    Ok(group)
}

/// Append one measurement to a single-page base sketch, updating count and
/// the observed 0x2328 selection marker. Host kinds are checked first.
pub fn append_to_base(
    base: &[u8],
    kind: ValueEquationKind,
    references: [u32; 2],
) -> Result<Vec<u8>, String> {
    if !base.starts_with(b"GSP4") {
        return Err("Expected GSP4 document".into());
    }
    let rs = records(base, 4)?;
    let pages = rs.iter().filter(|r| r.kind == 0x03e8).collect::<Vec<_>>();
    if pages.len() != 1 || pages[0].payload.len() < 28 {
        return Err("Expected one page".into());
    }
    if rs.iter().any(|r| r.kind == 0x2328) {
        return Err("Base already has a measurement marker".into());
    }
    let groups = rs.iter().filter(|r| r.kind == 0x07d0).collect::<Vec<_>>();
    let count = groups.len() as u32;
    if u32_at(base, pages[0].payload.start + 24) != count
        || references.iter().any(|&id| id == 0 || id > count)
    {
        return Err("Invalid base object count or references".into());
    }
    let code = |id: u32| {
        u16::from_le_bytes(
            base[groups[id as usize - 1].payload.start..groups[id as usize - 1].payload.start + 2]
                .try_into()
                .unwrap(),
        )
    };
    match kind {
        ValueEquationKind::PointValue
            if code(references[0]) == 15 && matches!(code(references[1]), 2 | 3 | 63 | 64) => {}
        ValueEquationKind::LineEquation
            if code(references[0]) == 63 && code(references[1]) == 61 => {}
        ValueEquationKind::CircleEquation
            if code(references[0]) == 3 && code(references[1]) == 61 => {}
        _ => return Err("Unsupported Value/Equation object dependencies".into()),
    }
    let trailer = rs
        .iter()
        .find(|r| r.kind == 0x03e9)
        .ok_or("Missing page trailer")?
        .start;
    let insert = rs
        .iter()
        .find(|r| r.kind == 0x232d)
        .map(|r| r.start)
        .unwrap_or(trailer);
    let mut result = base.to_vec();
    let page_count = pages[0].payload.start + 24;
    result[page_count..page_count + 4].copy_from_slice(&(count + 1).to_le_bytes());
    let mut added =
        if kind == ValueEquationKind::PointValue && matches!(code(references[1]), 63 | 64) {
            encode_straight_point_value_group(references)?
        } else {
            encode_group(kind, references)?
        };
    added.extend_from_slice(&4u32.to_le_bytes());
    added.extend_from_slice(&0x2328u32.to_le_bytes());
    added.extend_from_slice(&(count + 1).to_le_bytes());
    result.splice(insert..insert, added);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> CartesianFrame {
        CartesianFrame {
            origin: ScreenPoint { x: 199., y: 125. },
            x_unit: ScreenPoint {
                x: 236.79527559055118,
                y: 125.,
            },
            y_unit: ScreenPoint {
                x: 199.,
                y: 87.20472440944882,
            },
        }
    }
    #[test]
    fn original_groups_decode_and_reencode() {
        for (kind, refs) in [
            (ValueEquationKind::PointValue, [4, 3]),
            (ValueEquationKind::LineEquation, [3, 9]),
            (ValueEquationKind::CircleEquation, [3, 9]),
        ] {
            let source = template(kind);
            let range = last_group(source).unwrap();
            let decoded = decode_group(&source[range.clone()]).unwrap();
            assert_eq!(decoded.kind, kind);
            assert_eq!(decoded.references, refs);
            assert_eq!(encode_group(kind, refs).unwrap(), source[range]);
        }
    }
    #[test]
    fn ray_and_line_point_value_groups_match_native() {
        for (base, native) in [
            (
                include_bytes!("../tests/fixtures/point_on_line_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/point_on_line_value_native.gsp").as_slice(),
            ),
            (
                include_bytes!("../tests/fixtures/point_on_ray_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/point_on_ray_value_native.gsp").as_slice(),
            ),
        ] {
            let native_group = &native[last_group(native).unwrap()];
            assert_eq!(
                encode_straight_point_value_group([4, 3]).unwrap(),
                native_group
            );
            assert_eq!(decode_group(native_group).unwrap().references, [4, 3]);
            let generated = append_to_base(base, ValueEquationKind::PointValue, [4, 3]).unwrap();
            assert_eq!(&generated[last_group(&generated).unwrap()], native_group);
        }
    }
    #[test]
    fn both_arc_point_value_groups_match_native() {
        for (native, references, through_three_points, parameter) in [
            (
                include_bytes!("../tests/fixtures/point_on_arc_value_native.gsp").as_slice(),
                [5, 4],
                true,
                0.6145401831027362,
            ),
            (
                include_bytes!("../tests/fixtures/point_on_arc_value_t04_native.gsp").as_slice(),
                [5, 4],
                true,
                0.3999999999999999,
            ),
            (
                include_bytes!("../tests/fixtures/point_on_circle_arc_value_native.gsp").as_slice(),
                [7, 6],
                false,
                0.5999999999999999,
            ),
            (
                include_bytes!("../tests/fixtures/point_on_circle_arc_value_t08_native.gsp")
                    .as_slice(),
                [7, 6],
                false,
                0.8,
            ),
        ] {
            let group = &native[last_group(native).unwrap()];
            assert_eq!(decode_group(group).unwrap().references, references);
            assert_eq!(
                encode_arc_point_value_group(references, through_three_points, parameter).unwrap(),
                group
            );
        }
    }
    #[test]
    fn point_value_tracks_segment_and_circle_constraint() {
        let a = ScreenPoint { x: 150., y: 120. };
        let b = ScreenPoint { x: 250., y: 120. };
        assert_eq!(
            segment_point_value(ScreenPoint { x: 200., y: 120. }, a, b),
            Some(EquationValue::PointValue(0.5))
        );
        assert_eq!(
            segment_point_value(ScreenPoint { x: 220., y: 120. }, a, b),
            Some(EquationValue::PointValue(0.7))
        );
        assert!(segment_point_value(a, a, a).is_none());
        let value = circle_point_value((-2.41784f64).cos(), -(-2.41784f64).sin()).unwrap();
        let EquationValue::PointValue(value) = value else {
            panic!()
        };
        assert!((value - 0.3848).abs() < 0.001);
    }
    #[test]
    fn equations_track_edits_and_vertical_edge_case() {
        let a = ScreenPoint { x: 150., y: 120. };
        let b = ScreenPoint { x: 250., y: 170. };
        let Some(EquationValue::Line { slope, intercept }) = line_equation(frame(), a, b) else {
            panic!()
        };
        assert!((slope + 0.5).abs() < 1e-12);
        assert!((intercept + 0.5159375).abs() < 0.001);
        let moved = ScreenPoint { x: 270., y: 150. };
        let Some(EquationValue::Line { slope, intercept }) = line_equation(frame(), a, moved)
        else {
            panic!()
        };
        assert!((slope + 0.25).abs() < 1e-12);
        assert!((intercept + 0.191).abs() < 0.01);
        let Some(EquationValue::VerticalLine { x }) =
            line_equation(frame(), a, ScreenPoint { x: 150., y: 170. })
        else {
            panic!()
        };
        assert!((x + 1.2964583).abs() < 1e-6);
        let Some(EquationValue::Circle { center, radius }) =
            circle_equation(frame(), a, ScreenPoint { x: 220., y: 120. })
        else {
            panic!()
        };
        assert!((center.x + 1.2964583).abs() < 1e-6);
        assert!((radius - 1.85208).abs() < 0.001);
        let Some(EquationValue::Circle { radius, .. }) =
            circle_equation(frame(), a, ScreenPoint { x: 240., y: 120. })
        else {
            panic!()
        };
        assert!((radius - 2.38125).abs() < 0.001);
    }
    #[test]
    fn generated_documents_and_edited_originals_decode() {
        for (base, kind, refs) in [
            (
                include_bytes!("../tests/fixtures/value_equation_point_value_base.gsp").as_slice(),
                ValueEquationKind::PointValue,
                [4, 3],
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_line_base.gsp").as_slice(),
                ValueEquationKind::LineEquation,
                [3, 9],
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_circle_base.gsp").as_slice(),
                ValueEquationKind::CircleEquation,
                [3, 9],
            ),
        ] {
            let generated = append_to_base(base, kind, refs).unwrap();
            let group = last_group(&generated).unwrap();
            assert_eq!(decode_group(&generated[group]).unwrap().references, refs);
        }
        for (source, kind) in [
            (
                include_bytes!("../tests/fixtures/value_equation_point_value_moved.gsp").as_slice(),
                ValueEquationKind::PointValue,
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_line_moved.gsp").as_slice(),
                ValueEquationKind::LineEquation,
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_circle_moved.gsp").as_slice(),
                ValueEquationKind::CircleEquation,
            ),
        ] {
            let group = last_group(source).unwrap();
            assert_eq!(decode_group(&source[group]).unwrap().kind, kind);
        }
    }
    #[test]
    fn refuses_invalid_references_and_auxiliary_layout() {
        assert!(encode_group(ValueEquationKind::LineEquation, [0, 9]).is_err());
        assert!(
            append_to_base(
                include_bytes!("../tests/fixtures/value_equation_segment_base.gsp"),
                ValueEquationKind::LineEquation,
                [3, 9]
            )
            .is_err()
        );
        let mut group = encode_group(ValueEquationKind::PointValue, [4, 3]).unwrap();
        let rs = records(&group, 0).unwrap();
        group[rs[2].payload.start + 2] = 0xff;
        assert!(decode_group(&group).is_err());
    }

    #[test]
    fn remapped_wine_files_and_vertical_variant_reparse() {
        for (source, kind, refs) in [
            (
                include_bytes!("../tests/fixtures/value_equation_generated_point.gsp").as_slice(),
                ValueEquationKind::PointValue,
                [5, 4],
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_generated_line.gsp").as_slice(),
                ValueEquationKind::LineEquation,
                [4, 10],
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_generated_circle.gsp").as_slice(),
                ValueEquationKind::CircleEquation,
                [4, 10],
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_vertical_original.gsp").as_slice(),
                ValueEquationKind::LineEquation,
                [3, 9],
            ),
            (
                include_bytes!("../tests/fixtures/value_equation_circle_point_value_original.gsp")
                    .as_slice(),
                ValueEquationKind::PointValue,
                [4, 3],
            ),
        ] {
            let group = last_group(source).unwrap();
            let decoded = decode_group(&source[group]).unwrap();
            assert_eq!((decoded.kind, decoded.references), (kind, refs));
        }
    }
}
