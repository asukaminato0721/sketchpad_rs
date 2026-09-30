use super::{Document, Kind, Object};
use crate::table_records::{
    CalculationGroup, CalculationSourceKind, LengthPairCalculationGroup, MeasuredCalculationGroup,
    NumericUnit, ParameterCalculationGroup, ParameterGroup,
};
use crate::value_equation_records::ValueEquationKind;

pub(super) fn calculation_source_ids(
    first: usize,
    group: &ParameterCalculationGroup,
) -> Vec<usize> {
    let base = first.saturating_sub(group.source_id() as usize);
    group
        .sources()
        .into_iter()
        .map(|id| base + id as usize)
        .collect()
}
pub(super) fn calculation_source_units(
    group: &ParameterCalculationGroup,
    refs: &[usize],
    objects: &[Object],
) -> Option<Vec<NumericUnit>> {
    let kinds = group.source_kinds().ok()?;
    if refs.len() != kinds.len() {
        return None;
    }
    refs.iter()
        .zip(kinds)
        .map(|(id, kind)| {
            let parent = objects.get(id.checked_sub(1)?)?;
            if parent.deleted {
                return None;
            }
            match kind {
                CalculationSourceKind::Numeric => parent.kind.numeric_unit(),
                CalculationSourceKind::Function if parent.kind.is_function() => {
                    Some(NumericUnit::Scalar)
                }
                _ => None,
            }
        })
        .collect()
}
pub(super) fn parse_numeric_calculation(
    bytes: &[u8],
    refs: &[usize],
    objects: &[Object],
    page_base: usize,
) -> Option<Kind> {
    let group = ParameterCalculationGroup::parse(bytes).ok()?;
    if refs.is_empty()
        || group.sources()
            != refs
                .iter()
                .map(|id| id.checked_sub(page_base).map(|id| id as u32))
                .collect::<Option<Vec<_>>>()?
    {
        return None;
    }
    let units = calculation_source_units(&group, refs, objects)?;
    let group = group.with_source_units(&units).ok()?;
    // Match the ordinary decoder's established concrete representations. A
    // native abstract template can specialize into either legacy wire family.
    match refs {
        [first, second]
            if objects[*first - 1].kind.is_distance_value()
                && objects[*second - 1].kind.is_distance_value() =>
        {
            if let Ok(pair) = LengthPairCalculationGroup::parse(bytes)
                && pair.sources() == [(*first - page_base) as u32, (*second - page_base) as u32]
            {
                return Some(Kind::LengthPairCalculation(*first, *second, pair));
            }
        }
        [source] if objects[*source - 1].kind.is_distance_value() => {
            if let Ok(measured) = MeasuredCalculationGroup::parse(bytes)
                && measured.source_id() as usize == *source - page_base
            {
                return Some(Kind::MeasuredCalculation(*source, measured));
            }
        }
        _ => {}
    }
    Some(Kind::ParameterCalculation(refs[0], group))
}

impl Document {
    pub fn add_default_parameter(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add a parameter in this sketch".into());
        }
        let first = self.objects.iter().find(|object| {
            !object.deleted
                && matches!(&object.kind, Kind::Parameter(group) if group.name() == "t[1]")
        });
        let second = self.objects.iter().find(|object| {
            !object.deleted
                && matches!(&object.kind, Kind::Parameter(group) if group.name() == "t[2]")
        });
        let mut group = match (first, second) {
            (None, None) => ParameterGroup::default_value(),
            (Some(_), None) => ParameterGroup::second_default_value(),
            _ => return Err("The supported parameter sequence is t[1], then t[2]".into()),
        };
        group.set_anchor(anchor)?;
        self.checkpoint();
        Ok(self.append_object(Kind::Parameter(group)))
    }
    pub fn add_named_parameter(
        &mut self,
        name: &str,
        value: f64,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add a parameter in this sketch".into());
        }
        if self.objects.iter().any(|object| {
            !object.deleted
                && matches!(&object.kind, Kind::Parameter(group) if group.name() == name)
        }) {
            return Err("A parameter with this name already exists".into());
        }
        let mut group = ParameterGroup::named(name, value)?;
        group.set_anchor(anchor)?;
        self.checkpoint();
        Ok(self.append_object(Kind::Parameter(group)))
    }
    pub fn set_parameter_value(&mut self, id: usize, value: f64) -> Result<(), String> {
        self.update_parameter_value(id, value, true)
    }
    pub fn animate_parameter_value(&mut self, id: usize, value: f64) -> Result<(), String> {
        let Some(Kind::Parameter(group)) = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid parameter ID")?)
            .map(|object| &object.kind)
        else {
            return Err("Select a parameter".into());
        };
        let factor = 10_f64.powi(group.precision() as i32);
        let displayed = (value * factor).round() / factor;
        self.update_parameter_value(id, displayed, false)
    }
    pub(super) fn update_parameter_value(
        &mut self,
        id: usize,
        value: f64,
        checkpoint: bool,
    ) -> Result<(), String> {
        if id == 0
            || (self.page_count == 1 && (self.partial_view || !self.safe_structure))
            || (self.page_count > 1
                && (self.image_read_only
                    || self.source_count != self.objects.len()
                    || self.objects.iter().any(|object| {
                        object.deleted || matches!(object.kind, Kind::Unsupported(_))
                    })))
        {
            return Err("Cannot edit a parameter in this sketch".into());
        }
        let object = self.objects.get(id - 1).ok_or("Select a parameter")?;
        if object.deleted
            || object.page != self.active_page
            || (self.page_count > 1 && id > self.source_count)
        {
            return Err("Select a saved parameter on the current page".into());
        }
        let Kind::Parameter(group) = &object.kind else {
            return Err("Select a parameter".into());
        };
        let page = object.page;
        let mut group = group.clone();
        group.set_value(value)?;
        let mut tables = Vec::new();
        let mut single_tables = Vec::new();
        if group.name() == "t[1]" {
            for (index, object) in self.objects.iter().enumerate() {
                if let Kind::ValueTable(source, table) = &object.kind
                    && !object.deleted
                    && *source == id
                    && table.default_values().is_some()
                {
                    if object.page != page {
                        return Err("Parameter table crosses sketch pages".into());
                    }
                    let mut changed = table.clone();
                    changed.set_last_default_value(value)?;
                    single_tables.push((index, *source, changed));
                }
            }
        }
        if matches!(group.name(), "t[1]" | "t[2]") {
            for (index, object) in self.objects.iter().enumerate() {
                if let Kind::ValueTablePair(first, second, table) = &object.kind
                    && !object.deleted
                    && (*first == id || *second == id)
                {
                    if object.page != page {
                        return Err("Parameter table crosses sketch pages".into());
                    }
                    let mut changed = table.clone();
                    changed.set_last_pair_axis_value(usize::from(*second == id), value)?;
                    tables.push((index, *first, *second, changed));
                }
            }
        }
        let mut candidate = self.clone();
        candidate.objects[id - 1].kind = Kind::Parameter(group);
        for (index, source, table) in single_tables {
            candidate.objects[index].kind = Kind::ValueTable(source, table);
        }
        for (index, first, second, table) in tables {
            candidate.objects[index].kind = Kind::ValueTablePair(first, second, table);
        }
        candidate.dirty = true;
        if self.page_count > 1 {
            let bytes = candidate.encode()?;
            candidate.validate_reparsed_document(&Self::from_bytes(&bytes, None)?)?;
        }
        if checkpoint {
            self.checkpoint();
            candidate.undo = self.undo.clone();
            candidate.redo = self.redo.clone();
        }
        *self = candidate;
        Ok(())
    }
    pub fn add_calculation_expression(
        &mut self,
        expression: &str,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot add a calculation in this sketch safely".into());
        }
        let mut group = CalculationGroup::from_expression(expression)?;
        group.set_anchor(anchor)?;
        self.checkpoint();
        let mut kind = Kind::Calculation(group);
        self.apply_default_calculation_precision(&mut kind);
        Ok(self.append_object(kind))
    }
    pub fn set_calculation_expression(
        &mut self,
        id: usize,
        expression: &str,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot edit a calculation in this sketch safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid calculation ID")?)
            .filter(|object| !object.deleted && object.page == self.active_page)
            .ok_or("Select a calculation on the active page")?;
        let Kind::Calculation(group) = &object.kind else {
            return Err("Select a supported calculation".into());
        };
        let mut updated = group.clone();
        updated.set_expression(expression)?;
        self.checkpoint();
        self.objects[id - 1].kind = Kind::Calculation(updated);
        Ok(())
    }
    fn numeric_caption_height(&self, id: usize) -> usize {
        let Some(object) = id.checked_sub(1).and_then(|i| self.objects.get(i)) else {
            return 33;
        };
        match &object.kind {
            Kind::SegmentRatio(..) | Kind::PointRatio(..) => 64,
            Kind::ParameterCalculation(source, g) if *source < id => {
                calculation_source_ids(*source, g)
                    .into_iter()
                    .filter(|source| *source < id)
                    .map(|source| self.numeric_caption_height(source))
                    .max()
                    .unwrap_or(33)
                    + 31 * g.expression("v").matches('÷').count()
            }
            Kind::MeasuredCalculation(source, g) if *source < id => {
                self.numeric_caption_height(*source) + 31 * usize::from(g.divides())
            }
            Kind::LengthPairCalculation(a, b, g) if *a < id && *b < id => {
                self.numeric_caption_height(*a)
                    .max(self.numeric_caption_height(*b))
                    + 31 * usize::from(g.operator_text() == "÷")
            }
            Kind::Calculation(g) => 33 + 31 * g.expression().matches('÷').count(),
            _ => 33,
        }
    }
    /// Native tool outputs follow the last free-standing numeric caption on
    /// the active page, using its current anchor rather than the spatially
    /// lowest caption. Values attached to points do not occupy a row.
    pub(super) fn next_tool_numeric_anchor(&self) -> Result<(i16, i16), String> {
        use crate::table_records::{cached_numeric_caption_step, numeric_caption_font_index};
        let object = self
            .objects
            .iter()
            .filter(|object| {
                !object.deleted
                    && object.page == self.active_page
                    && (matches!(
                        object.kind,
                        Kind::Parameter(_)
                            | Kind::Calculation(_)
                            | Kind::ParameterCalculation(..)
                            | Kind::MeasuredCalculation(..)
                            | Kind::LengthPairCalculation(..)
                            | Kind::SegmentRatio(..)
                            | Kind::PointRatio(..)
                    ) || numeric_measurement_caption_shape(&object.kind).is_some())
            })
            .max_by_key(|object| object.id)
            .ok_or("No numeric caption precedes the tool output")?;
        // Native auxiliary numeric objects with header flag 2 have no caption
        // box. The next tool output reuses their anchor instead of advancing a
        // row, regardless of the numeric object's concrete kind.
        if object.presentation_suppressed {
            return match &object.kind {
                Kind::Parameter(group) => Ok(group.anchor()),
                Kind::Calculation(group) => Ok(group.anchor()),
                Kind::ParameterCalculation(_, group) => Ok(group.anchor()),
                Kind::MeasuredCalculation(_, group) => Ok(group.anchor()),
                Kind::LengthPairCalculation(_, _, group) => Ok(group.anchor()),
                Kind::SegmentRatio(..) | Kind::PointRatio(..) => object
                    .measurement_anchor
                    .ok_or_else(|| "Missing ratio caption anchor".into()),
                kind if numeric_measurement_caption_shape(kind).is_some() => object
                    .measurement_anchor
                    .ok_or_else(|| "Missing measurement caption anchor".into()),
                _ => Err("Suppressed numeric caption has no anchor".into()),
            };
        }
        // Some auxiliary formulas carry the equivalent suppression in their
        // group payload rather than the object header.
        if let Kind::ParameterCalculation(_, group) = &object.kind
            && group.caption_suppressed()
        {
            return Ok(group.anchor());
        }
        let (anchor, step) = match &object.kind {
            Kind::Parameter(group) => (group.anchor(), cached_numeric_caption_step(group.bbox())?),
            Kind::Calculation(group) => {
                (group.anchor(), cached_numeric_caption_step(group.bbox())?)
            }
            Kind::ParameterCalculation(_, group) => {
                let step = self.tool_flat_numeric_caption_step(
                    object,
                    numeric_caption_font_index(group.as_bytes())?,
                    false,
                )?;
                let base = crate::table_records::NumericCaptionLayout::flat(step - 1)?;
                let layout = if group.display_mode() == 1 {
                    base
                } else if group.display_mode() == 3
                    && object
                        .label
                        .as_ref()
                        .is_some_and(|label| label.name_utf8().is_ok_and(|name| !name.is_empty()))
                {
                    numeric_named_caption_layout(
                        base,
                        object
                            .label
                            .as_ref()
                            .unwrap()
                            .name_utf8()
                            .map_err(|error| error.to_string())?,
                    )?
                } else {
                    self.tool_numeric_expression_layout(object.id, base)?
                };
                (
                    group.anchor(),
                    layout
                        .height()?
                        .checked_add(1)
                        .ok_or("Numeric caption layout overflow")?,
                )
            }
            Kind::MeasuredCalculation(_, group) => {
                let step = self.tool_flat_numeric_caption_step(
                    object,
                    numeric_caption_font_index(group.as_bytes())?,
                    false,
                )?;
                let base = crate::table_records::NumericCaptionLayout::flat(step - 1)?;
                let parsed = ParameterCalculationGroup::parse(group.as_bytes())?;
                let layout = if parsed.display_mode() == 1 {
                    base
                } else {
                    self.tool_numeric_expression_layout(object.id, base)?
                };
                (
                    group.anchor(),
                    layout
                        .height()?
                        .checked_add(1)
                        .ok_or("Numeric caption layout overflow")?,
                )
            }
            Kind::LengthPairCalculation(_, _, group) => {
                let step = self.tool_flat_numeric_caption_step(
                    object,
                    numeric_caption_font_index(group.as_bytes())?,
                    false,
                )?;
                let base = crate::table_records::NumericCaptionLayout::flat(step - 1)?;
                let parsed = ParameterCalculationGroup::parse(group.as_bytes())?;
                let layout = if parsed.display_mode() == 1 {
                    base
                } else {
                    self.tool_numeric_expression_layout(object.id, base)?
                };
                (
                    group.anchor(),
                    layout
                        .height()?
                        .checked_add(1)
                        .ok_or("Numeric caption layout overflow")?,
                )
            }
            Kind::SegmentRatio(..) | Kind::PointRatio(..) => (
                object
                    .measurement_anchor
                    .ok_or("Missing ratio caption anchor")?,
                64,
            ),
            kind if numeric_measurement_caption_shape(kind).is_some() => {
                let font = if let Some(header) = object.header_offset {
                    let rs = super::records::records(&self.original)?;
                    let start = rs
                        .iter()
                        .position(|record| record.kind == 0x07d0 && record.data.start == header)
                        .ok_or("Missing numeric caption group")?;
                    let record = rs[start + 1..]
                        .iter()
                        .take_while(|record| record.kind != 0x07d7)
                        .find(|record| record.kind == 0x07d8 && record.data.len() == 2)
                        .ok_or("Missing measurement caption font")?;
                    u16::from_le_bytes(self.original[record.data.clone()].try_into().unwrap())
                        as usize
                } else {
                    1
                };
                let step = self.tool_flat_numeric_caption_step(object, font, true)?;
                let base = crate::table_records::NumericCaptionLayout::flat(step - 1)?;
                let layout = numeric_measurement_caption_shape(&object.kind)
                    .unwrap()
                    .layout(base)?;
                (
                    object
                        .measurement_anchor
                        .ok_or("Missing measurement caption anchor")?,
                    layout.height()? + 1,
                )
            }
            _ => return Err("Tool placement needs a verified numeric caption layout".into()),
        };
        let next = i32::from(anchor.1)
            .checked_add(step)
            .and_then(|value| i16::try_from(value).ok())
            .ok_or("Numeric caption placement overflow")?;
        Ok((anchor.0, next))
    }

    fn tool_numeric_expression_layout(
        &self,
        id: usize,
        base: crate::table_records::NumericCaptionLayout,
    ) -> Result<crate::table_records::NumericCaptionLayout, String> {
        let object = id
            .checked_sub(1)
            .and_then(|index| self.objects.get(index))
            .ok_or("Invalid numeric caption source")?;
        match &object.kind {
            Kind::Parameter(_) => Ok(base),
            kind if numeric_measurement_caption_shape(kind).is_some() => {
                // Square units decorate the measurement's value, not its source symbol.
                numeric_measurement_caption_shape(kind)
                    .unwrap()
                    .source_layout(base)
            }
            Kind::Calculation(group) => {
                if group.is_native_radian_angle() && group.display_mode() == 3 {
                    Ok(base)
                } else {
                    group.expression_caption_layout(base)
                }
            }
            Kind::ParameterCalculation(first, group) => {
                if group.display_mode() == 3
                    && let Some(name) = object
                        .label
                        .as_ref()
                        .and_then(|label| label.name_utf8().ok())
                        .filter(|name| !name.is_empty())
                {
                    return numeric_named_caption_layout(base, name);
                }
                if group.display_mode() != 0 && !group.caption_is_flat() {
                    return Err(
                        "Non-formula fraction source expansion needs a native control".into(),
                    );
                }
                let sources = calculation_source_ids(*first, group);
                let layouts = sources
                    .into_iter()
                    .map(|source| {
                        if source >= id {
                            return Err("Invalid numeric caption dependency order".into());
                        }
                        self.tool_numeric_expression_layout(source, base)
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                group.expression_caption_layout(base, &layouts)
            }
            Kind::MeasuredCalculation(source, group) => {
                if *source >= id {
                    return Err("Invalid numeric caption dependency order".into());
                }
                ParameterCalculationGroup::parse(group.as_bytes())?.expression_caption_layout(
                    base,
                    &[self.tool_numeric_expression_layout(*source, base)?],
                )
            }
            Kind::LengthPairCalculation(first, second, group) => {
                if *first >= id || *second >= id {
                    return Err("Invalid numeric caption dependency order".into());
                }
                ParameterCalculationGroup::parse(group.as_bytes())?.expression_caption_layout(
                    base,
                    &[
                        self.tool_numeric_expression_layout(*first, base)?,
                        self.tool_numeric_expression_layout(*second, base)?,
                    ],
                )
            }
            _ => Err("Numeric caption source layout needs a native control".into()),
        }
    }

    fn tool_flat_numeric_caption_step(
        &self,
        object: &Object,
        index: usize,
        measurement: bool,
    ) -> Result<i32, String> {
        if let Some(font) = &object.text_style_override {
            if font.family == "Times New Roman"
                && font.size_code == 16
                && !font.italic
                && !font.underline
            {
                return Ok(21);
            }
            if font.family == "Times New Roman"
                && font.size_code == 24
                && !font.bold
                && !font.italic
                && !font.underline
            {
                return Ok(19);
            }
            if font.family == "黑体"
                && font.size_code == 16
                && font.bold
                && !font.italic
                && !font.underline
            {
                return Ok(21);
            }
            if font.family == "宋体"
                && font.size_code == 16
                && font.bold
                && !font.italic
                && !font.underline
            {
                return Ok(19);
            }
            if font.family == "Tahoma"
                && font.size_code == 18
                && !font.bold
                && !font.italic
                && !font.underline
            {
                return Ok(27);
            }
            if font.family == "Tahoma"
                && font.size_code == 24
                && (font.bold || measurement)
                && !font.italic
                && !font.underline
            {
                return Ok(34);
            }
            return Err(format!(
                "Tool placement needs a verified numeric caption font layout for object {}: {font:?}",
                object.id
            ));
        }
        let rs = super::records::records(&self.original)?;
        let styles = super::embedded_tools::document_records(&rs)
            .filter(|record| record.kind == 0x04b3)
            .collect::<Vec<_>>();
        let style = styles.get(index).ok_or_else(|| {
            format!(
                "Missing numeric caption font style {index} for object {}",
                object.id
            )
        })?;
        let payload = &self.original[style.data.clone()];
        let family_index = payload
            .get(..2)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u16::from_le_bytes)
            .ok_or("Invalid numeric caption font style")? as usize;
        let family = super::embedded_tools::document_records(&rs)
            .filter(|record| record.kind == 0x04b1)
            .nth(family_index)
            .ok_or("Missing numeric caption font family")?;
        let family_name = crate::text_style_records::decode_font_family_payload(
            &self.original[family.data.clone()],
        )?;
        if object.text_style_override.is_none()
            && family_name == "Tahoma"
            && payload.len() == 20
            && payload[2..].iter().all(|byte| *byte == 0)
        {
            return Ok(21);
        }
        let font = object
            .text_style_override
            .clone()
            .or_else(|| {
                crate::text_style_records::decode_font_payloads(
                    payload,
                    &self.original[family.data.clone()],
                )
                .ok()
            })
            .ok_or("Unsupported numeric caption font style")?;
        if font.family == "Tahoma"
            && font.size_code == 18
            && !font.bold
            && !font.italic
            && !font.underline
        {
            return Ok(27);
        }
        if font.family == "Tahoma"
            && font.size_code == 24
            && (font.bold || measurement)
            && !font.italic
            && !font.underline
        {
            return Ok(34);
        }
        if font.family == "Times New Roman"
            && font.size_code == 16
            && !font.italic
            && !font.underline
        {
            return Ok(21);
        }
        if font.family == "Times New Roman"
            && font.size_code == 24
            && !font.bold
            && !font.italic
            && !font.underline
        {
            return Ok(19);
        }
        if font.family == "黑体"
            && font.size_code == 16
            && font.bold
            && !font.italic
            && !font.underline
        {
            return Ok(21);
        }
        if font.family == "宋体"
            && font.size_code == 16
            && font.bold
            && !font.italic
            && !font.underline
        {
            return Ok(19);
        }
        Err(format!(
            "Tool placement needs a verified numeric caption font layout for object {}: {font:?}",
            object.id
        ))
    }

    /// Place a new calculation below the existing measurements and numeric captions.
    pub fn next_calculation_anchor(&self) -> (i16, i16) {
        let objects = self
            .objects
            .iter()
            .filter(|o| !o.deleted && o.page == self.active_page)
            .collect::<Vec<_>>();
        let mut bottom = 5 + objects.iter().filter(|o| o.kind.is_measurement()).count() * 28;
        for object in objects {
            let anchor = match &object.kind {
                Kind::Parameter(g) => g.anchor(),
                Kind::Calculation(g) => g.anchor(),
                Kind::MeasuredCalculation(_, g) => g.anchor(),
                Kind::LengthPairCalculation(_, _, g) => g.anchor(),
                Kind::ParameterCalculation(_, g) => g.anchor(),
                _ => continue,
            };
            bottom = bottom.max(
                (i32::from(anchor.1) + self.numeric_caption_height(object.id) as i32).max(5)
                    as usize,
            );
        }
        (10, bottom.min(i16::MAX as usize) as i16)
    }

    pub fn add_parameter_calculation(
        &mut self,
        source: usize,
        expression: &str,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot add a calculation in this sketch safely".into());
        }
        let Some(object) = self.objects.get(
            source
                .checked_sub(1)
                .ok_or("Invalid calculation source ID")?,
        ) else {
            return Err("Invalid calculation source ID".into());
        };
        if object.deleted || object.page != self.active_page {
            return Err("Select a numeric value on the active page".into());
        }
        let source_name = if matches!(object.kind, Kind::Parameter(_)) {
            self.parameter_display_name(source)
                .ok_or("Parameter display name is unavailable")?
        } else {
            "v".to_owned()
        };
        let source_value = self
            .numeric_value(source)
            .ok_or("Select a defined scalar or angle value")?;
        let group = ParameterCalculationGroup::from_expression_with_source_unit(
            u32::try_from(source).map_err(|_| "Invalid calculation source ID")?,
            &source_name,
            expression,
            anchor,
            object
                .kind
                .numeric_unit()
                .ok_or("Unsupported calculation units")?,
        )?;
        if group
            .value_with_source_unit(
                source_value,
                object
                    .kind
                    .numeric_unit()
                    .ok_or("Unsupported calculation units")?,
            )
            .is_none()
        {
            return Err("Calculation is undefined for the current source value".into());
        }
        self.checkpoint();
        let mut kind = Kind::ParameterCalculation(source, group);
        self.apply_default_calculation_precision(&mut kind);
        Ok(self.append_object(kind))
    }
    pub fn add_multi_calculation(
        &mut self,
        sources: &[usize],
        expression: &str,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add a calculation in this sketch safely".into());
        }
        let units = sources
            .iter()
            .map(|id| {
                self.objects
                    .get(id.saturating_sub(1))
                    .filter(|o| !o.deleted && o.page == self.active_page)
                    .and_then(|o| {
                        o.kind
                            .numeric_unit()
                            .or_else(|| o.kind.is_function().then_some(NumericUnit::Scalar))
                    })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("Select numeric values on the active page")?;
        let ids = sources
            .iter()
            .map(|id| u32::try_from(*id).map_err(|_| "Invalid calculation source ID"))
            .collect::<Result<Vec<_>, _>>()?;
        let group = ParameterCalculationGroup::from_sources(&ids, expression, anchor, &units)?;
        calculation_source_units(&group, sources, &self.objects)
            .ok_or("Calculation reference type disagrees with its expression")?;
        self.checkpoint();
        let mut kind = Kind::ParameterCalculation(sources[0], group);
        self.apply_default_calculation_precision(&mut kind);
        Ok(self.append_object(kind))
    }
    pub fn add_measured_length_calculation(
        &mut self,
        source: usize,
        factor: u8,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        self.add_measured_length_scaled(source, factor, anchor, false)
    }
    pub fn add_length_pair_calculation(
        &mut self,
        first: usize,
        second: usize,
        subtract: bool,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        self.add_length_pair_operation(first, second, u8::from(subtract), anchor)
    }
    pub fn add_length_pair_operation(
        &mut self,
        first: usize,
        second: usize,
        operator: u8,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot add a calculation in this sketch safely".into());
        }
        for id in [first, second] {
            if !self
                .objects
                .get(id.checked_sub(1).ok_or("Invalid source ID")?)
                .is_some_and(|o| {
                    !o.deleted && o.page == self.active_page && o.kind.is_distance_value()
                })
            {
                return Err("Select two length measurements or length calculations".into());
            }
        }
        let group = LengthPairCalculationGroup::with_operator(
            first as u32,
            second as u32,
            operator,
            anchor,
        )?;
        if self
            .distance_value_pixels(first)
            .zip(self.distance_value_pixels(second))
            .and_then(|(a, b)| group.value(a, b))
            .is_none()
        {
            return Err("Calculation is undefined for these lengths".into());
        }
        self.checkpoint();
        let mut kind = Kind::LengthPairCalculation(first, second, group);
        self.apply_default_calculation_precision(&mut kind);
        Ok(self.append_object(kind))
    }
    pub fn length_pair_calculation_cm(&self, id: usize) -> Option<f64> {
        let Kind::LengthPairCalculation(_, _, group) = &self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        if group.length_power() != 1 {
            return None;
        }
        self.length_pair_calculation_value(id)
    }
    pub fn length_pair_calculation_value(&self, id: usize) -> Option<f64> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::LengthPairCalculation(first, second, group) = &o.kind else {
            return None;
        };
        if o.deleted || *first >= id || *second >= id {
            return None;
        }
        let a = self.distance_value_pixels(*first)? / 37.79527559055118;
        let b = self.distance_value_pixels(*second)? / 37.79527559055118;
        group.value(a, b)
    }
    pub fn add_measured_length_division(
        &mut self,
        source: usize,
        divisor: u8,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        self.add_measured_length_scaled(source, divisor, anchor, true)
    }
    fn add_measured_length_scaled(
        &mut self,
        source: usize,
        factor: u8,
        anchor: (i16, i16),
        divide: bool,
    ) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || !self.safe_structure
            || !self
                .objects
                .get(source.checked_sub(1).ok_or("Invalid measurement ID")?)
                .is_some_and(|object| !object.deleted && object.kind.is_distance_value())
        {
            return Err("Select a supported length or distance measurement".into());
        }
        let source = u32::try_from(source).map_err(|_| "Invalid measurement ID")?;
        let group = if divide {
            MeasuredCalculationGroup::divide(source, factor, anchor)?
        } else {
            MeasuredCalculationGroup::multiply(source, factor, anchor)?
        };
        self.checkpoint();
        let mut kind = Kind::MeasuredCalculation(source as usize, group);
        self.apply_default_calculation_precision(&mut kind);
        Ok(self.append_object(kind))
    }
    pub fn measured_calculation_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::MeasuredCalculation(source, group) = &object.kind else {
            return None;
        };
        if object.deleted || *source >= id {
            return None;
        }
        let source_kind = &self.objects.get(source.checked_sub(1)?)?.kind;
        if !source_kind.is_distance_value() {
            return None;
        }
        // Resolve the source as a construction value so a frozen measurement
        // cannot be reopened through its original geometric predecessors.
        let measurement = self.numeric_value(*source)?;
        Some(if group.divides() {
            measurement / f64::from(group.factor())
        } else {
            measurement * f64::from(group.factor())
        })
    }
    pub fn parameter_calculation_value(&self, id: usize) -> Option<f64> {
        let (group, values, units) = self.parameter_calculation_inputs(id)?;
        let Kind::ParameterCalculation(first, _) = self.objects[id - 1].kind else {
            return None;
        };
        let sources = calculation_source_ids(first, group);
        group.value_for_sources_with_functions_angle_mode(
            &values,
            &units,
            self.angle_unit_radians,
            &|index, argument, unit| {
                self.function_value_with_unit(*sources.get(index)?, argument, unit)
            },
        )
    }

    pub fn parameter_calculation_display_value(&self, id: usize) -> Option<f64> {
        let (group, values, units) = self.parameter_calculation_inputs(id)?;
        let Kind::ParameterCalculation(first, _) = self.objects[id - 1].kind else {
            return None;
        };
        let sources = calculation_source_ids(first, group);
        group.display_value_with_functions_angle_mode(
            &values,
            &units,
            self.angle_unit_radians,
            &|index, argument, unit| {
                self.function_value_with_unit(*sources.get(index)?, argument, unit)
            },
        )
    }

    fn parameter_calculation_inputs(
        &self,
        id: usize,
    ) -> Option<(&ParameterCalculationGroup, Vec<f64>, Vec<NumericUnit>)> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::ParameterCalculation(source, group) = &object.kind else {
            return None;
        };
        if object.deleted {
            return None;
        }
        let ids = calculation_source_ids(*source, group);
        if ids.iter().any(|source| {
            *source >= id
                || self
                    .objects
                    .get(source.saturating_sub(1))
                    .is_none_or(|o| o.deleted)
        }) {
            return None;
        }
        let values = ids
            .iter()
            .zip(group.source_kinds().ok()?)
            .map(|(source, kind)| {
                if kind == CalculationSourceKind::Function {
                    Some(0.)
                } else {
                    self.numeric_value(*source)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        let units = calculation_source_units(group, &ids, &self.objects)?;
        Some((group, values, units))
    }

    pub fn parameter_calculation_expression(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::ParameterCalculation(source, group) = &object.kind else {
            return None;
        };
        if object.deleted || *source >= id {
            return None;
        }
        let mut names = Vec::new();
        for source in calculation_source_ids(*source, group) {
            if source >= id {
                return None;
            }
            let source_object = self.objects.get(source.checked_sub(1)?)?;
            if source_object.deleted {
                return None;
            }
            let name = self.calculation_source_name(source)?;
            names.push(name);
        }
        Some(group.expression_for_sources(&names.iter().map(String::as_str).collect::<Vec<_>>()))
    }
    pub(super) fn calculation_source_name(&self, source: usize) -> Option<String> {
        let source_object = self.objects.get(source.checked_sub(1)?)?;
        if source_object.deleted {
            return None;
        }
        let source = &source;
        let name = match &source_object.kind {
            Kind::FunctionDefinition(_)
            | Kind::ReferencedFunction(_, _)
            | Kind::DerivativeFunction(_, _)
            | Kind::DataFunction { .. } => self.function_symbol(*source)?,
            Kind::Parameter(parameter) => {
                let name = self.parameter_display_name(*source)?;
                if parameter.display_mode() == 0 {
                    format!("({name})")
                } else {
                    name
                }
            }
            Kind::ParameterCalculation(_, g) if g.display_mode() == 3 => {
                if let Some(name) = source_object
                    .label
                    .as_ref()
                    .and_then(|label| label.name_utf8().ok())
                    .filter(|name| !name.is_empty())
                {
                    name.to_owned()
                } else {
                    self.parameter_calculation_expression(*source)?
                }
            }
            Kind::ParameterCalculation(_, _) => self.parameter_calculation_expression(*source)?,
            Kind::AngleMeasurement(a, b, c) => format!(
                "∠{}{}{}",
                self.coordinate_point_name(*a)?,
                self.coordinate_point_name(*b)?,
                self.coordinate_point_name(*c)?
            ),
            Kind::ArcAngle(_) => format!("arc angle {source}"),
            Kind::Calculation(g) if g.is_native_radian_angle() && g.display_mode() == 3 => {
                source_object
                    .label
                    .as_ref()
                    .and_then(|label| label.name_utf8().ok())
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned)
                    .unwrap_or_else(|| g.expression())
            }
            Kind::Calculation(g) => g.expression(),
            Kind::SegmentRatio(_, _) | Kind::PointRatio(_, _, _) => format!("Ratio {source}"),
            Kind::SlopeMeasurement(_, _) => format!("Slope {source}"),
            Kind::LengthPairCalculation(_, _, _)
            | Kind::MeasuredCalculation(_, _)
            | Kind::SegmentLength(_)
            | Kind::PointDistance(_, _)
            | Kind::PointLineDistance(_, _)
            | Kind::PolygonPerimeter(_)
            | Kind::CircleCircumference(_)
            | Kind::CircleRadius(_)
            | Kind::ArcLength(_)
            | Kind::CircleArcLength(_, _, _) => self.length_measurement_name(*source)?,
            Kind::CircleArea(_) | Kind::PolygonArea(_) => format!("Area {source}"),
            Kind::CoordinateMeasurement { kind, first, .. } => {
                format!("{kind:?} {}", self.coordinate_point_name(*first)?)
            }
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                first,
                ..
            } => format!("Value of {}", self.coordinate_point_name(*first)?),
            _ => return None,
        };
        Some(name)
    }
    pub fn length_measurement_name(&self, id: usize) -> Option<String> {
        if let Kind::ParameterCalculation(_, g) = &self.objects.get(id.checked_sub(1)?)?.kind {
            return (g.unit() == NumericUnit::Length)
                .then(|| self.parameter_calculation_expression(id))
                .flatten();
        }
        if let Kind::LengthPairCalculation(first, second, group) =
            &self.objects.get(id.checked_sub(1)?)?.kind
        {
            if *first >= id || *second >= id {
                return None;
            }
            return Some(format!(
                "({} {} {})",
                self.length_measurement_name(*first)?,
                group.operator_text(),
                self.length_measurement_name(*second)?
            ));
        }
        if let Kind::MeasuredCalculation(source, group) =
            &self.objects.get(id.checked_sub(1)?)?.kind
        {
            if *source >= id {
                return None;
            }
            let source_name = self.length_measurement_name(*source)?;
            return Some(format!(
                "({source_name} {} {})",
                if group.divides() { "÷" } else { "·" },
                group.factor()
            ));
        }
        let (first, second) = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::SegmentLength(segment) => {
                let Kind::Segment(first, second) = self.objects.get(segment.checked_sub(1)?)?.kind
                else {
                    return None;
                };
                (first, second)
            }
            Kind::PointDistance(first, second) => (first, second),
            Kind::PointLineDistance(_, _) => return Some("Distance".into()),
            Kind::PolygonPerimeter(_) => return Some("Perimeter".into()),
            Kind::CircleCircumference(_) => return Some("Circumference".into()),
            Kind::CircleRadius(_) => return Some("Radius".into()),
            Kind::ArcLength(_) | Kind::CircleArcLength(_, _, _) => {
                return Some("Arc Length".into());
            }
            _ => return None,
        };
        Some(format!(
            "{}{}",
            self.coordinate_point_name(first)?,
            self.coordinate_point_name(second)?
        ))
    }
}

fn numeric_measurement_caption_shape(
    kind: &Kind,
) -> Option<crate::table_records::NumericCaptionShape> {
    use crate::table_records::NumericCaptionShape;
    match kind {
        Kind::SegmentLength(_)
        | Kind::PointDistance(_, _)
        | Kind::NumericAlias { .. }
        | Kind::AngleMeasurement(_, _, _)
        | Kind::SlopeMeasurement(_, _)
        | Kind::CircleRadius(_)
        | Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            ..
        } => Some(NumericCaptionShape::Flat),
        Kind::PolygonArea(_) | Kind::CircleArea(_) => Some(NumericCaptionShape::SquareUnit),
        _ => None,
    }
}

fn numeric_named_caption_layout(
    mut base: crate::table_records::NumericCaptionLayout,
    name: &str,
) -> Result<crate::table_records::NumericCaptionLayout, String> {
    // Native m[1] has a subscript extending one pixel below the plain math row
    // in both independently controlled18Normal and24Bold profiles.
    if name == "m[1]" {
        base.below = base
            .below
            .checked_add(1)
            .ok_or("Numeric caption layout overflow")?;
    } else if name.contains('[') || name.contains(']') {
        return Err("Indexed numeric caption name needs a native glyph layout control".into());
    }
    Ok(base)
}
