//! Graph plotting records observed with Plot Points and Plot Table Data.
//! Cartesian and polar points from frame-only and parameter sketches are cross-opened.

use std::ops::Range;

#[allow(dead_code)]
#[path = "table_records.rs"]
mod table_records;

const CARTESIAN: &[u8] = include_bytes!("../tests/fixtures/plot_points_table_single_original.gsp");
const POLAR: &[u8] = include_bytes!("../tests/fixtures/plot_points_table_polar_original.gsp");

/// One degree in radians as stored by Sketchpad 5's polar point dialog.
pub const DEGREES_TO_RADIANS: f64 = 0.017453292519943334;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlotPointsMode {
    Cartesian,
    Polar,
}

impl PlotPointsMode {
    pub fn code(self) -> u32 {
        match self {
            Self::Cartesian => 67,
            Self::Polar => 68,
        }
    }
    fn template(self) -> &'static [u8] {
        match self {
            Self::Cartesian => CARTESIAN,
            Self::Polar => POLAR,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotPoint {
    pub mode: PlotPointsMode,
    /// Source anchor first, active coordinate system second.
    pub references: [u32; 2],
    /// Cartesian x/y; polar radius/angle in radians as stored in GSP4.
    pub stored: [f64; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CartesianBasis {
    pub origin: ScreenPoint,
    pub x_unit: ScreenPoint,
    pub y_unit: ScreenPoint,
}

impl PlotPoint {
    pub fn position(self, basis: CartesianBasis) -> Option<ScreenPoint> {
        let [x, y] = match self.mode {
            PlotPointsMode::Cartesian => self.stored,
            PlotPointsMode::Polar => {
                let [radius, angle] = self.stored;
                [radius * angle.cos(), radius * angle.sin()]
            }
        };
        let point = ScreenPoint {
            x: basis.origin.x
                + x * (basis.x_unit.x - basis.origin.x)
                + y * (basis.y_unit.x - basis.origin.x),
            y: basis.origin.y
                + x * (basis.x_unit.y - basis.origin.y)
                + y * (basis.y_unit.y - basis.origin.y),
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }
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

fn records(bytes: &[u8], mut at: usize) -> Result<Vec<Record>, String> {
    let mut result = Vec::new();
    while at < bytes.len() {
        if bytes.len() - at < 8 {
            return Err("Truncated GSP record".into());
        }
        let size = u32_at(bytes, at) as usize;
        let payload = at + 8;
        let payload_end = payload.checked_add(size).ok_or("Record length overflow")?;
        let end = payload_end
            .checked_add(size % 2)
            .ok_or("Record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated GSP record payload".into());
        }
        result.push(Record {
            kind: u32_at(bytes, at + 4),
            start: at,
            payload: payload..payload_end,
            end,
        });
        at = end;
    }
    Ok(result)
}

fn last_group(bytes: &[u8]) -> Result<Range<usize>, String> {
    if !bytes.starts_with(b"GSP4") {
        return Err("Expected GSP4 file".into());
    }
    let rs = records(bytes, 4)?;
    let last = rs
        .iter()
        .rposition(|r| r.kind == 0x07d7)
        .ok_or("Missing object group".to_string())?;
    let first = rs[..last]
        .iter()
        .rposition(|r| r.kind == 0x07d0)
        .ok_or("Missing group header".to_string())?;
    Ok(rs[first].start..rs[last].end)
}

fn object_group(rs: &[Record], ordinal: usize) -> Result<Range<usize>, String> {
    let start = rs
        .iter()
        .filter(|r| r.kind == 0x07d0)
        .nth(ordinal - 1)
        .ok_or("Missing object group")?
        .start;
    let end = rs
        .iter()
        .filter(|r| r.kind == 0x07d7 && r.start > start)
        .next()
        .ok_or("Unterminated object group")?
        .end;
    Ok(start..end)
}

pub fn decode_group(group: &[u8]) -> Result<PlotPoint, String> {
    let rs = records(group, 0)?;
    if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x07d2, 0x07d3, 0x07d7]
        || rs[0].payload.len() != 28
        || rs[1].payload.len() != 8
        || rs[2].payload.len() != 20
    {
        return Err("Unsupported Plot Points record sequence".into());
    }
    let code = u32_at(group, rs[0].payload.start);
    let mode = match code {
        67 => PlotPointsMode::Cartesian,
        68 => PlotPointsMode::Polar,
        _ => return Err("Unsupported Plot Points object code".into()),
    };
    if u32_at(group, rs[2].payload.start) != code {
        return Err("Invalid Plot Points payload tag".into());
    }
    let refs = [
        u32_at(group, rs[1].payload.start),
        u32_at(group, rs[1].payload.start + 4),
    ];
    if refs[0] == 0 || refs[1] == 0 || refs[0] == refs[1] {
        return Err("Invalid Plot Points dependencies".into());
    }
    let stored = [
        f64::from_le_bytes(
            group[rs[2].payload.start + 4..rs[2].payload.start + 12]
                .try_into()
                .unwrap(),
        ),
        f64::from_le_bytes(
            group[rs[2].payload.start + 12..rs[2].payload.start + 20]
                .try_into()
                .unwrap(),
        ),
    ];
    if stored.iter().any(|v| !v.is_finite()) {
        return Err("Non-finite Plot Points value".into());
    }
    Ok(PlotPoint {
        mode,
        references: refs,
        stored,
    })
}

/// Remap dependencies of an original group. The point remains t[1]=1.00 on
/// both coordinates, with the original's exact Cartesian/polar cache bytes.
pub fn encode_t1_group(mode: PlotPointsMode, references: [u32; 2]) -> Result<Vec<u8>, String> {
    let stored = decode_group(&mode.template()[last_group(mode.template())?])?.stored;
    encode_group(mode, references, stored)
}

pub fn encode_group(
    mode: PlotPointsMode,
    references: [u32; 2],
    stored: [f64; 2],
) -> Result<Vec<u8>, String> {
    if references[0] == 0 || references[1] == 0 || references[0] == references[1] {
        return Err("Invalid Plot Points dependencies".into());
    }
    if stored.iter().any(|value| !value.is_finite()) {
        return Err("Non-finite Plot Points coordinates".into());
    }
    let template = mode.template();
    let mut group = template[last_group(template)?].to_vec();
    let rs = records(&group, 0)?;
    group[rs[1].payload.start..rs[1].payload.start + 4]
        .copy_from_slice(&references[0].to_le_bytes());
    group[rs[1].payload.start + 4..rs[1].payload.start + 8]
        .copy_from_slice(&references[1].to_le_bytes());
    group[rs[2].payload.start + 4..rs[2].payload.start + 12]
        .copy_from_slice(&stored[0].to_le_bytes());
    group[rs[2].payload.start + 12..rs[2].payload.start + 20]
        .copy_from_slice(&stored[1].to_le_bytes());
    decode_group(&group)?;
    Ok(group)
}

/// Appends the observed one-column t[1] plot to a single-page original base.
/// The source and frame are validated by ID and code; unsupported table data
/// is rejected instead of silently plotted from an unrelated measurement.
pub fn append_t1_plot(
    base: &[u8],
    mode: PlotPointsMode,
    references: [u32; 2],
) -> Result<Vec<u8>, String> {
    if !base.starts_with(b"GSP4") {
        return Err("Expected GSP4 file".into());
    }
    let rs = records(base, 4)?;
    let page = rs.iter().filter(|r| r.kind == 0x03e8).collect::<Vec<_>>();
    let groups = rs.iter().filter(|r| r.kind == 0x07d0).collect::<Vec<_>>();
    if page.len() != 1 || page[0].payload.len() < 28 || groups.len() != 8 {
        return Err("Expected one-page t[1] table and coordinate system".into());
    }
    let codes = groups
        .iter()
        .map(|r| u32_at(base, r.payload.start) & 0xffff)
        .collect::<Vec<_>>();
    if codes != [0, 91, 0, 52, 58, 54, 58, 61]
        || references != [1, 8]
        || u32_at(base, page[0].payload.start + 24) != 8
    {
        return Err("Unsupported Plot Points source or frame".into());
    }
    let parameter = table_records::ParameterGroup::parse(&base[object_group(&rs, 1)?])?;
    let table = table_records::TableGroup::parse(&base[object_group(&rs, 2)?])?;
    if parameter.name() != "t[1]"
        || parameter.value() != 1.0
        || table.source_id() != 1
        || table.rows() == 0
    {
        return Err("Only a nonempty t[1]=1.00 table is supported".into());
    }
    let table_ref = rs
        .iter()
        .find(|r| r.kind == 0x07d2 && r.start > groups[1].start)
        .ok_or("Missing table source")?;
    if base[table_ref.payload.clone()] != 1_u32.to_le_bytes().repeat(2) {
        return Err("Only a table of t[1] is supported".into());
    }
    let marker = rs
        .iter()
        .find(|r| r.kind == 0x2328)
        .ok_or("Missing selected table marker")?;
    if marker.payload.len() != 4 || u32_at(base, marker.payload.start) != 2 {
        return Err("Expected selected one-column table".into());
    }
    if rs.iter().filter(|r| r.kind == 0x232d).count() != 1 {
        return Err("Expected one active coordinate frame".into());
    }
    let mut output = base.to_vec();
    output[page[0].payload.start + 24..page[0].payload.start + 28]
        .copy_from_slice(&9_u32.to_le_bytes());
    output[marker.payload.clone()].copy_from_slice(&9_u32.to_le_bytes());
    output.splice(
        marker.start..marker.start,
        encode_t1_group(mode, references)?,
    );
    records(&output, 4)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis() -> CartesianBasis {
        CartesianBasis {
            origin: ScreenPoint { x: 259., y: 195. },
            x_unit: ScreenPoint {
                x: 296.79527559055117,
                y: 195.,
            },
            y_unit: ScreenPoint {
                x: 259.,
                y: 157.20472440944883,
            },
        }
    }

    #[test]
    fn original_cartesian_and_polar_groups_decode() {
        let cart = decode_group(&CARTESIAN[last_group(CARTESIAN).unwrap()]).unwrap();
        assert_eq!(cart.mode, PlotPointsMode::Cartesian);
        assert_eq!(cart.references, [1, 8]);
        assert_eq!(cart.stored, [1., 1.]);
        assert_eq!(
            encode_t1_group(cart.mode, cart.references).unwrap(),
            &CARTESIAN[last_group(CARTESIAN).unwrap()]
        );
        let polar = decode_group(&POLAR[last_group(POLAR).unwrap()]).unwrap();
        assert_eq!(polar.mode, PlotPointsMode::Polar);
        assert_eq!(polar.references, [1, 8]);
        assert_eq!(polar.stored[0], 1.);
        assert!((polar.stored[1] - std::f64::consts::PI / 180.).abs() < 1e-15);
        assert_eq!(
            encode_t1_group(polar.mode, polar.references).unwrap(),
            &POLAR[last_group(POLAR).unwrap()]
        );
    }

    #[test]
    fn evaluated_positions_follow_observed_coordinate_modes() {
        let cart = decode_group(&CARTESIAN[last_group(CARTESIAN).unwrap()]).unwrap();
        let polar = decode_group(&POLAR[last_group(POLAR).unwrap()]).unwrap();
        let a = cart.position(basis()).unwrap();
        let b = polar.position(basis()).unwrap();
        assert!((a.x - 296.7953).abs() < 0.01 && (a.y - 157.2047).abs() < 0.01);
        assert!((b.x - 296.7895).abs() < 0.01 && (b.y - 194.3404).abs() < 0.01);
    }

    #[test]
    fn original_two_row_table_still_adds_one_point() {
        let file = include_bytes!("../tests/fixtures/plot_points_table_two_rows_original.gsp");
        let rs = records(file, 4).unwrap();
        assert_eq!(rs.iter().filter(|r| r.kind == 0x07d0).count(), 9);
        assert_eq!(
            decode_group(&file[last_group(file).unwrap()])
                .unwrap()
                .references,
            [1, 8]
        );
    }

    #[test]
    fn bounded_writer_and_invalid_inputs() {
        let base = include_bytes!("../tests/fixtures/plot_points_table_frame.gsp");
        let made = append_t1_plot(base, PlotPointsMode::Cartesian, [1, 8]).unwrap();
        let last = decode_group(&made[last_group(&made).unwrap()]).unwrap();
        assert_eq!(last.stored, [1., 1.]);
        let two_rows = include_bytes!("../tests/fixtures/plot_points_table_two_rows_base.gsp");
        assert!(append_t1_plot(two_rows, PlotPointsMode::Cartesian, [1, 8]).is_ok());
        assert!(append_t1_plot(base, PlotPointsMode::Cartesian, [2, 8]).is_err());
        assert!(append_t1_plot(&made, PlotPointsMode::Cartesian, [1, 8]).is_err());
        let mut broken = encode_t1_group(PlotPointsMode::Cartesian, [1, 8]).unwrap();
        let rs = records(&broken, 0).unwrap();
        broken[rs[2].payload.start + 4..rs[2].payload.start + 12]
            .copy_from_slice(&f64::NAN.to_le_bytes());
        assert!(decode_group(&broken).is_err());
    }
}
