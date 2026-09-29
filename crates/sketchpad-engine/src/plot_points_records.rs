//! Graph plotting records observed with Plot Points and Plot Table Data.
//! Cartesian and polar points from frame-only and parameter sketches are cross-opened.

use crate::table_records::{ParameterGroup, TableGroup};
use std::ops::Range;

const CARTESIAN: &[u8] =
    include_bytes!("../../../tests/fixtures/plot_points_table_single_original.gsp");
const POLAR: &[u8] = include_bytes!("../../../tests/fixtures/plot_points_table_polar_original.gsp");

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
    /// The sole geometric dependency: the active coordinate system.
    pub coordinate_system: u32,
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
        .find(|r| r.kind == 0x07d7 && r.start > start)
        .ok_or("Unterminated object group")?
        .end;
    Ok(start..end)
}

pub fn decode_group(group: &[u8]) -> Result<PlotPoint, String> {
    let rs = records(group, 0)?;
    let kinds = rs.iter().map(|r| r.kind).collect::<Vec<_>>();
    if !matches!(
        kinds.as_slice(),
        [0x07d0, 0x07d2, 0x07d3, 0x07d7]
            | [0x07d0, 0x07d2, 0x07d3, 0x232e, 0x07d7]
            | [0x07d0, 0x07d2, 0x07d3, 0x07d5, 0x07d8, 0x07d7]
            | [0x07d0, 0x07d2, 0x07d3, 0x07d5, 0x07d8, 0x232e, 0x07d7]
    ) || rs[0].payload.len() != 28
        || rs[2].payload.len() != 20
        || !rs.last().unwrap().payload.is_empty()
    {
        return Err("Unsupported Plot Points record sequence".into());
    }
    if kinds[3] == 0x07d5 {
        crate::text_records::LabelAux::parse(&group[rs[3].payload.clone()])?;
        if rs[4].payload.len() != 2 {
            return Err("Invalid Plot Points label font".into());
        }
    }
    if rs
        .iter()
        .any(|record| record.kind == 0x232e && !record.payload.is_empty())
    {
        return Err("Unsupported Plot Points attribute payload".into());
    }
    let header = u32_at(group, rs[0].payload.start);
    let code = header & 0xffff;
    let mode = match code {
        67 => PlotPointsMode::Cartesian,
        68 => PlotPointsMode::Polar,
        _ => return Err("Unsupported Plot Points object code".into()),
    };
    if u32_at(group, rs[2].payload.start) != code {
        return Err("Invalid Plot Points payload tag".into());
    }
    if header >> 16 > 4 || group[rs[0].payload.start + 6..rs[0].payload.start + 10] != [1, 0, 56, 0]
    {
        return Err("Unsupported Plot Points header".into());
    }
    let reference_data = &group[rs[1].payload.clone()];
    // 07d2 starts with a reference count, independent of the visibility flags
    // in the object header. Both Native templates and every observed corpus
    // point store [1, frame]; the first word is not an object dependency.
    if reference_data.len() != 8 || u32_at(reference_data, 0) != 1 {
        return Err("Invalid Plot Points dependency count".into());
    }
    let coordinate_system = u32_at(reference_data, 4);
    if coordinate_system == 0 {
        return Err("Invalid Plot Points coordinate system".into());
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
        coordinate_system,
        stored,
    })
}

/// Remap the frame of an original group. `references` is [count=1, frame].
/// The point remains t[1]=1.00 on
/// both coordinates, with the original's exact Cartesian/polar cache bytes.
pub fn encode_t1_group(mode: PlotPointsMode, references: [u32; 2]) -> Result<Vec<u8>, String> {
    let stored = decode_group(&mode.template()[last_group(mode.template())?])?.stored;
    encode_group(mode, references, stored)
}

/// Encode the observed counted packet `[1, frame]`. The first word is a
/// reference count; it never identifies a source object. Prefer the typed
/// `encode_frame_point_group` entry point for new callers.
pub fn encode_group(
    mode: PlotPointsMode,
    references: [u32; 2],
    stored: [f64; 2],
) -> Result<Vec<u8>, String> {
    if references[0] != 1 || references[1] == 0 {
        return Err("Expected Plot Points reference count 1 and a nonzero frame".into());
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

/// A literal point depending only on its coordinate system. The Native
/// dialog templates and corpus points encode the same counted dependency.
pub fn encode_frame_point_group(
    frame: u32,
    mode: PlotPointsMode,
    stored: [f64; 2],
) -> Result<Vec<u8>, String> {
    encode_group(mode, [1, frame], stored)
}

/// Appends the observed one-column t[1] plot to a single-page original base.
/// The table source and frame are validated by ID and code; the plotted point
/// itself depends only on the frame. `references` is the [count=1, frame] packet.
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
    let parameter = ParameterGroup::parse(&base[object_group(&rs, 1)?])?;
    let table = TableGroup::parse(&base[object_group(&rs, 2)?])?;
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
        assert_eq!(cart.coordinate_system, 8);
        assert_eq!(cart.stored, [1., 1.]);
        assert_eq!(
            encode_t1_group(cart.mode, [1, cart.coordinate_system]).unwrap(),
            &CARTESIAN[last_group(CARTESIAN).unwrap()]
        );
        let polar = decode_group(&POLAR[last_group(POLAR).unwrap()]).unwrap();
        assert_eq!(polar.mode, PlotPointsMode::Polar);
        assert_eq!(polar.coordinate_system, 8);
        assert_eq!(polar.stored[0], 1.);
        assert!((polar.stored[1] - std::f64::consts::PI / 180.).abs() < 1e-15);
        assert_eq!(
            encode_t1_group(polar.mode, [1, polar.coordinate_system]).unwrap(),
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
        let file =
            include_bytes!("../../../tests/fixtures/plot_points_table_two_rows_original.gsp");
        let rs = records(file, 4).unwrap();
        assert_eq!(rs.iter().filter(|r| r.kind == 0x07d0).count(), 9);
        let point = decode_group(&file[last_group(file).unwrap()]).unwrap();
        assert_eq!(point.coordinate_system, 8);
    }

    fn original_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn corpus_literal_points_have_one_frame_dependency_with_optional_labels() {
        // 2024年年历表.gsp object 21 and 心脏线.gsp object 9, extracted
        // from their actual drawing groups, not from reordered HTM object IDs.
        let cart = original_hex(concat!(
            "1c000000d007000043000100000001003800210180000000ffff515cc1bdf0ff00000000",
            "08000000d20700000100000014000000",
            "14000000d30700004300000000000000000000000000000000000000",
            "1a000000d5070000070001000100edffffff00000000000000000000000001004f00",
            "02000000d80700000300",
            "00000000d7070000"
        ));
        let polar = original_hex(concat!(
            "1c000000d0070000440000000000010038002101ff000000ffff515cc1bdf0ff00000000",
            "08000000d20700000100000007000000",
            "14000000d307000044000000000000000000e03f282d4454fb210940",
            "1a000000d50700000600000000000800050000000000986e12c1515c614101004400",
            "02000000d80700000000",
            "00000000d7070000"
        ));
        let a = decode_group(&cart).unwrap();
        assert_eq!(a.coordinate_system, 20);
        assert_eq!(a.mode, PlotPointsMode::Cartesian);
        assert_eq!(a.stored, [0.0, 0.0]);
        let b = decode_group(&polar).unwrap();
        assert_eq!(b.coordinate_system, 7);
        assert_eq!(b.mode, PlotPointsMode::Polar);
        assert_eq!(b.stored, [0.5, crate::table_records::CALCULATOR_PI]);
        let at_pi = b.position(basis()).unwrap();
        assert!((at_pi.x - (259.0 - 0.5 * (basis().x_unit.x - 259.0))).abs() < 1e-12);
        assert!((at_pi.y - 195.0).abs() < 1e-12);
        // Visibility flags are independent of dependency packet grammar.
        for flags in 0_u32..=4 {
            let mut group = cart.clone();
            group[8..12].copy_from_slice(&(67 | flags << 16).to_le_bytes());
            assert_eq!(decode_group(&group).unwrap(), a);
        }
    }

    #[test]
    fn original_empty_attributes_are_bounded_and_do_not_add_dependencies() {
        // 三角函数线.gsp object 97 and 函数图象(田野风).gsp object 1031.
        let labeled = original_hex(concat!(
            "1c000000d0070000430000000000010038002102ffffff00ffff0000c1bdf0ff00000000",
            "08000000d2070000010000005e000000",
            "14000000d307000043000000000000000000f03f0000000000000000",
            "20000000d50700000700010000000800e8ff00000000000000000000000006004128312c302900e4",
            "02000000d80700000100",
            "000000002e230000",
            "00000000d7070000"
        ));
        let bare = original_hex(concat!(
            "1c000000d0070000430001000000010038002101ff000000ffffee3fc1bdf0ff00000000",
            "08000000d2070000010000008f000000",
            "14000000d307000043000000000000000000f03f000000000000f03f",
            "000000002e230000",
            "00000000d7070000"
        ));
        for (group, frame, stored) in [(labeled, 94, [1.0, 0.0]), (bare, 143, [1.0, 1.0])] {
            let point = decode_group(&group).unwrap();
            assert_eq!(point.coordinate_system, frame);
            assert_eq!(point.stored, stored);
            let attribute = records(&group, 0)
                .unwrap()
                .into_iter()
                .find(|record| record.kind == 0x232e)
                .unwrap();
            let mut nonempty = group.clone();
            nonempty[attribute.start..attribute.start + 4].copy_from_slice(&2_u32.to_le_bytes());
            nonempty.splice(attribute.payload.start..attribute.payload.start, [0, 0]);
            assert!(decode_group(&nonempty).is_err());
        }
    }

    #[test]
    fn frame_writer_matches_original_payload_and_rejects_fake_sources() {
        // Unlabeled original object 130 in 2024年年历表.gsp, with its
        // visibility flag cleared because a newly authored point is visible.
        let original_visible = original_hex(concat!(
            "1c000000d0070000430000000000010038002102ff000000ffff515cc1bdf0ff00000000",
            "08000000d20700000100000014000000",
            "14000000d307000043000000000000000000f0bf0000000000000000",
            "00000000d7070000"
        ));
        assert_eq!(
            encode_frame_point_group(20, PlotPointsMode::Cartesian, [-1.0, 0.0]).unwrap(),
            original_visible
        );
        for mode in [PlotPointsMode::Cartesian, PlotPointsMode::Polar] {
            let group = encode_frame_point_group(1, mode, [2.5, -0.75]).unwrap();
            let point = decode_group(&group).unwrap();
            assert_eq!(point.coordinate_system, 1);
            assert_eq!(point.stored, [2.5, -0.75]);
            assert!(encode_frame_point_group(0, mode, [0.0, 0.0]).is_err());
            assert!(encode_frame_point_group(1, mode, [f64::INFINITY, 0.0]).is_err());
            assert!(encode_group(mode, [2, 20], [0.0, 0.0]).is_err());
            let rs = records(&group, 0).unwrap();
            for count in [0_u32, 2, u32::MAX] {
                let mut broken = group.clone();
                broken[rs[1].payload.start..rs[1].payload.start + 4]
                    .copy_from_slice(&count.to_le_bytes());
                assert!(decode_group(&broken).is_err());
            }
            let mut wrong_tag = group.clone();
            wrong_tag[rs[2].payload.start..rs[2].payload.start + 4]
                .copy_from_slice(&66_u32.to_le_bytes());
            assert!(decode_group(&wrong_tag).is_err());
            assert!(decode_group(&group[..group.len() - 1]).is_err());
        }
    }

    #[test]
    fn bounded_writer_and_invalid_inputs() {
        let base = include_bytes!("../../../tests/fixtures/plot_points_table_frame.gsp");
        let made = append_t1_plot(base, PlotPointsMode::Cartesian, [1, 8]).unwrap();
        let last = decode_group(&made[last_group(&made).unwrap()]).unwrap();
        assert_eq!(last.stored, [1., 1.]);
        let two_rows =
            include_bytes!("../../../tests/fixtures/plot_points_table_two_rows_base.gsp");
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
