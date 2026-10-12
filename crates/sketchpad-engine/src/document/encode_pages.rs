use super::embedded_tools::document_record_indices;
use super::kind::{
    is_arc_kind, is_multi_page_addition_kind, is_multi_page_deletion_kind, is_point_kind,
    is_segment_kind, is_straight_kind, polygon_vertex_count_for_encoding, straight_kind_mode,
};
use super::records::{header, page_title_slots, polygon_subtype, put_record, records, u32_at};
use super::{Document, Kind, LineMode, Object};
use crate::action_records::{
    AnimationButtonGroup, HideShowButtonGroup, LinkPageButtonGroup, LinkUrlButtonGroup,
    MovementButtonGroup, PresentationButtonGroup, ScrollButtonGroup, SoundButtonGroup,
};
use crate::style_records;
use std::collections::BTreeSet;

impl Document {
    pub(super) fn compact_deleted(&self, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        if !self.objects.iter().any(|o| o.deleted) {
            return Ok(bytes);
        }
        let mut ids = vec![0; self.objects.len() + 1];
        let mut kept = 0;
        for object in &self.objects {
            if !object.deleted {
                kept += 1;
                ids[object.id] = kept;
            }
        }
        for &source in self.merge_aliases.keys() {
            let mut target = source;
            let mut visited = BTreeSet::new();
            while let Some(&next) = self.merge_aliases.get(&target) {
                if !visited.insert(target) {
                    return Err("Merged point aliases contain a cycle".into());
                }
                target = next;
            }
            ids[source] = *ids
                .get(target)
                .filter(|&&mapped| mapped != 0)
                .ok_or("Merged point target was deleted")?;
        }
        let rs = records(&bytes)?;
        let drawing_indices: BTreeSet<_> = document_record_indices(&rs).into_iter().collect();
        let mut out = bytes[..4].to_vec();
        let mut group_id = 0;
        let mut dropping = false;
        for (index, record) in rs.iter().enumerate() {
            if !drawing_indices.contains(&index) {
                let end = rs.get(index + 1).map_or(bytes.len(), |next| next.start);
                out.extend_from_slice(&bytes[record.start..end]);
                continue;
            }
            if record.kind == 0x7d0 {
                group_id += 1;
                dropping = self
                    .objects
                    .get(group_id - 1)
                    .ok_or("Object group count mismatch")?
                    .deleted;
            }
            if !dropping && record.kind != 0x2328 {
                let end = rs.get(index + 1).map_or(bytes.len(), |next| next.start);
                let mut raw = bytes[record.start..end].to_vec();
                if record.kind == 0x232d {
                    if record.data.len() != 4 {
                        return Err("Invalid coordinate-system page link".into());
                    }
                    let old = u32_at(&bytes, record.data.start) as usize;
                    let new = *ids.get(old).ok_or("Invalid coordinate-system page link")?;
                    if new == 0 {
                        continue;
                    }
                    raw[8..12].copy_from_slice(&(new as u32).to_le_bytes());
                }
                if record.kind == 0x2331 {
                    let definition =
                        super::CustomTransformDefinition::parse(&bytes[record.data.clone()])
                            .ok_or("Invalid custom-transform definition")?;
                    let seed = *ids
                        .get(definition.seed())
                        .ok_or("Invalid custom-transform seed")?;
                    let output = *ids
                        .get(definition.output())
                        .ok_or("Invalid custom-transform output")?;
                    if seed == 0 || output == 0 {
                        continue;
                    }
                    raw[8..12].copy_from_slice(&(seed as u32).to_le_bytes());
                    raw[12..16].copy_from_slice(&(output as u32).to_le_bytes());
                }
                if record.kind == 0x3e8 {
                    if record.data.len() < 28 {
                        return Err("Short page preamble".into());
                    }
                    raw[32..36].copy_from_slice(&(kept as u32).to_le_bytes());
                }
                if record.kind == 0x7d2 {
                    let payload = &bytes[record.data.clone()];
                    if payload.len() < 4
                        || !(payload.len() - 4).is_multiple_of(4)
                        || u32_at(payload, 0) as usize != (payload.len() - 4) / 4
                    {
                        return Err("Invalid object references during deletion".into());
                    }
                    for j in 0..(payload.len() - 4) / 4 {
                        let old = u32_at(payload, 4 + j * 4) as usize;
                        let new = *ids
                            .get(old)
                            .filter(|&&id| id > 0)
                            .ok_or("Surviving object refers to a deleted object")?;
                        raw[12 + j * 4..16 + j * 4].copy_from_slice(&(new as u32).to_le_bytes());
                    }
                }
                out.extend_from_slice(&raw);
            }
            if record.kind == 0x7d7 {
                dropping = false;
            }
        }
        if group_id != self.objects.len() {
            return Err("Object group count mismatch".into());
        }
        records(&out)?;
        Ok(out)
    }
    pub(super) fn encode_multi_page_deletions(&self) -> Result<Vec<u8>, String> {
        if self.page_count < 2
            || self
                .objects
                .iter()
                .any(|object| !is_multi_page_deletion_kind(&object.kind))
        {
            return Err("Deleting these multi-page objects cannot be saved safely".into());
        }
        let mut unmarked = self.clone();
        for mut object in unmarked.objects.iter_mut() {
            object.deleted = false;
        }
        let bytes = if unmarked.has_pending_multi_page_additions() {
            unmarked.encode_multi_page_additions()?
        } else {
            unmarked.encode_existing_state()?
        };
        let mut ordered: Vec<_> = self.objects.iter().collect();
        ordered.sort_by_key(|object| (object.page, object.id > self.source_count, object.id));
        let mut page_objects = vec![Vec::new(); self.page_count];
        for object in ordered {
            page_objects
                .get_mut(object.page)
                .ok_or("Object belongs to a missing page")?
                .push(object);
        }
        let mut local_ids = Vec::with_capacity(self.page_count);
        let mut survivor_counts = Vec::with_capacity(self.page_count);
        for objects in &page_objects {
            let mut ids = vec![0u32; objects.len() + 1];
            let mut survivors = 0u32;
            for (index, object) in objects.iter().enumerate() {
                if !object.deleted {
                    survivors = survivors.checked_add(1).ok_or("Too many objects on page")?;
                    ids[index + 1] = survivors;
                }
            }
            local_ids.push(ids);
            survivor_counts.push(survivors);
        }
        let rs = records(&bytes)?;
        let mut out = bytes[..4].to_vec();
        let mut current_page = None;
        let mut pages_seen = 0;
        let mut group_ordinal = 0;
        let mut inside_group = false;
        let mut dropping = false;
        for (index, record) in rs.iter().enumerate() {
            let end = rs.get(index + 1).map_or(bytes.len(), |next| next.start);
            let mut raw = bytes[record.start..end].to_vec();
            match record.kind {
                0x03e8 => {
                    if current_page.is_some()
                        || inside_group
                        || record.data.len() < 28
                        || pages_seen >= self.page_count
                    {
                        return Err("Invalid page preamble during deletion".into());
                    }
                    current_page = Some(pages_seen);
                    group_ordinal = 0;
                    if u32_at(&bytes, record.data.start + 24) as usize
                        != page_objects[pages_seen].len()
                    {
                        return Err("Page object count differs from its preamble".into());
                    }
                    raw[32..36].copy_from_slice(&survivor_counts[pages_seen].to_le_bytes());
                    pages_seen += 1;
                }
                0x07d0 => {
                    if inside_group {
                        return Err("Nested object group during deletion".into());
                    }
                    let page = current_page.ok_or("Object group outside page")?;
                    let object = page_objects[page]
                        .get(group_ordinal)
                        .ok_or("Too many object groups on page")?;
                    group_ordinal += 1;
                    inside_group = true;
                    dropping = object.deleted;
                }
                0x07d2 if inside_group && !dropping => {
                    let page = current_page.ok_or("Object references outside page")?;
                    let payload = &bytes[record.data.clone()];
                    if payload.len() < 4
                        || !(payload.len() - 4).is_multiple_of(4)
                        || u32_at(payload, 0) as usize != (payload.len() - 4) / 4
                    {
                        return Err("Invalid page-local references during deletion".into());
                    }
                    for reference in 0..(payload.len() - 4) / 4 {
                        let old = u32_at(payload, 4 + reference * 4) as usize;
                        let new = *local_ids[page]
                            .get(old)
                            .filter(|&&id| id > 0)
                            .ok_or("Surviving object refers to a deleted object")?;
                        raw[12 + reference * 4..16 + reference * 4]
                            .copy_from_slice(&new.to_le_bytes());
                    }
                }
                0x07d2 if !inside_group => {
                    return Err("Object references outside group".into());
                }
                0x07d7 => {
                    if !inside_group {
                        return Err("Object terminator outside group".into());
                    }
                }
                0x03e9 => {
                    let page = current_page.ok_or("Page trailer without preamble")?;
                    if inside_group || group_ordinal != page_objects[page].len() {
                        return Err("Object group count mismatch on page".into());
                    }
                    current_page = None;
                }
                0x232d => {
                    let page = current_page.ok_or("Page link outside page")?;
                    if record.data.len() != 4 {
                        return Err("Invalid page link during deletion".into());
                    }
                    let old = u32_at(&bytes, record.data.start) as usize;
                    let new = *local_ids[page]
                        .get(old)
                        .filter(|&&id| id > 0)
                        .ok_or("Page link refers to a deleted object")?;
                    raw[8..12].copy_from_slice(&new.to_le_bytes());
                }
                0x2328 if !inside_group => continue,
                _ => {}
            }
            let literal_text_record = matches!(record.kind, 0x08fc | 0x0898)
                && current_page
                    .and_then(|page| page_objects[page].get(group_ordinal.saturating_sub(1)))
                    .is_some_and(|object| matches!(object.kind, Kind::PlainText { .. }));
            if inside_group
                && !literal_text_record
                && !matches!(
                    record.kind,
                    0x07d0 | 0x07d2 | 0x07d3 | 0x07d5 | 0x07d7 | 0x07d8 | 0x0899 | 0x0903 | 0x0906
                )
            {
                return Err("Object group has unsupported deletion records".into());
            }
            if !dropping {
                out.extend_from_slice(&raw);
            }
            if record.kind == 0x07d7 {
                inside_group = false;
                dropping = false;
            }
        }
        if pages_seen != self.page_count || current_page.is_some() || inside_group {
            return Err("Incomplete multi-page document during deletion".into());
        }
        records(&out)?;
        Ok(out)
    }
    pub(super) fn encode_added_pages(&self) -> Result<Vec<u8>, String> {
        let rs = records(&self.original)?;
        let old_count = rs.iter().filter(|record| record.kind == 0x03e8).count();
        if old_count == 0
            || self.page_count <= old_count
            || self.page_count > 32768
            || self.image_read_only
        {
            return Err("Cannot add pages to this document".into());
        }
        let header = rs
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .ok_or("Missing document page header")?;
        let end = rs
            .last()
            .filter(|record| record.kind == 0x0385)
            .ok_or("Missing document end record")?;
        let page_metadata: Vec<_> = rs
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
            .collect();
        if page_metadata.len() != old_count
            || u16::from_le_bytes(
                self.original[header.data.start + 2..header.data.start + 4]
                    .try_into()
                    .unwrap(),
            ) as usize
                != old_count
        {
            return Err("Document page metadata is inconsistent".into());
        }
        let blank = include_bytes!("../../../../assets/blank_page_records.bin");
        let mut expanded = self.original[..end.start].to_vec();
        for _ in old_count..self.page_count {
            expanded.extend_from_slice(blank);
        }
        expanded.extend_from_slice(&self.original[end.start..]);
        let expanded_records = records(&expanded)?;
        let expanded_header = expanded_records
            .iter()
            .find(|record| record.kind == 0x0384 && record.data.len() == 28)
            .ok_or("Missing expanded document header")?;
        let header_start = expanded_header.data.start;
        expanded[header_start + 2..header_start + 4]
            .copy_from_slice(&(self.page_count as u16).to_le_bytes());
        expanded[header_start + 16..header_start + 18]
            .copy_from_slice(&(self.active_page as u16).to_le_bytes());
        expanded[header_start + 24..header_start + 28]
            .copy_from_slice(&((self.page_count + 1) as u32).to_le_bytes());
        let page_marker = (self.page_count * 2 - 1) as u16;
        for record in expanded_records
            .iter()
            .filter(|record| record.kind == 0x0386 && record.data.len() == 216)
        {
            expanded[record.data.start + 6..record.data.start + 8]
                .copy_from_slice(&page_marker.to_le_bytes());
            expanded[record.data.start + 8..record.data.start + 10]
                .copy_from_slice(&page_marker.to_le_bytes());
        }
        let mut working = self.clone();
        working.original = expanded;
        working.encode()
    }
    fn multi_page_local_reference(
        &self,
        page: usize,
        current_id: usize,
        reference: usize,
    ) -> Result<u32, String> {
        let target = self
            .objects
            .get(reference.saturating_sub(1))
            .ok_or("Missing multi-page construction parent")?;
        if reference == 0 || reference >= current_id || target.page != page || target.deleted {
            return Err(
                "Multi-page construction parent must precede its child on the same page".into(),
            );
        }
        let preceding = self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| object.page == page)
            .chain(
                self.objects
                    .iter()
                    .skip(self.source_count)
                    .take_while(|object| object.id < current_id)
                    .filter(|object| object.page == page),
            );
        let ordinal = preceding
            .enumerate()
            .find_map(|(index, object)| (object.id == reference).then_some(index + 1))
            .ok_or("Missing page-local construction parent")?;
        u32::try_from(ordinal).map_err(|_| "Too many objects on page".into())
    }
    fn append_multi_page_reference_group(
        groups: &mut Vec<u8>,
        object: &Object,
        code: u16,
        refs: &[u32],
    ) -> Result<(), String> {
        let mut object_header = header(code);
        if !object.visible {
            object_header[2] = 1;
        }
        object_header[2] |= u8::from(object.presentation_suppressed) << 1;
        object_header[4] = u8::from(object.tracing);
        if let Some(style) = object.visual_style {
            style_records::set_visual_style(&mut object_header, style)?;
        }
        if let Kind::Polygon(vertices) = &object.kind {
            object_header[8..10].copy_from_slice(
                &polygon_subtype(vertices.len())
                    .ok_or("Polygon has too many vertices")?
                    .to_le_bytes(),
            );
            object_header[20..24].copy_from_slice(
                &object
                    .interior_order
                    .ok_or("Missing polygon drawing order")?
                    .to_le_bytes(),
            );
        }
        if matches!(
            object.kind,
            Kind::SectorBoundary(_) | Kind::CircularSegmentBoundary(_) | Kind::CircleInterior(_)
        ) {
            object_header[20..24].copy_from_slice(
                &object
                    .interior_order
                    .ok_or("Missing sector drawing order")?
                    .to_le_bytes(),
            );
        }
        put_record(groups, 0x07d0, &object_header);
        if is_point_kind(&object.kind)
            && let Some(label) = &object.label
        {
            groups.extend_from_slice(&crate::text_records::encode_point_label_records(label));
        }
        let mut references = Vec::with_capacity(4 + refs.len() * 4);
        references.extend_from_slice(
            &u32::try_from(refs.len())
                .map_err(|_| "Too many construction parents")?
                .to_le_bytes(),
        );
        for reference in refs {
            references.extend_from_slice(&reference.to_le_bytes());
        }
        put_record(groups, 0x07d2, &references);
        put_record(groups, 0x07d7, &[]);
        Ok(())
    }
    pub(super) fn encode_multi_page_additions(&self) -> Result<Vec<u8>, String> {
        let original = Self::from_bytes(&self.original, None)?;
        if self.page_count < 2
            || original.page_count != self.page_count
            || self.source_count > self.objects.len()
            || original.snap_points != self.snap_points
            || original.grid_dotted != self.grid_dotted
            || original.grid_form != self.grid_form
            || original.angle_unit_radians != self.angle_unit_radians
            || original.marked_coordinate_system != self.marked_coordinate_system
            || original.frame_styles != self.frame_styles
            || self.objects[self.source_count..].iter().any(|object| {
                object.deleted
                    || object.page >= self.page_count
                    || !is_multi_page_addition_kind(&object.kind)
            })
        {
            return Err("Multi-page edit contains unsupported additions".into());
        }
        let mut out = self.encode_existing_source_state()?;
        let rs = records(&out)?;
        let source_headers: Vec<_> = document_record_indices(&rs)
            .into_iter()
            .filter(|index| rs[*index].kind == 0x07d0)
            .collect();
        if source_headers.len() != self.source_count {
            return Err("Source object group count mismatch".into());
        }
        let preambles: Vec<_> = rs.iter().filter(|record| record.kind == 0x3e8).collect();
        if preambles.len() != self.page_count {
            return Err("Page preamble count mismatch".into());
        }
        let mut insertions = Vec::new();
        for (page, preamble) in preambles.iter().enumerate() {
            if preamble.data.len() < 28 {
                return Err("Short page preamble".into());
            }
            let next_page = preambles.get(page + 1).map_or(out.len(), |next| next.start);
            let trailer = rs
                .iter()
                .find(|record| {
                    record.kind == 0x3e9
                        && record.start > preamble.start
                        && record.start < next_page
                })
                .ok_or("Missing page trailer")?
                .start;
            let existing = original
                .objects
                .iter()
                .filter(|object| object.page == page)
                .count();
            if u32_at(&out, preamble.data.start + 24) as usize != existing {
                return Err("Page object count differs from its preamble".into());
            }
            let new_objects: Vec<_> = self.objects[self.source_count..]
                .iter()
                .filter(|object| object.page == page)
                .collect();
            if new_objects.is_empty() {
                continue;
            }
            let last_group_end = rs
                .iter()
                .filter(|record| {
                    record.kind == 0x7d7 && record.start > preamble.start && record.start < trailer
                })
                .map(|record| record.data.end)
                .max()
                .unwrap_or(preamble.data.end);
            let insert_at = rs
                .iter()
                .find(|record| {
                    matches!(record.kind, 0x2328 | 0x232d)
                        && record.start >= last_group_end
                        && record.start < trailer
                })
                .map(|record| record.start)
                .unwrap_or(trailer);
            let mut groups = Vec::new();
            let mut measurement_row = original
                .objects
                .iter()
                .filter(|object| object.page == page && object.kind.is_measurement())
                .count();
            for (added_index, object) in new_objects.into_iter().enumerate() {
                match &object.kind {
                    Kind::ReferencedText { parents, text } => {
                        let refs = parents
                            .iter()
                            .map(|id| self.multi_page_local_reference(page, object.id, *id))
                            .collect::<Result<Vec<_>, _>>()?;
                        groups.extend(super::referenced_captions::encode_group(
                            object, &refs, text,
                        ));
                    }
                    Kind::PlainText { text, rect } => {
                        let mut group = if let Some(template) = &object.rich_text_override {
                            template.encode_group(text, *rect)?
                        } else {
                            crate::text_object::encode_plain_text_group(text, *rect)?
                        };
                        group = Self::caption_group_with_label(&group, object.label.as_ref())?;
                        if !object.visible {
                            group[10] |= 1;
                        }
                        groups.extend_from_slice(&group);
                    }
                    Kind::CoordinatePoint(definition) => {
                        let refs = definition
                            .native_references()
                            .into_iter()
                            .map(|id| self.multi_page_local_reference(page, object.id, id))
                            .collect::<Result<Vec<_>, _>>()?;
                        groups.extend(super::coordinate_points::encode_group(
                            object, definition, &refs,
                        )?);
                    }
                    Kind::Colorized(definition) => {
                        put_record(
                            &mut groups,
                            0x07d0,
                            &self.colorized_object_header(object, definition)?,
                        );
                        let ids = definition.native_references();
                        let mut refs = (ids.len() as u32).to_le_bytes().to_vec();
                        for id in ids {
                            refs.extend_from_slice(
                                &self
                                    .multi_page_local_reference(page, object.id, id)?
                                    .to_le_bytes(),
                            );
                        }
                        put_record(&mut groups, 0x07d2, &refs);
                        put_record(&mut groups, 0x07d3, &definition.native_payload());
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::MappedElement(definition) => {
                        let refs = definition
                            .native_references()
                            .into_iter()
                            .map(|id| self.multi_page_local_reference(page, object.id, id))
                            .collect::<Result<Vec<_>, _>>()?;
                        Self::append_multi_page_reference_group(&mut groups, object, 101, &refs)?;
                    }
                    Kind::IterationPointAlias(binding) => {
                        let reference =
                            self.multi_page_local_reference(page, object.id, *binding)?;
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            88,
                            &[reference],
                        )?;
                    }
                    Kind::Free(point) => {
                        if !point.x.is_finite() || !point.y.is_finite() {
                            return Err("Multi-page point coordinates must be finite".into());
                        }
                        let mut object_header = header(0);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut coordinates = Vec::with_capacity(16);
                        coordinates.extend_from_slice(&point.x.to_le_bytes());
                        coordinates.extend_from_slice(&point.y.to_le_bytes());
                        put_record(&mut groups, 0x0899, &coordinates);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::PointOnSegment(host, t)
                    | Kind::PointOnRay(host, t)
                    | Kind::PointOnLine(host, t)
                    | Kind::PointOnArc(host, t)
                    | Kind::PointOnSector(host, t)
                    | Kind::PointOnCircularSegment(host, t)
                    | Kind::PointOnCurve(host, t) => {
                        if !t.is_finite()
                            || matches!(
                                object.kind,
                                Kind::PointOnSegment(_, _)
                                    | Kind::PointOnArc(_, _)
                                    | Kind::PointOnCircularSegment(_, _)
                            ) && !(0.0..=1.0).contains(t)
                            || matches!(object.kind, Kind::PointOnRay(_, _)) && *t < 0.
                        {
                            return Err("Invalid path-point parameter".into());
                        }
                        let host_object = self
                            .objects
                            .get(host.saturating_sub(1))
                            .ok_or("Missing point host")?;
                        if host_object.page != page
                            || matches!(object.kind, Kind::PointOnSegment(_, _))
                                && straight_kind_mode(&host_object.kind) != Some(LineMode::Segment)
                            || matches!(object.kind, Kind::PointOnRay(_, _))
                                && straight_kind_mode(&host_object.kind) != Some(LineMode::Ray)
                            || matches!(object.kind, Kind::PointOnLine(_, _))
                                && straight_kind_mode(&host_object.kind) != Some(LineMode::Line)
                            || matches!(object.kind, Kind::PointOnArc(_, _))
                                && !is_arc_kind(&host_object.kind)
                            || matches!(object.kind, Kind::PointOnSector(..))
                                && !super::sector_boundary::supported_host(
                                    &self.objects,
                                    *host,
                                    page,
                                    object.id,
                                )
                            || matches!(object.kind, Kind::PointOnCircularSegment(..))
                                && !super::sector_boundary::supported_circular_segment_host(
                                    &self.objects,
                                    *host,
                                    page,
                                    object.id,
                                )
                            || matches!(object.kind, Kind::PointOnCurve(..))
                                && !super::curve_points::free_supported(
                                    &self.objects,
                                    *host,
                                    page,
                                    object.id,
                                )
                        {
                            return Err("Invalid point host on this page".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, *host)?;
                        let mut object_header = header(15);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut references = Vec::with_capacity(8);
                        references.extend_from_slice(&1u32.to_le_bytes());
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        let code = match object.kind {
                            Kind::PointOnLine(host, _) => {
                                super::kind::straight_point_constraint_subtype(
                                    &self.objects[host - 1].kind,
                                )
                                .ok_or("Invalid straight-point host")?
                            }
                            Kind::PointOnRay(_, _) => 13,
                            Kind::PointOnSegment(_, _) => 14,
                            Kind::PointOnArc(_, _) => 67,
                            Kind::PointOnSector(_, _) => 68,
                            Kind::PointOnCircularSegment(_, _) => 69,
                            Kind::PointOnCurve(host, _) => {
                                super::curve_points::free_constraint_subtype(&self.objects, host)
                                    .ok_or("Invalid curve-point host")?
                            }
                            _ => unreachable!(),
                        };
                        let mut constraint = vec![15, 0, code, 0];
                        constraint.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut groups, 0x07d3, &constraint);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::PointOnPolygon(host, edge, t) => {
                        let host_object = self
                            .objects
                            .get(host.saturating_sub(1))
                            .ok_or("Missing polygon host")?;
                        let count = polygon_vertex_count_for_encoding(&self.objects, *host)
                            .ok_or("Invalid polygon boundary host")?;
                        if host_object.page != page
                            || *edge >= count
                            || !t.is_finite()
                            || !(0.0..=1.0).contains(t)
                        {
                            return Err("Invalid polygon boundary constraint".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, *host)?;
                        let mut object_header = header(15);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x7d0, &object_header);
                        let mut references = 1u32.to_le_bytes().to_vec();
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x7d2, &references);
                        let mut constraint = 15u16.to_le_bytes().to_vec();
                        constraint.extend_from_slice(
                            &super::decode::polygon_constraint_subtype(
                                &self.objects,
                                &self.original,
                                *host,
                                count,
                            )
                            .ok_or("Too many polygon vertices")?
                            .to_le_bytes(),
                        );
                        constraint.extend_from_slice(&t.to_le_bytes());
                        constraint.extend_from_slice(&(*edge as u32).to_le_bytes());
                        constraint.extend_from_slice(&[255, 255, 0, 0]);
                        put_record(&mut groups, 0x7d3, &constraint);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        put_record(&mut groups, 0x7d7, &[]);
                    }
                    Kind::PointOnCircle(host, x, y) => {
                        if !x.is_finite() || !y.is_finite() || (x * x + y * y - 1.).abs() > 1e-6 {
                            return Err("Invalid circle-point parameter".into());
                        }
                        let host_object = self
                            .objects
                            .get(host.saturating_sub(1))
                            .ok_or("Missing point host")?;
                        if host_object.page != page || !host_object.kind.is_circle() {
                            return Err("Invalid circle host on this page".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, *host)?;
                        let mut object_header = header(15);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut references = Vec::with_capacity(8);
                        references.extend_from_slice(&1u32.to_le_bytes());
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        let mut constraint = vec![15, 0, 28, 0];
                        constraint.extend_from_slice(&x.to_le_bytes());
                        constraint.extend_from_slice(&y.to_le_bytes());
                        put_record(&mut groups, 0x07d3, &constraint);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::Segment(a, b)
                    | Kind::Circle(a, b)
                    | Kind::Line(a, b)
                    | Kind::Ray(a, b) => {
                        if a == b
                            || !self
                                .objects
                                .get(a.saturating_sub(1))
                                .is_some_and(|parent| is_point_kind(&parent.kind))
                            || !self
                                .objects
                                .get(b.saturating_sub(1))
                                .is_some_and(|parent| is_point_kind(&parent.kind))
                            || self
                                .position(*a)
                                .zip(self.position(*b))
                                .is_none_or(|(first, second)| first.distance(second) <= 1e-9)
                        {
                            return Err(
                                "Straight object or circle needs two distinct points".into()
                            );
                        }
                        let first = self.multi_page_local_reference(page, object.id, *a)?;
                        let second = self.multi_page_local_reference(page, object.id, *b)?;
                        let code = match object.kind {
                            Kind::Segment(_, _) => 2,
                            Kind::Circle(_, _) => 3,
                            Kind::Line(_, _) => 63,
                            Kind::Ray(_, _) => 64,
                            _ => unreachable!(),
                        };
                        let mut object_header = header(code);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = Vec::with_capacity(12);
                        references.extend_from_slice(&2u32.to_le_bytes());
                        references.extend_from_slice(&first.to_le_bytes());
                        references.extend_from_slice(&second.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::SegmentMarker { .. } | Kind::SegmentArrow { .. } => {
                        let (host, parameter, marker_kind, marker_tail) = match object.kind {
                            Kind::SegmentMarker {
                                host,
                                parameter,
                                count,
                            } => (host, parameter, u32::from(count), 1_u32),
                            Kind::SegmentArrow {
                                host,
                                parameter,
                                forward,
                            } => (host, parameter, 0x0001_0001, u32::from(forward)),
                            _ => unreachable!(),
                        };
                        if !parameter.is_finite()
                            || !(0.0..=1.0).contains(&parameter)
                            || !matches!(marker_kind, 1..=3 | 0x0001_0001)
                            || !self
                                .objects
                                .get(host.saturating_sub(1))
                                .is_some_and(|parent| {
                                    !parent.deleted
                                        && parent.page == page
                                        && is_segment_kind(&parent.kind)
                                })
                        {
                            return Err("Invalid segment marker".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, host)?;
                        let mut object_header = header(121);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = 1_u32.to_le_bytes().to_vec();
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        let mut marker = parameter.to_le_bytes().to_vec();
                        marker.extend_from_slice(&marker_kind.to_le_bytes());
                        marker.extend_from_slice(&marker_tail.to_le_bytes());
                        put_record(&mut groups, 0x090f, &marker);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::Midpoint(_, _) | Kind::SegmentMidpoint(_) => {
                        let host = match object.kind {
                            Kind::SegmentMidpoint(host) => host,
                            Kind::Midpoint(a, b) => {
                                let host = object
                                    .midpoint_host
                                    .ok_or("Midpoint needs a segment host")?;
                                if !matches!(self.objects.get(host.saturating_sub(1)).map(|parent| &parent.kind), Some(Kind::Segment(first, second)) if (*first == a && *second == b) || (*first == b && *second == a))
                                {
                                    return Err("Midpoint host has different endpoints".into());
                                }
                                host
                            }
                            _ => unreachable!(),
                        };
                        let local_host = self.multi_page_local_reference(page, object.id, host)?;
                        let mut object_header = header(1);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = Vec::with_capacity(8);
                        references.extend_from_slice(&1u32.to_le_bytes());
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::CircleBySegment(center, radius) => {
                        if !self
                            .objects
                            .get(center.saturating_sub(1))
                            .is_some_and(|parent| is_point_kind(&parent.kind))
                            || !matches!(
                                self.objects
                                    .get(radius.saturating_sub(1))
                                    .map(|parent| &parent.kind),
                                Some(Kind::Segment(_, _))
                            )
                            || self
                                .line_points(*radius)
                                .is_none_or(|(first, second, _)| first.distance(second) <= 1e-9)
                        {
                            return Err(
                                "Circle by segment needs a point and nonzero segment".into()
                            );
                        }
                        let local_center =
                            self.multi_page_local_reference(page, object.id, *center)?;
                        let local_radius =
                            self.multi_page_local_reference(page, object.id, *radius)?;
                        let mut object_header = header(4);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = Vec::with_capacity(12);
                        references.extend_from_slice(&2u32.to_le_bytes());
                        references.extend_from_slice(&local_center.to_le_bytes());
                        references.extend_from_slice(&local_radius.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::Parallel(point, line) | Kind::Perpendicular(point, line) => {
                        if !self
                            .objects
                            .get(point.saturating_sub(1))
                            .is_some_and(|parent| is_point_kind(&parent.kind))
                            || self
                                .line_points(*line)
                                .is_none_or(|(first, second, _)| first.distance(second) <= 1e-9)
                        {
                            return Err(
                                "Derived line needs a point and nonzero straight object".into()
                            );
                        }
                        let local_point =
                            self.multi_page_local_reference(page, object.id, *point)?;
                        let local_line = self.multi_page_local_reference(page, object.id, *line)?;
                        let code = if matches!(object.kind, Kind::Parallel(_, _)) {
                            6
                        } else {
                            5
                        };
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            code,
                            &[local_point, local_line],
                        )?;
                    }
                    Kind::CurveIntersection(a, b, branch) => {
                        if !super::curve_intersection::supported(
                            &self.objects,
                            *a,
                            *b,
                            object.page,
                            object.id,
                        ) {
                            return Err("Invalid curve intersection".into());
                        }
                        let mut h = header(98);
                        if !object.visible {
                            h[2] = 1;
                        }
                        h[2] |= u8::from(object.presentation_suppressed) << 1;
                        h[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut h, style)?;
                        }
                        put_record(&mut groups, 0x7d0, &h);
                        let mut refs = 2u32.to_le_bytes().to_vec();
                        for id in [*a, *b] {
                            refs.extend_from_slice(
                                &self
                                    .multi_page_local_reference(page, object.id, id)?
                                    .to_le_bytes(),
                            );
                        }
                        put_record(&mut groups, 0x7d2, &refs);
                        let mut payload = 98u32.to_le_bytes().to_vec();
                        payload.extend_from_slice(&branch.to_le_bytes());
                        put_record(&mut groups, 0x7d3, &payload);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        put_record(&mut groups, 0x7d7, &[]);
                    }
                    Kind::BoundaryIntersection(a, b) => {
                        if !super::boundary_intersection::supported(
                            &self.objects,
                            *a,
                            *b,
                            object.page,
                            object.id,
                        ) {
                            return Err("Invalid polygon boundary intersection".into());
                        }
                        let refs = [
                            self.multi_page_local_reference(page, object.id, *a)?,
                            self.multi_page_local_reference(page, object.id, *b)?,
                        ];
                        Self::append_multi_page_reference_group(&mut groups, object, 93, &refs)?;
                    }
                    Kind::Intersection(a, b)
                    | Kind::LineCircleIntersection(a, b, _)
                    | Kind::CircleCircleIntersection(a, b, _) => {
                        let code = match object.kind {
                            Kind::Intersection(_, _)
                                if self.line_points(*a).is_some()
                                    && self.line_points(*b).is_some() =>
                            {
                                9
                            }
                            Kind::LineCircleIntersection(_, _, variant)
                                if variant <= 1
                                    && self.line_points(*a).is_some()
                                    && self.objects.get(b.saturating_sub(1)).is_some_and(
                                        |round| round.kind.is_circle() || is_arc_kind(&round.kind),
                                    ) =>
                            {
                                if variant == 0 {
                                    12
                                } else {
                                    11
                                }
                            }
                            Kind::CircleCircleIntersection(_, _, variant)
                                if variant <= 1
                                    && [*a, *b].iter().all(|&id| {
                                        self.objects.get(id.saturating_sub(1)).is_some_and(|o| {
                                            o.kind.is_circle() || is_arc_kind(&o.kind)
                                        })
                                    }) =>
                            {
                                if variant == 0 {
                                    14
                                } else {
                                    13
                                }
                            }
                            _ => return Err("Intersection needs compatible paths".into()),
                        };
                        if self.position(object.id).is_none() {
                            return Err("Intersection has no finite point".into());
                        }
                        let local_a = self.multi_page_local_reference(page, object.id, *a)?;
                        let local_b = self.multi_page_local_reference(page, object.id, *b)?;
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            code,
                            &[local_a, local_b],
                        )?;
                    }
                    Kind::Bisector(a, b, c)
                    | Kind::ThreePointArc(a, b, c)
                    | Kind::CenterArc(a, b, c) => {
                        if [*a, *b, *c].iter().any(|id| {
                            !self
                                .objects
                                .get(id.saturating_sub(1))
                                .is_some_and(|parent| is_point_kind(&parent.kind))
                        }) {
                            return Err("Construction needs three points".into());
                        }
                        let code = if matches!(object.kind, Kind::Bisector(_, _, _)) {
                            if self.line_points(object.id).is_none() {
                                return Err("Angle bisector has no finite direction".into());
                            }
                            7
                        } else {
                            if self.arc_geometry(object.id).is_none() {
                                return Err("Three-point arc has no finite geometry".into());
                            }
                            if matches!(object.kind, Kind::CenterArc(..)) {
                                80
                            } else {
                                81
                            }
                        };
                        let refs = [
                            self.multi_page_local_reference(page, object.id, *a)?,
                            self.multi_page_local_reference(page, object.id, *b)?,
                            self.multi_page_local_reference(page, object.id, *c)?,
                        ];
                        Self::append_multi_page_reference_group(&mut groups, object, code, &refs)?;
                    }
                    Kind::ArcOnCircle(circle, start, end) => {
                        if !self
                            .objects
                            .get(circle.saturating_sub(1))
                            .is_some_and(|o| !o.deleted && o.page == page && o.kind.is_circle())
                            || [*start, *end].iter().any(|id| {
                                !self.objects.get(id.saturating_sub(1)).is_some_and(|o| {
                                    !o.deleted && o.page == page && is_point_kind(&o.kind)
                                })
                            })
                        {
                            return Err("Arc needs a circle and two points on this page".into());
                        }
                        let refs = [
                            self.multi_page_local_reference(page, object.id, *circle)?,
                            self.multi_page_local_reference(page, object.id, *start)?,
                            self.multi_page_local_reference(page, object.id, *end)?,
                        ];
                        Self::append_multi_page_reference_group(&mut groups, object, 79, &refs)?;
                    }
                    Kind::SampledMapped(definition) => {
                        if !definition.supported(&self.objects, page, object.id) {
                            return Err("Invalid sampled mapped source roles".into());
                        }
                        let refs = definition
                            .native_references()
                            .into_iter()
                            .map(|id| self.multi_page_local_reference(page, object.id, id))
                            .collect::<Result<Vec<_>, _>>()?;
                        Self::append_multi_page_reference_group(&mut groups, object, 102, &refs)?;
                        groups.truncate(groups.len() - 8);
                        put_record(&mut groups, 0x0902, definition.range_payload());
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::SampledImage(definition) => {
                        if !definition.supported(&self.objects, page, object.id) {
                            return Err("Invalid sampled picture source roles".into());
                        }
                        let refs = definition
                            .native_references()
                            .into_iter()
                            .map(|id| self.multi_page_local_reference(page, object.id, id))
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut object_header = header(96);
                        object_header[2] = u8::from(!object.visible)
                            | (u8::from(object.presentation_suppressed) << 1);
                        object_header[4] = u8::from(object.tracing);
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = u32::try_from(refs.len())
                            .map_err(|_| "Too many sampled picture references")?
                            .to_le_bytes()
                            .to_vec();
                        for id in refs {
                            references.extend_from_slice(&id.to_le_bytes());
                        }
                        put_record(&mut groups, 0x07d2, &references);
                        let mut constraint = 96u32.to_le_bytes().to_vec();
                        constraint.extend_from_slice(&definition.resolution().to_le_bytes());
                        put_record(&mut groups, 0x07d3, &constraint);
                        put_record(&mut groups, 0x0902, definition.range_payload());
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::CircleInterior(circle) => {
                        if !super::circle_interior::supported_source(
                            &self.objects,
                            *circle,
                            page,
                            object.id,
                        ) {
                            return Err("Circle interior needs a live circle on this page".into());
                        }
                        let reference =
                            self.multi_page_local_reference(page, object.id, *circle)?;
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            10,
                            &[reference],
                        )?;
                    }
                    Kind::SectorBoundary(arc) => {
                        if !super::sector_boundary::supported_source(
                            &self.objects,
                            *arc,
                            page,
                            object.id,
                        ) {
                            return Err("Sector needs a live arc on this page".into());
                        }
                        let reference = self.multi_page_local_reference(page, object.id, *arc)?;
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            82,
                            &[reference],
                        )?;
                    }
                    Kind::CircularSegmentBoundary(arc) => {
                        if !super::sector_boundary::supported_circular_segment_source(
                            &self.objects,
                            *arc,
                            page,
                            object.id,
                        ) {
                            return Err("Circular segment needs a live arc on this page".into());
                        }
                        let reference = self.multi_page_local_reference(page, object.id, *arc)?;
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            83,
                            &[reference],
                        )?;
                    }
                    Kind::Polygon(vertices) => {
                        if vertices.len() < 3
                            || vertices.iter().collect::<BTreeSet<_>>().len() != vertices.len()
                            || vertices.iter().any(|id| {
                                !self
                                    .objects
                                    .get(id.saturating_sub(1))
                                    .is_some_and(|parent| is_point_kind(&parent.kind))
                            })
                            || self.polygon_points(object.id).is_none()
                        {
                            return Err("Polygon needs at least three distinct points".into());
                        }
                        let refs = vertices
                            .iter()
                            .map(|id| self.multi_page_local_reference(page, object.id, *id))
                            .collect::<Result<Vec<_>, _>>()?;
                        Self::append_multi_page_reference_group(&mut groups, object, 8, &refs)?;
                    }
                    Kind::SegmentLength(host)
                    | Kind::PolygonPerimeter(host)
                    | Kind::CircleCircumference(host)
                    | Kind::CircleRadius(host)
                    | Kind::CircleArea(host)
                    | Kind::PolygonArea(host)
                    | Kind::ArcAngle(host)
                    | Kind::ArcLength(host) => {
                        let host_kind = &self
                            .objects
                            .get(host.saturating_sub(1))
                            .ok_or("Missing measurement host")?
                            .kind;
                        let template: &[u8] = match &object.kind {
                            Kind::SegmentLength(_) if is_segment_kind(host_kind) => {
                                include_bytes!("../../../../assets/length_measurement_group.bin")
                            }
                            Kind::PolygonPerimeter(_)
                                if host_kind.is_polygon()
                                    || host_kind.is_sector()
                                    || host_kind.is_circular_segment() =>
                            {
                                include_bytes!("../../../../assets/perimeter_measurement_group.bin")
                            }
                            Kind::CircleCircumference(_) if host_kind.is_circle() => {
                                include_bytes!(
                                    "../../../../assets/circumference_measurement_group.bin"
                                )
                            }
                            Kind::CircleRadius(_)
                                if host_kind.is_circle()
                                    || host_kind.is_arc()
                                    || host_kind.is_sector() =>
                            {
                                include_bytes!("../../../../assets/radius_measurement_group.bin")
                            }
                            Kind::CircleArea(_) if host_kind.is_circle() => {
                                include_bytes!("../../../../assets/area_measurement_group.bin")
                            }
                            Kind::PolygonArea(_)
                                if host_kind.is_polygon()
                                    || host_kind.is_sector()
                                    || host_kind.is_circular_segment() =>
                            {
                                include_bytes!("../../../../assets/area_measurement_group.bin")
                            }
                            Kind::ArcAngle(_) if is_arc_kind(host_kind) => {
                                include_bytes!("../../../../assets/arc_angle_measurement_group.bin")
                            }
                            Kind::ArcLength(_)
                                if is_arc_kind(host_kind) || host_kind.is_sector() =>
                            {
                                include_bytes!(
                                    "../../../../assets/arc_length_measurement_group.bin"
                                )
                            }
                            _ => return Err("Measurement needs a compatible host".into()),
                        };
                        let local_host = self.multi_page_local_reference(page, object.id, *host)?;
                        let mut group = template.to_vec();
                        group[48..52].copy_from_slice(&local_host.to_le_bytes());
                        Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                        if !object.visible {
                            group[10] = 1;
                        }
                        groups.extend_from_slice(&group);
                    }
                    Kind::CircleArcLength(circle, start, end) => {
                        let parents = [*circle, *start, *end];
                        if !self
                            .objects
                            .get(circle.saturating_sub(1))
                            .is_some_and(|parent| parent.kind.is_circle())
                            || [start, end].iter().any(|id| {
                                !self
                                    .objects
                                    .get(id.saturating_sub(1))
                                    .is_some_and(|parent| is_point_kind(&parent.kind))
                            })
                        {
                            return Err("Arc length needs a circle and two points".into());
                        }
                        let local_parents = parents
                            .iter()
                            .map(|parent| self.multi_page_local_reference(page, object.id, *parent))
                            .collect::<Result<Vec<_>, _>>()?;
                        let (mut group, format_offset) =
                            Self::arc_length_measurement_group(&local_parents)?;
                        Self::place_measurement(&mut group, format_offset, &mut measurement_row)?;
                        if !object.visible {
                            group[10] = 1;
                        }
                        groups.extend_from_slice(&group);
                    }
                    Kind::PointDistance(a, b)
                    | Kind::PointLineDistance(a, b)
                    | Kind::SegmentRatio(a, b)
                    | Kind::AngleMeasurement(a, b, _)
                    | Kind::PointRatio(a, b, _) => {
                        let (parents, template, format_offset): (Vec<usize>, Option<&[u8]>, usize) =
                            match &object.kind {
                                Kind::PointDistance(_, _) => (
                                    vec![*a, *b],
                                    Some(include_bytes!(
                                        "../../../../assets/distance_measurement_group.bin"
                                    )),
                                    64,
                                ),
                                Kind::PointLineDistance(_, _) => (
                                    vec![*a, *b],
                                    Some(include_bytes!(
                                        "../../../../assets/point_line_distance_measurement_group.bin"
                                    )),
                                    64,
                                ),
                                Kind::SegmentRatio(_, _) => (vec![*a, *b], None, 64),
                                Kind::AngleMeasurement(_, _, c) => (
                                    vec![*a, *b, *c],
                                    Some(include_bytes!(
                                        "../../../../assets/angle_measurement_group.bin"
                                    )),
                                    68,
                                ),
                                Kind::PointRatio(_, _, c) => (
                                    vec![*a, *b, *c],
                                    Some(include_bytes!(
                                        "../../../../assets/ratio_measurement_group.bin"
                                    )),
                                    68,
                                ),
                                _ => unreachable!(),
                            };
                        if parents.iter().enumerate().any(|(index, id)| {
                            !self
                                .objects
                                .get(id.saturating_sub(1))
                                .is_some_and(|parent| {
                                    if matches!(object.kind, Kind::SegmentRatio(_, _)) {
                                        is_segment_kind(&parent.kind)
                                    } else if index == 1
                                        && matches!(object.kind, Kind::PointLineDistance(_, _))
                                    {
                                        is_straight_kind(&parent.kind)
                                    } else {
                                        is_point_kind(&parent.kind)
                                    }
                                })
                        }) {
                            return Err("Measurement needs point parents".into());
                        }
                        let mut local_parents = Vec::with_capacity(parents.len());
                        for parent in &parents {
                            let local =
                                self.multi_page_local_reference(page, object.id, *parent)?;
                            local_parents.push(local);
                        }
                        let (mut group, format_offset) =
                            if matches!(object.kind, Kind::SegmentRatio(_, _)) {
                                Self::ratio_measurement_group(&local_parents)?
                            } else {
                                let mut group = template.unwrap().to_vec();
                                for (index, local) in local_parents.iter().enumerate() {
                                    group[48 + index * 4..52 + index * 4]
                                        .copy_from_slice(&local.to_le_bytes());
                                }
                                (group, format_offset)
                            };
                        Self::place_measurement(&mut group, format_offset, &mut measurement_row)?;
                        if !object.visible {
                            group[10] = 1;
                        }
                        groups.extend_from_slice(&group);
                    }
                    Kind::LinkPageButton {
                        target_page,
                        target_button_ordinal,
                        label,
                        anchor,
                    } => {
                        if *target_page >= self.page_count || *target_page == page {
                            return Err("Invalid Page Link destination".into());
                        }
                        let destination_buttons = self.link_destination_buttons(*target_page);
                        if target_button_ordinal.is_some_and(|ordinal| {
                            ordinal == 0
                                || destination_buttons.get(ordinal - 1).is_none_or(|id| {
                                    !matches!(
                                        self.objects[id - 1].kind,
                                        Kind::HideShowButton { .. }
                                    )
                                })
                        }) {
                            return Err("Page Link destination button is unavailable".into());
                        }
                        let mut group = LinkPageButtonGroup::from_original_template(
                            *target_page,
                            label,
                            *anchor,
                        )?;
                        group.set_target_button_ordinal(*target_button_ordinal)?;
                        group.set_page_object_index(existing + added_index)?;
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                        if let Some(ordinal) = target_button_ordinal {
                            let destination = &self.objects[destination_buttons[ordinal - 1] - 1];
                            if destination.id <= self.source_count {
                                let header_index = *source_headers
                                    .get(destination.id - 1)
                                    .ok_or("Missing destination button group")?;
                                let action = rs[header_index + 1..]
                                    .iter()
                                    .take_while(|record| record.kind != 0x07d7)
                                    .find(|record| record.kind == 0x0906 && record.data.len() >= 20)
                                    .ok_or("Missing destination button action")?;
                                out[action.data.start + 18..action.data.start + 20]
                                    .copy_from_slice(&1u16.to_le_bytes());
                            }
                        }
                    }
                    Kind::LinkUrlButton { url, label, anchor } => {
                        let mut group =
                            LinkUrlButtonGroup::from_original_template(url, label, *anchor)?;
                        group.set_page_object_index(existing + added_index)?;
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                    }
                    Kind::HideShowButton {
                        targets,
                        action,
                        options,
                        caption_flags,
                        label,
                        anchor,
                    } => {
                        if targets.is_empty()
                            || targets.iter().any(|target| {
                                self.objects
                                    .get(target.saturating_sub(1))
                                    .is_none_or(|source| {
                                        source.deleted
                                            || source.page != page
                                            || matches!(source.kind, Kind::Unsupported(_))
                                    })
                            })
                        {
                            return Err("Invalid Hide/Show targets on this page".into());
                        }
                        let ids = targets
                            .iter()
                            .map(|&target| self.multi_page_local_reference(page, object.id, target))
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut group = HideShowButtonGroup::from_original_template_targets(
                            &ids, label, *anchor,
                        )?;
                        group.set_action(*action);
                        group.set_options(*options);
                        group.set_caption_flags(*caption_flags)?;
                        group.set_hidden(!object.visible);
                        group.set_presentation_suppressed(object.presentation_suppressed);
                        let destination_buttons = self.link_destination_buttons(page);
                        let linked = self.objects.iter().any(|source| {
                            matches!(
                                source.kind,
                                Kind::LinkPageButton {
                                    target_page,
                                    target_button_ordinal: Some(ordinal),
                                    ..
                                } if target_page == page
                                    && ordinal.checked_sub(1)
                                        .and_then(|index| destination_buttons.get(index))
                                        == Some(&object.id)
                            )
                        });
                        group.set_link_destination(linked);
                        groups.extend_from_slice(&group.encode());
                    }
                    Kind::AnimationButton {
                        targets,
                        label,
                        anchor,
                        settings,
                    } => {
                        if targets.is_empty() || targets.len() != settings.len() {
                            return Err("Invalid Animation targets on this page".into());
                        }
                        let ids = targets
                            .iter()
                            .map(|&target| self.multi_page_local_reference(page, object.id, target))
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut group = AnimationButtonGroup::from_original_template_targets(
                            &ids, label, *anchor,
                        )?;
                        group.set_settings_list(settings)?;
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                    }
                    Kind::SoundButton {
                        functions,
                        label,
                        anchor,
                    } => {
                        if !(1..=2).contains(&functions.len())
                            || functions.iter().any(|function| {
                                self.objects
                                    .get(function.saturating_sub(1))
                                    .is_none_or(|parent| !parent.kind.is_function())
                            })
                        {
                            return Err("Invalid Sound functions on this page".into());
                        }
                        let ids = functions
                            .iter()
                            .map(|&function| {
                                self.multi_page_local_reference(page, object.id, function)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut group =
                            SoundButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                    }
                    Kind::MovementButton {
                        pairs,
                        speed,
                        follow,
                        label,
                        anchor,
                    } => {
                        if pairs.is_empty()
                            || pairs.iter().any(|&(s, d)| !self.movement_pair_valid(s, d))
                        {
                            return Err("Invalid Movement pairs on this page".into());
                        }
                        let local_pairs = pairs
                            .iter()
                            .map(|&(s, d)| {
                                Ok((
                                    self.multi_page_local_reference(page, object.id, s)?,
                                    self.multi_page_local_reference(page, object.id, d)?,
                                ))
                            })
                            .collect::<Result<Vec<_>, String>>()?;
                        let mut group = MovementButtonGroup::from_original_template_pairs(
                            &local_pairs,
                            label,
                            *anchor,
                        )?;
                        group.set_speed(*speed);
                        group.set_follows_destination(*follow);
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                    }
                    Kind::ScrollButton {
                        target,
                        placement,
                        label,
                        anchor,
                    } => {
                        if !self
                            .objects
                            .get(target.saturating_sub(1))
                            .is_some_and(|point| is_point_kind(&point.kind))
                            || self.position(*target).is_none()
                        {
                            return Err("Invalid Scroll point on this page".into());
                        }
                        let local_target =
                            self.multi_page_local_reference(page, object.id, *target)?;
                        let mut group = ScrollButtonGroup::from_original_template(
                            local_target,
                            label,
                            *anchor,
                        )?;
                        group.set_placement(*placement);
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                    }
                    Kind::PresentationButton {
                        targets,
                        mode,
                        options,
                        label,
                        anchor,
                    } => {
                        if targets.is_empty()
                            || options.native_state as usize > targets.len()
                            || (*mode == crate::action_records::PresentationMode::Sequential
                                && options.stop
                                    != crate::action_records::PresentationStop::LastAction)
                            || targets.iter().any(|target| {
                                self.objects
                                    .get(target.saturating_sub(1))
                                    .is_none_or(|parent| !parent.kind.is_action_button())
                            })
                        {
                            return Err("Invalid Presentation buttons on this page".into());
                        }
                        let ids = targets
                            .iter()
                            .map(|&target| self.multi_page_local_reference(page, object.id, target))
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut group = PresentationButtonGroup::from_original_template_targets(
                            &ids, label, *anchor,
                        )?;
                        group.set_options(*mode, *options);
                        group.set_hidden(!object.visible);
                        group.set_presentation_suppressed(object.presentation_suppressed);
                        groups.extend_from_slice(&group.encode());
                    }
                    _ => return Err("Expected supported multi-page addition".into()),
                }
            }
            let count = u32::try_from(
                existing
                    + self.objects[self.source_count..]
                        .iter()
                        .filter(|object| object.page == page)
                        .count(),
            )
            .map_err(|_| "Too many objects on page")?;
            out[preamble.data.start + 24..preamble.data.start + 28]
                .copy_from_slice(&count.to_le_bytes());
            insertions.push((insert_at, groups));
        }
        for (insert_at, groups) in insertions.into_iter().rev() {
            out.splice(insert_at..insert_at, groups);
        }
        records(&out)?;
        self.patch_multi_page_text(out)
    }
    pub(super) fn encode_existing_state(&self) -> Result<Vec<u8>, String> {
        if self.source_count != self.objects.len() {
            return Err("Multi-page state edit cannot add objects".into());
        }
        self.patch_multi_page_text(self.encode_existing_source_state()?)
    }
    fn patch_multi_page_text(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        // Serialized groups are ordered by page, while additions have global IDs.
        let mut ordered = self.clone();
        let mut objects = ordered.objects.to_vec();
        objects.sort_by_key(|object| (object.page, object.id > self.source_count, object.id));
        ordered.objects = objects.into();
        let rs = records(&bytes)?;
        let mut edits = Vec::new();
        let starts = document_record_indices(&rs)
            .into_iter()
            .filter(|&index| rs[index].kind == 0x07d0)
            .collect::<Vec<_>>();
        if starts.len() != ordered.objects.len() {
            return Err("Multi-page object group count mismatch".into());
        }
        for (object, start) in ordered.objects.iter().zip(starts) {
            let Kind::PlainText { text, rect } = &object.kind else {
                continue;
            };
            let group = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x07d7)
                .collect::<Vec<_>>();
            let header = &rs[start];
            let rich = group
                .iter()
                .find(|record| record.kind == 0x08fc)
                .ok_or("Missing free-text payload")?;
            let bbox = group
                .iter()
                .find(|record| record.kind == 0x0898)
                .ok_or("Missing free-text box")?;
            if bytes[header.data.clone()] == crate::text_object::LEGACY_TEXT_HEADER
                && object
                    .rich_text_override
                    .as_ref()
                    .is_some_and(|template| !template.is_legacy_gbk())
            {
                let mut replacement = Vec::new();
                put_record(
                    &mut replacement,
                    0x07d0,
                    &crate::text_object::modern_text_header(
                        object.visible,
                        object.presentation_suppressed,
                    ),
                );
                edits.push((header.start, header.data.end, replacement));
            }
            let (old_text, old_rect) = crate::text_object::decode_plain_text(
                &bytes[rich.data.clone()],
                &bytes[bbox.data.clone()],
            )?;
            if old_text == *text && old_rect == *rect && object.rich_text_override.is_none() {
                continue;
            }
            let (new_rich, new_bbox) = if let Some(template) = &object.rich_text_override {
                template.encode_parts(text, *rect)?
            } else {
                crate::text_object::encode_from_template(
                    &bytes[rich.data.clone()],
                    &bytes[bbox.data.clone()],
                    text,
                    *rect,
                )?
            };
            for (record, payload) in [(*rich, new_rich), (*bbox, new_bbox.to_vec())] {
                if bytes[record.data.clone()] != payload {
                    let mut replacement = Vec::new();
                    put_record(&mut replacement, record.kind, &payload);
                    edits.push((
                        record.start,
                        record.data.end + record.data.len() % 2,
                        replacement,
                    ));
                }
            }
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
        for (start, end, replacement) in edits {
            bytes.splice(start..end, replacement);
        }
        ordered.patch_text_font_overrides(bytes)
    }
    fn encode_existing_source_state(&self) -> Result<Vec<u8>, String> {
        let original = Self::from_bytes(&self.original, None)?;
        if self.source_count > self.objects.len()
            || original.objects.len() != self.source_count
            || original.page_count != self.page_count
            || original.snap_points != self.snap_points
            || original.grid_dotted != self.grid_dotted
            || original.grid_form != self.grid_form
            || original.angle_unit_radians != self.angle_unit_radians
            || original.marked_coordinate_system != self.marked_coordinate_system
            || original.frame_styles != self.frame_styles
        {
            return Err("Multi-page state edit changes the document structure".into());
        }
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        let mut numeric_edits = Vec::new();
        let changed_parameters = original
            .objects
            .iter()
            .zip(&self.objects[..self.source_count])
            .filter_map(|(saved, current)| match (&saved.kind, &current.kind) {
                (Kind::Parameter(before), Kind::Parameter(after)) if before != after => Some((
                    current.id,
                    (current.page, after.name().to_owned(), after.value()),
                )),
                _ => None,
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for (saved, current) in original
            .objects
            .iter()
            .zip(&self.objects[..self.source_count])
        {
            let mut expected = saved.clone();
            expected.visible = current.visible;
            if saved.measurement_anchor.is_some() {
                expected.measurement_anchor = current.measurement_anchor;
            }
            if matches!(
                (&saved.kind, &current.kind),
                (Kind::PlainText { .. }, Kind::PlainText { .. })
            ) {
                expected.kind = current.kind.clone();
                expected.rich_text_override = current.rich_text_override.clone();
            }
            let numeric_replacement = match (&saved.kind, &current.kind) {
                (Kind::Parameter(before), Kind::Parameter(after)) if before != after => {
                    let value_only = [false, true].into_iter().any(|animated| {
                        let set = |group: &mut crate::table_records::ParameterGroup, value| {
                            if animated {
                                group.set_animated_value(value)
                            } else {
                                group.set_value(value)
                            }
                        };
                        let mut changed = before.clone();
                        if set(&mut changed, after.value()).is_ok() && changed == *after {
                            return true;
                        }
                        // A previous fractional edit can raise precision permanently.
                        changed = before.clone();
                        after.precision() >= before.precision()
                            && set(&mut changed, 10_f64.powi(-(after.precision() as i32))).is_ok()
                            && set(&mut changed, after.value()).is_ok()
                            && changed == *after
                    });
                    if !value_only {
                        let mut changed = before.clone();
                        changed.set_value(after.value())?;
                        changed.set_precision(after.precision())?;
                        changed.set_display_mode(after.display_mode())?;
                        changed.set_includes_edit_box(after.includes_edit_box())?;
                        changed.set_animation_settings(after.animation_settings())?;
                        if changed != *after {
                            return Err("Parameter edit changes unsupported records".into());
                        }
                    }
                    expected.kind = current.kind.clone();
                    Some(after.as_bytes().to_vec())
                }
                (Kind::ValueTable(old_source, before), Kind::ValueTable(source, after))
                    if old_source == source && before != after =>
                {
                    let (page, name, value) = changed_parameters
                        .get(source)
                        .ok_or("Multi-page table edit requires its parameter value change")?;
                    if *page != current.page || name != "t[1]" || before.default_values().is_none()
                    {
                        return Err("Unsupported multi-page parameter table edit".into());
                    }
                    let mut changed = before.clone();
                    changed.set_last_default_value(*value)?;
                    if changed != *after {
                        return Err("Multi-page table edit changes unsupported rows".into());
                    }
                    expected.kind = current.kind.clone();
                    Some(after.as_bytes().to_vec())
                }
                (
                    Kind::ValueTablePair(old_first, old_second, before),
                    Kind::ValueTablePair(first, second, after),
                ) if old_first == first && old_second == second && before != after => {
                    let mut changed = before.clone();
                    let mut updated = false;
                    for (axis, source) in [first, second].iter().enumerate() {
                        if let Some((page, name, value)) = changed_parameters.get(source) {
                            if *page != current.page || !matches!(name.as_str(), "t[1]" | "t[2]") {
                                return Err("Unsupported multi-page parameter table edit".into());
                            }
                            changed.set_last_pair_axis_value(axis, *value)?;
                            updated = true;
                        }
                    }
                    if !updated || changed != *after {
                        return Err("Multi-page table edit changes unsupported rows".into());
                    }
                    expected.kind = current.kind.clone();
                    Some(after.as_bytes().to_vec())
                }
                (before, after)
                    if super::calculations::calculation_bytes(before).is_some()
                        && super::calculations::calculation_bytes(after).is_some()
                        && before != after =>
                {
                    let old = super::calculations::calculation_bytes(before).unwrap();
                    let new = super::calculations::calculation_bytes(after).unwrap();
                    if before.dependencies() != after.dependencies()
                        || before.numeric_unit() != after.numeric_unit()
                        || !crate::table_records::calculation_expression_only_edit(old, new)?
                    {
                        return Err("Calculation edit changes unsupported records".into());
                    }
                    expected.kind = current.kind.clone();
                    Some(new.to_vec())
                }
                _ => None,
            };
            if matches!(saved.kind, Kind::PlainText { .. })
                || (is_point_kind(&saved.kind) && saved.label.is_some())
                || saved.kind.numeric_unit().is_some()
                || matches!(
                    saved.kind,
                    Kind::ValueEquation { .. } | Kind::CoordinateMeasurement { .. }
                )
            {
                expected.text_style_override = current.text_style_override.clone();
            }

            if saved.kind.has_measurement_precision()
                && current
                    .measurement_precision
                    .is_some_and(|precision| precision <= 5)
            {
                expected.measurement_precision = current.measurement_precision;
            }
            if saved.visual_style.is_some() {
                expected.tracing = current.tracing;
            }
            if saved.visual_style.is_some() && current.visual_style.is_some() {
                expected.visual_style = current.visual_style;
            }
            if saved.kind.is_filled_region() && current.interior_order.is_some() {
                expected.interior_order = current.interior_order;
            }
            if let (Kind::Free(_), Kind::Free(point)) = (&saved.kind, &current.kind) {
                if !point.x.is_finite() || !point.y.is_finite() {
                    return Err("Multi-page point coordinates must be finite".into());
                }
                expected.kind = current.kind.clone();
            }
            if let (Kind::CoordinatePoint(old), Kind::CoordinatePoint(new)) =
                (&saved.kind, &current.kind)
                && old.frame() == new.frame()
                && old.wire_flavor() == new.wire_flavor()
                && old.native_references() == new.native_references()
            {
                expected.kind = current.kind.clone();
            }
            match (&saved.kind, &current.kind) {
                (Kind::PointOnSegment(old_host, _), Kind::PointOnSegment(host, t))
                    if old_host == host && t.is_finite() && (0.0..=1.0).contains(t) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnRay(old_host, _), Kind::PointOnRay(host, t))
                    if old_host == host && t.is_finite() && *t >= 0. =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnLine(old_host, _), Kind::PointOnLine(host, t))
                    if old_host == host && t.is_finite() =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnArc(old_host, _), Kind::PointOnArc(host, t))
                    if old_host == host && t.is_finite() && (0.0..=1.0).contains(t) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnSector(old_host, _), Kind::PointOnSector(host, t))
                    if old_host == host
                        && t.is_finite()
                        && super::sector_boundary::supported_host(
                            &self.objects,
                            *host,
                            current.page,
                            current.id,
                        ) =>
                {
                    expected.kind = current.kind.clone();
                }
                (
                    Kind::PointOnCircularSegment(old_host, _),
                    Kind::PointOnCircularSegment(host, t),
                ) if old_host == host
                    && (0.0..=1.0).contains(t)
                    && super::sector_boundary::supported_circular_segment_host(
                        &self.objects,
                        *host,
                        current.page,
                        current.id,
                    ) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnCurve(old_host, _), Kind::PointOnCurve(host, t))
                    if old_host == host
                        && t.is_finite()
                        && super::curve_points::free_supported(
                            &self.objects,
                            *host,
                            current.page,
                            current.id,
                        ) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnPolygon(old_host, _, _), Kind::PointOnPolygon(host, edge, t))
                    if old_host == host
                        && t.is_finite()
                        && (0.0..=1.0).contains(t)
                        && self
                            .polygon_points(*host)
                            .is_some_and(|vertices| *edge < vertices.len()) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnCircle(old_host, _, _), Kind::PointOnCircle(host, x, y))
                    if old_host == host
                        && x.is_finite()
                        && y.is_finite()
                        && (x * x + y * y - 1.0).abs() <= 1e-6 =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::SampledImage(before), Kind::SampledImage(after))
                    if before.same_structure(after) =>
                {
                    expected.kind = current.kind.clone();
                }
                _ => {}
            }
            let compatible_action_edit = match (&saved.kind, &current.kind) {
                (
                    Kind::HideShowButton {
                        targets: before,
                        anchor: before_anchor,
                        ..
                    },
                    Kind::HideShowButton {
                        targets: after,
                        anchor: after_anchor,
                        ..
                    },
                )
                | (
                    Kind::AnimationButton {
                        targets: before,
                        anchor: before_anchor,
                        ..
                    },
                    Kind::AnimationButton {
                        targets: after,
                        anchor: after_anchor,
                        ..
                    },
                )
                | (
                    Kind::PresentationButton {
                        targets: before,
                        anchor: before_anchor,
                        ..
                    },
                    Kind::PresentationButton {
                        targets: after,
                        anchor: after_anchor,
                        ..
                    },
                )
                | (
                    Kind::SoundButton {
                        functions: before,
                        anchor: before_anchor,
                        ..
                    },
                    Kind::SoundButton {
                        functions: after,
                        anchor: after_anchor,
                        ..
                    },
                ) => before == after && before_anchor == after_anchor,
                (
                    Kind::MovementButton {
                        pairs: before,
                        anchor: before_anchor,
                        ..
                    },
                    Kind::MovementButton {
                        pairs: after,
                        anchor: after_anchor,
                        ..
                    },
                ) => before == after && before_anchor == after_anchor,
                (
                    Kind::ScrollButton {
                        target: before,
                        anchor: before_anchor,
                        ..
                    },
                    Kind::ScrollButton {
                        target: after,
                        anchor: after_anchor,
                        ..
                    },
                ) => before == after && before_anchor == after_anchor,
                (
                    Kind::LinkUrlButton { anchor: before, .. }
                    | Kind::LinkPageButton { anchor: before, .. },
                    Kind::LinkUrlButton { anchor: after, .. }
                    | Kind::LinkPageButton { anchor: after, .. },
                ) => before == after,
                _ => false,
            };
            if compatible_action_edit {
                expected.kind = current.kind.clone();
            }
            if expected != *current
                || matches!(current.kind, Kind::Unsupported(_)) && saved.visible != current.visible
            {
                return Err("Multi-page state edit changes unsupported object data".into());
            }
            let header = current.header_offset.ok_or("Missing object header")?;
            if saved.interior_order != current.interior_order {
                if !saved.kind.is_filled_region() {
                    return Err("Cannot change drawing order of an unsupported interior".into());
                }
                if !rs.iter().any(|record| {
                    record.kind == 0x07d0 && record.data.start == header && record.data.len() == 28
                }) {
                    return Err("Polygon drawing order is unavailable in this header".into());
                }
                out[header + 20..header + 24].copy_from_slice(
                    &current
                        .interior_order
                        .ok_or("Missing polygon drawing order")?
                        .to_le_bytes(),
                );
            }
            if saved.tracing != current.tracing {
                let record = rs
                    .iter()
                    .find(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing traced object header")?;
                if record.data.len() < 5 {
                    return Err("Short traced object header".into());
                }
                out[header + 4] = (out[header + 4] & !1) | u8::from(current.tracing);
            }
            if saved.visual_style != current.visual_style {
                let record = rs
                    .iter()
                    .find(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing styled object header")?;
                let style = current.visual_style.ok_or("Missing object style")?;
                style_records::set_visual_style(&mut out[record.data.clone()], style)
                    .map_err(|error| format!("Invalid style in object {}: {error}", current.id))?;
            }
            if let (Kind::Free(before), Kind::Free(point)) = (&saved.kind, &current.kind)
                && before != point
            {
                let at = current
                    .point_offset
                    .ok_or("Missing free-point coordinates")?;
                out[at..at + 8].copy_from_slice(&point.x.to_le_bytes());
                out[at + 8..at + 16].copy_from_slice(&point.y.to_le_bytes());
            }
            if let Kind::CoordinatePoint(d) = &current.kind {
                let at = current
                    .constraint_offset
                    .ok_or("Missing coordinate-point payload")?;
                if out.get(at..at + 4) != Some(&d.native_payload()[..4]) {
                    return Err("Invalid coordinate-point opcode".into());
                }
                out.get_mut(at..at + 12)
                    .ok_or("Short coordinate-point payload")?
                    .copy_from_slice(&d.native_payload());
            }
            if saved.kind != current.kind
                && matches!(
                    current.kind,
                    Kind::PointOnSegment(_, _)
                        | Kind::PointOnRay(_, _)
                        | Kind::PointOnLine(_, _)
                        | Kind::PointOnArc(_, _)
                        | Kind::PointOnSector(_, _)
                        | Kind::PointOnCircularSegment(_, _)
                        | Kind::PointOnCurve(_, _)
                        | Kind::PointOnPolygon(_, _, _)
                        | Kind::PointOnCircle(_, _, _)
                )
            {
                let at = current
                    .constraint_offset
                    .ok_or("Missing point constraint")?;
                match &current.kind {
                    Kind::PointOnSegment(_, t) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 14, 0]) {
                            return Err("Invalid segment-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short segment-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnRay(host, t) | Kind::PointOnLine(host, t) => {
                        let payload = out
                            .get(at..at + 4)
                            .ok_or("Short straight-point constraint")?;
                        if !super::kind::straight_point_constraint_matches(
                            &self.objects[*host - 1].kind,
                            payload,
                        ) && !(matches!(current.kind, Kind::PointOnLine(_, _))
                            && payload == [15, 0, 12, 0])
                        {
                            return Err("Invalid straight-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short straight-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnArc(_, t) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 67, 0]) {
                            return Err("Invalid arc-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short arc-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnSector(_, t) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 68, 0]) {
                            return Err("Invalid sector-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short sector-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnCircularSegment(_, t) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 69, 0]) {
                            return Err("Invalid circular-segment-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short circular-segment-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnCurve(host, t) => {
                        let subtype =
                            super::curve_points::free_constraint_subtype(&self.objects, *host)
                                .ok_or("Invalid curve-point host")?;
                        if out.get(at..at + 4) != Some(&[15, 0, subtype, 0]) {
                            return Err("Invalid curve-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short curve-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnPolygon(_, edge, t) => {
                        if out.get(at..at + 2) != Some(&[15, 0]) {
                            return Err("Invalid polygon-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short polygon-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                        out.get_mut(at + 12..at + 14)
                            .ok_or("Short polygon-point constraint")?
                            .copy_from_slice(&(*edge as u16).to_le_bytes());
                    }
                    Kind::PointOnCircle(_, x, y) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 28, 0]) {
                            return Err("Invalid circle-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short circle-point constraint")?
                            .copy_from_slice(&x.to_le_bytes());
                        out.get_mut(at + 12..at + 20)
                            .ok_or("Short circle-point constraint")?
                            .copy_from_slice(&y.to_le_bytes());
                    }
                    _ => {}
                }
            }
            if saved.visible != current.visible {
                let mut class = u32_at(&out, header);
                if current.visible {
                    class &= if matches!(current.kind, Kind::PathValuePoint(..)) {
                        !0x5_0000
                    } else {
                        !0x1_0000
                    };
                } else {
                    class |= 0x1_0000;
                }
                out[header..header + 4].copy_from_slice(&class.to_le_bytes());
            }
            if let Some(mut replacement) = numeric_replacement {
                let start_index = rs
                    .iter()
                    .position(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing saved numeric group header")?;
                let end_record = rs[start_index..]
                    .iter()
                    .find(|record| record.kind == 0x07d7)
                    .ok_or("Missing saved numeric group terminator")?;
                let start = rs[start_index].start;
                let end = end_record.data.end + end_record.data.len() % 2;
                if replacement.len() < 36 || rs[start_index].data.len() != 28 {
                    return Err("Invalid saved numeric group header".into());
                }
                replacement[..36].copy_from_slice(&out[start..header + 28]);
                numeric_edits.push((start, end, replacement));
            }
        }
        for (start, end, replacement) in numeric_edits.into_iter().rev() {
            out.splice(start..end, replacement);
        }
        self.patch_action_source_groups(out)
    }
    pub(super) fn encode_page_names(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let rs = records(&bytes)?;
        let slots = page_title_slots(&bytes, &rs)?;
        if slots.len() != self.page_count || self.page_names.len() != self.page_count {
            return Err("Page name count mismatch".into());
        }
        for page in (0..self.page_count).rev() {
            let (title, trailer, actual) = &slots[page];
            let desired = &self.page_names[page];
            if actual == desired {
                continue;
            }
            let mut replacement = Vec::new();
            if let Some(name) = desired {
                if name.as_bytes().contains(&0) || name.len() > 255 {
                    return Err("Invalid page name".into());
                }
                let mut payload = name.as_bytes().to_vec();
                payload.push(0);
                if !payload.len().is_multiple_of(2) {
                    payload.push(0);
                }
                put_record(&mut replacement, 0x044c, &payload);
            }
            let (start, end) = if let Some(index) = title {
                (rs[*index].start, rs[*index + 1].start)
            } else {
                (rs[*trailer].start, rs[*trailer].start)
            };
            bytes.splice(start..end, replacement);
        }
        records(&bytes)?;
        Ok(bytes)
    }
}
