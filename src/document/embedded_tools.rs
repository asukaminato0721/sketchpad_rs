use super::records::{put_record, u32_at};
use super::{
    Document, EmbeddedToolInput, EmbeddedToolInputKind, Kind, LineMode, Object, Record, records,
};
use std::ops::Range;

/// An embedded tool has its own canvas and object-reference namespace.
/// Its prototype records are retained separately from the drawing objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedToolSummary {
    pub name: Option<String>,
    pub object_codes: Vec<u16>,
    /// Ordered local object IDs required when invoking the tool.
    /// None means the input declaration is missing or malformed.
    pub input_ids: Option<Vec<u32>>,
    pub complete: bool,
}

pub(super) fn document_record_indices(records: &[Record]) -> Vec<usize> {
    let mut depth = 0usize;
    records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| match record.kind {
            0x03ea => {
                depth += 1;
                None
            }
            0x03eb if depth > 0 => {
                depth -= 1;
                None
            }
            _ => (depth == 0).then_some(index),
        })
        .collect()
}

pub(super) fn document_records(records: &[Record]) -> impl Iterator<Item = &Record> {
    document_record_indices(records)
        .into_iter()
        .map(|index| &records[index])
}

fn tool_ranges(records: &[Record]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, record) in records.iter().enumerate() {
        match record.kind {
            0x03ea => {
                if depth == 0 {
                    start = index;
                }
                depth += 1;
            }
            0x03eb if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    ranges.push(start..index + 1);
                }
            }
            _ => {}
        }
    }
    if depth > 0 {
        ranges.push(start..records.len());
    }
    ranges
}

impl Document {
    /// Return given categories in binding order, without guessing their type
    /// from a placeholder's cached coordinates or its decoded drawing kind.
    pub fn embedded_tool_inputs(&self, index: usize) -> Result<Vec<EmbeddedToolInput>, String> {
        let summaries = self.embedded_tools()?;
        let summary = summaries.get(index).ok_or("Unknown embedded tool")?;
        let ids = summary
            .input_ids
            .as_ref()
            .ok_or("Invalid embedded tool input declaration")?;
        let entries = records(&self.original)?;
        let range = tool_ranges(&entries)
            .get(index)
            .cloned()
            .ok_or("Unknown embedded tool")?;
        let headers: Vec<_> = entries[range]
            .iter()
            .filter(|record| record.kind == 0x07d0)
            .collect();
        ids.iter()
            .map(|&id| {
                let header = headers.get(id as usize - 1).ok_or("Invalid tool input")?;
                let payload = &self.original[header.data.clone()];
                if payload.len() != 28 || payload[..2] != [0, 0] {
                    return Err("Unsupported embedded tool given header".into());
                }
                Ok(EmbeddedToolInput {
                    local_id: id as usize,
                    native_class: u16::from_le_bytes(payload[6..8].try_into().unwrap()),
                    native_subtype: u16::from_le_bytes(payload[8..10].try_into().unwrap()),
                })
            })
            .collect()
    }
    /// Inspect preserved tool prototypes. Tool execution and editing are not
    /// implied by recognizing their canvas boundaries.
    pub fn embedded_tools(&self) -> Result<Vec<EmbeddedToolSummary>, String> {
        let entries = records(&self.original)?;
        Ok(tool_ranges(&entries)
            .into_iter()
            .map(|range| {
                let section = &entries[range];
                let name = section
                    .iter()
                    .find(|record| record.kind == 0x044c)
                    .map(|record| {
                        let payload = &self.original[record.data.clone()];
                        let end = payload
                            .iter()
                            .position(|byte| *byte == 0)
                            .unwrap_or(payload.len());
                        String::from_utf8_lossy(&payload[..end]).into_owned()
                    });
                let object_codes: Vec<u16> = section
                    .iter()
                    .filter(|record| record.kind == 0x07d0)
                    .filter_map(|record| {
                        self.original
                            .get(record.data.clone())
                            .and_then(|payload| payload.get(..2))
                            .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
                    })
                    .collect();
                let mut declarations = section.iter().filter(|r| r.kind == 0x04b4);
                let input_ids = declarations.next().and_then(|record| {
                    if declarations.next().is_some() {
                        return None;
                    }
                    let payload = &self.original[record.data.clone()];
                    let count = u32::from_le_bytes(payload.get(..4)?.try_into().ok()?) as usize;
                    if count.checked_mul(4)?.checked_add(4)? != payload.len() {
                        return None;
                    }
                    let ids: Vec<u32> = payload[4..]
                        .chunks_exact(4)
                        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
                        .collect();
                    let unique: std::collections::BTreeSet<_> = ids.iter().copied().collect();
                    (unique.len() == ids.len()
                        && ids
                            .iter()
                            .all(|&id| id > 0 && id as usize <= object_codes.len()))
                    .then_some(ids)
                });
                EmbeddedToolSummary {
                    name,
                    object_codes,
                    input_ids,
                    complete: section.last().is_some_and(|record| record.kind == 0x03eb),
                }
            })
            .collect())
    }

    /// Decode one prototype in its own object and font namespaces. Declared
    /// free-point inputs are placeholders, not stored drawing coordinates.
    pub fn embedded_tool_document(&self, index: usize) -> Result<Document, String> {
        let summaries = self.embedded_tools()?;
        let summary = summaries.get(index).ok_or("Unknown embedded tool")?;
        if !summary.complete {
            return Err("Embedded tool canvas is incomplete".into());
        }
        let inputs = summary
            .input_ids
            .as_ref()
            .ok_or("Invalid embedded tool input declaration")?;
        let entries = records(&self.original)?;
        let range = tool_ranges(&entries)
            .get(index)
            .cloned()
            .ok_or("Unknown embedded tool")?;
        let section = &entries[range];
        if section[0].data.len() != 36
            || u32_at(&self.original[section[0].data.clone()], 24) as usize
                != summary.object_codes.len()
            || !section
                .last()
                .is_some_and(|record| record.kind == 0x03eb && record.data.is_empty())
        {
            return Err("Invalid embedded tool canvas framing".into());
        }
        if section
            .iter()
            .filter(|record| record.kind == 0x03ea)
            .count()
            != 1
        {
            return Err("Nested tool canvases are unsupported".into());
        }
        let mut body = Vec::new();
        let mut fonts = Vec::new();
        let mut local_id = 0usize;
        let mut point_input = false;
        let mut saw_point = false;
        for record in &section[1..section.len() - 1] {
            let payload = &self.original[record.data.clone()];
            match record.kind {
                0x044c | 0x04b4 | 0x232f => continue,
                0x04b1 | 0x04b3 => {
                    put_record(&mut fonts, record.kind, payload);
                    continue;
                }
                0x07d0 => {
                    local_id += 1;
                    point_input = inputs.contains(&(local_id as u32))
                        && payload.len() == 28
                        && payload[..2] == [0, 0]
                        && payload[6..10] == [1, 0, 4, 0];
                    saw_point = false;
                }
                0x0899 if point_input => {
                    if saw_point || payload.len() != 16 {
                        return Err("Invalid tool point placeholder".into());
                    }
                    put_record(&mut body, 0x0899, &[0; 16]);
                    saw_point = true;
                    continue;
                }
                0x07d7 if point_input && !saw_point => {
                    put_record(&mut body, 0x0899, &[0; 16]);
                }
                _ => {}
            }
            put_record(&mut body, record.kind, payload);
        }
        if local_id != summary.object_codes.len() || fonts.is_empty() {
            return Err("Invalid embedded tool canvas".into());
        }
        let blank = Document::new();
        let baseline = records(&blank.original)?;
        let mut bytes = b"GSP4".to_vec();
        let mut inserted_fonts = false;
        for record in baseline {
            let payload = &blank.original[record.data.clone()];
            match record.kind {
                0x04b1 | 0x04b3 => {
                    if !inserted_fonts {
                        bytes.extend_from_slice(&fonts);
                        inserted_fonts = true;
                    }
                }
                0x03e8 => {
                    let mut preamble = payload.to_vec();
                    preamble[24..28].copy_from_slice(&(local_id as u32).to_le_bytes());
                    put_record(&mut bytes, record.kind, &preamble);
                }
                0x03e9 => {
                    bytes.extend_from_slice(&body);
                    put_record(&mut bytes, record.kind, payload);
                }
                _ => put_record(&mut bytes, record.kind, payload),
            }
        }
        let given_types = self
            .embedded_tool_inputs(index)?
            .into_iter()
            .map(|input| (input.local_id, input))
            .collect();
        let document = Document::from_bytes_with_tool_inputs(&bytes, None, &given_types)?;
        if document.objects.len() != local_id {
            return Err("Tool object count mismatch".into());
        }
        Ok(document)
    }

    /// Bind ordered prototype inputs and materialize its dependent graph.
    /// Invocation is committed only after the result saves and reparses exactly.
    pub fn invoke_embedded_tool(
        &mut self,
        index: usize,
        inputs: &[usize],
    ) -> Result<Vec<usize>, String> {
        let tool = self.embedded_tool_document(index)?;
        let input_types = self.embedded_tool_inputs(index)?;
        self.invoke_tool_prototype(tool, input_types, inputs)
    }

    /// Invoke an immutable tool definition from another document. Only the
    /// materialized construction becomes part of this drawing; the source
    /// document and its definition namespace remain independent.
    pub fn invoke_tool_from_document(
        &mut self,
        source: &Document,
        index: usize,
        inputs: &[usize],
    ) -> Result<Vec<usize>, String> {
        let tool = source.embedded_tool_document(index)?;
        let input_types = source.embedded_tool_inputs(index)?;
        self.invoke_tool_prototype(tool, input_types, inputs)
    }

    fn invoke_tool_prototype(
        &mut self,
        tool: Document,
        input_types: Vec<EmbeddedToolInput>,
        inputs: &[usize],
    ) -> Result<Vec<usize>, String> {
        let local_inputs: Vec<_> = input_types.iter()
            .map(|input| input.local_id as u32).collect();
        if inputs.len() != local_inputs.len() || inputs.is_empty() {
            return Err("Select the tool's ordered inputs".into());
        }
        if tool.partial_view || !tool.safe_structure {
            return Err("Tool prototype contains unsupported records or constructions".into());
        }
        if tool.objects.iter().any(|object| {
            matches!(object.kind, Kind::Free(_)) && !local_inputs.contains(&(object.id as u32))
        }) {
            return Err("Tool has an undeclared free-point placeholder".into());
        }
        let mut ids = vec![0; tool.objects.len() + 1];
        for ((&local, &destination), expected_type) in
            local_inputs.iter().zip(inputs).zip(input_types)
        {
            let expected = tool
                .objects
                .get(local as usize - 1)
                .ok_or("Invalid tool input")?;
            let actual = destination
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .filter(|object| !object.deleted && object.page == self.active_page)
                .ok_or("Tool input is not on the active page")?;
            match expected_type.kind() {
                Some(EmbeddedToolInputKind::Point)
                    if matches!(expected.kind, Kind::Free(_))
                        && super::kind::is_point_kind(&actual.kind)
                        && self.position(destination).is_some() => {}
                Some(EmbeddedToolInputKind::Path)
                    if matches!(expected.kind, Kind::ToolInput(EmbeddedToolInputKind::Path))
                        && (self.line_points(destination).is_some()
                            || self.circle_points(destination).is_some()
                            || self.arc_points(destination, 32).is_some()
                            || self.polygon_points(destination).is_some()) => {}
                Some(EmbeddedToolInputKind::Point) => {
                    return Err("This tool input requires an available point".into());
                }
                Some(EmbeddedToolInputKind::Path) => {
                    return Err("This tool input requires an available path".into());
                }
                Some(EmbeddedToolInputKind::Segment)
                    if matches!(
                        expected.kind,
                        Kind::ToolInput(EmbeddedToolInputKind::Segment)
                    ) && self
                        .line_points(destination)
                        .is_some_and(|(_, _, mode)| mode == LineMode::Segment) => {}
                Some(EmbeddedToolInputKind::Segment) => {
                    return Err("This tool input requires an available finite segment".into());
                }
                Some(EmbeddedToolInputKind::Numeric)
                    if matches!(
                        expected.kind,
                        Kind::ToolInput(EmbeddedToolInputKind::Numeric)
                    ) && self.numeric_value(destination).is_some_and(f64::is_finite) => {}
                Some(EmbeddedToolInputKind::Numeric) => {
                    return Err("This tool input requires an available numeric value".into());
                }
                None => return Err("This tool input category is unsupported".into()),
            }
            ids[local as usize] = destination;
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        candidate.selected.clear();
        let mut created = Vec::new();
        for object in &tool.objects {
            if ids[object.id] != 0 {
                continue;
            }
            if matches!(object.kind, Kind::Unsupported(_))
                || tool
                    .dependencies_of(object.id)
                    .ok_or("Invalid tool object")?
                    .iter()
                    .any(|&id| ids.get(id).copied().unwrap_or(0) == 0)
            {
                return Err("Tool has an unresolved or unsupported dependency".into());
            }
            let caption_style = if matches!(object.kind, Kind::ParameterCalculation(..)) {
                // Native uses its calculation default, independently of both
                // script fonts and the destination's last caption font.
                Some(crate::text_style_records::FontState {
                    family: "Tahoma".into(),
                    size_code: 18,
                    bold: false,
                    italic: false,
                    underline: false,
                })
            } else {
                None
            };
            let id = candidate.objects.len() + 1;
            let mut materialized: Object = object.clone();
            materialized.id = id;
            materialized.page = candidate.active_page;
            materialized.kind = match object.kind.remap_checked(&ids)? {
                Kind::ParameterCalculation(_, mut group) => {
                    group.set_anchor(candidate.next_tool_numeric_anchor()?)?;
                    let font = candidate.imported_caption_font_index(caption_style.as_ref().unwrap())?;
                    let group = group.with_caption_font(font)?;
                    let sources = group.sources();
                    let units = sources
                        .iter()
                        .map(|&id| {
                            candidate
                                .objects
                                .get(id as usize - 1)
                                .and_then(|object| object.kind.numeric_unit())
                                .ok_or("Tool calculation requires numeric sources")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let group = group.specialize_tool_sources(&sources, &units)?;
                    super::calculations::parse_numeric_calculation(
                        group.as_bytes(),
                        &sources.iter().map(|&id| id as usize).collect::<Vec<_>>(),
                        &candidate.objects,
                        0,
                    ).ok_or("Tool calculation cannot be materialized")?
                }
                Kind::SegmentMidpoint(host) => match candidate.objects[host - 1].kind {
                    Kind::Segment(a, b) => Kind::Midpoint(a, b),
                    _ => Kind::SegmentMidpoint(host),
                },
                Kind::ToolPointOnPath(host) => {
                    let fraction = tool_point_fraction();
                    match candidate.line_points(host).map(|(_, _, mode)| mode) {
                        Some(LineMode::Segment) => Kind::PointOnSegment(host, fraction),
                        Some(LineMode::Ray) => Kind::PointOnRay(host, fraction * 2.),
                        Some(LineMode::Line) => Kind::PointOnLine(host, fraction * 3. - 1.),
                        None if candidate.circle_points(host).is_some() => {
                            let angle = fraction * std::f64::consts::TAU;
                            Kind::PointOnCircle(host, angle.cos(), angle.sin())
                        }
                        None if candidate.arc_points(host, 32).is_some() => {
                            Kind::PointOnArc(host, fraction)
                        }
                        None if candidate.polygon_points(host).is_some() => {
                            let vertices = candidate.polygon_points(host).unwrap();
                            let parameter = fraction * vertices.len() as f64;
                            Kind::PointOnPolygon(
                                host,
                                parameter.floor() as usize,
                                parameter.fract(),
                            )
                        }
                        _ => return Err("Tool point requires a supported path".into()),
                    }
                }
                kind => kind,
            };
            materialized.midpoint_host = if let Kind::SegmentMidpoint(host) = object.kind {
                matches!(materialized.kind, Kind::Midpoint(..)).then_some(ids[host])
            } else {
                object.midpoint_host.map(|host| ids[host])
            };
            materialized.point_offset = None;
            materialized.constraint_offset = None;
            materialized.header_offset = None;
            materialized.text_style_override = caption_style;
            materialized.rich_text_override = tool.text_rich_template(object.id);
            candidate.objects.push(materialized);
            if let Some(style) = tool.frame_styles.get(&object.id) {
                candidate.frame_styles.insert(id, *style);
            }
            ids[object.id] = id;
            created.push(id);
            if object.visible && !object.presentation_suppressed {
                candidate.selected.insert(id);
            }
        }
        if created.is_empty() {
            return Err("Tool has no constructed outputs".into());
        }
        let encoded = candidate.encode()?;
        let reparsed = Document::from_bytes(&encoded, None)?;
        if candidate.has_pending_multi_page_additions() {
            let mut survivors: Vec<_> = candidate
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            survivors
                .sort_by_key(|object| (object.page, object.id > candidate.source_count, object.id));
            candidate.validate_reparsed_in_order(&reparsed, survivors, true)?;
        } else {
            candidate.validate_reparsed(&reparsed)?;
        }
        *self = candidate;
        Ok(created)
    }
}

// A tool's unspecified point placement is chosen once when it is invoked.
// The resulting parameter is stored in the ordinary constrained-point kind,
// so save, undo/redo, and later host edits retain that choice.
fn tool_point_fraction() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos() as u64);
    let mut value = time.wrapping_add(SEQUENCE.fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((value ^ (value >> 31)) >> 11) as f64 / (1_u64 << 53) as f64
}
