use super::embedded_tools::document_records;
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind};
use super::records::records;
use super::transform_model::ElementType;
use super::{Document, Kind, Point, TextStyleCommand};
use crate::text_object::TextRect;
use crate::text_records::LabelAux;
use crate::text_style_records::FontState;
use crate::{text_object, text_style_records};

fn can_create_geometry_label(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(..)
            | Kind::Line(..)
            | Kind::Ray(..)
            | Kind::Parallel(..)
            | Kind::Perpendicular(..)
            | Kind::Bisector(..)
            | Kind::Circle(..)
            | Kind::CircleBySegment(..)
    ) || kind.transform_definition().is_some_and(|definition| {
        matches!(
            definition.element(),
            ElementType::Segment
                | ElementType::Ray
                | ElementType::Line
                | ElementType::Circle
                | ElementType::Arc
        )
    }) || matches!(kind, Kind::Colorized(definition) if matches!(
        definition.element(),
        ElementType::Segment
            | ElementType::Ray
            | ElementType::Line
            | ElementType::Circle
            | ElementType::Arc
    )) || matches!(
        kind,
        Kind::ThreePointArc(..) | Kind::ArcOnCircle(..) | Kind::CenterArc(..)
    )
}

impl Document {
    /// Current displayed caption content and its saved placement.
    pub fn text_content(&self, id: usize) -> Option<(String, TextRect)> {
        if let Some(content) = self.referenced_caption_content(id) {
            return Some(content);
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PlainText { text, rect } = &object.kind else {
            return None;
        };
        Some((text.clone(), *rect))
    }
    fn can_edit_literal_text(&self) -> bool {
        if self.page_count == 1 {
            !self.partial_view
        } else {
            !self.image_read_only
                && self
                    .objects
                    .iter()
                    .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
                && self.objects[self.source_count..]
                    .iter()
                    .all(|object| super::kind::is_multi_page_addition_kind(&object.kind))
        }
    }

    pub fn add_text(&mut self, text: &str, at: Point) -> Result<usize, String> {
        let x = i16::try_from(at.x.round() as i64).map_err(|_| "Text x is out of range")?;
        let y = i16::try_from(at.y.round() as i64).map_err(|_| "Text y is out of range")?;
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let rect = TextRect::roomy_at(x, y, &normalized)?;
        text_object::encode_plain_text_group(&normalized, rect)?;
        if !self.can_edit_literal_text() {
            return Err("Cannot add text to this sketch safely".into());
        }
        Ok(self.add(Kind::PlainText {
            text: normalized,
            rect,
        }))
    }
    pub fn edit_text(&mut self, id: usize, text: &str) -> Result<(), String> {
        if !self.can_edit_literal_text() {
            return Err("Cannot edit text in a sketch with unsupported records safely".into());
        }
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let text = normalized.as_str();
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        let Kind::PlainText {
            rect,
            text: ref current,
        } = object.kind
        else {
            return Err("Object is not free text".into());
        };
        if object.deleted || object.page != self.active_page {
            return Err("Text is not on the active page".into());
        }
        let rect = TextRect::roomy_at(rect.left, rect.top, text)?;
        text_object::encode_plain_text_group(text, rect)?;
        let rich = if current != text {
            let runs = self.text_rich_runs(id).ok_or("Unsupported text markup")?;
            if runs
                .iter()
                .flatten()
                .any(|run| run.style != text_object::RichRunStyle::default())
                || object
                    .rich_text_override
                    .as_ref()
                    .is_some_and(text_object::RichTextTemplate::is_legacy_gbk)
            {
                Some(text_object::remap_rich_text(&runs, text, rect)?)
            } else {
                None
            }
        } else {
            object.rich_text_override.clone()
        };
        self.checkpoint();
        self.objects.edit(id - 1).kind = Kind::PlainText {
            text: text.to_owned(),
            rect,
        };
        self.objects.edit(id - 1).rich_text_override = rich;
        Ok(())
    }
    pub(super) fn text_rich_template(&self, id: usize) -> Option<text_object::RichTextTemplate> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PlainText { ref text, .. } = object.kind else {
            return None;
        };
        if let Some(template) = &object.rich_text_override {
            return (template.decode().ok()?.0 == *text).then(|| template.clone());
        }
        let header = object.header_offset?;
        let rs = records(&self.original).ok()?;
        let start = rs
            .iter()
            .position(|record| record.kind == 0x07d0 && record.data.start == header)?;
        let group = rs[start + 1..]
            .iter()
            .take_while(|record| record.kind != 0x07d7)
            .collect::<Vec<_>>();
        let rich = group.iter().find(|record| record.kind == 0x08fc)?;
        let bbox = group.iter().find(|record| record.kind == 0x0898)?;
        let template = text_object::RichTextTemplate::from_parts(
            &self.original[rich.data.clone()],
            &self.original[bbox.data.clone()],
        )
        .ok()?;
        (template.decode().ok()?.0 == *text).then_some(template)
    }
    pub fn text_rich_runs(&self, id: usize) -> Option<Vec<Vec<text_object::RichTextRun>>> {
        if let Some(rows) = self.referenced_caption_rich_runs(id) {
            return Some(rows);
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PlainText { ref text, .. } = object.kind else {
            return None;
        };
        if let Some(template) = self.text_rich_template(id) {
            return Some(template.decode().ok()?.2);
        }
        Some(
            text.split('\n')
                .map(|line| {
                    vec![text_object::RichTextRun {
                        text: line.to_owned(),
                        style: text_object::RichRunStyle::default(),
                    }]
                })
                .collect(),
        )
    }
    pub fn text_font_state(&self, id: usize) -> Option<FontState> {
        self.cached_text_font(id, || self.text_font_state_uncached(id))
    }

    fn text_font_state_uncached(&self, id: usize) -> Option<FontState> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if !matches!(object.kind, Kind::PlainText { .. })
            && object.label.is_none()
            && object.kind.numeric_unit().is_none()
            && !matches!(
                object.kind,
                Kind::ValueTable(_, _)
                    | Kind::ValueTablePair(_, _, _)
                    | Kind::DataTable(_, _)
                    | Kind::IterationValues(..)
                    | Kind::ValueEquation { .. }
                    | Kind::CoordinateMeasurement { .. }
                    | Kind::PointNumericLabel(_, _)
                    | Kind::PointReferencedText { .. }
                    | Kind::ReferencedText { .. }
            )
        {
            return None;
        }
        if let Some(style) = &object.text_style_override {
            return Some(style.clone());
        }
        if object.header_offset.is_none() && object.kind.numeric_unit().is_some() {
            return None;
        }
        let style = if let Some(header) = object.header_offset {
            let rs = records(&self.original).ok()?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x07d0 && record.data.start == header)?;
            let group = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x07d7);
            let record = group
                .clone()
                .find(|record| record.kind == 0x07d8 && record.data.len() == 2);
            if record.is_none()
                && object.label.is_none()
                && !group
                    .clone()
                    .any(|record| matches!(record.kind, 0x08fc | 0x07d5 | 0x0903 | 0x0907 | 0x090d))
            {
                return None;
            }
            record.map_or(0, |record| {
                u16::from_le_bytes(self.original[record.data.clone()].try_into().unwrap()) as usize
            })
        } else if let Kind::ValueTable(_, group)
        | Kind::ValueTablePair(_, _, group)
        | Kind::DataTable(_, group) = &object.kind
        {
            usize::from(group.font_reference())
        } else {
            0
        };
        let entries = records(&self.original).ok()?;
        let style_record = document_records(&entries)
            .filter(|r| r.kind == 0x04b3)
            .nth(style)?;
        let payload = &self.original[style_record.data.clone()];
        let family_index = u16::from_le_bytes(payload.get(..2)?.try_into().ok()?) as usize;
        let family_record = document_records(&entries)
            .filter(|r| r.kind == 0x04b1)
            .nth(family_index)?;
        text_style_records::decode_font_payloads(
            payload,
            &self.original[family_record.data.clone()],
        )
        .ok()
    }
    pub fn text_font_families(&self) -> Option<Vec<String>> {
        let entries = records(&self.original).ok()?;
        let mut families = document_records(&entries)
            .filter(|r| r.kind == 0x04b1)
            .map(|r| text_style_records::decode_font_family_payload(&self.original[r.data.clone()]))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        families.extend(self.run_font_families.iter().cloned());
        Some(families)
    }
    pub fn toggle_selected_text_style(
        &mut self,
        command: TextStyleCommand,
    ) -> Result<usize, String> {
        if !self.can_edit_literal_text() {
            return Err("Cannot format text in this sketch safely".into());
        }
        let updates = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.objects.get(id - 1).is_some_and(|object| {
                    !object.deleted
                        && object.page == self.active_page
                        && (matches!(
                            object.kind,
                            Kind::PlainText { .. }
                                | Kind::ReferencedText { .. }
                                | Kind::PointReferencedText { .. }
                                | Kind::PointNumericLabel(..)
                        ) || object.label.is_some()
                            || object.measurement_anchor.is_some())
                })
            })
            .map(|id| {
                let mut style = self.text_font_state(id).ok_or("Unsupported text font")?;
                match command {
                    TextStyleCommand::Bold => style.bold = !style.bold,
                    TextStyleCommand::Italic => style.italic = !style.italic,
                    TextStyleCommand::Underline => style.underline = !style.underline,
                    TextStyleCommand::IncreaseSize => {
                        style.size_code = match style.size_code {
                            14 => 16,
                            16 => 18,
                            18 => 24,
                            24 => 36,
                            36 => 48,
                            48 => 64,
                            _ => return Err("This text size has no verified larger step".into()),
                        }
                    }
                    TextStyleCommand::DecreaseSize => {
                        style.size_code = match style.size_code {
                            16 => 14,
                            18 => 16,
                            24 => 18,
                            36 => 24,
                            48 => 36,
                            64 => 48,
                            _ => return Err("This text size has no verified smaller step".into()),
                        }
                    }
                }
                Ok((id, style))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if updates.is_empty() {
            return Err("Select text, an object label, or a measurement caption to format".into());
        }
        let count = updates.len();
        self.checkpoint();
        for (id, style) in updates {
            self.objects.edit(id - 1).text_style_override = Some(style);
        }
        Ok(count)
    }
    pub fn set_selected_text_family(&mut self, family: &str) -> Result<usize, String> {
        if !self.can_edit_literal_text() {
            return Err("Cannot format text in this sketch safely".into());
        }
        text_style_records::encode_font_family_payload(family)?;
        let updates = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.objects.get(id - 1).is_some_and(|object| {
                    !object.deleted
                        && object.page == self.active_page
                        && (matches!(
                            object.kind,
                            Kind::PlainText { .. }
                                | Kind::ReferencedText { .. }
                                | Kind::PointReferencedText { .. }
                                | Kind::PointNumericLabel(..)
                        ) || object.label.is_some()
                            || object.measurement_anchor.is_some())
                })
            })
            .map(|id| {
                let mut style = self.text_font_state(id).ok_or("Unsupported text font")?;
                style.family = family.to_owned();
                Ok((id, style))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if updates.is_empty() {
            return Err("Select text, an object label, or a measurement caption to format".into());
        }
        let count = updates.len();
        self.checkpoint();
        for (id, style) in updates {
            self.objects.edit(id - 1).text_style_override = Some(style);
        }
        Ok(count)
    }
    pub fn format_text_range(
        &mut self,
        id: usize,
        start: usize,
        end: usize,
        command: TextStyleCommand,
    ) -> Result<(), String> {
        self.format_text_range_with(id, start, end, |style, base| {
            match command {
                TextStyleCommand::Bold | TextStyleCommand::Italic | TextStyleCommand::Underline => {
                    let (override_flag, baseline) = match command {
                        TextStyleCommand::Bold => (&mut style.bold, base.bold),
                        TextStyleCommand::Italic => (&mut style.italic, base.italic),
                        TextStyleCommand::Underline => (&mut style.underline, base.underline),
                        _ => unreachable!(),
                    };
                    let next = !override_flag.unwrap_or(baseline);
                    *override_flag = (next != baseline).then_some(next);
                }
                TextStyleCommand::IncreaseSize | TextStyleCommand::DecreaseSize => {
                    let current = style.size_code.unwrap_or(base.size_code);
                    let next = match (command, current) {
                        (TextStyleCommand::IncreaseSize, 14) => 16,
                        (TextStyleCommand::IncreaseSize, 16) => 18,
                        (TextStyleCommand::IncreaseSize, 18) => 24,
                        (TextStyleCommand::IncreaseSize, 24) => 36,
                        (TextStyleCommand::IncreaseSize, 36) => 48,
                        (TextStyleCommand::IncreaseSize, 48) => 64,
                        (TextStyleCommand::DecreaseSize, 16) => 14,
                        (TextStyleCommand::DecreaseSize, 18) => 16,
                        (TextStyleCommand::DecreaseSize, 24) => 18,
                        (TextStyleCommand::DecreaseSize, 36) => 24,
                        (TextStyleCommand::DecreaseSize, 48) => 36,
                        (TextStyleCommand::DecreaseSize, 64) => 48,
                        (TextStyleCommand::IncreaseSize, _) => {
                            return Err("This text size has no verified larger step".into());
                        }
                        _ => return Err("This text size has no verified smaller step".into()),
                    };
                    style.size_code = (next != base.size_code).then_some(next);
                }
            }
            Ok(())
        })
    }
    pub fn format_text_range_family(
        &mut self,
        id: usize,
        start: usize,
        end: usize,
        family: &str,
    ) -> Result<(), String> {
        text_style_records::encode_font_family_payload(family)?;
        let families = self
            .text_font_families()
            .ok_or("Unsupported font family table")?;
        let index = families.iter().position(|name| name == family);
        let next_index = index.unwrap_or(families.len());
        let next_index = u16::try_from(next_index).map_err(|_| "Too many font families")?;
        self.format_text_range_with(id, start, end, |style, base| {
            style.family_index = (family != base.family).then_some(next_index);
            Ok(())
        })?;
        if index.is_none() {
            self.run_font_families.push(family.to_owned());
        }
        Ok(())
    }
    fn format_text_range_with(
        &mut self,
        id: usize,
        start: usize,
        end: usize,
        mut modify: impl FnMut(&mut text_object::RichRunStyle, &FontState) -> Result<(), String>,
    ) -> Result<(), String> {
        if !self.can_edit_literal_text() {
            return Err("Cannot format text in this sketch safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        let Kind::PlainText { ref text, rect } = object.kind else {
            return Err("Object is not free text".into());
        };
        if object.deleted || object.page != self.active_page {
            return Err("Text is not on the active page".into());
        }
        if start >= end || end > text.chars().count() {
            return Err("Select a valid text range".into());
        }
        let base = self.text_font_state(id).ok_or("Unsupported text font")?;
        let rows = self.text_rich_runs(id).ok_or("Unsupported text markup")?;
        let mut position = 0;
        let mut changed = 0;
        let mut styled_rows = Vec::new();
        let row_count = rows.len();
        for (line_index, row) in rows.into_iter().enumerate() {
            let mut styled = Vec::<text_object::RichTextRun>::new();
            for run in row {
                for ch in run.text.chars() {
                    let mut style = run.style;
                    if (start..end).contains(&position) {
                        modify(&mut style, &base)?;
                        changed += 1;
                    }
                    if let Some(last) = styled.last_mut().filter(|last| last.style == style) {
                        last.text.push(ch);
                    } else {
                        styled.push(text_object::RichTextRun {
                            text: ch.to_string(),
                            style,
                        });
                    }
                    position += 1;
                }
            }
            if styled.is_empty() {
                styled.push(text_object::RichTextRun {
                    text: String::new(),
                    style: text_object::RichRunStyle::default(),
                });
            }
            styled_rows.push(styled);
            if line_index + 1 < row_count {
                position += 1;
            }
        }
        if changed == 0 {
            return Err("Select text characters to format".into());
        }
        let template = text_object::encode_rich_runs(&styled_rows, rect)?;
        if template.decode()?.0 != *text {
            return Err("Styled text did not round-trip".into());
        }
        self.checkpoint();
        self.objects.edit(id - 1).rich_text_override = Some(template);
        Ok(())
    }
    pub fn move_text(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_text_inner(id, at, true)
    }
    pub fn drag_text(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_text_inner(id, at, false)
    }
    fn move_text_inner(&mut self, id: usize, at: Point, checkpoint: bool) -> Result<(), String> {
        let x = i16::try_from(at.x.round() as i64).map_err(|_| "Text x is out of range")?;
        let y = i16::try_from(at.y.round() as i64).map_err(|_| "Text y is out of range")?;
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        if let Kind::ReferencedText { parents, text } = &object.kind {
            if object.deleted || object.page != self.active_page {
                return Err("Caption is not on the active page".into());
            }
            let parents = parents.clone();
            let text = text.with_origin((x, y));
            if checkpoint {
                self.checkpoint();
            }
            self.objects.edit(id - 1).kind = Kind::ReferencedText { parents, text };
            self.dirty = true;
            return Ok(());
        }
        let Kind::PlainText { ref text, rect } = object.kind else {
            return Err("Object is not free text".into());
        };
        let dx = i32::from(x) - i32::from(rect.left);
        let dy = i32::from(y) - i32::from(rect.top);
        let right =
            i16::try_from(i32::from(rect.right) + dx).map_err(|_| "Text x is out of range")?;
        let bottom =
            i16::try_from(i32::from(rect.bottom) + dy).map_err(|_| "Text y is out of range")?;
        let text = text.clone();
        if checkpoint {
            self.checkpoint();
        }
        self.objects.edit(id - 1).kind = Kind::PlainText {
            text,
            rect: TextRect {
                left: x,
                top: y,
                right,
                bottom,
            },
        };
        self.dirty = true;
        Ok(())
    }
    pub fn can_edit_geometry_label(&self, id: usize) -> bool {
        self.objects
            .get(id.saturating_sub(1))
            .is_some_and(|object| {
                !object.deleted
                    && object.page == self.active_page
                    && (is_point_kind(&object.kind)
                        || can_create_geometry_label(&object.kind)
                        || (object.label.is_some()
                            && (is_straight_kind(&object.kind)
                                || object.kind.is_circle()
                                || is_arc_kind(&object.kind))))
            })
    }

    pub fn object_label_anchor(&self, id: usize) -> Option<Point> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let label = object.label.as_ref()?;
        if is_point_kind(&object.kind) {
            return self.position(id);
        }
        if is_straight_kind(&object.kind) {
            let parameter = label.geometric_parameter()?;
            let (start, end, _) = self.line_points(id)?;
            return Some(Point {
                x: start.x + parameter * (end.x - start.x),
                y: start.y + parameter * (end.y - start.y),
            });
        }
        if object.kind.is_circle() {
            let parameter = label.geometric_parameter()?;
            let (center, radius) = self.circle_points(id)?;
            return Some(Point {
                x: center.x + radius * parameter.cos(),
                y: center.y - radius * parameter.sin(),
            });
        }
        if is_arc_kind(&object.kind) {
            return self.point_on_arc(id, label.geometric_parameter()?);
        }
        if matches!(
            object.kind,
            Kind::FunctionPlot(..)
                | Kind::ParameterLocus(..)
                | Kind::SampledMapped(..)
                | Kind::Locus(..)
        ) {
            return self.point_on_curve(id, label.geometric_parameter()?);
        }
        if object.kind.is_circle_interior() {
            let parameter = label.geometric_parameter()?;
            let (center, radius) = self.circle_interior_geometry(id)?;
            return Some(Point {
                x: center.x + radius * parameter.cos(),
                y: center.y - radius * parameter.sin(),
            });
        }
        if object.kind.is_sector() {
            return self.point_on_sector(id, label.geometric_parameter()?);
        }
        self.angle_marker_label_anchor(id)
    }

    pub fn set_object_label(
        &mut self,
        id: usize,
        name: &str,
        visible: bool,
    ) -> Result<bool, String> {
        if self.partial_view {
            return Err("Cannot edit labels in a sketch with unsupported records safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid object ID")?)
            .ok_or("Invalid object ID")?;
        if object.deleted || object.page != self.active_page {
            return Err("Object is not on the active page".into());
        }
        if !is_point_kind(&object.kind)
            && !is_straight_kind(&object.kind)
            && !object.kind.is_circle()
            && !is_arc_kind(&object.kind)
        {
            return Err("Object does not support editable geometric labels".into());
        }
        let transformed_element = object
            .kind
            .transform_definition()
            .map(|definition| definition.element());
        let colorized_element = match &object.kind {
            Kind::Colorized(definition) => Some(definition.element()),
            _ => None,
        };
        let mut label = if let Some(existing) = &object.label {
            existing.clone()
        } else if is_point_kind(&object.kind) {
            LabelAux::new_point(name, visible, (8, 5))?
        } else if !can_create_geometry_label(&object.kind) {
            return Err("Creating a label for this derived object is not yet supported".into());
        } else if matches!(
            transformed_element,
            Some(ElementType::Segment | ElementType::Ray | ElementType::Line)
        ) {
            LabelAux::new_geometric(name, visible, (-10, 5), 0.25)?
        } else if transformed_element == Some(ElementType::Circle) {
            LabelAux::new_geometric(name, visible, (1, -1), 7.0 * std::f64::consts::FRAC_PI_4)?
        } else if transformed_element == Some(ElementType::Arc) {
            LabelAux::new_geometric(name, visible, (-8, -33), 0.25)?
        } else if matches!(
            colorized_element,
            Some(ElementType::Segment | ElementType::Ray | ElementType::Line)
        ) {
            LabelAux::new_geometric(name, visible, (-10, 5), 0.25)?
        } else if colorized_element == Some(ElementType::Circle) {
            LabelAux::new_geometric(name, visible, (1, -1), 7.0 * std::f64::consts::FRAC_PI_4)?
        } else if colorized_element == Some(ElementType::Arc) {
            LabelAux::new_geometric(name, visible, (6, -23), 0.25)?
        } else if is_straight_kind(&object.kind) {
            LabelAux::new_geometric(name, visible, (-6, 5), 0.25)?
        } else if object.kind.is_circle() {
            LabelAux::new_geometric(name, visible, (2, -1), 7.0 * std::f64::consts::FRAC_PI_4)?
        } else if matches!(object.kind, Kind::ThreePointArc(..)) {
            LabelAux::new_geometric(name, visible, (-26, -30), 0.25)?
        } else if is_arc_kind(&object.kind) {
            LabelAux::new_geometric(name, visible, (6, -23), 0.25)?
        } else {
            return Err("Object does not support editable labels".into());
        };
        label.set_name(name)?;
        label.set_visible(visible);
        if object.label.as_ref() == Some(&label) {
            return Ok(false);
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        candidate.objects.edit(id - 1).label = Some(label);
        candidate.verify_roundtrip()?;
        *self = candidate;
        Ok(true)
    }

    pub fn set_selected_geometry_label_visibility(
        &mut self,
        visible: bool,
    ) -> Result<usize, String> {
        if self.partial_view {
            return Err("Cannot edit labels in a sketch with unsupported records safely".into());
        }
        let ids = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.can_edit_geometry_label(id)
                    && self.objects[id - 1]
                        .label
                        .as_ref()
                        .is_some_and(|label| label.visible() != visible)
            })
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return Ok(0);
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        for id in &ids {
            candidate
                .objects
                .edit(*id - 1)
                .label
                .as_mut()
                .unwrap()
                .set_visible(visible);
        }
        candidate.verify_roundtrip()?;
        *self = candidate;
        Ok(ids.len())
    }

    pub fn set_point_label(&mut self, id: usize, name: &str, visible: bool) -> Result<(), String> {
        if !self
            .objects
            .get(id.saturating_sub(1))
            .is_some_and(|object| is_point_kind(&object.kind))
        {
            return Err("Object is not a point".into());
        }
        self.set_object_label(id, name, visible)?;
        Ok(())
    }
}
