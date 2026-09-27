//! Record support for the four Cartesian point measurements observed in
//! Sketchpad 5. Unknown layouts are rejected rather than rewritten.

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateMeasureKind {
    Coordinates,
    Abscissa,
    Ordinate,
    CoordinateDistance,
}

impl CoordinateMeasureKind {
    pub fn code(self) -> u16 {
        match self {
            Self::Coordinates => 49,
            Self::Abscissa => 65,
            Self::Ordinate => 66,
            Self::CoordinateDistance => 86,
        }
    }
    fn from_code(code: u16) -> Option<Self> {
        match code {
            49 => Some(Self::Coordinates),
            65 => Some(Self::Abscissa),
            66 => Some(Self::Ordinate),
            86 => Some(Self::CoordinateDistance),
            _ => None,
        }
    }
    pub fn reference_count(self) -> usize {
        if self == Self::CoordinateDistance {
            3
        } else {
            2
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateBasis {
    pub origin: ScreenPoint,
    pub x_unit: ScreenPoint,
    pub y_unit: ScreenPoint,
}

impl CoordinateBasis {
    pub fn coordinates(self, point: ScreenPoint) -> Option<ScreenPoint> {
        let xx = self.x_unit.x - self.origin.x;
        let xy = self.x_unit.y - self.origin.y;
        let yx = self.y_unit.x - self.origin.x;
        let yy = self.y_unit.y - self.origin.y;
        let dx = point.x - self.origin.x;
        let dy = point.y - self.origin.y;
        let determinant = xx.mul_add(yy, -xy * yx);
        let norm = xx.hypot(xy) * yx.hypot(yy);
        if norm == 0.0
            || !norm.is_finite()
            || !determinant.is_finite()
            || determinant.abs() <= 1e-12 * norm
        {
            return None;
        }
        let x = dx.mul_add(yy, -dy * yx) / determinant;
        let y = xx.mul_add(dy, -xy * dx) / determinant;
        (x.is_finite() && y.is_finite()).then_some(ScreenPoint { x, y })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordinateMeasureValue {
    Pair(ScreenPoint),
    Scalar(f64),
}

pub fn evaluate(
    kind: CoordinateMeasureKind,
    frame: CoordinateBasis,
    first: ScreenPoint,
    second: Option<ScreenPoint>,
) -> Option<CoordinateMeasureValue> {
    let a = frame.coordinates(first)?;
    match kind {
        CoordinateMeasureKind::Coordinates => Some(CoordinateMeasureValue::Pair(a)),
        CoordinateMeasureKind::Abscissa => Some(CoordinateMeasureValue::Scalar(a.x)),
        CoordinateMeasureKind::Ordinate => Some(CoordinateMeasureValue::Scalar(a.y)),
        CoordinateMeasureKind::CoordinateDistance => {
            let b = frame.coordinates(second?)?;
            let distance = (a.x - b.x).hypot(a.y - b.y);
            distance
                .is_finite()
                .then_some(CoordinateMeasureValue::Scalar(distance))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinateMeasurement {
    pub kind: CoordinateMeasureKind,
    /// Point, frame for scalar/pair measurements; point A, point B, frame for distance.
    pub references: Vec<u32>,
    pub visible: bool,
}

#[derive(Clone, Debug)]
struct Record {
    kind: u32,
    start: usize,
    payload: Range<usize>,
    end: usize,
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn records(bytes: &[u8], start: usize) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    let mut at = start;
    while at < bytes.len() {
        if bytes.len() - at < 8 {
            return Err("Truncated GSP record".into());
        }
        let len = u32_at(bytes, at) as usize;
        let payload_start = at + 8;
        let payload_end = payload_start
            .checked_add(len)
            .ok_or("Record length overflow")?;
        let end = payload_end
            .checked_add(len & 1)
            .ok_or("Record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated GSP payload".into());
        }
        out.push(Record {
            kind: u32_at(bytes, at + 4),
            start: at,
            payload: payload_start..payload_end,
            end,
        });
        at = end;
    }
    Ok(out)
}

pub fn decode_group(group: &[u8]) -> Result<CoordinateMeasurement, String> {
    let rs = records(group, 0)?;
    if rs.len() < 4
        || rs[0].kind != 0x07d0
        || rs.last().unwrap().kind != 0x07d7
        || rs.last().unwrap().end != group.len()
        || rs[0].payload.len() != 28
    {
        return Err("Invalid coordinate measurement group".into());
    }
    let header = &group[rs[0].payload.clone()];
    let kind =
        CoordinateMeasureKind::from_code(u16::from_le_bytes(header[..2].try_into().unwrap()))
            .ok_or("Unsupported coordinate measurement code")?;
    let labeled = rs.get(3).is_some_and(|r| r.kind == 0x07d5);
    let style_index = if labeled { 4 } else { 3 };
    let expected: &[u32] = match kind {
        CoordinateMeasureKind::Coordinates => &[0x07d0, 0x07d2, 0x08fc, 0x07d8, 0x07d7],
        _ => &[0x07d0, 0x07d2, 0x0903, 0x07d8, 0x07d7],
    };
    if rs
        .iter()
        .enumerate()
        .filter(|(index, _)| !labeled || *index != 3)
        .map(|(_, record)| record.kind)
        .collect::<Vec<_>>()
        != expected
    {
        return Err("Unsupported coordinate measurement record layout".into());
    }
    let refs = &group[rs[1].payload.clone()];
    if refs.len() != 4 + kind.reference_count() * 4
        || u32_at(refs, 0) as usize != kind.reference_count()
    {
        return Err("Invalid coordinate measurement references".into());
    }
    let references = (0..kind.reference_count())
        .map(|i| u32_at(refs, 4 + i * 4))
        .collect::<Vec<_>>();
    if references.contains(&0)
        || references[..kind.reference_count() - 1].contains(references.last().unwrap())
    {
        return Err("Invalid coordinate measurement dependency".into());
    }
    let aux = &group[rs[2].payload.clone()];
    if kind == CoordinateMeasureKind::Coordinates {
        if aux.len() < 20
            || u32_at(aux, 0) != 0x08fc
            || u32_at(aux, 4) as usize != aux.len()
            || u32_at(aux, 8) != 4
        {
            return Err("Unsupported coordinate readout text".into());
        }
    } else if aux.len() != 32
        || aux[..12] != [0xfc, 0x08, 0, 0, 0x16, 0, 0, 0, 4, 0, 0, 0]
        || aux[18..22] != [0x3c, 0x30, 0x3e, 0]
        || u16::from_le_bytes(aux[24..26].try_into().unwrap()) != 6
        || !matches!(
            u16::from_le_bytes(aux[26..28].try_into().unwrap()),
            0 | 1 | 3
        )
        || u16::from_le_bytes(aux[28..30].try_into().unwrap()) > 5
    {
        return Err("Unsupported coordinate measurement format".into());
    }
    // The final presentation word varies in native coordinate records, as in
    // ordinary measurements. Preserve it without using it as geometry data.
    if labeled {
        crate::text_records::LabelAux::parse(&group[rs[3].payload.clone()])?;
    }
    let style = &group[rs[style_index].payload.clone()];
    if style.len() != 2 {
        return Err("Unsupported coordinate measurement style".into());
    }
    Ok(CoordinateMeasurement {
        kind,
        references,
        visible: u32_at(header, 0) & 0x10000 == 0,
    })
}

fn source(kind: CoordinateMeasureKind) -> &'static [u8] {
    match kind {
        CoordinateMeasureKind::Coordinates => {
            include_bytes!("../tests/fixtures/coordinate_measure_menu257.gsp")
        }
        CoordinateMeasureKind::Abscissa => {
            include_bytes!("../tests/fixtures/coordinate_measure_menu274.gsp")
        }
        CoordinateMeasureKind::Ordinate => {
            include_bytes!("../tests/fixtures/coordinate_measure_menu291.gsp")
        }
        CoordinateMeasureKind::CoordinateDistance => {
            include_bytes!("../tests/fixtures/coordinate_measure_menu308.gsp")
        }
    }
}

fn last_group(bytes: &[u8]) -> Result<Range<usize>, String> {
    if !bytes.starts_with(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let rs = records(bytes, 4)?;
    let end = rs
        .iter()
        .rposition(|r| r.kind == 0x07d7)
        .ok_or("No object group")?;
    let start = rs[..end]
        .iter()
        .rposition(|r| r.kind == 0x07d0)
        .ok_or("No object header")?;
    Ok(rs[start].start..rs[end].end)
}

/// Encode the observed default label/layout, patching only object references.
/// IDs are page-local GSP IDs; callers must validate their object kinds.
pub fn encode_group(kind: CoordinateMeasureKind, references: &[u32]) -> Result<Vec<u8>, String> {
    if references.len() != kind.reference_count() || references.contains(&0) {
        return Err("Invalid coordinate measurement references".into());
    }
    let template = source(kind);
    let group = last_group(template)?;
    let mut output = template[group].to_vec();
    let rs = records(&output, 0)?;
    let payload = rs[1].payload.clone();
    for (i, reference) in references.iter().enumerate() {
        output[payload.start + 4 + i * 4..payload.start + 8 + i * 4]
            .copy_from_slice(&reference.to_le_bytes());
    }
    decode_group(&output)?;
    Ok(output)
}

/// The observed pair readout stores one literal ASCII point name in its rich text.
/// Keep the record lengths fixed; reject names outside that layout.
pub fn set_coordinates_name(group: &mut [u8], name: &str) -> Result<(), String> {
    if decode_group(group)?.kind != CoordinateMeasureKind::Coordinates {
        return Err("Expected a Coordinates readout".into());
    }
    let [letter] = name.as_bytes() else {
        return Err("Coordinates readout requires a one-letter ASCII point name".into());
    };
    if !letter.is_ascii_alphabetic() {
        return Err("Coordinates readout requires a one-letter ASCII point name".into());
    }
    let marker = b"<T1xA>";
    let positions = group
        .windows(marker.len())
        .enumerate()
        .filter(|(_, window)| *window == marker)
        .map(|(at, _)| at)
        .collect::<Vec<_>>();
    if positions.len() != 1 {
        return Err("Coordinates readout name template changed".into());
    }
    group[positions[0] + 4] = *letter;
    decode_group(group)?;
    Ok(())
}

/// A reproducible writer proof against the original nine-object frame fixture.
pub fn encode_on_original_base(
    kind: CoordinateMeasureKind,
    references: &[u32],
) -> Result<Vec<u8>, String> {
    let mut bytes = include_bytes!("../tests/fixtures/coordinate_measure_base.gsp").to_vec();
    if !bytes.starts_with(b"GSP4") {
        return Err("Invalid base document".into());
    }
    let rs = records(&bytes, 4)?;
    let page = rs
        .iter()
        .find(|r| r.kind == 0x03e8)
        .ok_or("No page record")?;
    if u32_at(&bytes, page.payload.start + 24) != 9 {
        return Err("Unexpected base object count".into());
    }
    let page_count_offset = page.payload.start + 24;
    let link = rs
        .iter()
        .find(|r| r.kind == 0x232d)
        .ok_or("No coordinate-system link")?;
    if u32_at(&bytes, link.payload.start) != 9 {
        return Err("Unexpected coordinate-system link".into());
    }
    if references.last() != Some(&9)
        || references[..references.len() - 1]
            .iter()
            .any(|id| !matches!(id, 1 | 2 | 4))
    {
        return Err("References do not belong to the base sketch".into());
    }
    bytes[page_count_offset..page_count_offset + 4].copy_from_slice(&10u32.to_le_bytes());
    let mut inserted = encode_group(kind, references)?;
    inserted.extend_from_slice(&4u32.to_le_bytes());
    inserted.extend_from_slice(&0x2328u32.to_le_bytes());
    inserted.extend_from_slice(&10u32.to_le_bytes());
    bytes.splice(link.start..link.start, inserted);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_variants_decode_and_reencode() {
        for (kind, refs) in [
            (CoordinateMeasureKind::Coordinates, vec![1, 9]),
            (CoordinateMeasureKind::Abscissa, vec![1, 9]),
            (CoordinateMeasureKind::Ordinate, vec![1, 9]),
            (CoordinateMeasureKind::CoordinateDistance, vec![1, 4, 9]),
        ] {
            let bytes = source(kind);
            let range = last_group(bytes).unwrap();
            let decoded = decode_group(&bytes[range.clone()]).unwrap();
            assert_eq!(decoded.kind, kind);
            assert_eq!(decoded.references, refs);
            assert_eq!(encode_group(kind, &refs).unwrap(), bytes[range]);
        }
    }
    #[test]
    fn coordinate_values_follow_original_drags() {
        let frame = CoordinateBasis {
            origin: ScreenPoint { x: 199., y: 125. },
            x_unit: ScreenPoint {
                x: 236.79527559055118,
                y: 125.,
            },
            y_unit: ScreenPoint {
                x: 199.,
                y: 87.20472440944882,
            },
        };
        let a = ScreenPoint { x: 150., y: 120. };
        let moved = ScreenPoint { x: 170., y: 100. };
        let origin = frame.origin;
        let CoordinateMeasureValue::Pair(initial) =
            evaluate(CoordinateMeasureKind::Coordinates, frame, a, None).unwrap()
        else {
            panic!()
        };
        assert!((initial.x + 1.296458333333333).abs() < 1e-6);
        assert!((initial.y - 0.132291666666667).abs() < 1e-6);
        let CoordinateMeasureValue::Pair(after) =
            evaluate(CoordinateMeasureKind::Coordinates, frame, moved, None).unwrap()
        else {
            panic!()
        };
        assert!((after.x + 0.767291666666667).abs() < 1e-6);
        assert!((after.y - 0.661458333333333).abs() < 1e-6);
        let CoordinateMeasureValue::Scalar(distance) = evaluate(
            CoordinateMeasureKind::CoordinateDistance,
            frame,
            moved,
            Some(origin),
        )
        .unwrap() else {
            panic!()
        };
        assert!((distance - 1.013).abs() < 0.01);
        assert!(
            evaluate(
                CoordinateMeasureKind::CoordinateDistance,
                frame,
                moved,
                None
            )
            .is_none()
        );
    }
    #[test]
    fn rejects_unknown_format_and_invalid_ids() {
        let mut group = encode_group(CoordinateMeasureKind::Abscissa, &[1, 9]).unwrap();
        let rs = records(&group, 0).unwrap();
        group[rs[2].payload.start + 26..rs[2].payload.start + 28]
            .copy_from_slice(&2_u16.to_le_bytes());
        assert!(decode_group(&group).is_err());
        group[rs[2].payload.start + 26..rs[2].payload.start + 28]
            .copy_from_slice(&0_u16.to_le_bytes());
        group[rs[2].payload.start + 2] = 0xff;
        assert!(decode_group(&group).is_err());
        assert!(encode_group(CoordinateMeasureKind::Ordinate, &[0, 9]).is_err());
        assert!(encode_on_original_base(CoordinateMeasureKind::Coordinates, &[3, 9]).is_err());
    }
    #[test]
    fn generated_documents_have_one_measurement_and_valid_footer() {
        for (kind, refs) in [
            (CoordinateMeasureKind::Coordinates, vec![2, 9]),
            (CoordinateMeasureKind::Abscissa, vec![2, 9]),
            (CoordinateMeasureKind::Ordinate, vec![2, 9]),
            (CoordinateMeasureKind::CoordinateDistance, vec![2, 4, 9]),
        ] {
            let bytes = encode_on_original_base(kind, &refs).unwrap();
            let rs = records(&bytes, 4).unwrap();
            let page = rs.iter().find(|r| r.kind == 0x03e8).unwrap();
            assert_eq!(u32_at(&bytes, page.payload.start + 24), 10);
            let marker = rs.iter().find(|r| r.kind == 0x2328).unwrap();
            assert_eq!(u32_at(&bytes, marker.payload.start), 10);
            let group = last_group(&bytes).unwrap();
            assert_eq!(decode_group(&bytes[group]).unwrap().references, refs);
        }
    }
    #[test]
    fn original_dragged_groups_keep_their_dependencies() {
        for (source, kind, refs) in [
            (
                include_bytes!("../tests/fixtures/coordinate_measure_coordinates_moved.gsp")
                    .as_slice(),
                CoordinateMeasureKind::Coordinates,
                vec![1, 9],
            ),
            (
                include_bytes!("../tests/fixtures/coordinate_measure_distance_moved.gsp")
                    .as_slice(),
                CoordinateMeasureKind::CoordinateDistance,
                vec![1, 4, 9],
            ),
        ] {
            let range = last_group(source).unwrap();
            let decoded = decode_group(&source[range]).unwrap();
            assert_eq!(decoded.kind, kind);
            assert_eq!(decoded.references, refs);
        }
    }
}
