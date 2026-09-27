use super::calculations::parse_numeric_calculation;
use super::embedded_tools::{document_record_indices, document_records};
use super::iteration::{
    mapped_binding_kind, supported_branch_polygon_mapping, supported_mapped_point_iteration,
    supported_point_iteration_mapping, supported_point_parameter_iteration,
};
use super::kind::{
    expected_locus_refs, expected_parameter_locus_refs, is_arc_kind, is_coordinate_unit_kind,
    is_literal_plot_path_kind, is_locus_polygon_kind, is_locus_straight_kind, is_point_kind,
    is_segment_kind, is_straight_kind, is_value_plot_path_kind, polygon_vertex_count,
    straight_kind_mode,
};
use super::records::{f64_at, page_title_slots, polygon_subtype, records, u32_at};
use super::{
    Document, EmbeddedToolInput, EmbeddedToolInputKind, IterationDepth, Kind, LineMode, Object,
    Point,
};
use crate::action_records::{
    AnimationButtonGroup, HideShowButtonGroup, LinkPageButtonGroup, LinkUrlButtonGroup,
    MovementButtonGroup, PresentationButtonGroup, ScrollButtonGroup,
};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::graph::{GraphObject, GridForm};
use crate::image_records::ImageFile;
use crate::plot_points_records::PlotPointsMode;
use crate::table_records::{
    CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup, NumericUnit,
    ParameterCalculationGroup, ParameterGroup, TableGroup,
};
use crate::text_records::LabelAux;
use crate::value_equation_records::ValueEquationKind;
use crate::{
    coordinate_measure_records, function_records, graph, plot_axis_records, plot_points_records,
    style_records, text_object, value_equation_records,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Native transform literals use radians (0) or either observed degree form
/// (1/2). Check the direction cache in its declared unit before normalizing
/// to the shared model's degrees; preserving saves retain the original record.
pub(super) fn transform_angle_degrees(payload: &[u8]) -> Option<f64> {
    if payload.len() < 32 {
        return None;
    }
    let angle = f64_at(payload, 20);
    let (radians, degrees) = match u32_at(payload, 28) {
        0 => (angle, angle.to_degrees()),
        1 | 2 => (angle.to_radians(), angle),
        _ => return None,
    };
    let sine_cache = f64_at(payload, 4);
    let cosine_cache = f64_at(payload, 12);
    (radians.is_finite()
        && degrees.is_finite()
        && sine_cache.is_finite()
        && cosine_cache.is_finite()
        && (sine_cache + radians.sin()).abs() < 1e-6
        && (cosine_cache - radians.cos()).abs() < 1e-6)
        .then_some(degrees)
}

/// The polar distance cache stores magnitude * pixels-per-unit, as in the
/// Cartesian translation record. Preserve nonstandard imported scales too.
pub(super) fn polar_distance_scale(payload: &[u8]) -> Option<f64> {
    let declared = match u32_at(payload, 48) {
        1 => 37.79527559055118_f64,
        2 => 96.,
        _ => return None,
    };
    let magnitude = f64_at(payload, 40);
    let pixels = f64_at(payload, 32);
    if !magnitude.is_finite() || !pixels.is_finite() {
        return None;
    }
    if magnitude == 0. {
        return (pixels == 0.).then_some(declared);
    }
    let scale = pixels / magnitude;
    if !scale.is_finite() || scale <= 0. {
        return None;
    }
    // Avoid introducing a division rounding error for ordinary cm/inch units.
    Some(if (scale - declared).abs() <= declared * 1e-12 {
        declared
    } else {
        scale
    })
}

/// Structural saves may retain complete registered tool canvases unchanged.
/// Their local object IDs and opaque auxiliary records are never drawing IDs.
pub(super) fn registered_tool_structure_safe(bytes: &[u8], entries: &[super::Record]) -> bool {
    if !entries
        .iter()
        .any(|record| matches!(record.kind, 0x03ea | 0x03eb))
    {
        return true;
    }
    for (index, record) in entries.iter().enumerate() {
        if record.kind == 0x03ea
            && !index
                .checked_sub(1)
                .and_then(|previous| entries.get(previous))
                .is_some_and(|previous| previous.kind == 0x0386)
        {
            return false;
        }
        if record.kind == 0x03eb
            && !entries
                .get(index + 1)
                .is_some_and(|next| next.kind == 0x0387 && next.data.is_empty())
        {
            return false;
        }
    }
    let mut drawing_open = false;
    let mut group_open = false;
    let mut tool_expected = None;
    let mut tool_objects = 0usize;
    let mut tool_inputs = false;
    let mut tools = 0usize;
    let mut pages = 0usize;
    for record in entries {
        match record.kind {
            0x03e8 => {
                if drawing_open || group_open || tool_expected.is_some() {
                    return false;
                }
                drawing_open = true;
                pages += 1;
            }
            0x03e9 => {
                if !drawing_open || group_open || tool_expected.is_some() {
                    return false;
                }
                drawing_open = false;
            }
            0x03ea => {
                if drawing_open || group_open || tool_expected.is_some() || record.data.len() != 36
                {
                    return false;
                }
                tool_expected = Some(u32_at(bytes, record.data.start + 24) as usize);
                tool_objects = 0;
                tool_inputs = false;
            }
            0x03eb => {
                if group_open
                    || !tool_inputs
                    || record.data.len() != 0
                    || tool_expected != Some(tool_objects)
                {
                    return false;
                }
                tool_expected = None;
                tools += 1;
            }
            0x07d0 => {
                if group_open
                    || (!drawing_open && tool_expected.is_none())
                    || (tool_expected.is_some() && !matches!(record.data.len(), 16 | 28))
                {
                    return false;
                }
                group_open = true;
                if tool_expected.is_some() {
                    tool_objects += 1;
                }
            }
            0x07d7 => {
                if !group_open || !record.data.is_empty() {
                    return false;
                }
                group_open = false;
            }
            0x04b4 if tool_expected.is_some() => {
                let payload = &bytes[record.data.clone()];
                if group_open
                    || tool_inputs
                    || payload.len() < 4
                    || (payload.len() - 4) % 4 != 0
                    || u32_at(payload, 0) as usize != (payload.len() - 4) / 4
                {
                    return false;
                }
                let mut unique = BTreeSet::new();
                let limit = tool_expected.unwrap();
                for reference in payload[4..].chunks_exact(4) {
                    let id = u32::from_le_bytes(reference.try_into().unwrap()) as usize;
                    if id == 0 || id > limit || !unique.insert(id) {
                        return false;
                    }
                }
                tool_inputs = true;
            }
            0x07d2 if tool_expected.is_some() => {
                let payload = &bytes[record.data.clone()];
                if !group_open
                    || payload.len() < 4
                    || (payload.len() - 4) % 4 != 0
                    || u32_at(payload, 0) as usize != (payload.len() - 4) / 4
                {
                    return false;
                }
                let limit = tool_expected.unwrap();
                if payload[4..].chunks_exact(4).any(|reference| {
                    let id = u32::from_le_bytes(reference.try_into().unwrap()) as usize;
                    id == 0 || id > limit
                }) {
                    return false;
                }
            }
            _ => {}
        }
    }
    if drawing_open || group_open || tool_expected.is_some() {
        return false;
    }
    if tools == 0 {
        return true;
    }
    let mut headers = entries.iter().filter(|record| record.kind == 0x0384);
    let Some(header) = headers.next() else {
        return false;
    };
    header.data.len() == 28
        && headers.next().is_none()
        && u32_at(bytes, header.data.start + 4) as usize == tools
        && u32_at(bytes, header.data.start + 24) as usize == pages + tools + 1
}

impl Document {
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::from_bytes(
            &fs::read(path).map_err(|e| e.to_string())?,
            Some(path.to_path_buf()),
        )
    }
    pub fn from_bytes(b: &[u8], path: Option<PathBuf>) -> Result<Self, String> {
        Self::from_bytes_with_tool_inputs(b, path, &BTreeMap::new())
    }

    /// Decode a synthetic, isolated tool canvas with explicit local givens.
    /// Ordinary documents never infer this context from a code-zero header.
    pub(super) fn from_bytes_with_tool_inputs(
        b: &[u8],
        path: Option<PathBuf>,
        tool_inputs: &BTreeMap<usize, EmbeddedToolInput>,
    ) -> Result<Self, String> {
        let rs = records(b)?;
        if !tool_inputs.is_empty() {
            if rs
                .iter()
                .any(|record| matches!(record.kind, 0x03ea | 0x03eb))
                || rs.iter().filter(|record| record.kind == 0x03e8).count() != 1
            {
                return Err("Tool input context requires one isolated canvas".into());
            }
            let headers: Vec<_> = rs.iter().filter(|record| record.kind == 0x07d0).collect();
            for (&id, input) in tool_inputs {
                let header = id
                    .checked_sub(1)
                    .and_then(|index| headers.get(index))
                    .ok_or("Invalid contextual tool input ID")?;
                let payload = &b[header.data.clone()];
                if input.local_id != id
                    || payload.len() != 28
                    || payload[..2] != [0, 0]
                    || u16::from_le_bytes(payload[6..8].try_into().unwrap()) != input.native_class
                    || u16::from_le_bytes(payload[8..10].try_into().unwrap())
                        != input.native_subtype
                {
                    return Err("Tool input descriptor does not match its native header".into());
                }
            }
        }
        let document_indices: BTreeSet<_> = document_record_indices(&rs).into_iter().collect();
        let title_slots = page_title_slots(b, &rs)?;
        let title_offsets: BTreeSet<usize> = title_slots
            .iter()
            .filter_map(|(title, _, _)| title.map(|index| rs[index].start))
            .collect();
        let page_names: Vec<_> = title_slots.into_iter().map(|(_, _, name)| name).collect();
        let has_picture_resource = document_records(&rs).any(|record| record.kind == 0x0514);
        let image_file = has_picture_resource
            .then(|| ImageFile::parse(b))
            .transpose()
            .ok()
            .flatten();
        let image_read_only =
            has_picture_resource && !image_file.as_ref().is_some_and(ImageFile::can_edit);
        let baseline_records = records(include_bytes!("../../assets/point_template.gsp"))?;
        let angle_unit_radians = document_records(&rs)
            .find(|record| record.kind == 0x273a && record.data.len() == 2)
            .is_some_and(|record| b[record.data.clone()] == [0, 0]);
        let safe_outside: BTreeSet<u32> = baseline_records
            .iter()
            .map(|record| record.kind)
            .chain([0x2328, 0x232d])
            .collect();
        let text_style_count = document_records(&rs)
            .filter(|record| record.kind == 0x04b3)
            .count();
        let font_family_count = document_records(&rs)
            .filter(|record| record.kind == 0x04b1)
            .count();
        let mut objects: Vec<Object> = Vec::new();
        let total_pages = document_records(&rs)
            .filter(|record| record.kind == 0x3e8)
            .count();
        let mut i = 0;
        let mut safe = registered_tool_structure_safe(b, &rs);
        let mut snap_points = Vec::new();
        let mut grid_dotted = Vec::new();
        let mut grid_form = Vec::new();
        let mut frame_styles = BTreeMap::new();
        let (mut page, mut page_base, mut page_count) = (0, 0, 0);
        while i < rs.len() {
            if !document_indices.contains(&i) {
                i += 1;
                continue;
            }
            if rs[i].kind == 0x3e8 {
                if page_count > 0 {
                    page += 1;
                    page_base = objects.len();
                }
                page_count += 1;
                snap_points.push(graph::snap_points(&b[rs[i].data.clone()]).unwrap_or(false));
                grid_dotted.push(false);
                grid_form.push(GridForm::Square);
                i += 1;
                continue;
            }
            if rs[i].kind != 0x7d0 {
                if !safe_outside.contains(&rs[i].kind)
                    && !title_offsets.contains(&rs[i].start)
                    && !(rs[i].kind == 0x0514 && !image_read_only)
                {
                    safe = false;
                }
                i += 1;
                continue;
            }
            let h = &b[rs[i].data.clone()];
            if h.len() != 16 && h.len() != 28 {
                return Err(format!("Invalid object header at {:#x}", rs[i].start));
            }
            let header_offset = rs[i].data.start;
            let group_start = rs[i].start;
            let is_image = image_file
                .as_ref()
                .is_some_and(|picture| picture.picture_group_offset() == group_start);
            let raw = u32_at(h, 0);
            let code = raw as u16;
            let tool_input = tool_inputs.get(&(objects.len() + 1)).copied();
            let mut point = None;
            let mut point_offset = None;
            let mut constraint_payload: Option<(Vec<u8>, usize)> = None;
            let mut refs = Vec::new();
            let mut measurement_format = false;
            let mut measurement_precision = None;
            let mut measurement_anchor = None;
            let mut measurement_style = false;
            let mut graph_records: Vec<(u32, Vec<u8>)> = Vec::new();
            let mut function_expression: Option<Vec<u8>> = None;
            let mut function_range: Option<Vec<u8>> = None;
            let mut locus_range: Option<Vec<u8>> = None;
            let mut iteration_definition: Option<Vec<u8>> = None;
            let mut label: Option<LabelAux> = None;
            let labeled_shape = matches!(h[6..8], [1, 0] | [2, 0] | [3, 0] | [4, 0])
                && matches!(
                    code,
                    1 | 2
                        | 3
                        | 4
                        | 5
                        | 6
                        | 7
                        | 8
                        | 9
                        | 11
                        | 12
                        | 13
                        | 14
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
                        | 34
                        | 67
                        | 68
                        | 69
                        | 109
                        | 110
                        | 75
                        | 95
                        | 123
                );
            let numeric_label = matches!(
                code,
                36 | 37
                    | 38
                    | 39
                    | 40
                    | 41
                    | 42
                    | 43
                    | 44
                    | 46
                    | 47
                    | 48
                    | 49
                    | 50
                    | 51
                    | 65
                    | 66
                    | 86
                    | 87
                    | 94
            );
            let mut rich_text: Option<Vec<u8>> = None;
            let mut text_box: Option<Vec<u8>> = None;
            let mut text_style = false;
            i += 1;
            while i < rs.len() && rs[i].kind != 0x7d7 {
                let r = &rs[i];
                let d = &b[r.data.clone()];
                if matches!(code, 52 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 87) {
                    graph_records.push((r.kind, d.to_vec()));
                }
                if matches!(code, 0 | 71 | 78) && r.kind == 0x0907 {
                    function_expression = Some(d.to_vec());
                }
                if matches!(code, 72 | 126 | 127) && r.kind == 0x0902 {
                    function_range = Some(d.to_vec());
                }
                if matches!(code, 35 | 97) && r.kind == 0x0902 {
                    locus_range = Some(d.to_vec());
                }
                if matches!(code, 76 | 89) && r.kind == 0x090a {
                    iteration_definition = Some(d.to_vec());
                }
                match r.kind {
                    0x07d5
                        if matches!(code, 0 | 15)
                            || (code == 71 && h[6..10] == [11, 0, 57, 0])
                            || labeled_shape
                            || numeric_label =>
                    {
                        match LabelAux::parse(d) {
                            Ok(value)
                                if matches!(code, 0 | 15)
                                    || (code == 71 && h[6..10] == [11, 0, 57, 0])
                                    || numeric_label
                                    || h[6..8] == [1, 0] =>
                            {
                                label = Some(value)
                            }
                            Ok(_) => {}
                            Err(_) => safe = false,
                        }
                    }
                    0x08fc if code == 0 => rich_text = Some(d.to_vec()),
                    0x0898 if code == 0 => text_box = Some(d.to_vec()),
                    0x07d8
                        if matches!(code, 0 | 15)
                            || (code == 71 && h[6..10] == [11, 0, 57, 0])
                            || labeled_shape =>
                    {
                        text_style = d.len() == 2
                            && (u16::from_le_bytes([d[0], d[1]]) as usize) < text_style_count;
                        if (labeled_shape
                            || code == 15
                            || (code == 71 && h[6..10] == [11, 0, 57, 0]))
                            && !text_style
                        {
                            safe = false;
                        }
                    }
                    0x899 if d.len() == 16 => {
                        point = Some(Point {
                            x: f64_at(d, 0),
                            y: f64_at(d, 8),
                        });
                        point_offset = Some(r.data.start);
                    }
                    0x0903
                        if matches!(code, 36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47) =>
                    {
                        let sample: &[u8] = match code {
                            36 => {
                                &include_bytes!("../../assets/length_measurement_group.bin")[60..92]
                            }
                            37 => &include_bytes!("../../assets/distance_measurement_group.bin")
                                [64..96],
                            38 => &include_bytes!(
                                "../../assets/point_line_distance_measurement_group.bin"
                            )[64..96],
                            39 => &include_bytes!("../../assets/perimeter_measurement_group.bin")
                                [60..92],
                            40 => {
                                &include_bytes!("../../assets/circumference_measurement_group.bin")
                                    [60..92]
                            }
                            41 => {
                                &include_bytes!("../../assets/angle_measurement_group.bin")[68..100]
                            }
                            42 => {
                                &include_bytes!("../../assets/area_measurement_group.bin")[60..92]
                            }
                            43 => &include_bytes!("../../assets/arc_angle_measurement_group.bin")
                                [60..92],
                            44 => &include_bytes!("../../assets/arc_length_measurement_group.bin")
                                [60..92],
                            47 => {
                                &include_bytes!("../../assets/ratio_measurement_group.bin")[68..100]
                            }
                            _ => {
                                &include_bytes!("../../assets/radius_measurement_group.bin")[60..92]
                            }
                        };
                        measurement_format = d.len() == sample.len()
                            && u16::from_le_bytes(d[28..30].try_into().unwrap()) <= 5
                            && matches!(
                                u16::from_le_bytes(d[26..28].try_into().unwrap()),
                                0 | 1 | 3
                            )
                            && d.iter()
                                .zip(sample)
                                .enumerate()
                                .all(|(i, (actual, expected))| {
                                    // Label x/y and presentation bytes vary after edits.
                                    matches!(i, 12..=15 | 22 | 23 | 26..=31) || actual == expected
                                });
                        if measurement_format {
                            measurement_precision =
                                Some(u16::from_le_bytes(d[28..30].try_into().unwrap()));
                        }
                    }
                    0x0903 if matches!(code, 65 | 66 | 86 | 87 | 94) && d.len() == 32 => {
                        let precision = u16::from_le_bytes(d[28..30].try_into().unwrap());
                        if precision <= 5 {
                            measurement_precision = Some(precision);
                        }
                    }
                    0x07d8 if numeric_label => {
                        measurement_style = d.len() == 2
                            && (usize::from(u16::from_le_bytes([d[0], d[1]])) < text_style_count
                                || u16::from_le_bytes([d[0], d[1]])
                                    == if matches!(code, 49 | 50 | 51 | 65 | 66 | 86 | 87) {
                                        2
                                    } else {
                                        1
                                    });
                        if !measurement_style {
                            safe = false;
                        }
                    }
                    0x7d3
                        if matches!(
                            code,
                            15 | 17 | 18 | 19 | 21 | 24 | 27 | 30 | 52 | 54 | 55 | 56 | 57 | 75 | 109 | 110
                        ) =>
                    {
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d2 | 0x7d3 if d.len() >= 4 && (d.len() - 4) % 4 == 0 => {
                        let n = u32_at(d, 0) as usize;
                        if n == (d.len() - 4) / 4 {
                            refs = (0..n)
                                .map(|j| {
                                    let raw = u32_at(d, 4 + j * 4) as usize;
                                    if raw == 0 { 0 } else { page_base + raw }
                                })
                                .collect();
                        }
                    }
                    _ => {}
                }
                if numeric_label && r.kind == 0x0903 && d.len() == 32 {
                    measurement_anchor = Some((
                        i16::from_le_bytes(d[12..14].try_into().unwrap()),
                        i16::from_le_bytes(d[14..16].try_into().unwrap()),
                    ));
                }
                if !matches!(r.kind, 0x899 | 0x7d2 | 0x7d3)
                    && !(code == 0
                        && matches!(r.kind, 0x07d5 | 0x07d8 | 0x08fc | 0x0898 | 0x0906 | 0x08a2))
                    && !((code == 15 || (code == 71 && h[6..10] == [11, 0, 57, 0]))
                        && matches!(r.kind, 0x07d5 | 0x07d8))
                    && !((labeled_shape || numeric_label) && matches!(r.kind, 0x07d5 | 0x07d8))
                    && !(matches!(code, 36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47)
                        && matches!(r.kind, 0x0903 | 0x07d8))
                    && !(matches!(code, 52 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 87)
                        && matches!(r.kind, 0x0905 | 0x0904 | 0x0903 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 49 | 65 | 66 | 86)
                        && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(matches!(code, 50 | 51 | 94) && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(code == 62 && matches!(r.kind, 0x0906 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 0 | 71) && matches!(r.kind, 0x0907 | 0x08a3))
                    && !(matches!(code, 48 | 78) && matches!(r.kind, 0x0907 | 0x07d8))
                    && !(matches!(code, 48 | 67 | 68 | 73 | 75 | 90 | 94) && r.kind == 0x232e && d.is_empty())
                    && !(code == 0
                        && matches!(u16::from_le_bytes([h[8], h[9]]), 79..=81)
                        && r.kind == 0x232e
                        && d.is_empty())
                    && !(code == 91 && matches!(r.kind, 0x090d | 0x07d8))
                    && !(matches!(code, 73 | 90) && matches!(r.kind, 0x08fc | 0x07d8))
                    && !(code == 92 && matches!(r.kind, 0x090d | 0x07d8))
                    && !(is_image && matches!(r.kind, 0x090c | 0x08a8 | 0x1f44))
                    && !(matches!(code, 72 | 126 | 127) && r.kind == 0x0902)
                    && !(matches!(code, 35 | 97) && r.kind == 0x0902)
                    && !(matches!(code, 76 | 89) && r.kind == 0x090a)
                {
                    safe = false;
                }
                i += 1;
            }
            if i == rs.len() {
                return Err("Unterminated object group".into());
            }
            let action_bytes = (code == 62).then_some(&b[group_start..rs[i].data.end]);
            let action_group =
                action_bytes.and_then(|bytes| HideShowButtonGroup::parse(bytes).ok());
            let animation_group =
                action_bytes.and_then(|bytes| AnimationButtonGroup::parse(bytes).ok());
            let movement_group =
                action_bytes.and_then(|bytes| MovementButtonGroup::parse(bytes).ok());
            let scroll_group = action_bytes.and_then(|bytes| ScrollButtonGroup::parse(bytes).ok());
            let presentation_group =
                action_bytes.and_then(|bytes| PresentationButtonGroup::parse(bytes).ok());
            let link_url_group = (code == 0)
                .then_some(&b[group_start..rs[i].data.end])
                .and_then(|bytes| LinkUrlButtonGroup::parse(bytes).ok());
            let link_page_group = (code == 0)
                .then_some(&b[group_start..rs[i].data.end])
                .and_then(|bytes| LinkPageButtonGroup::parse(bytes).ok());
            if code == 0
                && rs.iter().any(|record| {
                    record.start >= group_start
                        && record.start < rs[i].start
                        && record.kind == 0x0906
                })
                && link_url_group.is_none()
                && link_page_group.is_none()
            {
                safe = false;
            }
            if code == 62
                && action_group.is_none()
                && animation_group.is_none()
                && movement_group.is_none()
                && scroll_group.is_none()
                && presentation_group.is_none()
            {
                safe = false;
            }
            let coordinate_group = matches!(code, 49 | 65 | 66 | 86)
                .then(|| coordinate_measure_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if coordinate_group.is_err() {
                safe = false;
            }
            let coordinate_group = coordinate_group.ok().flatten();
            let value_equation_group = matches!(code, 50 | 51 | 94)
                .then(|| value_equation_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if value_equation_group.is_err() {
                safe = false;
            }
            let value_equation_group = value_equation_group.ok().flatten();
            let plotted_point = matches!(code, 67 | 68)
                .then(|| plot_points_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if plotted_point.is_err() {
                safe = false;
            }
            let plotted_point = plotted_point.ok().flatten();
            let axis_value_refs = (code == 95)
                .then(|| plot_axis_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if axis_value_refs.is_err() {
                safe = false;
            }
            let axis_value_refs = axis_value_refs.ok().flatten();
            let literal_axis_value = (code == 123)
                .then(|| plot_axis_records::decode_literal_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if literal_axis_value.is_err() {
                safe = false;
            }
            let literal_axis_value = literal_axis_value.ok().flatten();
            let graph_borrowed = graph_records
                .iter()
                .map(|(kind, payload)| (*kind, payload.as_slice()))
                .collect::<Vec<_>>();
            let graph_kind = graph::decode_graph_object(code, &refs, &graph_borrowed);
            let mapped_iteration = (|| {
                if !matches!(code, 76 | 89) || h[6..10] != [14, 0, 64, 0] {
                    return None;
                }
                let data = iteration_definition.as_ref()?;
                if data.len() != 24
                    || data[..4] != [0; 4]
                    || !matches!(&data[4..6], [0, 0] | [1, 0])
                    || data[20..24] != [0; 4]
                {
                    return None;
                }
                let count = usize::from(u16::from_le_bytes(data[6..8].try_into().ok()?));
                let branches_count = u32_at(data, 8) as usize;
                if !(1..=16).contains(&count) || !(1..=8).contains(&branches_count) {
                    return None;
                }
                let start = usize::from(code == 89);
                if refs.len() <= start + count * (branches_count + 1) {
                    return None;
                }
                let seeds = refs[start..start + count].to_vec();
                let branch_end = start + count * (branches_count + 1);
                let branches = refs[start + count..branch_end]
                    .chunks_exact(count)
                    .map(<[usize]>::to_vec)
                    .collect::<Vec<_>>();
                let carried = refs[branch_end..].to_vec();
                if !supported_mapped_point_iteration(&objects, &seeds, &branches, &carried) {
                    return None;
                }
                let depth = if code == 89 {
                    let id = refs[0];
                    if !objects
                        .get(id.checked_sub(1)?)
                        .is_some_and(|o| o.kind.is_iteration_depth_value())
                    {
                        return None;
                    }
                    IterationDepth::Value(id)
                } else {
                    let value = u32_at(data, 16);
                    if !(1..=128).contains(&value) {
                        return None;
                    }
                    IterationDepth::Fixed(value)
                };
                Some(Kind::MappedPointIteration {
                    seeds,
                    branches,
                    carried,
                    depth,
                    final_only: data[4..6] == [1, 0],
                })
            })();
            let kind = if let Some(input) = tool_input {
                if !refs.is_empty()
                    || constraint_payload.is_some()
                    || rs.iter().any(|record| {
                        record.start > group_start
                            && record.start < rs[i].start
                            && record.kind == 0x07d2
                    })
                {
                    return Err("Tool input placeholder must not have dependencies".into());
                }
                if input.kind() == Some(EmbeddedToolInputKind::Numeric) {
                    super::tool_inputs::numeric_given_layout(&b[group_start..rs[i].data.end])?;
                } else if input.kind().is_some() {
                    let body: Vec<_> = rs
                        .iter()
                        .filter(|record| record.start > group_start && record.start < rs[i].start)
                        .collect();
                    if body
                        .iter()
                        .any(|record| !matches!(record.kind, 0x0899 | 0x07d5 | 0x07d8))
                        || body.iter().filter(|record| record.kind == 0x0899).count() != 1
                        || body
                            .iter()
                            .any(|record| record.kind == 0x0899 && record.data.len() != 16)
                        || body.iter().filter(|record| record.kind == 0x07d5).count() > 1
                        || body.iter().filter(|record| record.kind == 0x07d8).count() > 1
                    {
                        return Err("Unsupported tool input placeholder layout".into());
                    }
                }
                match input.kind() {
                    Some(EmbeddedToolInputKind::Point) => Kind::Free(Point { x: 0.0, y: 0.0 }),
                    Some(
                        category @ (EmbeddedToolInputKind::Path
                        | EmbeddedToolInputKind::Segment
                        | EmbeddedToolInputKind::Numeric),
                    ) => {
                        point_offset = None;
                        Kind::ToolInput(category)
                    }
                    None => Kind::Unsupported(0),
                }
            } else if refs
                .iter()
                .any(|&reference| reference == 0 || reference > objects.len())
            {
                Kind::Unsupported(code)
            } else if !tool_inputs.is_empty() && code == 48 && h[6..10] == [11, 0, 33, 0] {
                // An abstract tool calculation is typed only after binding. Its
                // native expression and references remain exact in the template.
                let mut normalized = b[group_start..rs[i].data.end].to_vec();
                normalized[16..18].copy_from_slice(&51_u16.to_le_bytes());
                match ParameterCalculationGroup::parse(&normalized) {
                    Ok(group)
                        if !refs.is_empty()
                            && group.sources()
                                == refs
                                    .iter()
                                    .map(|id| (*id - page_base) as u32)
                                    .collect::<Vec<_>>()
                            && refs.iter().all(|id| {
                                objects.get(id - 1).is_some_and(|object| {
                                    object.kind.numeric_unit().is_some()
                                        || matches!(
                                            object.kind,
                                            Kind::ToolInput(EmbeddedToolInputKind::Numeric)
                                        )
                                })
                            }) =>
                    {
                        Kind::ParameterCalculation(refs[0], group)
                    }
                    _ => Kind::Unsupported(code),
                }
            } else if let Some(graph_kind) = graph_kind {
                match graph_kind {
                    GraphObject::HorizontalUnit { origin, step }
                        if objects
                            .get(origin - 1)
                            .is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        Kind::GraphHorizontalUnit(origin, step)
                    }
                    GraphObject::VerticalUnit {
                        horizontal_unit,
                        step,
                    } if objects
                        .get(horizontal_unit - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::GraphHorizontalUnit(_, _))) =>
                    {
                        Kind::GraphVerticalUnit(horizontal_unit, step)
                    }
                    GraphObject::Axis { start, end, axis }
                        if objects
                            .get(start - 1)
                            .is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(end - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        Kind::GraphAxis(start, end, axis)
                    }
                    GraphObject::CircleAxis { circle, axis }
                        if objects.get(circle - 1).is_some_and(|o| o.kind.is_circle()) =>
                    {
                        Kind::CircleAxis(circle, axis)
                    }
                    GraphObject::DistanceAxis {
                        origin,
                        distance,
                        axis,
                    } if objects
                        .get(origin - 1)
                        .is_some_and(|o| is_point_kind(&o.kind))
                        && objects
                            .get(distance - 1)
                            .is_some_and(|o| is_coordinate_unit_kind(&o.kind)) =>
                    {
                        Kind::DistanceAxis(origin, distance, axis)
                    }
                    GraphObject::DistanceVerticalUnit {
                        x_axis,
                        step,
                        independent,
                    } if objects
                        .get(x_axis - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::DistanceAxis(_, _, 1))) =>
                    {
                        Kind::DistanceVerticalUnit(x_axis, step, independent)
                    }
                    GraphObject::CoordinateSystem {
                        x_axis,
                        y_axis,
                        form,
                        dotted,
                    } if (objects.get(x_axis - 1).is_some_and(|o| {
                        matches!(o.kind, Kind::GraphAxis(_, _, 1 | 0x1_0001))
                    }) && objects
                        .get(y_axis - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::GraphAxis(_, _, 0))))
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { kind: Kind::CircleAxis(a, 1), .. }),
                             Some(Object { kind: Kind::CircleAxis(b, 0), .. })) if a == b
                        )
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { kind: Kind::DistanceAxis(a, _, 1), .. }),
                             Some(Object { kind: Kind::DistanceAxis(b, _, 0), .. })) if a == b
                        )
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { kind: Kind::DistanceAxis(origin, _, 1), .. }),
                             Some(Object { kind: Kind::GraphAxis(y_origin, unit, 0), .. }))
                                if origin == y_origin && objects.get(unit - 1)
                                    .is_some_and(|o| matches!(o.kind, Kind::DistanceVerticalUnit(host, _, _) if host == x_axis))
                        ) =>
                    {
                        grid_dotted[page] = dotted;
                        grid_form[page] = form;
                        frame_styles.insert(objects.len() + 1, (form, dotted));
                        Kind::CoordinateSystem(x_axis, y_axis)
                    }
                    GraphObject::Slope { line, coordinates }
                        if objects
                            .get(line.saturating_sub(1))
                            .is_some_and(|o| is_straight_kind(&o.kind))
                            && objects.get(coordinates.saturating_sub(1)).is_some_and(|o| {
                                matches!(o.kind, Kind::CoordinateSystem(_, _))
                            }) =>
                    {
                        Kind::SlopeMeasurement(line, coordinates)
                    }
                    _ => Kind::Unsupported(code),
                }
            } else if let Some(caption) = super::referenced_captions::decode(
                &objects, page, objects.len() + 1, h, &refs,
                &b[group_start..rs[i].data.end], text_style_count,
            ) {
                caption
            } else if let Some(definition) = super::coordinate_points::decode(
                &objects, page, objects.len() + 1, h, &refs,
                constraint_payload.as_ref().map(|(payload, _)| payload.as_slice()),
            ) {
                Kind::CoordinatePoint(definition)
            } else if let Some(definition) = super::colorized::decode_colorized(
                &objects, page, objects.len() + 1, h, &refs,
                constraint_payload.as_ref().map(|(payload, _)| payload.as_slice()),
            ) {
                Kind::Colorized(definition)
            } else if let Some(definition) = super::mapped_element::decode_mapped_element(
                &objects, page, objects.len() + 1, h, &refs,
                constraint_payload.as_ref().map(|(payload, _)| payload.as_slice()),
            ) {
                Kind::MappedElement(definition)
            } else if let Some(definition) = super::transform_arc_decode::decode_arc_transform(
                &objects, page, objects.len() + 1, h, &refs,
                constraint_payload.as_ref().map(|(payload, _)| payload.as_slice()),
            ) {
                Kind::Transformed(definition)
            } else {
                match (code, point, refs.as_slice()) {
                    (89, None, [parameter, seed, first, second, carried_second, carried_first, polygon, segments @ ..])
                        if h[6..10] == [14,0,64,0] && second == carried_second && first == carried_first
                            && objects.get(parameter.saturating_sub(1)).is_some_and(|o| o.kind.is_iteration_depth_value())
                            && supported_branch_polygon_mapping(&objects,*seed,*first,*second,*polygon,segments)
                            && iteration_definition.as_ref().is_some_and(|d| d.len()==24 && d[..8]==[0,0,0,0,0,0,1,0] && u32_at(d,8)==2 && d[20..24]==[0;4]) => {
                            Kind::BranchingPolygonIteration { seed:*seed, first:*first, second:*second, polygon:*polygon, segments:segments.to_vec(), depth:IterationDepth::Value(*parameter) }
                        }
                    (76, None, [seed, first, second, carried_second, carried_first, polygon, segments @ ..])
                        if h[6..10] == [14,0,64,0] && second == carried_second && first == carried_first
                            && supported_branch_polygon_mapping(&objects,*seed,*first,*second,*polygon,segments)
                            && iteration_definition.as_ref().is_some_and(|d| d.len()==24 && d[..8]==[0,0,0,0,0,0,1,0] && u32_at(d,8)==2 && (1..=128).contains(&u32_at(d,16)) && d[20..24]==[0;4]) => {
                            Kind::BranchingPolygonIteration { seed:*seed, first:*first, second:*second, polygon:*polygon, segments:segments.to_vec(), depth:IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(),16)) }
                        }
                    (76,None,[seed,parameter,image,calculation,carried_value,carried_point,label])
                        if h[6..10] == [14,0,64,0] && calculation==carried_value && image==carried_point
                            && supported_point_parameter_iteration(&objects,*seed,*parameter,*image,*calculation,*label)
                            && iteration_definition.as_ref().is_some_and(|d|d.len()==24 && d[..8]==[0,0,0,0,0,0,2,0] && u32_at(d,8)==1 && (1..=128).contains(&u32_at(d,16)) && d[20..24]==[0;4]) => {
                            Kind::PointParameterIteration(*seed,*parameter,*image,*calculation,*label,IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(),16)))
                        }
                    (89,None,[depth_parameter,seed,parameter,image,calculation,carried_value,carried_point,label])
                        if h[6..10] == [14,0,64,0] && calculation==carried_value && image==carried_point
                            && *depth_parameter != *parameter
                            && objects.get(depth_parameter.saturating_sub(1)).is_some_and(|o| !o.deleted && o.kind.is_iteration_depth_value())
                            && supported_point_parameter_iteration(&objects,*seed,*parameter,*image,*calculation,*label)
                            && iteration_definition.as_ref().is_some_and(|d|d.len()==24 && d[..8]==[0,0,0,0,0,0,2,0] && u32_at(d,8)==1 && d[20..24]==[0;4]) => {
                            Kind::PointParameterIteration(*seed,*parameter,*image,*calculation,*label,IterationDepth::Value(*depth_parameter))
                        }
                    (76 | 89, None, _) if mapped_iteration.as_ref().is_some_and(|kind| matches!(kind, Kind::MappedPointIteration { seeds,.. } if seeds.len() > 1)) => mapped_iteration.unwrap(),
                    (92,None,[iteration,calculation]) if *iteration>0 && objects.get(iteration-1).is_some_and(|o|matches!(o.kind,Kind::PointParameterIteration(_,_,_,value,_,_) if value==*calculation))
                        && {
                            let group=&b[group_start..rs[i].data.end];
                            let template=include_bytes!("../../assets/point_parameter_iteration_values_group.bin");
                            group.len()==template.len() && group.iter().zip(template).enumerate().all(|(at,(a,b))|at==10 || (48..56).contains(&at) || (100..102).contains(&at) || a==b)
                        } => Kind::IterationValues(*iteration,*calculation),
                    (90,None,_) if super::point_numeric_labels::decode_numeric_label(
                        &objects,page,objects.len()+1,h,&refs,&b[group_start..rs[i].data.end],text_style_count).is_some() =>
                        super::point_numeric_labels::decode_numeric_label(
                            &objects,page,objects.len()+1,h,&refs,&b[group_start..rs[i].data.end],text_style_count).unwrap(),
                    (89, None, [parameter, seed, image, carried, ray])
                        if *parameter > 0 && *seed > 0 && *image > 0 && *ray > 0
                            && h[6..10] == [14, 0, 64, 0]
                            && image == carried
                            && iteration_definition.as_ref().is_some_and(|definition| {
                                definition.len() == 24
                                    && definition[..8] == [0, 0, 0, 0, 0, 0, 1, 0]
                                    && u32_at(definition, 8) == 1
                                    && definition[20..24] == [0; 4]
                            })
                            && objects.get(parameter - 1).is_some_and(|o| o.kind.is_iteration_depth_value())
                            && objects.get(seed - 1).is_some_and(|o| matches!(o.kind, Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(ray - 1).is_some_and(|o| matches!(o.kind, Kind::Ray(a, b) if a == *seed && b == *image)) => {
                        Kind::RayIteration(*seed, *image, *ray, *parameter)
                    }
                    (89,None,[depth,seed,image,carried,segment])
                        if h[6..10]==[14,0,64,0] && image==carried
                            && objects.get(depth.saturating_sub(1)).is_some_and(|o|o.kind.is_iteration_depth_value())
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(seed.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
                            && objects.get(segment.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Segment(a,b) if a==*seed && b==*image))
                            && iteration_definition.as_ref().is_some_and(|d|d.len()==24 && d[..8]==[0,0,0,0,0,0,1,0] && u32_at(d,8)==1 && d[20..24]==[0;4]) => Kind::SegmentIteration(*seed,*image,*segment,IterationDepth::Value(*depth)),
                    (89,None,[depth,seed,image,carried,circle])
                        if h[6..10]==[14,0,64,0] && image==carried
                            && objects.get(depth.saturating_sub(1)).is_some_and(|o|o.kind.is_iteration_depth_value())
                            && objects.get(seed.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(circle.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Circle(a,b) if a==*seed&&b==*image))
                            && iteration_definition.as_ref().is_some_and(|d|d.len()==24 && d[..8]==[0,0,0,0,0,0,1,0] && u32_at(d,8)==1 && d[20..24]==[0;4]) => Kind::CircleIteration(*seed,*image,*circle,IterationDepth::Value(*depth)),
                    (76,None,[seed,image,carried,circle])
                        if h[6..10]==[14,0,64,0] && image==carried
                            && objects.get(seed.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(circle.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Circle(a,b) if a==*seed&&b==*image))
                            && iteration_definition.as_ref().is_some_and(|d|d.len()==24 && d[..8]==[0,0,0,0,0,0,1,0] && u32_at(d,8)==1 && (1..=128).contains(&u32_at(d,16)) && d[20..24]==[0;4]) => Kind::CircleIteration(*seed,*image,*circle,IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(),16))),
                    (76, None, [seed, image, carried, segment])
                        if h[6..10] == [14, 0, 64, 0]
                            && image == carried
                            && iteration_definition.as_ref().is_some_and(|definition| {
                                definition.len() == 24
                                    && definition[..8] == [0, 0, 0, 0, 0, 0, 1, 0]
                                    && u32_at(definition, 8) == 1
                                    && (1..=128).contains(&u32_at(definition, 16))
                                    && definition[20..24] == [0; 4]
                            })
                            && objects.get(seed - 1).is_some_and(|o| matches!(o.kind, Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(segment - 1).is_some_and(|o| matches!(o.kind, Kind::Segment(a, b) if a == *seed && b == *image)) => {
                        Kind::SegmentIteration(*seed, *image, *segment, IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(), 16)))
                    }
                    (89,None,[depth,seed,image,carried])
                        if h[6..10]==[14,0,64,0] && image==carried
                            && objects.get(depth.saturating_sub(1)).is_some_and(|o|o.kind.is_iteration_depth_value())
                            && objects.get(seed.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && iteration_definition.as_ref().is_some_and(|d|d.len()==24 && d[..8]==[0,0,0,0,0,0,1,0] && u32_at(d,8)==1 && d[20..24]==[0;4]) => Kind::PointIteration(*seed,*image,IterationDepth::Value(*depth)),
                    (76, None, [seed, image, carried])
                        if h[6..10] == [14, 0, 64, 0]
                            && image == carried
                            && iteration_definition.as_ref().is_some_and(|definition| {
                                definition.len() == 24
                                    && definition[..8] == [0, 0, 0, 0, 0, 0, 1, 0]
                                    && u32_at(definition, 8) == 1
                                    && (1..=128).contains(&u32_at(definition, 16))
                                    && definition[20..24] == [0; 4]
                            })
                            && objects.get(seed - 1).is_some_and(|o| matches!(o.kind, Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image) => {
                        Kind::PointIteration(*seed, *image, IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(), 16)))
                    }
                    (76 | 89, None, _) if mapped_iteration.is_some() => mapped_iteration.unwrap(),
                    (77, None, [image, iteration])
                        if h[6..10] == [15, 0, 65, 0]
                            && objects.get(iteration - 1).is_some_and(|o| match o.kind {
                                Kind::BranchingPolygonIteration { ref segments, polygon, first, second, .. } => polygon == *image || first == *image || second == *image || segments.contains(image),
                                Kind::PointIteration(_, mapped, _) => mapped == *image,
                                Kind::MappedPointIteration { ref carried, .. } => carried.contains(image) && objects.get(image.saturating_sub(1)).is_some_and(|source| mapped_binding_kind(&source.kind)),
                                Kind::SegmentIteration(_, mapped, segment, _) | Kind::CircleIteration(_,mapped,segment,_) => mapped == *image || segment == *image,
                                Kind::RayIteration(_, mapped, ray, _) => mapped == *image || ray == *image,
                                Kind::PointParameterIteration(_,_,mapped,_,label,_) => mapped==*image || label==*image,
                                _ => false,
                            }) => {
                        Kind::IterationBinding(*image, *iteration)
                    }
                    (97, None, _)
                        if h[6..10] == [6, 0, 42, 0]
                            && refs.first().zip(refs.get(1)).and_then(|(&target, &driver)| expected_parameter_locus_refs(&objects, driver, target)).as_ref() == Some(&refs)
                            && locus_range.as_ref().is_some_and(|data| data.len() == 40
                                && f64_at(data, 0).is_finite() && f64_at(data, 8).is_finite()
                                && f64_at(data, 0) < f64_at(data, 8)
                                && (2..=4096).contains(&u32_at(data, 16))
                                && u32_at(data, 20) == 0x00030004
                                && f64_at(data, 24).is_finite()
                                && refs.get(1).and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|driver| match &driver.kind {
                                    Kind::Parameter(group) => group.locus_unit_tag() == Some(u32_at(data, 32)),
                                    _ => false,
                                })
                                && u32_at(data, 36) == 0) => {
                        let data = locus_range.as_ref().unwrap();
                        Kind::ParameterLocus(refs.clone(), f64_at(data, 0), f64_at(data, 8), u32_at(data, 16))
                    }
                    (35, None, _)
                        if refs.first().and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| {
                            if is_locus_straight_kind(&o.kind) || o.kind.is_circle() || is_arc_kind(&o.kind) || is_locus_polygon_kind(&o.kind) {
                                h[6..10] == [6, 0, 43, 0]
                            } else {
                                h[6..10] == [6, 0, 42, 0]
                            }
                        })
                            && refs.first().zip(refs.get(2)).and_then(|(&target, &driver)| {
                                expected_locus_refs(&objects, driver, target)
                            }).as_ref() == Some(&refs)
                            && locus_range.as_ref().is_some_and(|data| {
                                data.len() == 40
                                    && f64_at(data, 0) == 0.0
                                    && (2..=10_000).contains(&u32_at(data, 16))
                                    && refs.get(2).and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| match o.kind {
                                        Kind::PointOnSegment(_, _) | Kind::PointOnArc(_, _) => f64_at(data, 8) == 1.0 && data[20..24] == [4, 0, 1, 0],
                                        Kind::PointOnCircle(_, _, _) => (f64_at(data, 8) - std::f64::consts::TAU).abs() < 1e-12 && data[20..24] == [4, 0, 0, 0],
                                        _ => false,
                                    })
                                    && f64_at(data, 24).is_finite()
                                    && refs.first().and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| {
                                        data[32..] == if is_locus_straight_kind(&o.kind) || o.kind.is_circle() || is_arc_kind(&o.kind) || is_locus_polygon_kind(&o.kind) {
                                            [0, 0, 0, 0, 0, 0, 0, 0]
                                        } else {
                                            [1, 0, 0, 0, 0, 0, 0, 0]
                                        }
                                    })
                            }) => Kind::Locus(refs.clone(), u32_at(locus_range.as_ref().unwrap(), 16)),
                    (123, _, [path])
                        if literal_axis_value
                            .is_some_and(|(local_path, _)| local_path as usize + page_base == *path)
                            && objects.get(path - 1).is_some_and(|o| o.page == page && is_literal_plot_path_kind(&o.kind)) => {
                        Kind::PathLiteralPoint(*path, literal_axis_value.unwrap().1)
                    }
                    (95, _, [source, path])
                        if axis_value_refs == Some([(*source - page_base) as u32, (*path - page_base) as u32])
                            && objects.get(source - 1).is_some_and(|o| o.page == page && o.kind.numeric_unit() == Some(NumericUnit::Scalar))
                            && objects.get(path - 1).is_some_and(|o| o.page == page && is_value_plot_path_kind(&o.kind)) => {
                        Kind::PathValuePoint(*source, *path)
                    }
                    (69, None, [x, y, frame])
                        if h[6..10] == [1, 0, 56, 0]
                            && [x, y].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| o.page == page && o.kind.numeric_unit().is_some()))
                            && objects.get(frame.saturating_sub(1)).is_some_and(|o| o.page == page && matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::NumericCoordinatePoint(*x, *y, *frame)
                    }
                    (67 | 68, _, [frame])
                        if plotted_point.as_ref().is_some_and(|plot| {
                            plot.coordinate_system == (*frame - page_base) as u32
                        })
                        && objects.get(frame.saturating_sub(1)).is_some_and(|o| o.page == page && matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        let plot = plotted_point.unwrap();
                        Kind::PlottedPoint(*frame, plot.mode, plot.stored)
                    }
                    (94, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::PointValue && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first.saturating_sub(1)).is_some_and(|o| o.page == page && is_point_kind(&o.kind))
                        && objects.get(second.saturating_sub(1)).is_some_and(|o| o.page == page && (is_straight_kind(&o.kind) || o.kind.is_circle() || is_arc_kind(&o.kind) || o.kind.is_polygon())) => {
                        Kind::ValueEquation { kind: ValueEquationKind::PointValue, first: *first, second: *second }
                    }
                    (50, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::LineEquation && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first.saturating_sub(1)).is_some_and(|o| o.kind.is_line())
                        && objects.get(second - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::ValueEquation { kind: ValueEquationKind::LineEquation, first: *first, second: *second }
                    }
                    (51, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::CircleEquation && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first.saturating_sub(1)).is_some_and(|o| o.kind.is_circle())
                        && objects.get(second - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::ValueEquation { kind: ValueEquationKind::CircleEquation, first: *first, second: *second }
                    }
                    (49 | 65 | 66, _, [first, frame]) if coordinate_group.as_ref().is_some_and(|g| {
                        g.references == vec![(*first - page_base) as u32, (*frame - page_base) as u32]
                    }) && objects.get(first - 1).is_some_and(|o| is_point_kind(&o.kind))
                        && objects.get(frame - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::CoordinateMeasurement { kind: coordinate_group.as_ref().unwrap().kind, first: *first, second: None, frame: *frame }
                    }
                    (86, _, [first, second, frame]) if coordinate_group.as_ref().is_some_and(|g| {
                        g.references == vec![(*first - page_base) as u32, (*second - page_base) as u32, (*frame - page_base) as u32]
                    }) && [first, second].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind)))
                        && objects.get(frame - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::CoordinateMeasurement { kind: CoordinateMeasureKind::CoordinateDistance, first: *first, second: Some(*second), frame: *frame }
                    }
                    (0, None, []) if is_image => {
                        let picture = image_file.as_ref().unwrap();
                        Kind::Image { png: picture.png().to_vec(), transform: picture.transform() }
                    }
                    (0, None, []) if function_expression.is_some() && rich_text.is_none() && text_box.is_none() => {
                        let group = &b[group_start..rs[i].data.end];
                        if let Ok(parameter) = ParameterGroup::parse(group) {
                            let generated_name_valid = !parameter.name().is_empty()
                                || parameter.display_mode() == 0
                                || u32::try_from(objects.iter().filter(|object| object.page == page
                                    && matches!(&object.kind, Kind::Parameter(group) if group.name().is_empty())).count())
                                    .ok().and_then(|ordinal| super::parameter_names::parameter_name_counter(b, page)?.checked_add(ordinal)).is_some();
                            if generated_name_valid { Kind::Parameter(parameter) } else { Kind::Unsupported(code) }
                        } else if let Ok(calculation) = CalculationGroup::parse(group) {
                            Kind::Calculation(calculation)
                        } else {
                            match function_records::decode_simple_expression(function_expression.as_ref().unwrap()) {
                                Ok(expr) => Kind::FunctionDefinition(expr),
                                Err(_) => Kind::Unsupported(code),
                            }
                        }
                    }
                    (71, None, sources) if !sources.is_empty() && h[6..10] == [11, 0, 57, 0] && function_expression.is_some() => {
                        match function_records::decode_expression_with_sources(function_expression.as_ref().unwrap(), sources.len()) {
                            Ok(expression) if super::functions::function_sources_valid(expression, sources, &objects, page)
                                && sources.iter().all(|&id| id > 0 && id <= objects.len()) => Kind::ReferencedFunction(sources.to_vec(), expression),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (48, None, [first, second]) if *first > 0 && *second > 0
                        && objects.get(first - 1).is_some_and(|o| o.kind.is_distance_value())
                        && objects.get(second - 1).is_some_and(|o| o.kind.is_distance_value()) => {
                        match LengthPairCalculationGroup::parse(&b[group_start..rs[i].data.end]) {
                            Ok(group) if group.sources() == [(*first-page_base) as u32, (*second-page_base) as u32] => Kind::LengthPairCalculation(*first, *second, group),
                            _ => parse_numeric_calculation(&b[group_start..rs[i].data.end], &[*first,*second], &objects, page_base).unwrap_or(Kind::Unsupported(code)),
                        }
                    }
                    (48, None, sources) if sources.len() >= 2 => parse_numeric_calculation(&b[group_start..rs[i].data.end], sources, &objects, page_base).unwrap_or(Kind::Unsupported(code)),
                    (48, None, [source]) if *source > 0
                        && objects.get(source - 1).is_some_and(|object| object.kind.is_distance_value()) => {
                        match MeasuredCalculationGroup::parse(&b[group_start..rs[i].data.end]) {
                            Ok(group) if group.source_id() as usize == source - page_base => Kind::MeasuredCalculation(*source, group),
                            _ => parse_numeric_calculation(&b[group_start..rs[i].data.end], &[*source], &objects, page_base).unwrap_or(Kind::Unsupported(code)),
                        }
                    }
                    (48, None, [source]) if *source > 0
                        && objects.get(source - 1).is_some_and(|object| object.kind.numeric_unit().is_some()
                            || matches!(object.kind, Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _))) => {
                        parse_numeric_calculation(&b[group_start..rs[i].data.end], &[*source], &objects, page_base).unwrap_or(Kind::Unsupported(code))
                    }
                    (91, None, [source]) if source.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o| matches!(o.kind, Kind::Parameter(_))) => {
                        match TableGroup::parse(&b[group_start..rs[i].data.end]) {
                            Ok(table) if source.checked_sub(page_base) == Some(table.source_id() as usize)
                                && source.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o| match &o.kind {
                                    Kind::Parameter(parameter) if parameter.unit() == NumericUnit::Scalar => match table.named_label_value() {
                                        Some((name, _)) => name == parameter.name(),
                                        None => parameter.name() == "t[1]",
                                    },
                                    _ => false,
                                }) => Kind::ValueTable(*source, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (91, None, [first, second]) if objects.get(first - 1).is_some_and(|o| {
                        o.page == page && matches!(&o.kind, Kind::Parameter(group) if group.name() == "t[2]" && group.unit() == NumericUnit::Scalar)
                    }) && objects.get(second - 1).is_some_and(|o| {
                        o.page == page && matches!(&o.kind, Kind::Parameter(group) if group.name() == "t[1]" && group.unit() == NumericUnit::Scalar)
                    }) => {
                        match TableGroup::parse_pair(&b[group_start..rs[i].data.end]) {
                            Ok(table) if table.pair_source_ids() == Some([
                                (*first - page_base) as u32,
                                (*second - page_base) as u32,
                            ]) => Kind::ValueTablePair(*first, *second, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (72, None, [definition, frame]) if function_range.is_some()
                        && matches!(objects.get(definition - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(frame - 1).map(|o| &o.kind), Some(Kind::CoordinateSystem(_, _))) => {
                        match function_records::decode_plot_range(function_range.as_ref().unwrap()) {
                            Ok((min,max,samples)) => Kind::FunctionPlot(*definition,*frame,min,max,samples),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (78, None, [source]) if *source > 0 && function_expression.is_some()
                        && matches!(objects.get(source - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _))) => {
                        match function_records::decode_simple_expression(function_expression.as_ref().unwrap()) {
                            Ok(expression) => Kind::DerivativeFunction(*source, expression),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (126 | 127, None, [x, y, frame]) if function_range.is_some()
                        && *x > 0 && *y > 0 && *frame > 0 && x != y
                        && matches!(objects.get(x - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(y - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(frame - 1).map(|o| &o.kind), Some(Kind::CoordinateSystem(_, _))) => {
                        match function_records::decode_parametric_range(function_range.as_ref().unwrap(), code == 127) {
                            Ok((min,max,samples)) => Kind::ParametricPlot(*x,*y,*frame,min,max,samples,if code == 127 { PlotPointsMode::Polar } else { PlotPointsMode::Cartesian }),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if !targets.is_empty() && action_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = action_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::HideShowButton {
                                targets: targets.to_vec(),
                                action: group.action(),
                                options: group.options(),
                                caption_flags: group.caption_flags(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if !targets.is_empty() && animation_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = animation_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::AnimationButton {
                                targets: targets.to_vec(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                                settings: group.settings_list().unwrap(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if movement_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target-page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = movement_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::MovementButton {
                                pairs: targets.chunks_exact(2).map(|pair| (pair[0],pair[1])).collect(),
                                speed: group.speed(),
                                follow: group.follows_destination(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, [target]) if scroll_group.as_ref().is_some_and(|group| {
                        group.target_id() == (*target - page_base) as u32
                    }) => {
                        let group = scroll_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::ScrollButton {
                                target: *target,
                                placement: group.placement(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if !targets.is_empty() && presentation_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = presentation_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::PresentationButton {
                                targets: targets.to_vec(),
                                mode: group.mode(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (0, None, []) if link_url_group.is_some() => {
                        let group = link_url_group.as_ref().unwrap();
                        match (group.url_utf8(), group.label_utf8()) {
                            (Ok(url), Ok(label)) => Kind::LinkUrlButton {
                                url: url.to_owned(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (0, None, []) if link_page_group.is_some() => {
                        let group = link_page_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => {
                                Kind::LinkPageButton {
                                    target_page: group.target_page(),
                                    target_button_ordinal: group.target_button_ordinal(),
                                    label: label.to_owned(),
                                    anchor: group.screen_anchor(),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (0, Some(p), _) if rich_text.is_none() && text_box.is_none() => Kind::Free(p),
                    (0, None, []) if rich_text.is_some() && text_box.is_some()
                        && text_style && label.is_none() => {
                        match text_object::decode_rich_text(rich_text.as_ref().unwrap(), text_box.as_ref().unwrap()) {
                            Ok((text, rect, runs)) if runs.iter().flatten().all(|run| run.style.family_index.is_none_or(|index| usize::from(index) < font_family_count)) => Kind::PlainText { text, rect },
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (36, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| is_segment_kind(&o.kind)) =>
                    {
                        Kind::SegmentLength(*host)
                    }
                    (37, _, [a, b])
                        if measurement_format && measurement_style
                            && objects.get(a - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(b - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        Kind::PointDistance(*a, *b)
                    }
                    (38, _, [point, host])
                        if measurement_format && measurement_style
                            && objects.get(point.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| is_straight_kind(&o.kind)) =>
                    {
                        Kind::PointLineDistance(*point, *host)
                    }
                    (47, _, [a, b, c])
                        if measurement_format && measurement_style
                            && [a, b, c].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::PointRatio(*a, *b, *c)
                    }
                    (39, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon()) =>
                    {
                        Kind::PolygonPerimeter(*host)
                    }
                    (40, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| o.kind.is_circle()) =>
                    {
                        Kind::CircleCircumference(*host)
                    }
                    (41, _, [a, b, c])
                        if measurement_format && measurement_style
                            && [a, b, c].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::AngleMeasurement(*a, *b, *c)
                    }
                    (42, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| o.kind.is_circle()) =>
                    {
                        Kind::CircleArea(*host)
                    }
                    (42, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon()) =>
                    {
                        Kind::PolygonArea(*host)
                    }
                    (43, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| is_arc_kind(&o.kind)) =>
                    {
                        Kind::ArcAngle(*host)
                    }
                    (44, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| is_arc_kind(&o.kind)) =>
                    {
                        Kind::ArcLength(*host)
                    }
                    (46, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| o.kind.is_circle()) =>
                    {
                        Kind::CircleRadius(*host)
                    }
                    (15, None, [host]) if !tool_inputs.is_empty()
                        && h[6..10] == [1, 0, 10, 0]
                        && constraint_payload.is_none()
                        && objects.get(host.saturating_sub(1))
                            .is_some_and(|object| matches!(object.kind, Kind::ToolInput(EmbeddedToolInputKind::Path))
                                || straight_kind_mode(&object.kind).is_some()
                                || object.kind.is_circle() || is_arc_kind(&object.kind)
                                || object.kind.is_polygon()) =>
                    {
                        Kind::ToolPointOnPath(*host)
                    }
                    (15, _, [host]) => {
                        match (objects.get(host - 1).map(|o| &o.kind), &constraint_payload) {
                            (Some(kind), Some((payload, _)))
                                if straight_kind_mode(kind) == Some(LineMode::Segment)
                                    && payload.len() == 12
                                    && payload[..4] == [15, 0, 14, 0]
                                    && f64_at(payload, 4).is_finite() =>
                            {
                                Kind::PointOnSegment(*host, f64_at(payload, 4))
                            }
                            (Some(kind), Some((payload, _)))
                                if straight_kind_mode(kind) == Some(LineMode::Ray)
                                    && payload.len() == 12
                                    && payload[..4] == [15, 0, 13, 0]
                                    && f64_at(payload, 4).is_finite() =>
                            {
                                Kind::PointOnRay(*host, f64_at(payload, 4))
                            }
                            (Some(kind), Some((payload, _)))
                                if straight_kind_mode(kind) == Some(LineMode::Line)
                                    && payload.len() == 12
                                    && (payload[..4] == [15, 0, 12, 0]
                                        || super::kind::straight_point_constraint_matches(kind, payload))
                                    && f64_at(payload, 4).is_finite() =>
                            {
                                Kind::PointOnLine(*host, f64_at(payload, 4))
                            }
                            (
                                Some(kind),
                                Some((payload, _)),
                            ) if kind.is_circle() && payload.len() == 20
                                && payload[..4] == [15, 0, 28, 0]
                                && f64_at(payload, 4).is_finite()
                                && f64_at(payload, 12).is_finite() =>
                            {
                                Kind::PointOnCircle(*host, f64_at(payload, 4), f64_at(payload, 12))
                            }
                            (
                                Some(kind),
                                Some((payload, _)),
                            ) if is_arc_kind(kind) && payload.len() == 12
                                && payload[..4] == [15, 0, 67, 0]
                                && f64_at(payload, 4).is_finite() =>
                            {
                                Kind::PointOnArc(*host, f64_at(payload, 4))
                            }
                            (Some(kind), Some((payload, _)))
                                if kind.is_polygon() && objects.get(host - 1).is_some_and(|object| !object.deleted && object.page == page)
                                    && payload.len() == 20
                                    && polygon_vertex_count(&objects, *host).and_then(polygon_subtype).is_some_and(|subtype| payload[..4] == [15, 0, subtype as u8, (subtype >> 8) as u8])
                                    && f64_at(payload, 4).is_finite()
                                    && (0.0..=1.0).contains(&f64_at(payload, 4))
                                    && polygon_vertex_count(&objects, *host).is_some_and(|count| (u16::from_le_bytes([payload[12], payload[13]]) as usize) < count) => {
                                Kind::PointOnPolygon(*host, u16::from_le_bytes([payload[12], payload[13]]) as usize, f64_at(payload, 4))
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (21, _, [origin]) => match &constraint_payload {
                        Some((payload, _))
                            if payload.len() == 60
                                && payload[..4] == [21, 0, 0, 0]
                                && f64_at(payload, 20).is_finite()
                                && f64_at(payload, 32).is_finite()
                                && f64_at(payload, 40).is_finite()
                                && polar_distance_scale(payload).is_some()
                                && transform_angle_degrees(payload).is_some()
                                && *origin > 0
                                && objects.get(origin - 1).is_some_and(|o| is_point_kind(&o.kind) || is_segment_kind(&o.kind) || o.kind.is_circle() || o.kind.is_polygon()) =>
                        {
                            if h[8..10] == [14, 0] && is_segment_kind(&objects[origin - 1].kind) {
                                Kind::PolarTranslatedSegment(*origin, transform_angle_degrees(payload).unwrap(), polar_distance_scale(payload).unwrap(), f64_at(payload, 40))
                            } else if h[8..10] == [28, 0] && objects[origin - 1].kind.is_circle() {
                                Kind::PolarTranslatedCircle(*origin, transform_angle_degrees(payload).unwrap(), polar_distance_scale(payload).unwrap(), f64_at(payload, 40))
                            } else if h[8..10] == [4, 0] && is_point_kind(&objects[origin - 1].kind) {
                                Kind::PolarTranslatedPoint(*origin, transform_angle_degrees(payload).unwrap(), polar_distance_scale(payload).unwrap(), f64_at(payload, 40))
                            } else if objects[origin - 1].kind.is_polygon()
                                && polygon_vertex_count(&objects, *origin).and_then(polygon_subtype) == Some(u16::from_le_bytes([h[8], h[9]]))
                                && h.len() == 28 && h[6..8] == [4, 0] {
                                let distance = f64_at(payload, 40);
                                let units = polar_distance_scale(payload).unwrap();
                                Kind::PolarTranslatedPolygon(*origin, transform_angle_degrees(payload).unwrap(), units, distance)
                            } else {
                                Kind::Unsupported(code)
                            }
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (17, _, [source]) => match &constraint_payload {
                        Some((payload, _))
                            if payload.len() == 52
                                && payload[..4] == [17, 0, 0, 0]
                                && u32_at(payload, 20) == 1
                                && u32_at(payload, 40) == 1
                                && [4, 12, 24, 32].iter().all(|&offset| f64_at(payload, offset).is_finite())
                                && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5
                                && (f64_at(payload, 24) - f64_at(payload, 32) * 37.79527559055118).abs() < 1e-5
                                && *source > 0
                                && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind) || matches!(o.kind, Kind::Segment(_, _) | Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::Polygon(_))) =>
                        {
                            if h[8..10] == [14, 0] && matches!(objects[source - 1].kind, Kind::Segment(_, _)) {
                                Kind::FixedRectangularTranslatedSegment(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else if h[8..10] == [28, 0] && matches!(objects[source - 1].kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) {
                                Kind::FixedRectangularTranslatedCircle(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else if h[8..10] == [4, 0] && is_point_kind(&objects[source - 1].kind) {
                                Kind::FixedRectangularTranslatedPoint(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else if matches!(&objects[source - 1].kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]])))
                                && h.len() == 28 && h[6..8] == [4, 0] {
                                Kind::FixedRectangularTranslatedPolygon(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else {
                                Kind::Unsupported(code)
                            }
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (18 | 19, _, [source, distance])
                        if matches!((objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]),
                            (Some(Kind::Segment(_, _)), [14, 0]) | (Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)), [28, 0]))
                            && objects.get(distance.saturating_sub(1)).is_some_and(|o| o.kind.is_marked_distance_value()) => match &constraint_payload {
                        Some((payload, _)) if payload.len() == 28
                            && payload[..4] == [code as u8, 0, 0, 0]
                            && u32_at(payload, 20) == 1
                            && payload[24..28] == [0, 0, 0, 0]
                            && f64_at(payload, 4).is_finite()
                            && f64_at(payload, 12).is_finite()
                            && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5 => {
                                if h[8..10] == [14, 0] {
                                    Kind::MixedRectangularTranslatedSegment(*source, *distance, f64_at(payload, 12), code == 18)
                                } else {
                                    Kind::MixedRectangularTranslatedCircle(*source, *distance, f64_at(payload, 12), code == 18)
                                }
                            },
                        _ => Kind::Unsupported(code),
                    },
                    (18 | 19, _, [source, distance])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind)) => match &constraint_payload {
                        Some((payload, _))
                            if payload.len() == 28
                                && payload[..4] == [code as u8, 0, 0, 0]
                                && u32_at(payload, 20) == 1
                                && payload[24..28] == [0, 0, 0, 0]
                                && f64_at(payload, 4).is_finite()
                                && f64_at(payload, 12).is_finite()
                                && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5
                                && *source > 0 && *distance > 0
                                && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                                && objects.get(distance - 1).is_some_and(|o| o.kind.is_marked_distance_value()) =>
                        {
                            Kind::MixedRectangularTranslatedPoint(*source, *distance, f64_at(payload, 12), code == 18)
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (18 | 19, _, [source, distance])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| matches!(&o.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && objects.get(distance.saturating_sub(1)).is_some_and(|o| o.kind.is_marked_distance_value())
                            && h.len() == 28 && h[6..8] == [4, 0] => match &constraint_payload {
                        Some((payload, _)) if payload.len() == 28
                            && payload[..4] == [code as u8, 0, 0, 0]
                            && u32_at(payload, 20) == 1
                            && payload[24..28] == [0, 0, 0, 0]
                            && f64_at(payload, 4).is_finite()
                            && f64_at(payload, 12).is_finite()
                            && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5 =>
                            Kind::MixedRectangularTranslatedPolygon(*source, *distance, f64_at(payload, 12), code == 18),
                        _ => Kind::Unsupported(code),
                    },
                    (24, _, [source, distance])
                        if *source > 0 && *distance > 0
                            && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind) || is_segment_kind(&o.kind) || o.kind.is_circle() || o.kind.is_polygon())
                            && objects.get(distance - 1).is_some_and(|o| o.kind.is_marked_distance_value()) =>
                    {
                        match &constraint_payload {
                            Some((payload, _))
                                if payload.len() == 36
                                    && payload[..4] == [24, 0, 0, 0]
                                    && transform_angle_degrees(payload).is_some()
                                    && f64_at(payload, 20).is_finite()
                                    && f64_at(payload, 4).is_finite()
                                    && f64_at(payload, 12).is_finite() =>
                            {
                                match (&objects[source - 1].kind, &h[8..10]) {
                                    (kind, [4, 0]) if is_point_kind(kind) => Kind::MarkedDistanceTranslatedPoint(*source, *distance, transform_angle_degrees(payload).unwrap()),
                                    (kind, [14, 0]) if is_segment_kind(kind) => Kind::MarkedDistanceTranslatedSegment(*source, *distance, transform_angle_degrees(payload).unwrap()),
                                    (kind, [28, 0]) if kind.is_circle() => Kind::MarkedDistanceTranslatedCircle(*source, *distance, transform_angle_degrees(payload).unwrap()),
                                    (kind, _) if kind.is_polygon() && polygon_vertex_count(&objects, *source).and_then(polygon_subtype) == Some(u16::from_le_bytes([h[8], h[9]]))
                                        && h.len() == 28 && h[6..8] == [4, 0] => Kind::MarkedDistanceTranslatedPolygon(*source, *distance, transform_angle_degrees(payload).unwrap()),
                                    _ => Kind::Unsupported(code),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (20, None, [source, horizontal, vertical])
                        if matches!((objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]),
                            (Some(Kind::Segment(_, _)), [14, 0]) | (Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)), [28, 0]))
                            && [horizontal, vertical].iter().all(|&&id| id > 0 && objects.get(id - 1).is_some_and(|o| o.kind.is_marked_distance_value())) => {
                            if h[8..10] == [14, 0] {
                                Kind::RectangularMarkedTranslatedSegment(*source, *horizontal, *vertical)
                            } else {
                                Kind::RectangularMarkedTranslatedCircle(*source, *horizontal, *vertical)
                            }
                        },
                    (20, None, [source, horizontal, vertical])
                        if *source > 0 && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && [horizontal, vertical].iter().all(|&&id| {
                                id > 0 && objects.get(id - 1).is_some_and(|o| o.kind.is_marked_distance_value())
                            }) =>
                    {
                        Kind::RectangularMarkedTranslatedPoint(*source, *horizontal, *vertical)
                    }
                    (20, None, [source, horizontal, vertical])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| matches!(&o.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && [horizontal, vertical].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| o.kind.is_marked_distance_value()))
                            && h.len() == 28 && h[6..8] == [4, 0] => {
                        Kind::RectangularMarkedTranslatedPolygon(*source, *horizontal, *vertical)
                    }
                    (16, _, [source, start, end])
                        if [source, start, end].iter().all(|&&id| id > 0 && objects.get(id - 1).is_some())
                            && [start, end].iter().all(|&&id| is_point_kind(&objects[id - 1].kind)) => {
                        match (&objects[source - 1].kind, &h[8..10]) {
                            (kind, [4, 0]) if is_point_kind(kind) => Kind::VectorTranslatedPoint(*source, *start, *end),
                            (kind, [14, 0]) if is_segment_kind(kind) => Kind::VectorTranslatedSegment(*source, *start, *end),
                            (kind, [28, 0]) if kind.is_circle() => Kind::VectorTranslatedCircle(*source, *start, *end),
                            (kind, _) if kind.is_polygon() && h[6..8] == [4, 0]
                                && polygon_vertex_count(&objects, *source).and_then(polygon_subtype) == Some(u16::from_le_bytes([h[8], h[9]])) => Kind::VectorTranslatedPolygon(*source, *start, *end),
                            _ => Kind::Unsupported(code),
                        }
                    },
                    (29, None, [source, center, angle])
                        if objects.get(center.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(angle.saturating_sub(1)).is_some_and(|o| o.kind.numeric_unit() == Some(NumericUnit::Degrees)) => {
                        match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                            (Some(kind), [4, 0]) if is_point_kind(kind) && h[6..8] == [1, 0] => Kind::CalculatedAngleRotatedPoint(*source, *center, *angle),
                            (Some(kind), [14, 0]) if is_segment_kind(kind) && h[6..8] == [2, 0] => Kind::CalculatedAngleRotatedSegment(*source, *center, *angle),
                            (Some(kind), [13, 0]) if kind.fixed_transform_straight_mode() == Some(LineMode::Ray) && h.len() == 28 && h[6..8] == [2, 0] => Kind::CalculatedAngleRotatedRay(*source, *center, *angle),
                            (Some(kind), [12, 0]) if kind.fixed_transform_straight_mode() == Some(LineMode::Line) && h.len() == 28 && h[6..8] == [2, 0] => Kind::CalculatedAngleRotatedLine(*source, *center, *angle),
                            (Some(kind), [28, 0]) if kind.is_circle() && h[6..8] == [3, 0] => Kind::CalculatedAngleRotatedCircle(*source, *center, *angle),
                            (Some(kind), _) if kind.is_polygon() && h[6..8] == [4, 0] && polygon_vertex_count(&objects, *source).and_then(polygon_subtype) == Some(u16::from_le_bytes([h[8], h[9]])) => Kind::CalculatedAngleRotatedPolygon(*source, *center, *angle),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (33, None, [source, center, angle])
                        if objects.get(center.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(angle.saturating_sub(1)).is_some_and(|o| o.kind.numeric_unit() == Some(NumericUnit::Scalar)) => {
                        match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                            (Some(kind), [4, 0]) if is_point_kind(kind) && h[6..8] == [1, 0] => Kind::CalculatedRatioScaledPoint(*source, *center, *angle),
                            (Some(kind), [14, 0]) if is_segment_kind(kind) && h[6..8] == [2, 0] => Kind::CalculatedRatioScaledSegment(*source, *center, *angle),
                            (Some(kind), [13, 0]) if kind.fixed_transform_straight_mode() == Some(LineMode::Ray) && h.len() == 28 && h[6..8] == [2, 0] => Kind::CalculatedRatioScaledRay(*source, *center, *angle),
                            (Some(kind), [12, 0]) if kind.fixed_transform_straight_mode() == Some(LineMode::Line) && h.len() == 28 && h[6..8] == [2, 0] => Kind::CalculatedRatioScaledLine(*source, *center, *angle),
                            (Some(kind), [28, 0]) if kind.is_circle() && h[6..8] == [3, 0] => Kind::CalculatedRatioScaledCircle(*source, *center, *angle),
                            (Some(kind), _) if kind.is_polygon() && h[6..8] == [4, 0] && polygon_vertex_count(&objects, *source).and_then(polygon_subtype) == Some(u16::from_le_bytes([h[8], h[9]])) => Kind::CalculatedRatioScaledPolygon(*source, *center, *angle),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (28, _, [source, center, first, vertex, last])
                        if [center, first, vertex, last].iter().all(|&&id| {
                            objects.get(id - 1).is_some_and(|object| is_point_kind(&object.kind))
                        }) => match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                            (Some(kind), [4, 0]) if is_point_kind(kind) => Kind::AngleRotatedPoint(*source, *center, *first, *vertex, *last),
                            (Some(Kind::Segment(_, _)), [14, 0]) => Kind::AngleRotatedSegment(*source, *center, *first, *vertex, *last),
                            (Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)), [28, 0]) => Kind::AngleRotatedCircle(*source, *center, *first, *vertex, *last),
                            (Some(Kind::Polygon(vertices)), _) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                && h.len() == 28 && h[6..8] == [4, 0] => Kind::AngleRotatedPolygon(*source, *center, *first, *vertex, *last),
                            _ => Kind::Unsupported(code),
                        },
                    (32, _, [source, center, origin, base, target])
                        if [source, center, origin, base, target].iter().all(|&&id| {
                            objects.get(id - 1).is_some_and(|object| is_point_kind(&object.kind))
                        }) => Kind::RatioScaledPoint(*source, *center, *origin, *base, *target),
                    (32, _, [source, center, origin, base, target])
                        if objects.get(source - 1).is_some_and(|object| object.kind.is_circle())
                            && [center, origin, base, target].iter().all(|&&id| {
                                objects.get(id - 1).is_some_and(|object| is_point_kind(&object.kind))
                            }) => Kind::RatioScaledCircle(*source, *center, *origin, *base, *target),
                    (32, _, [source, center, origin, base, target])
                        if objects.get(source.saturating_sub(1)).is_some_and(|object| matches!(object.kind, Kind::Segment(_, _)))
                            && [center, origin, base, target].iter().all(|&&id| {
                                objects.get(id.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                            }) && h[8..10] == [14, 0] => Kind::RatioScaledSegment(*source, *center, *origin, *base, *target),
                    (32, _, [source, center, origin, base, target])
                        if objects.get(source.saturating_sub(1)).is_some_and(|object| matches!(&object.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && [center, origin, base, target].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind)))
                            && h.len() == 28 && h[6..8] == [4, 0] => Kind::RatioScaledPolygon(*source, *center, *origin, *base, *target),
                    (34, _, [source, mirror])
                        if objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(mirror - 1).is_some_and(|o| is_straight_kind(&o.kind)) =>
                    {
                        Kind::ReflectedPoint(*source, *mirror)
                    }
                    (34, _, [source, mirror])
                        if objects.get(source - 1).is_some_and(|o| o.kind.is_circle())
                            && objects.get(mirror - 1).is_some_and(|o| is_straight_kind(&o.kind)) =>
                    {
                        Kind::ReflectedCircle(*source, *mirror)
                    }
                    (34, _, [source, mirror])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| is_segment_kind(&o.kind))
                            && objects.get(mirror.saturating_sub(1)).is_some_and(|o| is_straight_kind(&o.kind))
                            && h[6..8] == [2, 0] && h[8..10] == [14, 0] => Kind::ReflectedSegment(*source, *mirror),
                    (34, _, [source, mirror])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon())
                            && polygon_vertex_count(&objects, *source).and_then(polygon_subtype) == Some(u16::from_le_bytes([h[8], h[9]]))
                            && objects.get(mirror.saturating_sub(1)).is_some_and(|o| is_straight_kind(&o.kind))
                            && h.len() == 28 && h[6..8] == [4, 0] => Kind::ReflectedPolygon(*source, *mirror),
                    (30, _, [source, center])
                        if objects.get(center - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _)) if payload.len() == 12 && payload[..4] == [30, 0, 0, 0] && f64_at(payload, 4).is_finite() => {
                                let factor = f64_at(payload, 4);
                                match objects.get(source - 1).map(|o| &o.kind) {
                                    Some(kind) if is_point_kind(kind) => Kind::ScaledPoint(*source, *center, factor),
                                    Some(kind) if kind.is_circle() => Kind::ScaledCircle(*source, *center, factor),
                                    Some(kind) if is_segment_kind(kind) && h[6..8] == [2, 0] && h[8..10] == [14, 0] => Kind::ScaledSegment(*source, *center, factor),
                                    Some(kind) if kind.fixed_transform_straight_mode() == Some(LineMode::Ray) && h.len() == 28 && h[6..8] == [2, 0] && h[8..10] == [13, 0] => Kind::ScaledRay(*source, *center, factor),
                                    Some(kind) if kind.fixed_transform_straight_mode() == Some(LineMode::Line) && h.len() == 28 && h[6..8] == [2, 0] && h[8..10] == [12, 0] => Kind::ScaledLine(*source, *center, factor),
                                    Some(Kind::Polygon(vertices)) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                        && h.len() == 28 && h[6..8] == [4, 0] => Kind::ScaledPolygon(*source, *center, factor),
                                    _ => Kind::Unsupported(code),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (27, _, [source, center])
                        if objects.get(center.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _)) if payload.len() == 36 && payload[..4] == [27, 0, 0, 0] && f64_at(payload, 20).is_finite()
                                && f64_at(payload, 4).is_finite() && f64_at(payload, 12).is_finite()
                                && transform_angle_degrees(payload).is_some() => {
                                match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                                    (Some(kind), [4, 0]) if is_point_kind(kind) => Kind::RotatedPoint(*source, *center, transform_angle_degrees(payload).unwrap()),
                                    (Some(kind), [14, 0]) if is_segment_kind(kind) && h[6..8] == [2, 0] => Kind::RotatedSegment(*source, *center, transform_angle_degrees(payload).unwrap()),
                                    (Some(kind), [13, 0]) if kind.fixed_transform_straight_mode() == Some(LineMode::Ray) && h.len() == 28 && h[6..8] == [2, 0] => Kind::RotatedRay(*source, *center, transform_angle_degrees(payload).unwrap()),
                                    (Some(kind), [12, 0]) if kind.fixed_transform_straight_mode() == Some(LineMode::Line) && h.len() == 28 && h[6..8] == [2, 0] => Kind::RotatedLine(*source, *center, transform_angle_degrees(payload).unwrap()),
                                    (Some(kind), [28, 0]) if kind.is_circle() && h[6..8] == [3, 0] => Kind::RotatedCircle(*source, *center, transform_angle_degrees(payload).unwrap()),
                                    (Some(Kind::Polygon(vertices)), _) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                        && h.len() == 28 && h[6..8] == [4, 0] => Kind::RotatedPolygon(*source, *center, transform_angle_degrees(payload).unwrap()),
                                    _ => Kind::Unsupported(code),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (1, _, [segment]) => {
                        match objects.get(segment.saturating_sub(1)).map(|o| &o.kind) {
                            Some(Kind::Segment(a, b)) => Kind::Midpoint(*a, *b),
                            Some(kind) if is_segment_kind(kind) => Kind::SegmentMidpoint(*segment),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (2, _, [a, b]) => Kind::Segment(*a, *b),
                    (3, _, [a, b]) => Kind::Circle(*a, *b),
                    (4, _, [a, b]) => Kind::CircleBySegment(*a, *b),
                    (8, _, vertices)
                        if vertices.len() >= 3
                            && vertices.iter().all(|&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::Polygon(vertices.to_vec())
                    }
                    (5, _, [a, b]) => Kind::Perpendicular(*a, *b),
                    (6, _, [a, b]) => Kind::Parallel(*a, *b),
                    (7, _, [a, b, c]) => Kind::Bisector(*a, *b, *c),
                    (9, _, [a, b]) => Kind::Intersection(*a, *b),
                    (11, _, [a, b])
                        if objects
                            .get(a.saturating_sub(1))
                            .is_some_and(|object| is_straight_kind(&object.kind))
                            && objects.get(b.saturating_sub(1)).is_some_and(|object| {
                                object.kind.is_circle()
                            }) =>
                    {
                        Kind::LineCircleIntersection(*a, *b, 1)
                    }
                    (12, _, [a, b])
                        if objects
                            .get(a.saturating_sub(1))
                            .is_some_and(|object| is_straight_kind(&object.kind))
                            && objects.get(b.saturating_sub(1)).is_some_and(|object| {
                                object.kind.is_circle()
                            }) =>
                    {
                        Kind::LineCircleIntersection(*a, *b, 0)
                    }
                    (13, _, [a, b]) => Kind::CircleCircleIntersection(*a, *b, 1),
                    (14, _, [a, b]) => Kind::CircleCircleIntersection(*a, *b, 0),
                    (63, _, [a, b]) => Kind::Line(*a, *b),
                    (64, _, [a, b]) => Kind::Ray(*a, *b),
                    (80, None, [center, start, end])
                        if h.len() == 28 && h[6..10] == [12, 0, 67, 0]
                            && constraint_payload.is_none()
                            && [center, start, end].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind))) => {
                        Kind::CenterArc(*center, *start, *end)
                    }
                    (81, _, [a, b, c]) => Kind::ThreePointArc(*a, *b, *c),
                    (79, _, [circle, start, end])
                        if objects.get(circle - 1).is_some_and(|o| o.kind.is_circle())
                            && objects.get(start - 1).is_some_and(|o| matches!(o.kind, Kind::PointOnCircle(host, _, _) if host == *circle))
                            && objects.get(end - 1).is_some_and(|o| matches!(o.kind, Kind::PointOnCircle(host, _, _) if host == *circle)) =>
                    {
                        Kind::ArcOnCircle(*circle, *start, *end)
                    }
                    _ => Kind::Unsupported(code),
                }
            };
            if matches!(kind, Kind::Unsupported(_))
                || matches!(kind, Kind::LinkPageButton { target_page, .. } if target_page >= total_pages.max(1))
            {
                safe = false;
            }
            let function_name = if matches!(
                kind,
                Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _)
            ) {
                label
                    .as_ref()
                    .and_then(|value| value.name_utf8().ok())
                    .map(str::to_owned)
            } else {
                None
            };
            if matches!(
                kind,
                Kind::FunctionDefinition(_)
                    | Kind::ReferencedFunction(_, _)
                    | Kind::Parameter(_)
                    | Kind::LinkUrlButton { .. }
                    | Kind::LinkPageButton { .. }
            ) {
                label = None;
            }
            if label.is_some()
                && !is_point_kind(&kind)
                && kind.numeric_unit().is_none()
                && !matches!(
                    kind,
                    Kind::ValueEquation { .. } | Kind::CoordinateMeasurement { .. }
                )
                && !(tool_input.is_some() && matches!(kind, Kind::ToolInput(_)))
            {
                safe = false;
                label = None;
            }
            let function_anchor = if matches!(
                kind,
                Kind::FunctionDefinition(_)
                    | Kind::ReferencedFunction(_, _)
                    | Kind::DerivativeFunction(_, _)
            ) {
                function_expression
                    .as_deref()
                    .and_then(function_records::expression_anchor)
            } else {
                None
            };
            let visual_style = if let Kind::IterationBinding(source, iteration) = &kind
                && objects
                    .get(iteration.saturating_sub(1))
                    .is_some_and(|o| !matches!(o.kind, Kind::Unsupported(_)))
                && objects
                    .get(source.saturating_sub(1))
                    .is_some_and(|o| is_point_kind(&o.kind))
            {
                style_records::iteration_point_binding_visual_style(h)
            } else if let Kind::IterationBinding(source, _) = &kind
                && objects.get(source.saturating_sub(1)).is_some_and(|o| {
                    is_straight_kind(&o.kind)
                        || o.kind.is_circle()
                        || o.kind.is_polygon()
                        || is_arc_kind(&o.kind)
                })
            {
                style_records::iteration_binding_visual_style(
                    h,
                    objects
                        .get(source.saturating_sub(1))
                        .is_some_and(|o| o.kind.is_polygon()),
                )
            } else {
                style_records::visual_style(h)
            };
            objects.push(Object {
                id: objects.len() + 1,
                page,
                midpoint_host: matches!(kind, Kind::Midpoint(_, _)).then(|| refs[0]),
                kind,
                visible: raw & 0x10000 == 0,
                presentation_suppressed: raw & 0x20000 != 0,
                tracing: h[4] & 1 != 0,
                deleted: false,
                point_offset,
                constraint_offset: constraint_payload.map(|(_, offset)| offset),
                header_offset: Some(header_offset),
                label,
                visual_style,
                interior_order: (h.len() == 28 && h[6..8] == [4, 0])
                    .then(|| i32::from_le_bytes(h[20..24].try_into().unwrap())),
                text_style_override: None,
                rich_text_override: None,
                function_anchor,
                function_name,
                measurement_precision,
                measurement_anchor,
            });
            if matches!(objects.last().map(|o| &o.kind), Some(Kind::Unsupported(_))) {
                safe = false;
            }
            i += 1;
        }
        let source_count = objects.len();
        let preambles: Vec<_> = document_records(&rs)
            .filter(|r| r.kind == 0x3e8 && r.data.len() >= 28)
            .collect();
        if preambles.len() != 1 || u32_at(b, preambles[0].data.start + 24) as usize != source_count
        {
            safe = false;
        }
        if document_records(&rs)
            .filter(|record| record.kind == 0x232d)
            .any(|record| {
                record.data.len() != 4
                    || !objects
                        .get((u32_at(b, record.data.start) as usize).saturating_sub(1))
                        .is_some_and(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)))
            })
        {
            safe = false;
        }
        let mut marked_coordinate_system = vec![None; page_count.max(1)];
        let mut link_page = 0usize;
        let mut seen_page = false;
        for record in document_records(&rs) {
            if record.kind == 0x3e8 {
                if seen_page {
                    link_page += 1;
                }
                seen_page = true;
            } else if record.kind == 0x232d && record.data.len() == 4 {
                let id = u32_at(b, record.data.start) as usize;
                if objects.get(id.saturating_sub(1)).is_some_and(|object| {
                    object.page == link_page && matches!(object.kind, Kind::CoordinateSystem(_, _))
                }) {
                    marked_coordinate_system[link_page] = Some(id);
                } else {
                    safe = false;
                }
            }
        }
        for (page, marked) in marked_coordinate_system.iter().enumerate() {
            if let Some((form, dotted)) = marked.and_then(|id| frame_styles.get(&id).copied()) {
                grid_form[page] = form;
                grid_dotted[page] = dotted;
            }
        }
        let active_page = rs
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .and_then(|record| {
                let count = u16::from_le_bytes(
                    b[record.data.start + 2..record.data.start + 4]
                        .try_into()
                        .ok()?,
                ) as usize;
                let index = u16::from_le_bytes(
                    b[record.data.start + 16..record.data.start + 18]
                        .try_into()
                        .ok()?,
                ) as usize;
                (count == page_count && index < count).then_some(index)
            })
            .unwrap_or(0);
        Ok(Self {
            objects,
            path,
            selected: BTreeSet::new(),
            dirty: false,
            partial_view: !safe,
            active_page,
            page_count: page_count.max(1),
            snap_points: if snap_points.is_empty() {
                vec![false]
            } else {
                snap_points
            },
            grid_dotted: if grid_dotted.is_empty() {
                vec![false]
            } else {
                grid_dotted
            },
            grid_form: if grid_form.is_empty() {
                vec![GridForm::Square]
            } else {
                grid_form
            },
            page_names,
            angle_unit_radians,
            marked_coordinate_system,
            frame_styles,
            evaluation_context: None,
            original: b.to_vec(),
            run_font_families: Vec::new(),
            source_count,
            last_moved_object: None,
            safe_structure: safe,
            image_read_only,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }
}
