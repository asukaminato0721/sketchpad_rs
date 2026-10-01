// Native's generic Polygon subtype 15 is an imported wire identity, not a
// vertex-count encoding. Keep canonical identities for newly authored sources.
pub(super) fn polygon_constraint_subtype(
    objects: &[super::Object],
    bytes: &[u8],
    host: usize,
    count: usize,
) -> Option<u16> {
    let object = objects.get(host.checked_sub(1)?)?;
    if matches!(&object.kind, super::Kind::Polygon(vertices) if vertices.len() == count)
        && let Some(at) = object.header_offset
        && bytes.get(at..at + 2) == Some(&[8, 0])
        && bytes.get(at + 6..at + 10) == Some(&[4, 0, 15, 0])
        && (count == 9 || matches!(bytes.get(at + 2..at + 4), Some([0, 0] | [1, 0])))
    {
        return Some(15);
    }
    super::records::polygon_subtype(count)
}

use super::calculations::parse_numeric_calculation;
use super::embedded_tools::{document_record_indices, document_records};
use super::iteration::{
    mapped_binding_kind, supported_branch_polygon_mapping, supported_iteration_values,
    supported_mapped_point_iteration, supported_point_iteration_mapping,
    supported_point_parameter_iteration, supported_tool_mapped_point_iteration,
};
use super::kind::{
    is_arc_kind, is_coordinate_unit_kind, is_literal_plot_path_kind, is_locus_polygon_kind,
    is_locus_straight_kind, is_point_kind, is_segment_kind, is_straight_kind,
    is_value_plot_path_kind, parameter_locus_refs_match, polygon_vertex_count, straight_kind_mode,
};
use super::records::{f64_at, page_title_slots, polygon_subtype, records, u32_at};
use super::{
    Document, EmbeddedToolInput, EmbeddedToolInputKind, ImagePlacement, IterationDepth, Kind,
    LineMode, Object, Point,
};
use crate::action_records::{
    AnimationButtonGroup, HideShowButtonGroup, LinkPageButtonGroup, LinkUrlButtonGroup,
    MovementButtonGroup, PresentationButtonGroup, ScrollButtonGroup, SoundButtonGroup,
};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::graph::{GraphObject, GridForm};
use crate::image_records::{ImageFile, ReadOnlyImageAttachment};
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

fn iteration_definition_matches(data: &[u8], columns: u16, branches: u32) -> bool {
    data.len() == 24
        && data[..4] == [0; 4]
        && matches!(data[4..6], [0, 0] | [1, 0])
        && data[6..8] == columns.to_le_bytes()
        && u32_at(data, 8) == branches
        && data[20..24] == [0; 4]
}

fn transformed_image_png(
    objects: &[Object],
    page: usize,
    id: usize,
    limit: usize,
) -> Option<&Vec<u8>> {
    let mut current = id;
    loop {
        if current == 0 || current >= limit {
            return None;
        }
        let object = objects.get(current - 1)?;
        if object.deleted || object.page != page {
            return None;
        }
        match &object.kind {
            Kind::Image { png, .. } => return Some(png),
            Kind::CroppedImage { source, .. } => current = *source,
            Kind::TransformedCroppedImage(definition) => current = definition.source(),
            _ => return None,
        }
    }
}

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

/// Validate one fixed-Cartesian distance and return its declared pixel scale.
/// Native tag 1 is centimeters and tag 2 is inches.
pub(super) fn cartesian_distance_scale(
    payload: &[u8],
    pixel_offset: usize,
    magnitude_offset: usize,
    unit_offset: usize,
) -> Option<f64> {
    let scale = match u32_at(payload, unit_offset) {
        0 => 1.,
        1 => 37.79527559055118,
        2 => 96.,
        _ => return None,
    };
    let pixels = f64_at(payload, pixel_offset);
    let magnitude = f64_at(payload, magnitude_offset);
    (pixels.is_finite() && magnitude.is_finite() && (pixels - magnitude * scale).abs() < 1e-5)
        .then_some(scale)
}

fn mapped_iteration_internal_label(payload: &[u8]) -> bool {
    let Ok(label) = LabelAux::parse(payload) else {
        return false;
    };
    payload.len() == 28
        && label
            .name_utf8()
            .is_ok_and(|name| matches!(name.as_bytes(), [b'[', b'1'..=b'9', b']']))
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
                    || !record.data.is_empty()
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
                    || !(payload.len() - 4).is_multiple_of(4)
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
                    || !(payload.len() - 4).is_multiple_of(4)
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
        let cropped_images = if has_picture_resource {
            crate::image_records::imported_cropped_image_catalog(b)
                .unwrap_or_default()
                .into_iter()
                .map(|crop| (crop.group_offset, crop))
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };
        let imported_images = if has_picture_resource && image_file.is_none() {
            crate::image_records::imported_image_catalog(b)
                .unwrap_or_default()
                .into_iter()
                .map(|image| (image.group_offset, image))
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };
        let attached_images = if has_picture_resource {
            crate::image_records::imported_attached_image_catalog(b)
                .unwrap_or_default()
                .into_iter()
                .map(|image| (image.group_offset, image))
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };
        let recognized_picture_resources = image_file.is_some()
            || !cropped_images.is_empty()
            || !imported_images.is_empty()
            || !attached_images.is_empty();
        let image_read_only =
            has_picture_resource && !image_file.as_ref().is_some_and(ImageFile::can_edit);
        let baseline_records = records(include_bytes!("../../../../assets/point_template.gsp"))?;
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
        // Old Windows Chinese sketches use code page 936 for every short-header
        // free-text object. Detect the document cohort from an unambiguously
        // non-UTF-8 member so byte pairs that are coincidentally valid UTF-8
        // (for example GBK `CA B1`, "时") use the same encoding.
        let legacy_gbk_text = rs.iter().enumerate().any(|(index, record)| {
            document_indices.contains(&index)
                && record.kind == 0x07d0
                && b[record.data.clone()] == text_object::LEGACY_TEXT_HEADER
                && rs
                    .get(index + 1..)
                    .into_iter()
                    .flatten()
                    .take_while(|record| record.kind != 0x07d7)
                    .find(|record| record.kind == 0x08fc)
                    .is_some_and(|rich| {
                        text_object::rich_text_requires_legacy_gbk(&b[rich.data.clone()])
                    })
        });
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
                    && !(rs[i].kind == 0x0514 && recognized_picture_resources)
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
                .is_some_and(|picture| picture.picture_group_offset() == group_start)
                || imported_images.contains_key(&group_start);
            let is_attached_image = attached_images.contains_key(&group_start);
            let is_cropped_image = cropped_images.contains_key(&group_start);
            let raw = u32_at(h, 0);
            let code = raw as u16;
            let tool_input = tool_inputs.get(&(objects.len() + 1)).copied();
            let mut point = None;
            let mut point_offset = None;
            let mut constraint_payload: Option<(Vec<u8>, usize)> = None;
            let mut curve_constraint_count = 0usize;
            let mut point_curve_constraint_count = 0usize;
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
            let mut rich_text_override = None;
            let mut angle_marker_style = None;
            let angle_marker_label =
                h.len() == 28 && matches!(code, 113..=116) && h[6..10] == [18, 0, 95, 0];
            let labeled_shape = (code == 10 && matches!(h[6..10], [5, 0, 16, 0]))
                || (matches!(code, 82 | 83) && matches!(h[6..10], [13, 0, 68, 0] | [13, 0, 69, 0]))
                || matches!(h[6..8], [1, 0] | [2, 0] | [3, 0] | [4, 0])
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
                            | 26
                            | 27
                            | 28
                            | 29
                            | 30
                            | 32
                            | 33
                            | 34
                            | 63
                            | 64
                            | 67
                            | 68
                            | 69
                            | 88
                            | 93
                            | 98
                            | 101
                            | 109
                            | 110
                            | 119
                            | 75
                            | 95
                            | 123
                    );
            let arc_label = h[6..10] == [12, 0, 67, 0]
                && (matches!(code, 75 | 79..=81 | 119)
                    || matches!(code, 16..=21 | 24 | 27..=30 | 32..=34));
            // Polygon groups also carry 07d5 naming metadata, but that record
            // is not a displayed label. Keep it inert while admitting the
            // proved point, straight, circle, base-arc, and transformed-arc
            // presentations.
            let geometric_label =
                arc_label || labeled_shape && matches!(h[6..8], [1, 0] | [2, 0] | [3, 0]);
            let image_internal_label = matches!(code, 99 | 100) && h[6..10] == [16, 0, 78, 0];
            let internal_iteration_label = (matches!(code, 76 | 89) && h[6..10] == [14, 0, 64, 0])
                || (code == 77 && h[6..10] == [15, 0, 65, 0]);
            let locus_internal_label = (code == 35
                && matches!(h[6..10], [6, 0, 42, 0] | [6, 0, 43, 0]))
                || (!tool_inputs.is_empty()
                    && code == 97
                    && matches!(h[6..10], [6, 0, 42, 0] | [6, 0, 43, 0]));
            let referenced_caption = matches!(code, 73 | 90) && h[6..10] == [0, 0, 59, 0];
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
                    | 120
                    | 124
                    | 125
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
                if matches!(code, 0 | 71 | 78 | 122) && r.kind == 0x0907 {
                    function_expression = Some(d.to_vec());
                }
                if matches!(code, 72 | 126 | 127) && r.kind == 0x0902 {
                    function_range = Some(d.to_vec());
                }
                if matches!(code, 35 | 96 | 97 | 102) && r.kind == 0x0902 {
                    locus_range = Some(d.to_vec());
                }
                if matches!(code, 76 | 89) && r.kind == 0x090a {
                    iteration_definition = Some(d.to_vec());
                }
                if angle_marker_label && r.kind == 0x090e && d.len() == 12 {
                    angle_marker_style = Some(super::AngleMarkerStyle {
                        arrow: d[2] != 0,
                        opacity: u16::from_le_bytes([h[16], h[17]]),
                    });
                }
                match r.kind {
                    0x07d5 if image_internal_label => {
                        if !crate::image_records::internal_image_label(d) {
                            safe = false;
                        }
                    }
                    0x07d5 if internal_iteration_label => {
                        if !mapped_iteration_internal_label(d) {
                            safe = false;
                        }
                    }
                    0x07d5 if locus_internal_label => {
                        if LabelAux::parse(d).is_err() {
                            safe = false;
                        }
                    }
                    0x07d5 if referenced_caption => match LabelAux::parse(d) {
                        Ok(value) => label = Some(value),
                        Err(_) => safe = false,
                    },
                    0x07d5
                        if matches!(code, 0 | 15 | 92)
                            || (code == 71 && h[6..10] == [11, 0, 57, 0])
                            || labeled_shape
                            || arc_label
                            || angle_marker_label
                            || numeric_label =>
                    {
                        match LabelAux::parse(d) {
                            Ok(value)
                                if matches!(code, 0 | 15 | 92)
                                    || (code == 71 && h[6..10] == [11, 0, 57, 0])
                                    || numeric_label
                                    || angle_marker_label
                                    || geometric_label =>
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
                        if matches!(code, 0 | 15 | 92)
                            || (code == 71 && h[6..10] == [11, 0, 57, 0])
                            || labeled_shape
                            || arc_label
                            || angle_marker_label
                            || locus_internal_label =>
                    {
                        text_style = d.len() == 2
                            && (u16::from_le_bytes([d[0], d[1]]) as usize) < text_style_count;
                        if (labeled_shape
                            || arc_label
                            || angle_marker_label
                            || locus_internal_label
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
                        if matches!(
                            code,
                            36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47 | 120
                        ) =>
                    {
                        let sample: &[u8] = match code {
                            36 => {
                                &include_bytes!("../../../../assets/length_measurement_group.bin")
                                    [60..92]
                            }
                            37 => {
                                &include_bytes!("../../../../assets/distance_measurement_group.bin")
                                    [64..96]
                            }
                            38 => &include_bytes!(
                                "../../../../assets/point_line_distance_measurement_group.bin"
                            )[64..96],
                            39 => &include_bytes!(
                                "../../../../assets/perimeter_measurement_group.bin"
                            )[60..92],
                            40 => &include_bytes!(
                                "../../../../assets/circumference_measurement_group.bin"
                            )[60..92],
                            41 | 120 => {
                                &include_bytes!("../../../../assets/angle_measurement_group.bin")
                                    [68..100]
                            }
                            42 => &include_bytes!("../../../../assets/area_measurement_group.bin")
                                [60..92],
                            43 => &include_bytes!(
                                "../../../../assets/arc_angle_measurement_group.bin"
                            )[60..92],
                            44 => &include_bytes!(
                                "../../../../assets/arc_length_measurement_group.bin"
                            )[60..92],
                            47 => &include_bytes!("../../../../assets/ratio_measurement_group.bin")
                                [68..100],
                            _ => &include_bytes!("../../../../assets/radius_measurement_group.bin")
                                [60..92],
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
                    0x090d if code == 92 && d.len() == 28 => {
                        measurement_anchor = Some((
                            i16::from_le_bytes(d[12..14].try_into().unwrap()),
                            i16::from_le_bytes(d[14..16].try_into().unwrap()),
                        ));
                    }
                    0x0903
                        if matches!(code, 65 | 66 | 86 | 87 | 94 | 124 | 125) && d.len() == 32 =>
                    {
                        let precision = u16::from_le_bytes(d[28..30].try_into().unwrap());
                        if precision <= 5 {
                            measurement_precision = Some(precision);
                        }
                    }
                    0x07d8 if numeric_label => {
                        measurement_style = d.len() == 2
                            && (usize::from(u16::from_le_bytes([d[0], d[1]])) < text_style_count
                                || u16::from_le_bytes([d[0], d[1]])
                                    == if matches!(
                                        code,
                                        49 | 50 | 51 | 65 | 66 | 86 | 87 | 124 | 125
                                    ) {
                                        2
                                    } else {
                                        1
                                    });
                        if !measurement_style {
                            safe = false;
                        }
                    }
                    0x7d3 if code == 98 => {
                        curve_constraint_count += 1;
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d3 if code == 96 => {
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d3 if matches!(code, 11 | 12 | 13 | 14 | 93) => {
                        // Bounded arc intersections have references only.
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d3 if code == 33 && h[6..10] == [1, 0, 89, 0] => {
                        // Inherited mapped-point dilation has references only;
                        // retain every payload length so malformed operands reject.
                        constraint_payload = Some((d.to_vec(), r.data.start));
                        safe = false;
                    }
                    0x7d3 if code == 16 => {
                        // Proved vector packets have references only. Retain
                        // unproved payloads for strict typing without treating
                        // an otherwise unsupported record as safe structure.
                        constraint_payload = Some((d.to_vec(), r.data.start));
                        safe = false;
                    }
                    0x7d3 if code == 15 => {
                        point_curve_constraint_count += 1;
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d3
                        if matches!(
                            code,
                            15 | 17
                                | 18
                                | 19
                                | 21
                                | 22
                                | 23
                                | 24
                                | 27
                                | 30
                                | 52
                                | 54
                                | 55
                                | 56
                                | 57
                                | 75
                                | 88
                                | 109
                                | 110
                        ) =>
                    {
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d2 | 0x7d3 if d.len() >= 4 && (d.len() - 4).is_multiple_of(4) => {
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
                    && !(r.kind == 0x232e && d.is_empty())
                    && !(code == 0
                        && matches!(r.kind, 0x07d5 | 0x07d8 | 0x08fc | 0x0898 | 0x0906 | 0x08a2))
                    && !((matches!(code, 15 | 92) || (code == 71 && h[6..10] == [11, 0, 57, 0]))
                        && matches!(r.kind, 0x07d5 | 0x07d8))
                    && !((labeled_shape || arc_label || numeric_label || angle_marker_label)
                        && matches!(r.kind, 0x07d5 | 0x07d8))
                    && !(image_internal_label && r.kind == 0x07d5)
                    && !(internal_iteration_label && r.kind == 0x07d5)
                    && !(locus_internal_label && matches!(r.kind, 0x07d5 | 0x07d8))
                    && !(matches!(
                        code,
                        36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47 | 120
                    ) && matches!(r.kind, 0x0903 | 0x07d8))
                    && !(matches!(code, 52 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 87)
                        && matches!(r.kind, 0x0905 | 0x0904 | 0x0903 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 49 | 65 | 66 | 86 | 124 | 125)
                        && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(matches!(code, 50 | 51 | 94) && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(code == 62 && matches!(r.kind, 0x0906 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 0 | 71) && matches!(r.kind, 0x0907 | 0x08a3))
                    && !(code == 122 && matches!(r.kind, 0x0907 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 48 | 78) && matches!(r.kind, 0x0907 | 0x07d8))
                    && !(matches!(
                        code,
                        10 | 48
                            | 65
                            | 66
                            | 67
                            | 68
                            | 73
                            | 75
                            | 79
                            | 82
                            | 90
                            | 94
                            | 95
                            | 98
                            | 123
                            | 92
                    ) && r.kind == 0x232e
                        && d.is_empty())
                    && !(code == 0
                        && matches!(u16::from_le_bytes([h[8], h[9]]), 79..=81)
                        && r.kind == 0x232e
                        && d.is_empty())
                    && !(code == 91 && matches!(r.kind, 0x090d | 0x07d8))
                    && !(referenced_caption && matches!(r.kind, 0x07d5 | 0x08fc | 0x07d8))
                    && !(code == 92 && matches!(r.kind, 0x090d | 0x07d8))
                    && !(is_image && matches!(r.kind, 0x090c | 0x08a8 | 0x1f44))
                    && !(is_attached_image
                        && (matches!(r.kind, 0x090c | 0x1f44) || r.kind == 0x232e && d.is_empty()))
                    && !(is_cropped_image && (r.kind == 0x090c || r.kind == 0x232e && d.is_empty()))
                    && !(matches!(code, 16..=21 | 24 | 27..=30 | 32..=34)
                        && h[6..10] == [16, 0, 78, 0]
                        && r.kind == 0x090c
                        && d.len() == 16)
                    && !(matches!(code, 72 | 126 | 127) && r.kind == 0x0902)
                    && !(matches!(code, 35 | 96 | 97 | 102) && r.kind == 0x0902)
                    && !(matches!(code, 76 | 89) && r.kind == 0x090a)
                    && !(angle_marker_label && r.kind == 0x232e && d.is_empty())
                    && !(angle_marker_label
                        && r.kind == 0x090e
                        && d.len() == 12
                        && matches!(u16::from_le_bytes([d[0], d[1]]), 1 | 2)
                        && (code == 113 || d[0] == 1)
                        && matches!(u16::from_le_bytes([d[2], d[3]]), 0 | 1)
                        && d[4..] == [0, 0, 0, 0, 254, 254, 254, 0])
                    && !(code == 121 && r.kind == 0x090f && d.len() == 16)
                    && !(code == 121 && r.kind == 0x232e && d.is_empty())
                    // Native custom-tool prototypes attach this empty inert
                    // marker to several otherwise ordinary typed objects. It
                    // is not materialized into the invoked drawing.
                    && !(!tool_inputs.is_empty() && r.kind == 0x232e && d.is_empty())
                {
                    safe = false;
                }
                i += 1;
            }
            if i == rs.len() {
                return Err("Unterminated object group".into());
            }
            let empty_arrow_marker = rs.iter().find(|record| {
                record.start > group_start
                    && record.start < rs[i].start
                    && record.kind == 0x232e
                    && record.data.is_empty()
            });
            let arrow_selectable = empty_arrow_marker.is_none();
            let semantic_group_storage = empty_arrow_marker.map(|marker| {
                let mut group = b[group_start..marker.start].to_vec();
                group.extend_from_slice(&b[marker.data.end..rs[i].data.end]);
                group
            });
            let semantic_group = semantic_group_storage
                .as_deref()
                .unwrap_or(&b[group_start..rs[i].data.end]);
            let action_bytes = (code == 62).then_some(semantic_group);
            let action_group =
                action_bytes.and_then(|bytes| HideShowButtonGroup::parse(bytes).ok());
            let animation_group =
                action_bytes.and_then(|bytes| AnimationButtonGroup::parse(bytes).ok());
            let sound_group = action_bytes.and_then(|bytes| SoundButtonGroup::parse(bytes).ok());
            let movement_group =
                action_bytes.and_then(|bytes| MovementButtonGroup::parse(bytes).ok());
            let scroll_group = action_bytes.and_then(|bytes| ScrollButtonGroup::parse(bytes).ok());
            let presentation_group =
                action_bytes.and_then(|bytes| PresentationButtonGroup::parse(bytes).ok());
            let link_url_group = (code == 0)
                .then_some(semantic_group)
                .and_then(|bytes| LinkUrlButtonGroup::parse(bytes).ok());
            let link_page_group = (code == 0)
                .then_some(semantic_group)
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
                && sound_group.is_none()
                && movement_group.is_none()
                && scroll_group.is_none()
                && presentation_group.is_none()
            {
                safe = false;
            }
            let coordinate_group = matches!(code, 49 | 65 | 66 | 86 | 124 | 125)
                .then(|| coordinate_measure_records::decode_group(semantic_group))
                .transpose();
            if coordinate_group.is_err() {
                safe = false;
            }
            let coordinate_group = coordinate_group.ok().flatten();
            let value_equation_group = matches!(code, 50 | 51 | 94)
                .then(|| value_equation_records::decode_group(semantic_group))
                .transpose();
            if value_equation_group.is_err() {
                safe = false;
            }
            let value_equation_group = value_equation_group.ok().flatten();
            let plotted_point = matches!(code, 67 | 68)
                .then(|| plot_points_records::decode_group(semantic_group))
                .transpose();
            if plotted_point.is_err() {
                safe = false;
            }
            let plotted_point = plotted_point.ok().flatten();
            let axis_value_refs = (code == 95)
                .then(|| plot_axis_records::decode_group(semantic_group))
                .transpose();
            if axis_value_refs.is_err() {
                safe = false;
            }
            let axis_value_refs = axis_value_refs.ok().flatten();
            let literal_axis_value = (code == 123)
                .then(|| plot_axis_records::decode_literal_group(semantic_group))
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
                    || !(data[..4] == [0; 4] || (code == 89 && data[..4] == [0, 0, 1, 0]))
                    || !matches!(&data[4..6], [0, 0] | [1, 0])
                    || data[20..24] != [0; 4]
                {
                    return None;
                }
                let count = usize::from(u16::from_le_bytes(data[6..8].try_into().ok()?));
                let branches_count = u32_at(data, 8) as usize;
                if !(1..=128).contains(&count) || !(1..=16).contains(&branches_count) {
                    return None;
                }
                let start = usize::from(code == 89);
                if refs.len() <= start + count * (branches_count + 1) {
                    return None;
                }
                let raw_seeds = &refs[start..start + count];
                let branch_end = start + count * (branches_count + 1);
                let raw_branches = refs[start + count..branch_end]
                    .chunks_exact(count)
                    .map(<[usize]>::to_vec)
                    .collect::<Vec<_>>();
                // Native packets may repeat identical substitution columns as well as
                // carried prerequisites. Both describe one reevaluated DAG, so keep
                // the first occurrence while requiring duplicate columns to agree.
                let mut first_seed_columns = BTreeMap::new();
                let mut columns = Vec::new();
                for (column, seed) in raw_seeds.iter().copied().enumerate() {
                    if let Some(first) = first_seed_columns.get(&seed).copied() {
                        if raw_branches
                            .iter()
                            .any(|branch| branch[column] != branch[first])
                        {
                            return None;
                        }
                    } else {
                        first_seed_columns.insert(seed, column);
                        columns.push(column);
                    }
                }
                let seeds = columns
                    .iter()
                    .map(|column| raw_seeds[*column])
                    .collect::<Vec<_>>();
                let branches = raw_branches
                    .iter()
                    .map(|branch| columns.iter().map(|column| branch[*column]).collect())
                    .collect::<Vec<_>>();
                let mut seen = BTreeSet::new();
                let carried = refs[branch_end..]
                    .iter()
                    .copied()
                    .filter(|id| seen.insert(*id))
                    .collect::<Vec<_>>();
                let supported = if tool_inputs.is_empty() {
                    supported_mapped_point_iteration(&objects, &seeds, &branches, &carried)
                } else {
                    supported_tool_mapped_point_iteration(&objects, &seeds, &branches, &carried)
                };
                if !supported {
                    return None;
                }
                let depth = if code == 89 {
                    let id = refs[0];
                    if !objects.get(id.checked_sub(1)?).is_some_and(|o| {
                        o.kind.is_iteration_depth_value()
                            || !tool_inputs.is_empty()
                                && matches!(o.kind, Kind::ToolInput(EmbeddedToolInputKind::Numeric))
                    }) {
                        return None;
                    }
                    IterationDepth::Value(id)
                } else {
                    let value = u32_at(data, 16);
                    if !(1..=super::iteration::MAX_MAPPED_ITERATION_DEPTH).contains(&value) {
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
            let iteration_values_group = code == 92
                && h.len() == 28
                && h[2] & !3 == 0
                && {
                    let template = include_bytes!(
                        "../../../../assets/point_parameter_iteration_values_group.bin"
                    );
                    h.iter()
                        .zip(&template[8..36])
                        .enumerate()
                        .all(|(at, (actual, expected))| {
                            matches!(at, 2 | 12..=15) || actual == expected
                        })
                }
                && {
                    let group = rs
                        .iter()
                        .filter(|record| record.start >= group_start && record.start <= rs[i].start)
                        .collect::<Vec<_>>();
                    let kinds = group.iter().map(|record| record.kind).collect::<Vec<_>>();
                    let layout = matches!(
                        kinds.as_slice(),
                        [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
                            | [0x07d0, 0x07d2, 0x090d, 0x07d5, 0x07d8, 0x07d7]
                            | [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x232e, 0x07d7]
                            | [0x07d0, 0x07d2, 0x090d, 0x07d5, 0x07d8, 0x232e, 0x07d7]
                    );
                    let references = group.iter().find(|record| record.kind == 0x07d2);
                    let placement = group.iter().find(|record| record.kind == 0x090d);
                    layout
                        && text_style
                        && references.is_some_and(|record| record.data.len() == 4 + refs.len() * 4)
                        && placement.is_some_and(|record| {
                            let data = &b[record.data.clone()];
                            data.len() == 28
                                && data[..12] == [13, 9, 0, 0, 28, 0, 0, 0, 16, 0, 0, 0]
                                && data[16..] == [0; 12]
                        })
                        && group
                            .iter()
                            .all(|record| record.kind != 0x232e || record.data.is_empty())
                };
            let segment_marker = (|| {
                if code != 121
                    || h.len() != 28
                    || h[6..10] != [19, 0, 96, 0]
                    || point.is_some()
                    || constraint_payload.is_some()
                {
                    return None;
                }
                let [host] = refs.as_slice() else {
                    return None;
                };
                let segment_host = objects.get(host.checked_sub(1)?).is_some_and(|object| {
                    !object.deleted && object.page == page && is_segment_kind(&object.kind)
                });
                let arrow_host = super::segment_marker::arrow_host_supported(
                    &objects,
                    *host,
                    page,
                    objects.len() + 1,
                );
                if !segment_host && !arrow_host {
                    return None;
                }
                let group = rs
                    .iter()
                    .filter(|record| record.start >= group_start && record.start <= rs[i].start)
                    .collect::<Vec<_>>();
                let layout = group
                    .iter()
                    .map(|record| (record.kind, record.data.len()))
                    .collect::<Vec<_>>();
                if layout != [(0x07d0, 28), (0x07d2, 8), (0x090f, 16), (0x07d7, 0)]
                    && layout
                        != [
                            (0x07d0, 28),
                            (0x07d2, 8),
                            (0x090f, 16),
                            (0x232e, 0),
                            (0x07d7, 0),
                        ]
                {
                    return None;
                }
                let payload = &b[group
                    .iter()
                    .find(|record| record.kind == 0x090f)?
                    .data
                    .clone()];
                let parameter = f64_at(payload, 0);
                if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
                    return None;
                }
                match (u32_at(payload, 8), u32_at(payload, 12)) {
                    (count @ 1..=3, 1) if segment_host => Some(Kind::SegmentMarker {
                        host: *host,
                        parameter,
                        count: count as u16,
                    }),
                    (0x0001_0001, direction @ 0..=1) => Some(Kind::SegmentArrow {
                        host: *host,
                        parameter,
                        forward: direction == 1,
                    }),
                    _ => None,
                }
            })();
            let polymorphic_preimage_transform = !tool_inputs.is_empty()
                && matches!(code, 16 | 33)
                && h.len() == 28
                && h[6..10] == [1, 0, 92, 0]
                && constraint_payload.is_none()
                && refs.len() == 3
                && refs
                    .first()
                    .and_then(|id| objects.get(id.saturating_sub(1)))
                    .is_some_and(|object| {
                        matches!(
                            object.kind,
                            Kind::ToolInput(EmbeddedToolInputKind::Preimage)
                        ) || object
                            .header_offset
                            .is_some_and(|at| b.get(at + 6..at + 10) == Some(&[1, 0, 92, 0]))
                    });
            let polymorphic_polygon_transform = !tool_inputs.is_empty()
                && matches!(code, 16 | 29)
                && h.len() == 28
                && h[6..10] == [1, 0, 15, 0]
                && constraint_payload.is_none()
                && refs.len() == 3
                && refs
                    .first()
                    .and_then(|id| objects.get(id.saturating_sub(1)))
                    .is_some_and(|object| {
                        matches!(object.kind, Kind::ToolInput(EmbeddedToolInputKind::Polygon))
                            || object
                                .header_offset
                                .is_some_and(|at| b.get(at + 6..at + 10) == Some(&[1, 0, 15, 0]))
                    });
            let mut transform_header = h.to_vec();
            if polymorphic_preimage_transform {
                transform_header[6..10].copy_from_slice(&[1, 0, 4, 0]);
            } else if polymorphic_polygon_transform {
                transform_header[6..10].copy_from_slice(&[4, 0, 15, 0]);
            }
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
                    let bytes = semantic_group;
                    match input.native_subtype {
                        33 => super::tool_inputs::numeric_given_layout(bytes)?,
                        51 => super::tool_inputs::calculation_given_layout(bytes)?,
                        _ => return Err("Unsupported numeric tool given subtype".into()),
                    }
                } else if input.kind() == Some(EmbeddedToolInputKind::Distance) {
                    super::tool_inputs::distance_given_layout(semantic_group)?;
                } else if input.kind() == Some(EmbeddedToolInputKind::Function) {
                    super::tool_inputs::function_given_layout(semantic_group)?;
                } else if input.kind() == Some(EmbeddedToolInputKind::ScalarParameter) {
                    super::tool_inputs::scalar_parameter_given_layout(semantic_group)?;
                } else if input.kind() == Some(EmbeddedToolInputKind::DistanceParameter) {
                    super::tool_inputs::distance_parameter_given_layout(semantic_group)?;
                } else if input.kind() == Some(EmbeddedToolInputKind::AngleParameter) {
                    super::tool_inputs::angle_parameter_given_layout(semantic_group)?;
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
                        | EmbeddedToolInputKind::SegmentDistance
                        | EmbeddedToolInputKind::Polygon
                        | EmbeddedToolInputKind::CoordinateSystem
                        | EmbeddedToolInputKind::Numeric
                        | EmbeddedToolInputKind::Distance
                        | EmbeddedToolInputKind::Function
                        | EmbeddedToolInputKind::ScalarParameter
                        | EmbeddedToolInputKind::DistanceParameter
                        | EmbeddedToolInputKind::AngleParameter
                        | EmbeddedToolInputKind::Preimage),
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
            } else if let Some(marker) = segment_marker {
                marker
            } else if !tool_inputs.is_empty() && code == 48 && h[6..10] == [11, 0, 33, 0] {
                // An abstract tool calculation is typed only after binding. Its
                // native expression and references remain exact in the template.
                let mut normalized = semantic_group.to_vec();
                normalized[16..18].copy_from_slice(&51_u16.to_le_bytes());
                match ParameterCalculationGroup::parse(&normalized) {
                    Ok(group)
                        if !refs.is_empty()
                            && group.sources()
                                == refs
                                    .iter()
                                    .map(|id| (*id - page_base) as u32)
                                    .collect::<Vec<_>>()
                            && group.source_kinds().is_ok_and(|kinds| {
                                kinds.len() == refs.len()
                                    && refs.iter().zip(kinds).all(|(id, kind)| {
                                        objects.get(id - 1).is_some_and(|object| match kind {
                                            crate::table_records::CalculationSourceKind::Numeric => {
                                                object.kind.numeric_unit().is_some()
                                                    || matches!(
                                                        object.kind,
                                                        Kind::ToolInput(
                                                            EmbeddedToolInputKind::Numeric
                                                        )
                                                    )
                                            }
                                            crate::table_records::CalculationSourceKind::Function => {
                                                object.kind.is_function()
                                            }
                                        })
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
                            (Some(Object { node: sketchpad_ir::ObjectNode { kind: Kind::CircleAxis(a, 1), .. }, .. }),
                             Some(Object { node: sketchpad_ir::ObjectNode { kind: Kind::CircleAxis(b, 0), .. }, .. })) if a == b
                        )
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { node: sketchpad_ir::ObjectNode { kind: Kind::DistanceAxis(a, _, 1), .. }, .. }),
                             Some(Object { node: sketchpad_ir::ObjectNode { kind: Kind::DistanceAxis(b, _, 0), .. }, .. })) if a == b
                        )
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { node: sketchpad_ir::ObjectNode { kind: Kind::DistanceAxis(origin, _, 1), .. }, .. }),
                             Some(Object { node: sketchpad_ir::ObjectNode { kind: Kind::GraphAxis(y_origin, unit, 0), .. }, .. }))
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
                            && objects
                                .get(coordinates.saturating_sub(1))
                                .is_some_and(|o| o.kind.is_coordinate_system()) =>
                    {
                        Kind::SlopeMeasurement(line, coordinates)
                    }
                    _ => Kind::Unsupported(code),
                }
            } else if let Some(caption) = super::referenced_captions::decode(
                &objects,
                page,
                objects.len() + 1,
                h,
                &refs,
                semantic_group,
                text_style_count,
            ) {
                caption
            } else if let Some(alias) = super::iteration_endpoint::decode(
                &objects,
                page,
                objects.len() + 1,
                h,
                &refs,
                constraint_payload
                    .as_ref()
                    .map(|(payload, _)| payload.as_slice()),
            ) {
                alias
            } else if let Some(definition) = super::coordinate_points::decode(
                &objects,
                page,
                objects.len() + 1,
                h,
                &refs,
                constraint_payload
                    .as_ref()
                    .map(|(payload, _)| payload.as_slice()),
            ) {
                Kind::CoordinatePoint(definition)
            } else if let Some(definition) = super::colorized::decode_colorized(
                &objects,
                page,
                objects.len() + 1,
                h,
                &refs,
                constraint_payload
                    .as_ref()
                    .map(|(payload, _)| payload.as_slice()),
                !tool_inputs.is_empty(),
            )
            .filter(|definition| {
                if definition.element() == super::transform_model::ElementType::Polygon
                    && h[6..10] == [4, 0, 15, 0]
                {
                    let group = rs
                        .iter()
                        .filter(|r| r.start >= group_start && r.start <= rs[i].start)
                        .map(|r| (r.kind, r.data.len()))
                        .collect::<Vec<_>>();
                    matches!(
                        group.as_slice(),
                        [(0x07d0, 28), (0x07d2, 12 | 20), (0x07d3, 24), (0x07d7, 0)]
                            | [
                                (0x07d0, 28),
                                (0x07d2, 12 | 20),
                                (0x07d3, 24),
                                (0x232e, 0),
                                (0x07d7, 0)
                            ]
                    )
                } else {
                    true
                }
            }) {
                Kind::Colorized(definition)
            } else if let Some(definition) = super::mapped_element::decode_mapped_element(
                &objects,
                page,
                objects.len() + 1,
                h,
                &refs,
                constraint_payload
                    .as_ref()
                    .map(|(payload, _)| payload.as_slice()),
                !tool_inputs.is_empty(),
            ) {
                Kind::MappedElement(definition)
            } else if let Some(definition) = super::transform_decode::decode_native_transform(
                &objects,
                page,
                objects.len() + 1,
                &transform_header,
                &refs,
                constraint_payload
                    .as_ref()
                    .map(|(payload, _)| payload.as_slice()),
                super::transform_decode::DecodeContext {
                    custom_polar_translation: code == 26
                        && matches!(h[6..10], [1, 0, 4, 0] | [2, 0, 14, 0])
                        && matches!(
                            rs.iter()
                                .filter(|record| {
                                    record.start >= group_start && record.start <= rs[i].start
                                })
                                .map(|record| record.kind)
                                .collect::<Vec<_>>()
                                .as_slice(),
                            [0x07d0, 0x07d2, 0x07d7] | [0x07d0, 0x07d2, 0x07d5, 0x07d8, 0x07d7]
                        ),
                    source_is_generic_polygon: refs
                        .first()
                        .and_then(|source| objects.get(source.saturating_sub(1)))
                        .and_then(|source| source.header_offset)
                        .is_some_and(|at| {
                            b.get(at + 6..at + 10) == Some(&[4, 0, 15, 0])
                                || polymorphic_polygon_transform
                                    && b.get(at + 6..at + 10) == Some(&[1, 0, 15, 0])
                        }),
                    legacy_angle_rotation_layout: code == 119
                        && matches!(
                            rs.iter()
                                .filter(|record| {
                                    record.start >= group_start && record.start <= rs[i].start
                                })
                                .map(|record| (record.kind, record.data.len()))
                                .collect::<Vec<_>>()
                                .as_slice(),
                            [(0x07d0, 28), (0x07d2, 16), (0x07d7, 0)]
                                | [
                                    (0x07d0, 28),
                                    (0x07d2, 16),
                                    (0x07d5, 28),
                                    (0x07d8, 2),
                                    (0x07d7, 0)
                                ]
                        ),
                    polymorphic_preimage: polymorphic_preimage_transform,
                    polymorphic_polygon: polymorphic_polygon_transform,
                },
            ) {
                if definition.element() == super::transform_model::ElementType::Image {
                    let source = definition.source();
                    let source_object = objects.get(source.saturating_sub(1));
                    let source_png =
                        transformed_image_png(&objects, page, source, objects.len() + 1);
                    let source_crop_cache = source_object
                        .filter(|object| {
                            matches!(
                                object.kind,
                                Kind::CroppedImage { .. } | Kind::TransformedCroppedImage(..)
                            )
                        })
                        .and_then(|object| object.header_offset)
                        .and_then(|header| {
                            let start = rs.iter().position(|record| {
                                record.kind == 0x07d0 && record.data.start == header
                            })?;
                            let group = rs[start..]
                                .iter()
                                .take_while(|record| record.kind != 0x07d7)
                                .collect::<Vec<_>>();
                            (group.iter().filter(|record| record.kind == 0x090c).count() == 1).then(
                                || {
                                    let record =
                                        group.iter().find(|record| record.kind == 0x090c).unwrap();
                                    &b[record.data.clone()]
                                },
                            )
                        });
                    let group = rs
                        .iter()
                        .filter(|record| record.start >= group_start && record.start <= rs[i].start)
                        .collect::<Vec<_>>();
                    let cache = group
                        .iter()
                        .find(|record| record.kind == 0x090c)
                        .filter(|_| {
                            group.iter().filter(|record| record.kind == 0x090c).count() == 1
                                && group.iter().filter(|record| record.kind == 0x07d2).count() == 1
                                && group.iter().filter(|record| record.kind == 0x07d3).count()
                                    == usize::from(constraint_payload.is_some())
                                && group.iter().all(|record| {
                                    matches!(
                                        record.kind,
                                        0x07d0 | 0x07d2 | 0x07d3 | 0x090c | 0x07d7
                                    ) || (record.kind == 0x232e && record.data.is_empty())
                                })
                        })
                        .map(|record| &b[record.data.clone()]);
                    match (source_object.map(|object| &object.kind), source_png, cache) {
                        (Some(Kind::Image { .. }), Some(png), Some(cache)) if cache.len() == 16 => {
                            let dimensions = ImageFile::png_size(png).ok();
                            let cached = (
                                u32_at(cache, 0),
                                u32_at(cache, 4),
                                u32_at(cache, 8),
                                u32_at(cache, 12),
                            );
                            if dimensions.is_some_and(|(width, height)| {
                                cached == (width, height, width, height)
                            }) {
                                Kind::Image {
                                    png: png.clone(),
                                    placement: ImagePlacement::Transformed(definition),
                                }
                            } else {
                                Kind::Unsupported(code)
                            }
                        }
                        (
                            Some(Kind::CroppedImage { .. } | Kind::TransformedCroppedImage(..)),
                            Some(_),
                            Some(cache),
                        ) if cache.len() == 16 && source_crop_cache == Some(cache) => {
                            Kind::TransformedCroppedImage(definition)
                        }
                        _ => Kind::Unsupported(code),
                    }
                } else {
                    Kind::Transformed(definition)
                }
            } else if matches!(code, 16 | 21 | 27 | 29 | 30 | 33 | 34)
                && h.len() == 28
                && h[6..8] == [2, 0]
                && matches!(u16::from_le_bytes([h[8], h[9]]), 12 | 13)
            {
                // A known ray/line output that failed the shared schema must
                // not fall through to a legacy point/circle construction.
                Kind::Unsupported(code)
            } else {
                match (code, point, refs.as_slice()) {
                    (89, None, [parameter, seed, first, second, carried_second, carried_first, polygon, segments @ ..])
                        if h[6..10] == [14,0,64,0] && second == carried_second && first == carried_first
                            && objects.get(parameter.saturating_sub(1)).is_some_and(|o| o.kind.is_iteration_depth_value())
                            && supported_branch_polygon_mapping(&objects,*seed,*first,*second,*polygon,segments)
                            && iteration_definition.as_ref().is_some_and(|d| iteration_definition_matches(d, 1, 2)) => {
                            Kind::BranchingPolygonIteration { seed:*seed, first:*first, second:*second, polygon:*polygon, segments:segments.to_vec(), depth:IterationDepth::Value(*parameter) }
                        }
                    (76, None, [seed, first, second, carried_second, carried_first, polygon, segments @ ..])
                        if h[6..10] == [14,0,64,0] && second == carried_second && first == carried_first
                            && supported_branch_polygon_mapping(&objects,*seed,*first,*second,*polygon,segments)
                            && iteration_definition.as_ref().is_some_and(|d| iteration_definition_matches(d, 1, 2) && (1..=128).contains(&u32_at(d,16))) => {
                            Kind::BranchingPolygonIteration { seed:*seed, first:*first, second:*second, polygon:*polygon, segments:segments.to_vec(), depth:IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(),16)) }
                        }
                    (76,None,[seed,parameter,image,calculation,carried_value,carried_point,label])
                        if h[6..10] == [14,0,64,0] && calculation==carried_value && image==carried_point
                            && supported_point_parameter_iteration(&objects,*seed,*parameter,*image,*calculation,*label)
                            && iteration_definition.as_ref().is_some_and(|d|iteration_definition_matches(d, 2, 1) && (1..=128).contains(&u32_at(d,16))) => {
                            Kind::PointParameterIteration(*seed,*parameter,*image,*calculation,*label,IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(),16)))
                        }
                    (89,None,[depth_parameter,seed,parameter,image,calculation,carried_value,carried_point,label])
                        if h[6..10] == [14,0,64,0] && calculation==carried_value && image==carried_point
                            && *depth_parameter != *parameter
                            && objects.get(depth_parameter.saturating_sub(1)).is_some_and(|o| !o.deleted && o.kind.is_iteration_depth_value())
                            && supported_point_parameter_iteration(&objects,*seed,*parameter,*image,*calculation,*label)
                            && iteration_definition.as_ref().is_some_and(|d|iteration_definition_matches(d, 2, 1)) => {
                            Kind::PointParameterIteration(*seed,*parameter,*image,*calculation,*label,IterationDepth::Value(*depth_parameter))
                        }
                    (76 | 89, None, _) if mapped_iteration.as_ref().is_some_and(|kind| matches!(kind, Kind::MappedPointIteration { seeds,.. } if seeds.len() > 1)) => mapped_iteration.unwrap(),
                    (92,None,[iteration,values @ ..]) if iteration_values_group
                        && supported_iteration_values(
                            &objects,
                            page,
                            objects.len() + 1,
                            *iteration,
                            values,
                        ) => Kind::IterationValues(*iteration,values.to_vec()),
                    (90,None,_) if super::point_numeric_labels::decode_numeric_label(
                        &objects,page,objects.len()+1,h,&refs,semantic_group,text_style_count).is_some() =>
                        super::point_numeric_labels::decode_numeric_label(
                            &objects,page,objects.len()+1,h,&refs,semantic_group,text_style_count).unwrap(),
                    (90,None,_) if super::point_numeric_labels::decode_referenced_label(
                        &objects,page,objects.len()+1,h,&refs,semantic_group,text_style_count).is_some() =>
                        super::point_numeric_labels::decode_referenced_label(
                            &objects,page,objects.len()+1,h,&refs,semantic_group,text_style_count).unwrap(),
                    (89, None, [parameter, seed, image, carried, ray])
                        if *parameter > 0 && *seed > 0 && *image > 0 && *ray > 0
                            && h[6..10] == [14, 0, 64, 0]
                            && image == carried
                            && iteration_definition.as_ref().is_some_and(|definition| iteration_definition_matches(definition, 1, 1))
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
                            && iteration_definition.as_ref().is_some_and(|d|iteration_definition_matches(d, 1, 1)) => Kind::SegmentIteration(*seed,*image,*segment,IterationDepth::Value(*depth)),
                    (89,None,[depth,seed,image,carried,circle])
                        if h[6..10]==[14,0,64,0] && image==carried
                            && objects.get(depth.saturating_sub(1)).is_some_and(|o|o.kind.is_iteration_depth_value())
                            && objects.get(seed.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(circle.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Circle(a,b) if a==*seed&&b==*image))
                            && iteration_definition.as_ref().is_some_and(|d|iteration_definition_matches(d, 1, 1)) => Kind::CircleIteration(*seed,*image,*circle,IterationDepth::Value(*depth)),
                    (76,None,[seed,image,carried,circle])
                        if h[6..10]==[14,0,64,0] && image==carried
                            && objects.get(seed.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
                            && supported_point_iteration_mapping(&objects,*seed,*image)
                            && objects.get(circle.saturating_sub(1)).is_some_and(|o|matches!(o.kind,Kind::Circle(a,b) if a==*seed&&b==*image))
                            && iteration_definition.as_ref().is_some_and(|d|iteration_definition_matches(d, 1, 1) && (1..=128).contains(&u32_at(d,16))) => Kind::CircleIteration(*seed,*image,*circle,IterationDepth::Fixed(u32_at(iteration_definition.as_ref().unwrap(),16))),
                    (76, None, [seed, image, carried, segment])
                        if h[6..10] == [14, 0, 64, 0]
                            && image == carried
                            && iteration_definition.as_ref().is_some_and(|definition| {
                                iteration_definition_matches(definition, 1, 1)
                                    && (1..=128).contains(&u32_at(definition, 16))
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
                            && iteration_definition.as_ref().is_some_and(|d|iteration_definition_matches(d, 1, 1)) => Kind::PointIteration(*seed,*image,IterationDepth::Value(*depth)),
                    (76, None, [seed, image, carried])
                        if h[6..10] == [14, 0, 64, 0]
                            && image == carried
                            && iteration_definition.as_ref().is_some_and(|definition| {
                                iteration_definition_matches(definition, 1, 1)
                                    && (1..=128).contains(&u32_at(definition, 16))
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
                    (102, None, _)
                        if h.len() == 28 && h[6..10] == [6, 0, 90, 0]
                            && constraint_payload.is_none()
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x07d2).count() == 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x0902).count() == 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start).all(|r| matches!(r.kind, 0x07d0 | 0x07d2 | 0x0902 | 0x07d5 | 0x07d8 | 0x07d7) || (r.kind == 0x232e && r.data.is_empty())) => {
                        locus_range.as_ref()
                            .and_then(|range| super::sampled_mapped::decode_sampled_mapped(
                                &objects, page, objects.len() + 1, h, &refs, range,
                                constraint_payload.as_ref().map(|(payload, _)| payload.as_slice()),
                            ))
                            .map(Kind::SampledMapped)
                            .unwrap_or(Kind::Unsupported(code))
                    }
                    (96, None, _)
                        if h.len() == 28
                            && h[6..10] == [6, 0, 91, 0]
                            && constraint_payload.is_some()
                            && locus_range.is_some()
                            && {
                                let group = rs
                                    .iter()
                                    .filter(|record| {
                                        record.start >= group_start && record.start <= rs[i].start
                                    })
                                    .map(|record| (record.kind, record.data.len()))
                                    .collect::<Vec<_>>();
                                group
                                    == [
                                        (0x07d0, 28),
                                        (0x07d2, 4 + refs.len() * 4),
                                        (0x07d3, 12),
                                        (0x0902, 40),
                                        (0x07d7, 0),
                                    ]
                            } => super::sampled_image::decode_sampled_image(
                                &objects,
                                page,
                                objects.len() + 1,
                                h,
                                &refs,
                                &constraint_payload.as_ref().unwrap().0,
                                locus_range.as_ref().unwrap(),
                            )
                            .map(Kind::SampledImage)
                            .unwrap_or(Kind::Unsupported(code)),
                    (97, None, _)
                        if refs.first().is_some_and(|target| {
                            super::kind::parameter_locus_subtype(&objects, *target)
                                .is_some_and(|subtype| h[6..10] == [6, 0, subtype as u8, 0])
                        })
                            && parameter_locus_refs_match(&objects, &refs)
                            && locus_range.as_ref().is_some_and(|data| data.len() == 40
                                && f64_at(data, 0).is_finite() && f64_at(data, 8).is_finite()
                                && f64_at(data, 0) < f64_at(data, 8)
                                && u32_at(data, 16) >= 2
                                && u32_at(data, 20) == 0x00030004
                                && f64_at(data, 24).is_finite()
                                && refs.get(1).and_then(|driver| {
                                    super::kind::parameter_locus_driver_profile(&objects, *driver)
                                }).is_some_and(|profile| profile.matches_tag(u32_at(data, 32)))
                                && u32_at(data, 36) == 0) => {
                        let data = locus_range.as_ref().unwrap();
                        Kind::ParameterLocus(refs.clone(), f64_at(data, 0), f64_at(data, 8), u32_at(data, 16))
                    }
                    (35, None, _)
                        if refs.first().and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| {
                            if is_locus_straight_kind(&o.kind) || o.kind.is_circle() || is_arc_kind(&o.kind) || is_locus_polygon_kind(&o.kind) || refs.first().is_some_and(|id| super::kind::is_locus_curve_collection(&objects, *id)) {
                                h[6..10] == [6, 0, 43, 0]
                            } else {
                                h[6..10] == [6, 0, 42, 0]
                            }
                        })
                            && super::kind::locus_refs_match(&objects, &refs)
                            && locus_range.as_ref().is_some_and(|data| {
                                data.len() == 40
                                    && u32_at(data, 16) >= 2
                                    && u32_at(data, 20) & 0xffff == 4
                                    && refs.get(1).zip(refs.get(2)).is_some_and(|(path, driver)| super::locus::locus_driver_range_matches(
                                        &objects, *driver, *path, f64_at(data, 0), f64_at(data, 8), u32_at(data, 20) >> 16))
                                    // Both native point and geometric sweeps retain either
                                    // observed presentation flag. The output header and the
                                    // validated target dependency define the element family.
                                    && data[36..] == [0; 4]
                                    && u32_at(data, 32) <= 1
                            }) => Kind::Locus(refs.clone(), u32_at(locus_range.as_ref().unwrap(), 16)),
                    (123, _, [path])
                        if literal_axis_value
                            .is_some_and(|(local_path, _)| local_path as usize + page_base == *path)
                            && objects.get(path - 1).is_some_and(|o| {
                                o.page == page
                                    && (is_literal_plot_path_kind(&o.kind)
                                        || super::curve_points::literal_supported(
                                            &objects,
                                            *path,
                                            page,
                                            objects.len() + 1,
                                        )
                                        || !tool_inputs.is_empty()
                                            && matches!(
                                                o.kind,
                                                Kind::ToolInput(EmbeddedToolInputKind::Path)
                                            ))
                            }) => {
                        Kind::PathLiteralPoint(*path, literal_axis_value.unwrap().1)
                    }
                    (95, _, [source, path])
                        if h[6..10] == [1, 0, 56, 0]
                            && axis_value_refs == Some([(*source - page_base) as u32, (*path - page_base) as u32])
                            && objects.get(source - 1).is_some_and(|o| {
                                o.page == page && o.kind.numeric_unit().is_some()
                            })
                            && objects.get(path - 1).is_some_and(|o| o.page == page && (is_value_plot_path_kind(&o.kind) || super::curve_points::supported(&objects, *path, page, objects.len() + 1))) => {
                        Kind::PathValuePoint(*source, *path)
                    }
                    (code @ (69 | 70), None, [x, y, frame])
                        if h[6..10] == [1, 0, 56, 0]
                            && [x, y].iter().all(|&&id| {
                                objects.get(id.saturating_sub(1)).is_some_and(|o| {
                                    o.page == page
                                        && (o.kind.numeric_unit().is_some()
                                            || !tool_inputs.is_empty()
                                                && matches!(
                                                    o.kind,
                                                    Kind::ToolInput(
                                                        EmbeddedToolInputKind::Numeric
                                                    )
                                                ))
                                })
                            })
                            && objects.get(frame.saturating_sub(1)).is_some_and(|o| o.page == page && o.kind.is_coordinate_system()) => {
                        Kind::NumericCoordinatePoint(*x, *y, *frame, if code == 70 { PlotPointsMode::Polar } else { PlotPointsMode::Cartesian })
                    }
                    (112, None, [value, frame])
                        if h[6..10] == [1, 0, 56, 0]
                            && objects.get(value.saturating_sub(1)).is_some_and(|o| {
                                o.page == page && o.kind.numeric_unit().is_some()
                            })
                            && objects.get(frame.saturating_sub(1)).is_some_and(|o| {
                                o.page == page && o.kind.is_coordinate_system()
                            })
                            && {
                                let group = rs.iter().filter(|record| {
                                    record.start >= group_start && record.start <= rs[i].start
                                }).collect::<Vec<_>>();
                                group.iter().map(|record| record.kind).collect::<Vec<_>>()
                                    == [0x07d0, 0x07d2, 0x07d3, 0x07d7]
                                    && group[2].data.len() == 12
                                    && b[group[2].data.clone()]
                                        == [112, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                            } => Kind::GraphValuePoint(*value, *frame),
                    (67 | 68, _, [frame])
                        if plotted_point.as_ref().is_some_and(|plot| {
                            plot.coordinate_system == (*frame - page_base) as u32
                        })
                        && objects.get(frame.saturating_sub(1)).is_some_and(|o| o.page == page && o.kind.is_coordinate_system()) => {
                        let plot = plotted_point.unwrap();
                        Kind::PlottedPoint(*frame, plot.mode, plot.stored)
                    }
                    (94, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::PointValue && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && (super::measurements::point_value_refs_match(&objects, *first, *second, page, objects.len() + 1)
                            || !tool_inputs.is_empty()
                                && objects.get(first.saturating_sub(1)).is_some_and(|object| {
                                    !object.deleted && object.page == page && is_point_kind(&object.kind)
                                })
                                && objects.get(second.saturating_sub(1)).is_some_and(|object| {
                                    !object.deleted
                                        && object.page == page
                                        && matches!(object.kind, Kind::ToolInput(EmbeddedToolInputKind::Path))
                                })) => {
                        Kind::ValueEquation { kind: ValueEquationKind::PointValue, first: *first, second: *second }
                    }
                    (50, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::LineEquation && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first.saturating_sub(1)).is_some_and(|o| o.kind.is_line())
                        && objects.get(second - 1).is_some_and(|o| o.kind.is_coordinate_system()) => {
                        Kind::ValueEquation { kind: ValueEquationKind::LineEquation, first: *first, second: *second }
                    }
                    (51, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::CircleEquation && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first.saturating_sub(1)).is_some_and(|o| o.kind.is_circle())
                        && objects.get(second - 1).is_some_and(|o| o.kind.is_coordinate_system()) => {
                        Kind::ValueEquation { kind: ValueEquationKind::CircleEquation, first: *first, second: *second }
                    }
                    (49 | 65 | 66 | 124 | 125, _, [first, frame]) if coordinate_group.as_ref().is_some_and(|g| {
                        g.references == vec![(*first - page_base) as u32, (*frame - page_base) as u32]
                    }) && objects.get(first - 1).is_some_and(|o| is_point_kind(&o.kind))
                        && objects.get(frame - 1).is_some_and(|o| o.kind.is_coordinate_system()) => {
                        Kind::CoordinateMeasurement { kind: coordinate_group.as_ref().unwrap().kind, first: *first, second: None, frame: *frame }
                    }
                    (86, _, [first, second, frame]) if coordinate_group.as_ref().is_some_and(|g| {
                        g.references == vec![(*first - page_base) as u32, (*second - page_base) as u32, (*frame - page_base) as u32]
                    }) && [first, second].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind)))
                        && objects.get(frame - 1).is_some_and(|o| o.kind.is_coordinate_system()) => {
                        Kind::CoordinateMeasurement { kind: CoordinateMeasureKind::CoordinateDistance, first: *first, second: Some(*second), frame: *frame }
                    }
                    (0, None, []) if is_image => {
                        if let Some(picture) = imported_images.get(&group_start) {
                            Kind::Image { png: picture.png.clone(), placement: ImagePlacement::Independent(picture.transform) }
                        } else {
                            let picture = image_file.as_ref().unwrap();
                            Kind::Image { png: picture.png().to_vec(), placement: ImagePlacement::Independent(picture.transform()) }
                        }
                    }
                    (99, None, [anchor]) if h[6..10] == [16, 0, 78, 0]
                        && objects.get(anchor.saturating_sub(1)).is_some_and(|object| object.page == page && is_point_kind(&object.kind))
                        && attached_images.get(&group_start).is_some_and(|image| matches!(image.attachment, ReadOnlyImageAttachment::Point { point_ref, .. } if point_ref == (*anchor - page_base) as u32)) => {
                        let image = &attached_images[&group_start];
                        let width = image.dimensions.0 as f64;
                        let height = image.dimensions.1 as f64;
                        let ReadOnlyImageAttachment::Point { scale_x, scale_y, rotation, .. } = image.attachment else { unreachable!() };
                        let (sin, cos) = rotation.sin_cos();
                        let a = scale_x * cos;
                        let b = scale_y * sin;
                        let c = -scale_x * sin;
                        let d = scale_y * cos;
                        Kind::Image {
                            png: image.png.clone(),
                            placement: ImagePlacement::Point {
                                anchor: *anchor,
                                relative: [
                                    a, b, -a * width / 2. - b * height / 2.,
                                    c, d, -c * width / 2. - d * height / 2.,
                                ],
                            },
                        }
                    }
                    (85, None, [bottom_right, top_left]) if h[6..10] == [16, 0, 78, 0]
                        && [bottom_right, top_left].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|object| object.page == page && is_point_kind(&object.kind)))
                        && attached_images.get(&group_start).is_some_and(|image| matches!(image.attachment, ReadOnlyImageAttachment::Rectangle { bottom_right: stored_bottom_right, top_left: stored_top_left } if stored_bottom_right == (*bottom_right - page_base) as u32 && stored_top_left == (*top_left - page_base) as u32)) => {
                        Kind::Image {
                            png: attached_images[&group_start].png.clone(),
                            placement: ImagePlacement::Rectangle { bottom_right: *bottom_right, top_left: *top_left },
                        }
                    }
                    (100, None, [top_left, bottom_right, bottom_left]) if h[6..10] == [16, 0, 78, 0]
                        && [top_left, bottom_right, bottom_left].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|object| object.page == page && is_point_kind(&object.kind)))
                        && attached_images.get(&group_start).is_some_and(|image| matches!(image.attachment, ReadOnlyImageAttachment::Parallelogram { top_left: stored_top_left, bottom_right: stored_bottom_right, bottom_left: stored_bottom_left } if stored_top_left == (*top_left - page_base) as u32 && stored_bottom_right == (*bottom_right - page_base) as u32 && stored_bottom_left == (*bottom_left - page_base) as u32)) => {
                        Kind::Image {
                            png: attached_images[&group_start].png.clone(),
                            placement: ImagePlacement::Parallelogram { top_left: *top_left, bottom_right: *bottom_right, bottom_left: *bottom_left },
                        }
                    }
                    (108, None, [source, polygon])
                        if cropped_images.get(&group_start).is_some_and(|crop| {
                            crop.source as usize + page_base == *source
                                && crop.polygon as usize + page_base == *polygon
                                && crop.visible == (raw & 0x1_0000 == 0)
                        })
                        && objects.get(source.saturating_sub(1)).is_some_and(|object| {
                            object.page == page
                                && matches!(&object.kind, Kind::Image { png, .. }
                                    if ImageFile::png_size(png).ok() == Some(cropped_images[&group_start].dimensions))
                        })
                        && objects.get(polygon.saturating_sub(1)).is_some_and(|object| {
                            object.page == page && object.kind.is_polygon()
                        }) => {
                        Kind::CroppedImage { source: *source, polygon: *polygon }
                    }
                    (122, None, [source, frame]) => {
                        super::data_function::decode_data_function(
                            &objects,
                            page,
                            h,
                            &[*source, *frame],
                            semantic_group,
                        )
                        .unwrap_or(Kind::Unsupported(code))
                    }
                    (0, None, []) if function_expression.is_some() && rich_text.is_none() && text_box.is_none() => {
                        let group = semantic_group;
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
                            Ok(expression) if super::functions::function_sources_valid(&expression, sources, &objects, page)
                                && sources.iter().all(|&id| id > 0 && id <= objects.len()) => Kind::ReferencedFunction(sources.to_vec(), expression),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (48, None, [first, second]) if *first > 0 && *second > 0
                        && objects.get(first - 1).is_some_and(|o| o.kind.is_distance_value())
                        && objects.get(second - 1).is_some_and(|o| o.kind.is_distance_value()) => {
                        match LengthPairCalculationGroup::parse(semantic_group) {
                            Ok(group) if group.sources() == [(*first-page_base) as u32, (*second-page_base) as u32] => Kind::LengthPairCalculation(*first, *second, group),
                            _ => parse_numeric_calculation(semantic_group, &[*first,*second], &objects, page_base).unwrap_or(Kind::Unsupported(code)),
                        }
                    }
                    (48, None, sources) if sources.len() >= 2 => parse_numeric_calculation(semantic_group, sources, &objects, page_base).unwrap_or(Kind::Unsupported(code)),
                    (48, None, [source]) if *source > 0
                        && objects.get(source - 1).is_some_and(|object| object.kind.is_distance_value()) => {
                        match MeasuredCalculationGroup::parse(semantic_group) {
                            Ok(group) if group.source_id() as usize == source - page_base => Kind::MeasuredCalculation(*source, group),
                            _ => parse_numeric_calculation(semantic_group, &[*source], &objects, page_base).unwrap_or(Kind::Unsupported(code)),
                        }
                    }
                    (48, None, [source]) if *source > 0
                        && objects.get(source - 1).is_some_and(|object| object.kind.numeric_unit().is_some()
                            || matches!(object.kind, Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _))) => {
                        parse_numeric_calculation(semantic_group, &[*source], &objects, page_base).unwrap_or(Kind::Unsupported(code))
                    }
                    (91, None, [source]) if source.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o| matches!(o.kind, Kind::Parameter(_))) => {
                        match TableGroup::parse(semantic_group) {
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
                        match TableGroup::parse_pair(semantic_group) {
                            Ok(table) if table.pair_source_ids() == Some([
                                (*first - page_base) as u32,
                                (*second - page_base) as u32,
                            ]) => Kind::ValueTablePair(*first, *second, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (91, None, sources) if sources.len() >= 2
                        && sources.iter().all(|source| objects.get(source - 1).is_some_and(|object| {
                            object.page == page
                                && !object.deleted
                                && (object.kind.numeric_unit().is_some()
                                    || matches!(object.kind, Kind::ReferencedText { .. }))
                        })) => {
                        match TableGroup::parse_general(semantic_group) {
                            Ok(table) if table.general_source_ids().as_deref()
                                == Some(&sources.iter().map(|source| (*source - page_base) as u32).collect::<Vec<_>>()) => {
                                Kind::DataTable(sources.to_vec(), table)
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (72, None, [definition, frame]) if function_range.is_some()
                        && matches!(objects.get(definition - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                        && objects.get(frame - 1).is_some_and(|o| o.kind.is_coordinate_system()) => {
                        match function_records::decode_plot_range(function_range.as_ref().unwrap()) {
                            Ok((min,max,samples,mode)) => Kind::FunctionPlot(*definition,*frame,min,max,samples,mode),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (78, None, sources) if !sources.is_empty() && function_expression.is_some()
                        && matches!(sources.first().and_then(|source| objects.get(source - 1)).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _))) => {
                        match function_records::decode_expression_with_sources(function_expression.as_ref().unwrap(), sources.len()) {
                            Ok(expression) if super::functions::function_sources_valid(&expression, sources, &objects, page) => Kind::DerivativeFunction(sources.to_vec(), expression),
                            Err(_) => Kind::Unsupported(code),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (126 | 127, None, [x, y, frame]) if function_range.is_some()
                        && *x > 0 && *y > 0 && *frame > 0 && x != y
                        && matches!(objects.get(x - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(y - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))
                        && objects.get(frame - 1).is_some_and(|o| o.kind.is_coordinate_system()) => {
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
                            && group.settings_list().is_ok_and(|settings| settings.iter().zip(targets).all(|(settings, target)| {
                                objects.get(target.saturating_sub(1)).is_some_and(|object| {
                                    (matches!(object.kind,
                                        Kind::ToolInput(_)
                                            | Kind::ToolPointOnPath(_)
                                            | Kind::Free(_)
                                            | Kind::Parameter(_)
                                            | Kind::PointOnSegment(_, _)
                                            | Kind::PointOnRay(_, _)
                                            | Kind::PointOnLine(_, _)
                                            | Kind::PointOnCircle(_, _, _)
                                            | Kind::PointOnArc(_, _)
                                            | Kind::PointOnPolygon(_, _, _)
                                            | Kind::PointOnCurve(_, _)
                                            | Kind::PointOnSector(_, _)
                                            | Kind::PointOnCircularSegment(_, _)
                                    ) || object.kind.is_point() && object.kind.transform_definition().is_some())
                                    && (matches!(object.kind, Kind::ToolInput(_) | Kind::ToolPointOnPath(_))
                                        || matches!(object.kind, Kind::Parameter(_)) == settings.parameter.is_some())
                                })
                            }))
                    }) => {
                        let group = animation_group.as_ref().unwrap();
                        match group.label_text() {
                            Ok(label) => Kind::AnimationButton {
                                targets: targets.to_vec(),
                                label,
                                anchor: group.screen_anchor(),
                                settings: group.settings_list().unwrap(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, functions) if sound_group.as_ref().is_some_and(|group| {
                        group.target_ids() == functions.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                            && functions.iter().all(|&function| objects.get(function.saturating_sub(1)).is_some_and(|object| object.kind.is_function()))
                    }) => {
                        let group = sound_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::SoundButton {
                                functions: functions.to_vec(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
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
                                options: group.options(),
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
                        && text_style => {
                        let force_legacy_gbk = legacy_gbk_text
                            && h == text_object::LEGACY_TEXT_HEADER;
                        match text_object::decode_rich_text_with_legacy_gbk(
                            rich_text.as_ref().unwrap(),
                            text_box.as_ref().unwrap(),
                            force_legacy_gbk,
                        ) {
                            Ok((text, rect, runs)) if runs.iter().flatten().all(|run| run.style.family_index.is_none_or(|index| usize::from(index) < font_family_count)) => {
                                if force_legacy_gbk {
                                    rich_text_override = text_object::RichTextTemplate::from_parts_with_legacy_gbk(
                                        rich_text.as_ref().unwrap(),
                                        text_box.as_ref().unwrap(),
                                        true,
                                    ).ok();
                                }
                                Kind::PlainText { text, rect }
                            },
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
                    (47, _, [first, second])
                        if measurement_format && measurement_style
                            && [first, second].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_segment_kind(&o.kind))) =>
                    {
                        Kind::SegmentRatio(*first, *second)
                    }
                    (39, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon() || o.kind.is_sector() || o.kind.is_circular_segment()) =>
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
                    (113..=116, None, [a, vertex, c])
                        if angle_marker_label
                            && constraint_payload.is_none()
                            && [a, vertex, c].iter().all(|&&id| id.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o| !o.deleted && o.page == page && is_point_kind(&o.kind)))
                            && {
                                let group = rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind != 0x232e)
                                    .map(|r| (r.kind, r.data.len())).collect::<Vec<_>>();
                                group == [(0x07d0, 28), (0x07d2, 16), (0x090e, 12), (0x07d7, 0)]
                                    || (group.len() == 6
                                        && group[0..3] == [(0x07d0, 28), (0x07d2, 16), (0x090e, 12)]
                                        && group[3].0 == 0x07d5 && label.is_some()
                                        && group[4..] == [(0x07d8, 2), (0x07d7, 0)])
                            } => {
                        let marker = rs.iter().find(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x090e).unwrap();
                        match code {
                            114 => Kind::SignedAngleValue(*a, *vertex, *c),
                            115 => Kind::PolarAngleValue(*a, *vertex, *c),
                            116 => Kind::VertexAngleValue(*a, *vertex, *c),
                            _ => Kind::AngleMarker(*a, *vertex, *c, u16::from_le_bytes(b[marker.data.start..marker.data.start + 2].try_into().unwrap())),
                        }
                    }
                    (120, None, [source])
                        if h.len() == 28 && h[6..10] == [7, 0, 49, 0]
                            && measurement_format && measurement_style
                            && source.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o|
                                !o.deleted && o.page == page
                                    && matches!(o.kind, Kind::SignedAngleValue(..) | Kind::PolarAngleValue(..) | Kind::VertexAngleValue(..) | Kind::AngleMarker(..)))
                            && {
                                let group = rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start)
                                    .map(|r| (r.kind, r.data.len())).collect::<Vec<_>>();
                                group == [(0x07d0, 28), (0x07d2, 8), (0x0903, 32), (0x07d8, 2), (0x07d7, 0)]
                                    || (group.len() == 6
                                        && group[0..3] == [(0x07d0, 28), (0x07d2, 8), (0x0903, 32)]
                                        && group[3].0 == 0x07d5 && matches!(group[3].1, 26 | 28 | 30 | 40)
                                        && label.is_some()
                                        && group[4..] == [(0x07d8, 2), (0x07d7, 0)])
                            } => Kind::NumericAlias { source: *source, unit: NumericUnit::Degrees },
                    (42, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| o.kind.is_circle()) =>
                    {
                        Kind::CircleArea(*host)
                    }
                    (42, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon() || o.kind.is_sector() || o.kind.is_circular_segment()) =>
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
                            && objects.get(host - 1).is_some_and(|o| is_arc_kind(&o.kind) || o.kind.is_sector()) =>
                    {
                        Kind::ArcLength(*host)
                    }
                    (44, _, [circle, start, end])
                        if measurement_format && measurement_style
                            && objects.get(circle.saturating_sub(1)).is_some_and(|o| o.kind.is_circle())
                            && [start, end].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::CircleArcLength(*circle, *start, *end)
                    }
                    (46, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| {
                                o.kind.is_circle() || o.kind.is_arc() || o.kind.is_sector()
                            }) =>
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
                                || object.kind.is_polygon()
                                || super::curve_points::free_supported(
                                    &objects,
                                    *host,
                                    page,
                                    objects.len() + 1,
                                )) =>
                    {
                        Kind::ToolPointOnPath(*host)
                    }
                    (15, None, [host]) if point_curve_constraint_count == 1
                        && h.len() == 28 && h[6..10] == [1, 0, 10, 0]
                        && constraint_payload.as_ref().is_some_and(|(payload, _)| payload.len() == 12 && payload[..4] == [15, 0, 68, 0] && f64_at(payload, 4).is_finite())
                        && super::sector_boundary::supported_host(&objects, *host, page, objects.len() + 1)
                        && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x07d2).count() == 1
                        && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start).all(|r| match r.kind {
                            0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => true,
                            0x07d2 => r.data.len() == 8,
                            0x07d3 => r.data.len() == 12,
                            _ => false,
                        }) => {
                        Kind::PointOnSector(*host, f64_at(&constraint_payload.as_ref().unwrap().0, 4))
                    }
                    (15, None, [host]) if point_curve_constraint_count == 1
                        && h.len() == 28 && h[6..10] == [1, 0, 10, 0]
                        && constraint_payload.as_ref().is_some_and(|(payload, _)| {
                            payload.len() == 12
                                && payload[..4] == [15, 0, 69, 0]
                                && (0.0..=1.0).contains(&f64_at(payload, 4))
                        })
                        && super::sector_boundary::supported_circular_segment_host(&objects, *host, page, objects.len() + 1)
                        && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x07d2).count() == 1
                        && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start).all(|r| match r.kind {
                            0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => true,
                            0x07d2 => r.data.len() == 8,
                            0x07d3 => r.data.len() == 12,
                            _ => false,
                        }) => {
                        Kind::PointOnCircularSegment(*host, f64_at(&constraint_payload.as_ref().unwrap().0, 4))
                    }
                    (15, None, [host]) if point_curve_constraint_count == 1
                        && constraint_payload.as_ref().is_some_and(|(payload, _)| payload.len() == 12 && matches!(payload.get(..4), Some([15, 0, 42 | 58 | 90 | 99, 0]))) =>
                        super::curve_points::decode(&objects, *host, page, objects.len() + 1, h, &constraint_payload.as_ref().unwrap().0).unwrap_or(Kind::Unsupported(code)),
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
                                    && super::kind::straight_point_constraint_matches(kind, payload)
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
                                    && h[6..10] == [1, 0, 10, 0]
                                    && point_curve_constraint_count == 1
                                    && payload.len() == 20
                                    && polygon_vertex_count(&objects, *host).and_then(|count| polygon_constraint_subtype(&objects, b, *host, count)).is_some_and(|subtype| payload[..4] == [15, 0, subtype as u8, (subtype >> 8) as u8])
                                    && (payload[..4] != [15, 0, 15, 0]
                                        || polygon_vertex_count(&objects, *host) == Some(9)
                                        || {
                                            let exact_layout = |expected: &[u32]| {
                                                rs.iter()
                                                    .filter(|record| record.start >= group_start && record.start <= rs[i].start)
                                                    .map(|record| record.kind)
                                                    .eq(expected.iter().copied())
                                            };
                                            exact_layout(&[0x07d0, 0x07d2, 0x07d3, 0x07d7])
                                                || exact_layout(&[
                                                    0x07d0, 0x07d2, 0x07d3, 0x232e, 0x07d7,
                                                ])
                                                || exact_layout(&[0x07d0, 0x07d2, 0x07d3, 0x07d5, 0x07d8, 0x07d7])
                                                || exact_layout(&[
                                                    0x07d0, 0x07d2, 0x07d3, 0x07d5, 0x07d8,
                                                    0x232e, 0x07d7,
                                                ])
                                        })
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
                                && cartesian_distance_scale(payload, 4, 12, 20).is_some()
                                && cartesian_distance_scale(payload, 24, 32, 40).is_some()
                                && *source > 0
                                && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind) || matches!(o.kind, Kind::Segment(_, _) | Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::Polygon(_))) =>
                        {
                            let horizontal_scale =
                                cartesian_distance_scale(payload, 4, 12, 20).unwrap();
                            let vertical_scale =
                                cartesian_distance_scale(payload, 24, 32, 40).unwrap();
                            if h[8..10] == [14, 0] && matches!(objects[source - 1].kind, Kind::Segment(_, _)) {
                                Kind::FixedRectangularTranslatedSegment(*source, horizontal_scale, f64_at(payload, 12), vertical_scale, f64_at(payload, 32))
                            } else if h[8..10] == [28, 0] && matches!(objects[source - 1].kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) {
                                Kind::FixedRectangularTranslatedCircle(*source, horizontal_scale, f64_at(payload, 12), vertical_scale, f64_at(payload, 32))
                            } else if h[8..10] == [4, 0] && is_point_kind(&objects[source - 1].kind) {
                                Kind::FixedRectangularTranslatedPoint(*source, horizontal_scale, f64_at(payload, 12), vertical_scale, f64_at(payload, 32))
                            } else if matches!(&objects[source - 1].kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]])))
                                && h.len() == 28 && h[6..8] == [4, 0] {
                                Kind::FixedRectangularTranslatedPolygon(*source, horizontal_scale, f64_at(payload, 12), vertical_scale, f64_at(payload, 32))
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
                            // Native copies the mapped point identity onto its vector image.
                            // Read the actual source header so arbitrary point kinds cannot
                            // acquire an unexplained application subtype.
                            (kind, [89, 0]) if is_point_kind(kind)
                                && h.len() == 28 && h[6..8] == [1, 0]
                                && constraint_payload.is_none()
                                && [source, start, end].iter().all(|&&id| id < objects.len() + 1 && objects[id - 1].page == page && !objects[id - 1].deleted)
                                && objects[source - 1].header_offset.is_some_and(|at| b.get(at + 6..at + 10) == Some(&[1, 0, 89, 0])) => Kind::VectorTranslatedPoint(*source, *start, *end),
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
                            && objects.get(angle.saturating_sub(1)).is_some_and(|o| {
                                o.kind.numeric_unit() == Some(NumericUnit::Scalar)
                                    || !tool_inputs.is_empty()
                                        && matches!(
                                            o.kind,
                                            Kind::ToolInput(EmbeddedToolInputKind::Numeric)
                                        )
                            }) => {
                        match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                            (Some(kind), [4, 0]) if is_point_kind(kind) && h[6..8] == [1, 0] => Kind::CalculatedRatioScaledPoint(*source, *center, *angle),
                            // Native preserves the stored mapped point identity through
                            // numeric dilation, including vector-transformed mapped points.
                            (Some(kind), [89, 0]) if is_point_kind(kind)
                                && h.len() == 28 && h[6..8] == [1, 0]
                                && constraint_payload.is_none()
                                && [source, center, angle].iter().all(|&&id| id > 0 && id < objects.len() + 1 && objects[id - 1].page == page && !objects[id - 1].deleted)
                                && objects[source - 1].header_offset.is_some_and(|at| b.get(at + 6..at + 10) == Some(&[1, 0, 89, 0])) => Kind::CalculatedRatioScaledPoint(*source, *center, *angle),
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
                    (11, None, [a, b])
                        if !tool_inputs.is_empty()
                            && h.len() == 28
                            && h[6..10] == [1, 0, 45, 0]
                            && point.is_none()
                            && constraint_payload.is_none()
                            && objects
                                .get(a.saturating_sub(1))
                                .is_some_and(|object| is_straight_kind(&object.kind))
                            && objects.get(b.saturating_sub(1)).is_some_and(|object| {
                                matches!(
                                    object.kind,
                                    Kind::ToolInput(EmbeddedToolInputKind::Path)
                                )
                            }) =>
                    {
                        Kind::LineCircleIntersection(*a, *b, 1)
                    }
                    (98, None, _) if curve_constraint_count == 1 => super::curve_intersection::decode(&objects, page, objects.len() + 1, h, &refs, constraint_payload.as_ref().map(|(p, _)| p.as_slice())).unwrap_or(Kind::Unsupported(code)),
                    (93, None, _) => super::boundary_intersection::decode(&objects, page, objects.len() + 1, h, &refs, constraint_payload.as_ref().map(|(p, _)| p.as_slice())).unwrap_or(Kind::Unsupported(code)),
                    (11, _, [a, b])
                        if objects
                            .get(a.saturating_sub(1))
                            .is_some_and(|object| is_straight_kind(&object.kind))
                            && objects.get(b.saturating_sub(1)).is_some_and(|object| {
                                object.kind.is_circle()
                                    || (is_arc_kind(&object.kind)
                                        && object.page == page
                                        && objects[a - 1].page == page
                                        && h.len() == 28
                                        && h[6..10] == [1, 0, 45, 0]
                                        && point.is_none()
                                        && constraint_payload.is_none())
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
                                    || (is_arc_kind(&object.kind)
                                        && object.page == page
                                        && objects[a - 1].page == page
                                        && h.len() == 28
                                        && h[6..10] == [1, 0, 45, 0]
                                        && point.is_none()
                                        && constraint_payload.is_none())
                            }) =>
                    {
                        Kind::LineCircleIntersection(*a, *b, 0)
                    }
                    (13 | 14, _, [a, b]) => {
                        let arc_parent = [a, b].iter().any(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| is_arc_kind(&o.kind)));
                        if !arc_parent || (h.len() == 28 && h[6..10] == [1, 0, 45, 0] && point.is_none() && constraint_payload.is_none()
                            && [a, b].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| !o.deleted && o.page == page && (o.kind.is_circle() || is_arc_kind(&o.kind))))) {
                            Kind::CircleCircleIntersection(*a, *b, if code == 13 { 1 } else { 0 })
                        } else { Kind::Unsupported(code) }
                    },
                    (63, _, [a, b]) => Kind::Line(*a, *b),
                    (64, _, [a, b]) => Kind::Ray(*a, *b),
                    (80, None, [center, start, end])
                        if h.len() == 28 && h[6..10] == [12, 0, 67, 0]
                            && constraint_payload.is_none()
                            && [center, start, end].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind))) => {
                        Kind::CenterArc(*center, *start, *end)
                    }
                    (81, _, [a, b, c]) => Kind::ThreePointArc(*a, *b, *c),
                    (10, None, [circle])
                        if matches!(h.len(), 16 | 28) && h[6..10] == [5, 0, 16, 0]
                            && constraint_payload.is_none()
                            && super::circle_interior::supported_source(&objects, *circle, page, objects.len() + 1)
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x07d2).count() == 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x232e).count() <= 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start).all(|r| match r.kind {
                                0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => true,
                                0x07d2 => r.data.len() == 8,
                                0x232e => r.data.is_empty(),
                                _ => false,
                            }) => Kind::CircleInterior(*circle),
                    (82, None, [arc])
                        if h.len() == 28 && h[6..10] == [13, 0, 68, 0]
                            && constraint_payload.is_none()
                            && super::sector_boundary::supported_source(&objects, *arc, page, objects.len() + 1)
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x07d2).count() == 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x232e).count() <= 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start).all(|r| match r.kind {
                                0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => true,
                                0x07d2 => r.data.len() == 8,
                                0x232e => r.data.is_empty(),
                                _ => false,
                            }) => Kind::SectorBoundary(*arc),
                    (83, None, [arc])
                        if h.len() == 28 && h[6..10] == [13, 0, 69, 0]
                            && constraint_payload.is_none()
                            && super::sector_boundary::supported_circular_segment_source(&objects, *arc, page, objects.len() + 1)
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x07d2).count() == 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start && r.kind == 0x232e).count() <= 1
                            && rs.iter().filter(|r| r.start >= group_start && r.start <= rs[i].start).all(|r| match r.kind {
                                0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => true,
                                0x07d2 => r.data.len() == 8,
                                0x232e => r.data.is_empty(),
                                _ => false,
                            }) => Kind::CircularSegmentBoundary(*arc),
                    (79, None, [circle, start, end])
                        if h.len() == 28 && h[6..10] == [12, 0, 67, 0]
                            && constraint_payload.is_none()
                            && rs.iter().filter(|record| record.start >= group_start && record.start <= rs[i].start)
                                .filter(|record| record.kind == 0x07d2).count() == 1
                            && rs.iter().filter(|record| record.start >= group_start && record.start <= rs[i].start)
                                .all(|record| match record.kind {
                                    0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => true,
                                    0x07d2 => record.data.len() == 16,
                                    0x232e => record.data.is_empty(),
                                    _ => false,
                                })
                            && objects.get(circle - 1).is_some_and(|o| !o.deleted && o.page == page && o.kind.is_circle())
                            && [start, end].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| !o.deleted && o.page == page && is_point_kind(&o.kind))) =>
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
                Kind::FunctionDefinition(_)
                    | Kind::ReferencedFunction(_, _)
                    | Kind::DataFunction { .. }
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
                    | Kind::DataFunction { .. }
                    | Kind::Parameter(_)
                    | Kind::LinkUrlButton { .. }
                    | Kind::LinkPageButton { .. }
            ) {
                label = None;
            }
            if label.is_some()
                && !is_point_kind(&kind)
                && !is_straight_kind(&kind)
                && !kind.is_circle()
                && !is_arc_kind(&kind)
                && kind.numeric_unit().is_none()
                && !matches!(
                    kind,
                    Kind::ValueEquation { .. }
                        | Kind::CoordinateMeasurement { .. }
                        | Kind::IterationValues(..)
                        | Kind::PlainText { .. }
                        | Kind::ReferencedText { .. }
                        | Kind::PointNumericLabel(..)
                        | Kind::PointReferencedText { .. }
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
                    | Kind::DataFunction { .. }
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
                        || matches!(o.kind, Kind::CircleInterior(_))
                        || matches!(&o.kind, Kind::Colorized(definition) if definition.element() == super::transform_model::ElementType::Disk)
                        || is_arc_kind(&o.kind)
                        || super::locus::is_mapped_stroke_locus(&objects, &o.kind)
                        || super::locus::is_point_locus(&objects, &o.kind)
                        || super::locus::is_mapped_polygon_locus(&objects, &o.kind)
                        || matches!(o.kind, Kind::SampledMapped(..))
                        || matches!(o.kind, Kind::FunctionPlot(..))
                })
            {
                style_records::iteration_binding_visual_style(
                    h,
                    objects.get(source.saturating_sub(1)).is_some_and(|o| {
                        o.kind.is_polygon() || matches!(o.kind, Kind::CircleInterior(_))
                            || matches!(&o.kind, Kind::Colorized(definition) if definition.element() == super::transform_model::ElementType::Disk)
                    }),
                )
            } else if super::kind::is_straight_locus(&objects, &kind) {
                style_records::locus_straight_visual_style(h)
            } else {
                style_records::visual_style(h)
            };
            let visible =
                raw & if matches!(kind, Kind::PathValuePoint(..)) {
                    0x5_0000
                } else {
                    0x1_0000
                } == 0;
            let angle_marker_style =
                super::AngleMarkerStyle::for_kind(&kind).and(angle_marker_style);
            let iteration_final_only = matches!(
                kind,
                Kind::PointParameterIteration(..)
                    | Kind::BranchingPolygonIteration { .. }
                    | Kind::MappedPointIteration { .. }
                    | Kind::PointIteration(..)
                    | Kind::SegmentIteration(..)
                    | Kind::CircleIteration(..)
                    | Kind::RayIteration(..)
            ) && iteration_definition
                .as_ref()
                .is_some_and(|definition| definition.get(4..6) == Some(&[1, 0]));
            objects.push(Object {
                midpoint_host: matches!(kind, Kind::Midpoint(_, _)).then(|| refs[0]),
                locus_interval: (matches!(kind, Kind::Locus(..))
                    && matches!(
                        objects[refs[2] - 1].kind,
                        Kind::PointOnRay(..) | Kind::PointOnLine(..)
                    ))
                .then(|| {
                    let data = locus_range.as_ref().unwrap();
                    [f64_at(data, 0), f64_at(data, 8)]
                }),
                node: sketchpad_ir::ObjectNode::new(objects.len() + 1, page, kind),
                visible,
                arrow_selectable,
                presentation_suppressed: raw & 0x20000 != 0,
                iteration_final_only,
                tracing: h[4] & 1 != 0,
                deleted: false,
                point_offset,
                constraint_offset: constraint_payload.map(|(_, offset)| offset),
                header_offset: Some(header_offset),
                label,
                visual_style,
                angle_marker_style,
                interior_order: (h.len() == 28
                    && (h[6..8] == [4, 0]
                        || matches!(h[6..10], [13, 0, 68, 0] | [13, 0, 69, 0] | [5, 0, 16, 0])))
                .then(|| i32::from_le_bytes(h[20..24].try_into().unwrap())),
                text_style_override: None,
                rich_text_override,
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
        let mut document = Self {
            objects,
            path,
            selected: BTreeSet::new(),
            dirty: false,
            partial_view: !safe || image_read_only,
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
            object_color_defaults: Default::default(),
            object_style_defaults: Default::default(),
            measurement_precision_defaults: Default::default(),
            new_polygon_opacity: 32767,
            randomize_interior_colors: false,
            interior_color_cursor: super::initial_interior_color_cursor(),
            merge_aliases: Default::default(),
            measurement_names: Default::default(),
            evaluation_context: None,
            original: b.to_vec(),
            run_font_families: Vec::new(),
            source_count,
            last_moved_object: None,
            safe_structure: safe,
            image_read_only,
            undo: Vec::new(),
            redo: Vec::new(),
        };
        document.refresh_measurement_names();
        Ok(document)
    }
}
