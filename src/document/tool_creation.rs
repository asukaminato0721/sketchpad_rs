use super::records::{header, put_record, records};
use super::{Document, EmbeddedToolInputKind, Kind};
use crate::table_records::ParameterCalculationGroup;
use std::collections::BTreeSet;

struct NumericToolGraph {
    inputs: Vec<usize>,
    calculations: Vec<usize>,
}

impl Document {
    /// Create a document tool from the selected construction and its prerequisites.
    /// Selected independent inputs become ordered givens; path boundaries exclude
    /// their unselected geometry prerequisites.
    pub fn create_embedded_tool(&mut self, name: &str) -> Result<usize, String> {
        let name = name.trim();
        if name.is_empty() || name.contains('\0') || name.chars().any(char::is_control) {
            return Err("Enter a tool name without control characters".into());
        }
        if self.page_count != 1 || !self.safe_structure {
            return Err("Tool creation requires a supported drawing".into());
        }
        if self.selected.len() < 2 {
            return Err("Select initial objects and constructed results to define a tool".into());
        }
        let numeric_boundary = self.numeric_tool_boundary()?;
        let (prototype, prototype_bytes, path_groups, input_count, object_count, mapping) =
            if let Some(graph) = numeric_boundary.as_ref() {
                let bytes = self.encode()?;
                let groups = numeric_template_groups(self, graph)?;
                (
                    self.clone(),
                    bytes,
                    Some(groups),
                    graph.inputs.len(),
                    graph.inputs.len() + graph.calculations.len(),
                    Vec::new(),
                )
            } else {
                let path_boundary = self.path_tool_boundary()?;
                let mut clipboard = self.copy_selected()?;
                if path_boundary.is_none()
                    && clipboard.objects.iter().any(|object| {
                        matches!(object.kind, Kind::Free(_))
                            && !self.selected.contains(&object.old_id)
                    })
                {
                    return Err(
                        "Select all independent point inputs and the constructed results".into(),
                    );
                }
                let input_count = clipboard
                    .objects
                    .iter()
                    .filter(|object| matches!(object.kind, Kind::Free(_)))
                    .count();
                if input_count == 0 || input_count == clipboard.len() {
                    return Err(
                "Select a construction with independent point inputs and at least one result"
                    .into(),
            );
                }
                // Native stores given point roots in reverse construction order, then
                // declares their binding order in 04b4 independently of object order.
                clipboard.objects.sort_by(|a, b| {
                    match (
                        matches!(a.kind, Kind::Free(_)),
                        matches!(b.kind, Kind::Free(_)),
                    ) {
                        (true, true) => b.old_id.cmp(&a.old_id),
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        (false, false) => a.old_id.cmp(&b.old_id),
                    }
                });
                let max_id = clipboard
                    .objects
                    .iter()
                    .map(|object| object.old_id)
                    .max()
                    .unwrap();
                let mut mapping = vec![0; max_id + 1];
                for (index, object) in clipboard.objects.iter().enumerate() {
                    mapping[object.old_id] = index + 1;
                }
                for object in &mut clipboard.objects {
                    object.kind = object.kind.remap_checked(&mapping)?;
                    object.midpoint_host = object.midpoint_host.map(|id| mapping[id]);
                    object.old_id = mapping[object.old_id];
                }
                clipboard.hide_on_paste = clipboard
                    .hide_on_paste
                    .iter()
                    .map(|id| mapping[*id])
                    .collect();
                clipboard.rename_on_paste = clipboard.rename_on_paste.map(|id| mapping[id]);
                let mut prototype = Document::new();
                prototype.paste(&clipboard)?;
                let prototype_bytes = prototype.encode()?;
                let path_groups = path_boundary
                    .as_ref()
                    .map(|(host, input_kind, outputs)| {
                        let given = mapping[*host];
                        let outputs: Vec<_> = outputs.iter().map(|id| mapping[*id]).collect();
                        path_template_groups(
                            &prototype_bytes,
                            &prototype,
                            given,
                            *input_kind,
                            &outputs,
                        )
                    })
                    .transpose()?;
                let input_count = if path_groups.is_some() {
                    1
                } else {
                    input_count
                };
                let object_count = path_boundary
                    .as_ref()
                    .map_or(prototype.objects.len(), |(_, _, outputs)| outputs.len() + 1);
                (
                    prototype,
                    prototype_bytes,
                    path_groups,
                    input_count,
                    object_count,
                    mapping,
                )
            };
        let prototype_records = records(&prototype_bytes)?;
        let mut section = Vec::new();
        // Native's tool compatibility block is separate from page view metadata.
        put_record(
            &mut section,
            0x0386,
            include_bytes!("../../assets/native_tool_compatibility.bin"),
        );
        let mut preamble = [0u8; 36];
        preamble[16..20].copy_from_slice(&[1, 0, 1, 0]);
        let count = u32::try_from(object_count).map_err(|_| "Too many tool objects")?;
        preamble[24..28].copy_from_slice(&count.to_le_bytes());
        put_record(&mut section, 0x03ea, &preamble);
        let mut title = name.as_bytes().to_vec();
        title.push(0);
        // Native tool names include their even-byte padding in the payload.
        if title.len() % 2 != 0 {
            title.push(0);
        }
        put_record(&mut section, 0x044c, &title);
        if numeric_boundary.is_some() {
            section.extend_from_slice(include_bytes!("../../assets/native_numeric_tool_fonts.bin"));
        } else {
            for record in prototype_records
                .iter()
                .filter(|record| matches!(record.kind, 0x04b1 | 0x04b3))
            {
                put_record(
                    &mut section,
                    record.kind,
                    &prototype_bytes[record.data.clone()],
                );
            }
        }
        let inputs: BTreeSet<_> = (1..=input_count).collect();
        let selected_local: BTreeSet<_> = self
            .selected
            .iter()
            .filter_map(|id| mapping.get(*id).copied())
            .collect();
        if let Some(groups) = path_groups {
            section.extend_from_slice(&groups);
        } else {
            let mut local_id = 0;
            let mut in_group = false;
            for record in &prototype_records {
                if record.kind == 0x07d0 {
                    local_id += 1;
                    in_group = true;
                }
                if !in_group {
                    continue;
                }
                if record.kind == 0x07d3
                    && matches!(
                        prototype.objects[local_id - 1].kind,
                        Kind::PointOnSegment(_, _)
                            | Kind::PointOnRay(_, _)
                            | Kind::PointOnLine(_, _)
                            | Kind::PointOnCircle(_, _, _)
                            | Kind::PointOnArc(_, _)
                            | Kind::PointOnPolygon(_, _, _)
                    )
                {
                    // Tool placement is chosen at invocation, including when its
                    // host is constructed from the tool's point givens.
                    continue;
                }
                if inputs.contains(&local_id) && matches!(record.kind, 0x07d1 | 0x0899) {
                    // Given point coordinates are supplied at invocation, rather
                    // than captured as an absolute position in the definition.
                    put_record(&mut section, 0x0899, &[0; 16]);
                } else if record.kind == 0x07d0
                    && !inputs.contains(&local_id)
                    && !selected_local.contains(&local_id)
                {
                    let mut payload = prototype_bytes[record.data.clone()].to_vec();
                    if payload.len() != 28 {
                        return Err("Tool helper header is unsupported".into());
                    }
                    // Native omits unselected constructed helpers from display using flag2.
                    payload[2] = 2;
                    put_record(&mut section, record.kind, &payload);
                } else {
                    put_record(
                        &mut section,
                        record.kind,
                        &prototype_bytes[record.data.clone()],
                    );
                }
                if record.kind == 0x07d7 {
                    in_group = false;
                }
            }
        }
        let mut declaration = (input_count as u32).to_le_bytes().to_vec();
        for id in (1..=input_count).rev() {
            declaration.extend_from_slice(&(id as u32).to_le_bytes());
        }
        put_record(&mut section, 0x04b4, &declaration);
        put_record(&mut section, 0x03eb, &[]);
        put_record(&mut section, 0x0387, &[]);

        let base = self.encode()?;
        let entries = records(&base)?;
        let header = entries
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .ok_or("Missing native document header")?;
        let end = entries
            .last()
            .filter(|record| record.kind == 0x0385)
            .ok_or("Missing native document terminator")?;
        let tool_count = self.embedded_tools()?.len();
        let new_count = u32::try_from(tool_count + 1).map_err(|_| "Too many tools")?;
        let mut bytes = base[..end.start].to_vec();
        bytes.extend_from_slice(&section);
        bytes.extend_from_slice(&base[end.start..]);
        bytes[header.data.start + 4..header.data.start + 8]
            .copy_from_slice(&new_count.to_le_bytes());
        let sections = (self.page_count as u32)
            .checked_add(new_count)
            .and_then(|n| n.checked_add(1))
            .ok_or("Too many document canvases")?;
        bytes[header.data.start + 24..header.data.start + 28]
            .copy_from_slice(&sections.to_le_bytes());
        let mut result = Document::from_bytes(&bytes, self.path.clone())?;
        self.validate_reparsed(&result)?;
        if result.embedded_tools()?.len() != tool_count + 1 || !result.safe_structure {
            return Err("Created tool cannot be saved safely".into());
        }
        let mut history = self.clone();
        history.checkpoint_structure();
        result.undo = history.undo;
        result.redo = history.redo;
        result.selected = self.selected.clone();
        result.dirty = true;
        *self = result;
        Ok(tool_count)
    }
}

impl Document {
    fn numeric_tool_boundary(&self) -> Result<Option<NumericToolGraph>, String> {
        let numeric: Vec<_> = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                id.checked_sub(1)
                    .and_then(|index| self.objects.get(index))
                    .is_some_and(|object| object.kind.numeric_unit().is_some())
            })
            .collect();
        if numeric.is_empty() {
            return Ok(None);
        }
        if numeric.len() != self.selected.len() {
            return Err("Mixed numeric and geometry tool templates are not supported yet".into());
        }
        let inputs: BTreeSet<_> = numeric
            .iter()
            .copied()
            .filter(|&id| {
                matches!(
                    self.objects[id - 1].kind,
                    Kind::Parameter(_) | Kind::SegmentLength(_)
                )
            })
            .collect();
        if inputs.is_empty() || inputs.len() == numeric.len() {
            return Err("Select supported numeric inputs and constructed calculations".into());
        }
        let mut calculations = BTreeSet::new();
        let mut used_inputs = BTreeSet::new();
        for id in numeric {
            if !inputs.contains(&id) {
                self.collect_numeric_tool_graph(id, &inputs, &mut calculations, &mut used_inputs)?;
            }
        }
        if used_inputs != inputs {
            return Err("All selected numeric givens must feed the selected calculations".into());
        }
        // Native stores u before t, but declares the binding order t then u.
        Ok(Some(NumericToolGraph {
            inputs: inputs.into_iter().rev().collect(),
            calculations: calculations.into_iter().collect(),
        }))
    }

    fn collect_numeric_tool_graph(
        &self,
        id: usize,
        inputs: &BTreeSet<usize>,
        calculations: &mut BTreeSet<usize>,
        used_inputs: &mut BTreeSet<usize>,
    ) -> Result<(), String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid numeric tool source")?)
            .filter(|object| !object.deleted && object.page == self.active_page)
            .ok_or("Numeric tool sources must be live on the active page")?;
        if !self.numeric_value(id).is_some_and(f64::is_finite) {
            return Err("Numeric tool sources must be defined".into());
        }
        if inputs.contains(&id) {
            used_inputs.insert(id);
            return Ok(());
        }
        if calculations.contains(&id) {
            return Ok(());
        }
        let group = numeric_tool_calculation(&object.kind)?;
        let sources: Vec<_> = group.sources().into_iter().map(|id| id as usize).collect();
        if self.dependencies_of(id).as_ref() != Some(&sources) {
            return Err(
                "Numeric tool calculation references disagree with its semantic graph".into(),
            );
        }
        for source in sources {
            if source >= id {
                return Err("Numeric tool calculations require earlier sources".into());
            }
            self.collect_numeric_tool_graph(source, inputs, calculations, used_inputs)?;
        }
        calculations.insert(id);
        Ok(())
    }

    fn constructible_from_selected_points(
        &self,
        id: usize,
        visiting: &mut BTreeSet<usize>,
    ) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|index| self.objects.get(index)) else {
            return false;
        };
        if matches!(object.kind, Kind::Free(_)) {
            return self.selected.contains(&id);
        }
        let Some(dependencies) = self.dependencies_of(id) else {
            return false;
        };
        if dependencies.is_empty() || !visiting.insert(id) {
            return false;
        }
        let result = dependencies
            .iter()
            .all(|dependency| self.constructible_from_selected_points(*dependency, visiting));
        visiting.remove(&id);
        result
    }

    fn path_tool_boundary(
        &self,
    ) -> Result<Option<(usize, EmbeddedToolInputKind, Vec<usize>)>, String> {
        let mut boundaries = Vec::new();
        for &id in &self.selected {
            let object = self
                .objects
                .get(id.checked_sub(1).ok_or("Invalid tool selection")?)
                .filter(|object| !object.deleted && object.page == self.active_page)
                .ok_or("Tool selection is not available on the active page")?;
            let path = self.line_points(id).is_some()
                || self.circle_points(id).is_some()
                || self.arc_points(id, 32).is_some()
                || self.polygon_points(id).is_some();
            if !path {
                continue;
            }
            if self.constructible_from_selected_points(id, &mut BTreeSet::new()) {
                continue;
            }
            if self.selected.iter().any(|&other| {
                other != id
                    && self
                        .dependencies_of(other)
                        .is_some_and(|dependencies| dependencies.contains(&id))
            }) {
                boundaries.push(object.id);
            }
        }
        if boundaries.is_empty() {
            return Ok(None);
        }
        if boundaries.len() != 1 {
            return Err("Tools with multiple path boundaries are not supported yet".into());
        }
        let host = boundaries[0];
        let mut outputs = Vec::new();
        let mut category = None;
        for &id in &self.selected {
            if id == host || matches!(self.objects[id - 1].kind, Kind::Free(_)) {
                continue;
            }
            let (output_host, input_kind) = match self.objects[id - 1].kind {
                Kind::PointOnSegment(h, _) | Kind::PointOnRay(h, _) | Kind::PointOnLine(h, _)
                | Kind::PointOnCircle(h, _, _) | Kind::PointOnArc(h, _) | Kind::PointOnPolygon(h, _, _) => (h, EmbeddedToolInputKind::Path),
                Kind::Midpoint(_, _) => (self.objects[id - 1].midpoint_host.ok_or("Missing midpoint host")?, EmbeddedToolInputKind::Segment),
                Kind::SegmentMidpoint(h) => (h, EmbeddedToolInputKind::Segment),
                _ => return Err("This path-given result requires an unsupported tool input category or record schema".into()),
            };
            if output_host != host {
                return Err("Path-given results must use the selected path boundary".into());
            }
            if category.is_some_and(|kind| kind != input_kind) {
                return Err(
                    "Mixed path and segment-given result templates are not supported yet".into(),
                );
            }
            category = Some(input_kind);
            outputs.push(id);
        }
        if outputs.is_empty() {
            return Err("Select a path and constructed points on that path".into());
        }
        Ok(Some((host, category.unwrap(), outputs)))
    }
}

fn path_template_groups(
    bytes: &[u8],
    prototype: &Document,
    given: usize,
    category: EmbeddedToolInputKind,
    outputs: &[usize],
) -> Result<Vec<u8>, String> {
    let mut groups = Vec::new();
    let mut placeholder = header(0);
    // Independently observed Native abstract Path/Segment givens.
    let (subtype, stamp) = match category {
        EmbeddedToolInputKind::Path => (5u16, 1u16),
        EmbeddedToolInputKind::Segment => (14u16, 0u16),
        _ => return Err("Unsupported abstract tool input category".into()),
    };
    placeholder[8..10].copy_from_slice(&subtype.to_le_bytes());
    placeholder[11] = 1;
    placeholder[12..16].fill(0);
    placeholder[18..20].copy_from_slice(&stamp.to_le_bytes());
    put_record(&mut groups, 0x07d0, &placeholder);
    put_record(&mut groups, 0x0899, &[0; 16]);
    put_record(&mut groups, 0x07d7, &[]);
    let entries = records(bytes)?;
    for &output in outputs {
        let object = &prototype.objects[output - 1];
        let references_match = match category {
            EmbeddedToolInputKind::Path => object.kind.dependencies() == vec![given],
            EmbeddedToolInputKind::Segment => {
                object.midpoint_host == Some(given)
                    || matches!(object.kind, Kind::SegmentMidpoint(host) if host == given)
            }
            _ => false,
        };
        if !references_match {
            return Err("Path output has unsupported auxiliary references".into());
        }
        let start = entries
            .iter()
            .enumerate()
            .filter(|(_, record)| record.kind == 0x07d0)
            .nth(output - 1)
            .map(|(index, _)| index)
            .ok_or("Missing path output group")?;
        let end = (start + 1..entries.len())
            .find(|&index| entries[index].kind == 0x07d7)
            .ok_or("Missing path output terminator")?;
        for record in &entries[start..=end] {
            match record.kind {
                0x07d3 => {} // Native chooses output placement when invoked.
                0x07d2 => {
                    let payload = &bytes[record.data.clone()];
                    if payload.len() != 8
                        || payload[..4] != 1u32.to_le_bytes()
                        || payload[4..] != (given as u32).to_le_bytes()
                    {
                        return Err("Path output reference layout is unsupported".into());
                    }
                    let mut references = 1u32.to_le_bytes().to_vec();
                    references.extend_from_slice(&1u32.to_le_bytes());
                    put_record(&mut groups, 0x07d2, &references);
                }
                0x07d0 | 0x07d5 | 0x07d8 | 0x07d7 => {
                    put_record(&mut groups, record.kind, &bytes[record.data.clone()])
                }
                _ => return Err("Path output contains an unsupported auxiliary record".into()),
            }
        }
    }
    Ok(groups)
}

/// Project supported AST records into Native's abstract Numeric graph. Source
/// references are remapped through the typed calculation parser, never caches.
fn numeric_template_groups(
    document: &Document,
    graph: &NumericToolGraph,
) -> Result<Vec<u8>, String> {
    let mut template = b"GSP4".to_vec();
    template.extend_from_slice(include_bytes!("../../assets/native_numeric_tool_given.bin"));
    let mut groups = Vec::new();
    let mut mapping = vec![0u32; document.objects.len() + 1];
    for (index, &given) in graph.inputs.iter().enumerate() {
        mapping[given] = (index + 1) as u32;
        let name = match &document.objects[given - 1].kind {
            Kind::Parameter(_) => document
                .parameter_display_name(given)
                .ok_or("Numeric given has no available parameter name")?,
            Kind::SegmentLength(_) => format!("m[{}]", index + 1),
            _ => return Err("Unsupported numeric tool given".into()),
        };
        for record in records(&template)? {
            if record.kind == 0x07d5 {
                let mut label =
                    crate::text_records::LabelAux::parse(&template[record.data.clone()])?;
                label.set_name(&name)?;
                put_record(&mut groups, record.kind, label.payload());
            } else {
                put_record(&mut groups, record.kind, &template[record.data]);
            }
        }
    }
    for (index, &id) in graph.calculations.iter().enumerate() {
        mapping[id] = (graph.inputs.len() + index + 1) as u32;
        let calculation = numeric_tool_calculation(&document.objects[id - 1].kind)?;
        let sources = calculation
            .sources()
            .iter()
            .map(|&source| {
                mapping
                    .get(source as usize)
                    .copied()
                    .filter(|&id| id != 0)
                    .ok_or("Numeric tool source is outside the declared graph")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let remapped = calculation.remap_sources(&sources)?;
        let mut framed = b"GSP4".to_vec();
        framed.extend_from_slice(remapped.as_bytes());
        for record in records(&framed)? {
            let mut payload = framed[record.data].to_vec();
            if record.kind == 0x07d0 {
                if payload.len() != 28
                    || payload[0..2] != 48u16.to_le_bytes()
                    || payload[6..8] != 11u16.to_le_bytes()
                {
                    return Err("Unsupported numeric calculation tool header".into());
                }
                payload[8..10].copy_from_slice(&33u16.to_le_bytes());
                if !document.selected.contains(&id) {
                    // Native includes necessary unselected calculations as flag2 helpers.
                    payload[2] = 2;
                }
            }
            if record.kind == 0x07d8 {
                if payload.len() != 2 {
                    return Err("Unsupported numeric template caption font".into());
                }
                payload.copy_from_slice(&1u16.to_le_bytes());
            }
            put_record(&mut groups, record.kind, &payload);
        }
    }
    Ok(groups)
}

fn numeric_tool_calculation(kind: &Kind) -> Result<ParameterCalculationGroup, String> {
    match kind {
        Kind::ParameterCalculation(_, group) => Ok(group.clone()),
        Kind::MeasuredCalculation(_, group) => ParameterCalculationGroup::parse(group.as_bytes()),
        _ => Err("This numeric given or calculation template family is not supported yet".into()),
    }
}
