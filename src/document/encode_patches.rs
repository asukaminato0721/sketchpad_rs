use super::calculations::calculation_source_ids;
use super::embedded_tools::{document_record_indices, document_records};
use super::kind::is_point_kind;
use super::records::{put_record, records};
use super::{Document, Kind, Object};
use crate::plot_points_records::PlotPointsMode;
use crate::{function_records, style_records, text_object, text_style_records};

impl Document {
    pub(super) fn patch_marked_coordinate_link(&self, mut out: Vec<u8>) -> Result<Vec<u8>, String> {
        let Some(frame) = self.active_coordinate_system() else {
            return Ok(out);
        };
        let frame = u32::try_from(frame).map_err(|_| "Too many objects")?;
        let rs = records(&out)?;
        if let Some(link) = document_records(&rs).find(|record| record.kind == 0x232d) {
            if link.data.len() != 4 {
                return Err("Invalid coordinate-system page link".into());
            }
            out[link.data.clone()].copy_from_slice(&frame.to_le_bytes());
        } else {
            let trailer = rs
                .iter()
                .find(|record| record.kind == 0x03e9)
                .ok_or("Missing document trailer")?
                .start;
            let mut link = Vec::new();
            put_record(&mut link, 0x232d, &frame.to_le_bytes());
            out.splice(trailer..trailer, link);
        }
        Ok(out)
    }
    pub(super) fn patch_defined_coordinate_grid_style(
        &self,
        mut out: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        if !self.objects.iter().skip(self.source_count).any(|object| {
            !object.deleted
                && matches!(
                    object.kind,
                    Kind::CircleAxis(_, _) | Kind::DistanceAxis(_, _, _)
                )
        }) {
            return Ok(out);
        }
        let rs = records(&out)?;
        let styles: Vec<_> = document_records(&rs)
            .filter(|record| record.kind == 0x04b3)
            .collect();
        if styles.iter().any(|record| {
            record.data.len() >= 4 && out[record.data.start + 2..record.data.start + 4] == [14, 0]
        }) {
            return Ok(out);
        }
        // Sketchpad adds this page style for circle- and distance-defined frames.
        // Without it the native app displays the axes but omits the grid lines.
        let mut style = Vec::new();
        put_record(
            &mut style,
            0x04b3,
            &[0, 0, 14, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
        );
        let anchor = styles
            .last()
            .copied()
            .or_else(|| document_records(&rs).find(|record| record.kind == 0x04b1))
            .ok_or("Missing page style records")?;
        let at = anchor.data.end + anchor.data.len() % 2;
        out.splice(at..at, style);
        Ok(out)
    }
    pub(super) fn patch_active_page(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        if self.active_page >= self.page_count {
            return Err("Selected page is outside this document".into());
        }
        let rs = records(&bytes)?;
        let Some(record) = rs.iter().find(|record| record.kind == 0x0384) else {
            return if self.active_page == 0 {
                Ok(bytes)
            } else {
                Err("Missing page selection record".into())
            };
        };
        if record.data.len() != 28
            || usize::from(u16::from_le_bytes(
                bytes[record.data.start + 2..record.data.start + 4]
                    .try_into()
                    .unwrap(),
            )) != self.page_count
        {
            return Err("Invalid page selection record".into());
        }
        let page = u16::try_from(self.active_page).map_err(|_| "Too many pages")?;
        bytes[record.data.start + 16..record.data.start + 18].copy_from_slice(&page.to_le_bytes());
        Ok(bytes)
    }
    pub(super) fn patch_measurement_precisions(
        &self,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        if !self.dirty {
            return Ok(bytes);
        }
        let rs = records(&bytes)?;
        let mut objects = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect::<Vec<_>>();
        objects.sort_by_key(|object| object.page);
        let mut objects = objects.into_iter();
        let mut current = None;
        for record in document_records(&rs) {
            if record.kind == 0x07d0 {
                current = Some(
                    objects
                        .next()
                        .ok_or("Object group count mismatch in patch_measurement_precisions")?,
                );
            } else if record.kind == 0x0903 {
                if let Some(object) = current {
                    // Unsupported groups retain their source records. Their
                    // numeric-looking metadata does not authorize rewriting a
                    // measurement format that has not been decoded.
                    if matches!(object.kind, Kind::Unsupported(_)) {
                        continue;
                    }
                    if let Some(anchor) = object.measurement_anchor {
                        if record.data.len() != 32 {
                            return Err("Invalid measurement caption record".into());
                        }
                        bytes[record.data.start + 12..record.data.start + 14]
                            .copy_from_slice(&anchor.0.to_le_bytes());
                        bytes[record.data.start + 14..record.data.start + 16]
                            .copy_from_slice(&anchor.1.to_le_bytes());
                    }
                    if let Some(precision) = object.measurement_precision {
                        if !object.kind.has_measurement_precision()
                            || precision > 5
                            || record.data.len() != 32
                        {
                            return Err("Invalid measurement precision record".into());
                        }
                        bytes[record.data.start + 28..record.data.start + 30]
                            .copy_from_slice(&precision.to_le_bytes());
                    }
                }
            } else if record.kind == 0x07d7 {
                current = None;
            }
        }
        if objects.next().is_some() {
            return Err("Object group count mismatch in patch_measurement_precisions".into());
        }
        Ok(bytes)
    }
    pub(super) fn patch_last_moved_marker(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let Some(id) = self.last_moved_object else {
            return Ok(bytes);
        };
        if !self.objects.iter().any(|object| {
            object
                .kind
                .transform_definition()
                .is_some_and(|definition| {
                    definition.element() == super::transform_model::ElementType::Polygon
                })
        }) {
            return Ok(bytes);
        }
        let entries = records(&bytes)?;
        if let Some(marker) = document_records(&entries)
            .find(|record| record.kind == 0x2328 && record.data.len() == 4)
        {
            bytes[marker.data.clone()].copy_from_slice(&(id as u32).to_le_bytes());
        }
        Ok(bytes)
    }
    /// Resolve a caption style before storing its font reference in a native group.
    /// Pending overrides are appended in object order by the font patcher.
    pub(super) fn imported_caption_font_index(
        &self,
        style: &text_style_records::FontState,
    ) -> Result<u16, String> {
        let entries = records(&self.original)?;
        let count = document_records(&entries)
            .filter(|record| record.kind == 0x04b3)
            .count();
        let mut known = (0..count)
            .map(|index| text_style_records::decode_font_state_at(&self.original, index).ok())
            .collect::<Vec<_>>();
        if !known.iter().any(Option::is_some) {
            return Err("No supported font style template".into());
        }
        for pending in self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .filter_map(|object| object.text_style_override.as_ref())
        {
            if !known.iter().any(|entry| entry.as_ref() == Some(pending)) {
                known.push(Some(pending.clone()));
            }
        }
        let index = known
            .iter()
            .position(|entry| entry.as_ref() == Some(style))
            .unwrap_or(known.len());
        u16::try_from(index).map_err(|_| "Too many text styles".into())
    }

    pub(super) fn patch_text_font_overrides(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let overrides = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .enumerate()
            .filter_map(|(ordinal, object)| {
                object
                    .text_style_override
                    .as_ref()
                    .map(|style| (ordinal, object, style))
            })
            .collect::<Vec<_>>();
        if overrides.is_empty() && self.run_font_families.is_empty() {
            return Ok(bytes);
        }
        if overrides.iter().any(|(_, object, _)| {
            !matches!(object.kind, Kind::PlainText { .. })
                && !(is_point_kind(&object.kind) && object.label.is_some())
                && object.kind.numeric_unit().is_none()
                && !matches!(
                    object.kind,
                    Kind::ValueTable(_, _)
                        | Kind::ValueTablePair(_, _, _)
                        | Kind::ValueEquation { .. }
                        | Kind::CoordinateMeasurement { .. }
                        | Kind::PointNumericLabel(_, _)
                        | Kind::ReferencedText { .. }
                )
        }) {
            return Err("Object has no supported caption font style".into());
        }
        let rs = records(&bytes)?;
        let last_family = document_records(&rs)
            .filter(|record| record.kind == 0x04b1)
            .last()
            .ok_or("Missing font family table")?;
        let style_records = document_records(&rs)
            .filter(|record| record.kind == 0x04b3)
            .collect::<Vec<_>>();
        style_records.last().ok_or("Missing font style table")?;
        let mut known = (0..style_records.len())
            .map(|index| text_style_records::decode_font_state_at(&bytes, index).ok())
            .collect::<Vec<_>>();
        let mut families = text_style_records::font_families(&bytes)?;
        let mut appended_families = Vec::new();
        let mut appended = Vec::new();
        let mut targets = Vec::new();
        for family in &self.run_font_families {
            if families.contains(family) {
                return Err("Duplicate pending text font family".into());
            }
            if families.len() > u16::MAX as usize {
                return Err("Too many font families".into());
            }
            let payload = text_style_records::encode_font_family_payload(family)?;
            put_record(&mut appended_families, 0x04b1, &payload);
            families.push(family.clone());
        }
        for (ordinal, _, style) in overrides {
            let family_index =
                if let Some(index) = families.iter().position(|name| name == &style.family) {
                    index
                } else {
                    let index = families.len();
                    if index > u16::MAX as usize {
                        return Err("Too many font families".into());
                    }
                    let payload = text_style_records::encode_font_family_payload(&style.family)?;
                    put_record(&mut appended_families, 0x04b1, &payload);
                    families.push(style.family.clone());
                    index
                };
            let index =
                if let Some(index) = known.iter().position(|known| known.as_ref() == Some(style)) {
                    index
                } else {
                    let family_template = known
                        .iter()
                        .take(style_records.len())
                        .position(|known| {
                            known
                                .as_ref()
                                .is_some_and(|known| known.family == style.family)
                        })
                        .or_else(|| known.iter().position(Option::is_some))
                        .ok_or("No supported font style template")?;
                    let index = known.len();
                    if index > u16::MAX as usize {
                        return Err("Too many text styles".into());
                    }
                    let template = &bytes[style_records[family_template].data.clone()];
                    let mut payload = text_style_records::encode_style_payload(template, style)?;
                    payload[..2].copy_from_slice(&(family_index as u16).to_le_bytes());
                    put_record(&mut appended, 0x04b3, &payload);
                    known.push(Some(style.clone()));
                    index
                };
            targets.push((ordinal, index as u16));
        }
        if !appended_families.is_empty() {
            let insert_at = last_family.data.end + last_family.data.len() % 2;
            bytes.splice(insert_at..insert_at, appended_families);
        }
        if !appended.is_empty() {
            let rs = records(&bytes)?;
            let last = document_records(&rs)
                .filter(|record| record.kind == 0x04b3)
                .last()
                .ok_or("Missing font style table")?;
            let insert_at = last.data.end + last.data.len() % 2;
            bytes.splice(insert_at..insert_at, appended);
        }
        let rs = records(&bytes)?;
        let starts = document_record_indices(&rs)
            .into_iter()
            .filter(|&index| rs[index].kind == 0x07d0)
            .collect::<Vec<_>>();
        for (ordinal, index) in targets {
            let start = *starts.get(ordinal).ok_or("Missing styled text object")?;
            let group = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x07d7)
                .collect::<Vec<_>>();
            if !group
                .iter()
                .any(|record| matches!(record.kind, 0x08fc | 0x07d5 | 0x0903 | 0x0907 | 0x090d))
            {
                return Err("Font style target has no text".into());
            }
            let reference = group
                .iter()
                .find(|record| record.kind == 0x07d8 && record.data.len() == 2)
                .ok_or("Missing free-text font reference")?;
            bytes[reference.data.clone()].copy_from_slice(&index.to_le_bytes());
        }
        Ok(bytes)
    }
    pub(super) fn patch_trace_flags(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let entries = records(&bytes)?;
        let mut objects = self.objects.iter().filter(|o| !o.deleted);
        for record in document_records(&entries).filter(|r| r.kind == 0x07d0) {
            let object = objects
                .next()
                .ok_or("Object group count mismatch in patch_trace_flags")?;
            if record.data.len() < 5 {
                return Err("Short object header".into());
            }
            let flag = &mut bytes[record.data.start + 4];
            *flag = (*flag & !1) | u8::from(object.tracing);
        }
        if objects.next().is_some() {
            return Err("Object group count mismatch in patch_trace_flags".into());
        }
        Ok(bytes)
    }
    pub(super) fn patch_mapped_iteration_image_levels(
        &self,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let rs = records(&bytes)?;
        let mut objects = self.objects.iter().filter(|o| !o.deleted);
        let mut current = None;
        for record in document_records(&rs) {
            if record.kind == 0x07d0 {
                current =
                    Some(objects.next().ok_or(
                        "Object group count mismatch in patch_mapped_iteration_image_levels",
                    )?);
            } else if record.kind == 0x090a
                && let Some(object) = current
                && let Kind::MappedPointIteration { final_only, .. } = object.kind
            {
                if record.data.len() != 24 {
                    return Err("Invalid mapped iteration definition".into());
                }
                bytes[record.data.start + 4..record.data.start + 6]
                    .copy_from_slice(&u16::from(final_only).to_le_bytes());
            }
        }
        if objects.next().is_some() {
            return Err(
                "Object group count mismatch in patch_mapped_iteration_image_levels".into(),
            );
        }
        Ok(bytes)
    }

    pub(super) fn patch_visual_styles(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let rs = records(&bytes)?;
        let survivors: Vec<_> = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect();
        let mut ordinal = 0;
        for record in document_records(&rs).filter(|record| record.kind == 0x07d0) {
            let object = survivors
                .get(ordinal)
                .ok_or("Object group count mismatch in patch_visual_styles")?;
            ordinal += 1;
            if object.kind.is_polygon() || object.kind.is_sector() {
                let payload = bytes
                    .get_mut(record.data.clone())
                    .filter(|payload| matches!(payload.len(), 16 | 28) && (payload[6..8] == [4, 0] || (payload.len() == 28 && payload[6..10] == [13, 0, 68, 0])))
                    .ok_or("Invalid polygon drawing-order header")?;
                match (payload.len(), object.interior_order) {
                    (28, Some(order)) => payload[20..24].copy_from_slice(&order.to_le_bytes()),
                    (16, None) => {}
                    _ => return Err("Polygon drawing order is unavailable in this header".into()),
                }
            } else if let Some(order) = object.interior_order {
                let payload = &bytes[record.data.clone()];
                if payload.len() != 28
                    || payload[6..8] != [4, 0]
                    || payload[20..24] != order.to_le_bytes()
                {
                    return Err("Cannot change drawing order of an unsupported interior".into());
                }
            }
            if let Some(style) = object.visual_style {
                let styled_binding = matches!(object.kind, Kind::IterationBinding(..));
                let result = if styled_binding {
                    style_records::set_iteration_binding_visual_style(
                        &mut bytes[record.data.clone()],
                        style,
                    )
                } else {
                    style_records::set_visual_style(&mut bytes[record.data.clone()], style)
                };
                result
                    .map_err(|error| format!("Invalid style in object {}: {error}", object.id))?;
            }
        }
        if ordinal != survivors.len() {
            return Err("Object group count mismatch in patch_visual_styles".into());
        }
        Ok(bytes)
    }
    pub(super) fn patch_text_and_labels(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let rs = records(&bytes)?;
        let mut edits: Vec<(usize, usize, Vec<u8>)> = Vec::new();
        let mut ordinal = 0usize;
        let mut current_group: Option<usize> = None;
        let survivors: Vec<&Object> = self.objects.iter().filter(|o| !o.deleted).collect();
        let mut ids = vec![0; self.objects.len() + 1];
        for (index, object) in survivors.iter().enumerate() {
            ids[object.id] = index + 1;
        }
        for index in document_record_indices(&rs) {
            let record = &rs[index];
            if record.kind == 0x07d0 {
                ordinal += 1;
                current_group = Some(index);
            }
            if record.kind != 0x07d7 {
                continue;
            }
            let start_index = current_group
                .take()
                .ok_or("Object terminator without header")?;
            let object = survivors
                .get(ordinal - 1)
                .ok_or("Object group count mismatch in caption patch")?;
            if object.id > self.source_count {
                continue;
            }
            let group = &rs[start_index..=index];
            if let Kind::ParameterLocus(_, min, max, samples) = object.kind {
                let range = group
                    .iter()
                    .find(|record| record.kind == 0x0902)
                    .ok_or("Missing saved parameter locus range")?;
                let original = &bytes[range.data.clone()];
                if original.len() != 40
                    || u32::from_le_bytes(original[20..24].try_into().unwrap()) != 0x0003_0004
                {
                    return Err("Unsupported saved parameter locus descriptor".into());
                }
                let mut payload = original.to_vec();
                payload[..8].copy_from_slice(&min.to_le_bytes());
                payload[8..16].copy_from_slice(&max.to_le_bytes());
                payload[16..20].copy_from_slice(&samples.to_le_bytes());
                if payload != original {
                    let mut replacement = Vec::new();
                    put_record(&mut replacement, 0x0902, &payload);
                    edits.push((range.start, range.data.end, replacement));
                }
                continue;
            }
            if let Some((min, max, samples, polar)) = match object.kind {
                Kind::FunctionPlot(_, _, min, max, samples) => Some((min, max, samples, None)),
                Kind::ParametricPlot(_, _, _, min, max, samples, mode) => {
                    Some((min, max, samples, Some(mode == PlotPointsMode::Polar)))
                }
                _ => None,
            } {
                let range = group
                    .iter()
                    .find(|record| record.kind == 0x0902)
                    .ok_or("Missing saved plot range")?;
                let original = &bytes[range.data.clone()];
                let previous = match polar {
                    Some(polar) => function_records::decode_parametric_range(original, polar)?,
                    None => function_records::decode_plot_range(original)?,
                };
                if previous != (min, max, samples) {
                    let mut payload = original.to_vec();
                    payload[..8].copy_from_slice(&min.to_le_bytes());
                    payload[8..16].copy_from_slice(&max.to_le_bytes());
                    payload[16..20].copy_from_slice(&samples.to_le_bytes());
                    let mut replacement = Vec::new();
                    put_record(&mut replacement, 0x0902, &payload);
                    edits.push((
                        range.start,
                        range.data.end + original.len() % 2,
                        replacement,
                    ));
                }
                continue;
            }
            if let Kind::FunctionDefinition(expression)
            | Kind::ReferencedFunction(_, expression)
            | Kind::DerivativeFunction(_, expression) = object.kind
            {
                let record = group
                    .iter()
                    .find(|record| record.kind == 0x0907)
                    .ok_or("Missing saved function expression")?;
                let original = &bytes[record.data.clone()];
                let count = self
                    .function_sources(object.id)
                    .ok_or("Invalid function sources")?
                    .len();
                if function_records::decode_expression_with_sources(original, count)? != expression
                {
                    edits.push((
                        record.start,
                        record.data.end + original.len() % 2,
                        function_records::encode_expression_record_with_sources(
                            original, expression, count,
                        )?,
                    ));
                }
                continue;
            }
            if let Some(mut replacement) = match &object.kind {
                Kind::Parameter(value) => Some(value.as_bytes().to_vec()),
                Kind::Calculation(value) => Some(value.as_bytes().to_vec()),
                Kind::MeasuredCalculation(source, value) => {
                    Some(value.remap_source(ids[*source] as u32)?.as_bytes().to_vec())
                }
                Kind::LengthPairCalculation(first, second, value) => Some(
                    value
                        .remap_sources(ids[*first] as u32, ids[*second] as u32)?
                        .as_bytes()
                        .to_vec(),
                ),
                Kind::ParameterCalculation(source, value) => {
                    let sources = calculation_source_ids(*source, value)
                        .into_iter()
                        .map(|id| ids[id] as u32)
                        .collect::<Vec<_>>();
                    Some(value.remap_sources(&sources)?.as_bytes().to_vec())
                }
                Kind::ValueTable(_, value) | Kind::ValueTablePair(_, _, value) => {
                    Some(value.remap_sources(&ids)?.as_bytes().to_vec())
                }
                _ => None,
            } {
                replacement[10] = (replacement[10] & !1) | u8::from(!object.visible);
                if bytes[group[0].start..record.data.end] != replacement {
                    edits.push((group[0].start, record.data.end, replacement));
                }
                continue;
            }
            if let Kind::ReferencedText { text, .. } = &object.kind {
                let rich = group.iter().find(|r| r.kind == 0x08fc).ok_or("Missing referenced caption payload")?;
                if bytes[rich.data.clone()] != *text.rich_bytes() {
                    let mut replacement = Vec::new();
                    super::records::put_record(&mut replacement, 0x08fc, text.rich_bytes());
                    edits.push((rich.start, rich.data.end + rich.data.len()%2, replacement));
                }
                continue;
            }
            if let Kind::PlainText { text, rect } = &object.kind {
                let rich = group
                    .iter()
                    .find(|r| r.kind == 0x08fc)
                    .ok_or("Missing free-text payload")?;
                let bbox = group
                    .iter()
                    .find(|r| r.kind == 0x0898)
                    .ok_or("Missing free-text box")?;
                let (old_text, old_rect) = text_object::decode_plain_text(
                    &bytes[rich.data.clone()],
                    &bytes[bbox.data.clone()],
                )?;
                if old_text != *text || old_rect != *rect || object.rich_text_override.is_some() {
                    let (new_rich, new_bbox) = if let Some(template) = &object.rich_text_override {
                        template.encode_parts(text, *rect)?
                    } else {
                        text_object::encode_from_template(
                            &bytes[rich.data.clone()],
                            &bytes[bbox.data.clone()],
                            text,
                            *rect,
                        )?
                    };
                    if bytes[rich.data.clone()] != new_rich {
                        let mut rich_record = Vec::new();
                        put_record(&mut rich_record, 0x08fc, &new_rich);
                        edits.push((rich.start, rich.data.end + rich.data.len() % 2, rich_record));
                    }
                    if bytes[bbox.data.clone()] != new_bbox {
                        let mut box_record = Vec::new();
                        put_record(&mut box_record, 0x0898, &new_bbox);
                        edits.push((bbox.start, bbox.data.end + bbox.data.len() % 2, box_record));
                    }
                }
            }
            if let Some(label) = &object.label {
                if let Some(old) = group.iter().find(|r| r.kind == 0x07d5) {
                    if bytes[old.data.clone()] != *label.payload() {
                        let mut replacement = Vec::new();
                        put_record(&mut replacement, 0x07d5, label.payload());
                        edits.push((old.start, old.data.end + old.data.len() % 2, replacement));
                    }
                } else {
                    let insertion =
                        if object
                            .kind
                            .transform_definition()
                            .is_some_and(|definition| {
                                definition.element() == super::transform_model::ElementType::Point
                            })
                        {
                            group
                                .iter()
                                .find(|item| matches!(item.kind, 0x07d8 | 0x232e))
                                .map_or(record.start, |item| item.start)
                        } else if matches!(
                            object.kind,
                            Kind::NumericCoordinatePoint(..)
                                | Kind::PathValuePoint(..)
                                | Kind::PathLiteralPoint(..)
                                | Kind::PointOnArc(..)
                                | Kind::CircleCircleIntersection(..)
                        ) {
                            group
                                .iter()
                                .find(|record| record.kind == 0x07d8)
                                .map_or(record.start, |font| font.start)
                        } else {
                            group[0].data.end + group[0].data.len() % 2
                        };
                    edits.push((
                        insertion,
                        insertion,
                        if group.iter().any(|record| record.kind == 0x07d8) {
                            let mut label_record = Vec::new();
                            put_record(&mut label_record, 0x07d5, label.payload());
                            label_record
                        } else {
                            crate::text_records::encode_point_label_records(label)
                        },
                    ));
                }
            }
        }
        if ordinal != survivors.len() {
            return Err("Object group count mismatch in caption patch".into());
        }
        edits.sort_by(|a, b| b.0.cmp(&a.0));
        for (start, end, replacement) in edits {
            bytes.splice(start..end, replacement);
        }
        records(&bytes)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod caption_font_namespace_tests {
    use super::*;

    #[test]
    fn imported_numeric_caption_font_uses_drawing_table_and_preserves_tool_bytes() {
        let original =
            include_bytes!("../../tests/fixtures/embedded_tool_scalar_created_native_verified.gsp");
        let mut document = Document::from_bytes(original, None).unwrap();
        let prototype = document.embedded_tool_document(0).unwrap();
        let style = prototype.text_font_state(2).unwrap();
        assert_eq!(style.size_code, 16);
        assert!(!style.bold);
        let index = document.imported_caption_font_index(&style).unwrap();
        assert_eq!(index, 1);
        document.objects[1].text_style_override = Some(style.clone());
        assert_eq!(document.imported_caption_font_index(&style).unwrap(), index);
        let output = document
            .patch_text_font_overrides(original.to_vec())
            .unwrap();
        assert_eq!(
            text_style_records::decode_font_state_at(&output, index as usize).unwrap(),
            style
        );
        assert!(text_style_records::decode_font_state_at(&output, 2).is_err());
        let tools = |bytes: &[u8]| {
            let entries = records(bytes).unwrap();
            let drawing = document_record_indices(&entries);
            entries
                .iter()
                .enumerate()
                .filter(|(index, _)| !drawing.contains(index))
                .flat_map(|(index, record)| {
                    let end = entries
                        .get(index + 1)
                        .map_or(bytes.len(), |next| next.start);
                    bytes[record.start..end].to_vec()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(tools(original), tools(&output));
        let reopened = Document::from_bytes(&output, None).unwrap();
        assert_eq!(reopened.text_font_state(2), Some(style));
    }
}
