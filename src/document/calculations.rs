use super::{Document, Kind, Object};
use crate::table_records::{
    CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup, NumericUnit,
    ParameterCalculationGroup, ParameterGroup,
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
    let units = refs
        .iter()
        .map(|id| {
            objects
                .get(id.checked_sub(1)?)
                .and_then(|o| o.kind.numeric_unit())
        })
        .collect::<Option<Vec<_>>>()?;
    let group = group.with_source_units(&units).ok()?;
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
    fn update_parameter_value(
        &mut self,
        id: usize,
        value: f64,
        checkpoint: bool,
    ) -> Result<(), String> {
        if id == 0 || (self.page_count == 1 && (self.partial_view || !self.safe_structure))
            || (self.page_count > 1 && (self.image_read_only
                || self.source_count != self.objects.len()
                || self.objects.iter().any(|object| object.deleted || matches!(object.kind, Kind::Unsupported(_))))) {
            return Err("Cannot edit a parameter in this sketch".into());
        }
        let object = self.objects.get(id - 1).ok_or("Select a parameter")?;
        if object.deleted || object.page != self.active_page
            || (self.page_count > 1 && id > self.source_count) {
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
                if let Kind::ValueTablePair(first, second, table) = &object.kind {
                    if !object.deleted && (*first == id || *second == id) {
                        if object.page != page {
                            return Err("Parameter table crosses sketch pages".into());
                        }
                        let mut changed = table.clone();
                        changed.set_last_pair_axis_value(usize::from(*second == id), value)?;
                        tables.push((index, *first, *second, changed));
                    }
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
        Ok(self.append_object(Kind::Calculation(group)))
    }
    fn numeric_caption_height(&self, id: usize) -> usize {
        let Some(object) = id.checked_sub(1).and_then(|i| self.objects.get(i)) else {
            return 33;
        };
        match &object.kind {
            Kind::PointRatio(..) => 64,
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
        let source_name = if let Kind::Parameter(parameter) = &object.kind {
            parameter.name().to_owned()
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
        Ok(self.append_object(Kind::ParameterCalculation(source, group)))
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
                    .and_then(|o| o.kind.numeric_unit())
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("Select numeric values on the active page")?;
        let ids = sources
            .iter()
            .map(|id| u32::try_from(*id).map_err(|_| "Invalid calculation source ID"))
            .collect::<Result<Vec<_>, _>>()?;
        let group = ParameterCalculationGroup::from_sources(&ids, expression, anchor, &units)?;
        self.checkpoint();
        Ok(self.append_object(Kind::ParameterCalculation(sources[0], group)))
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
        Ok(self.append_object(Kind::LengthPairCalculation(first, second, group)))
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
        Ok(self.append_object(Kind::MeasuredCalculation(source as usize, group)))
    }
    pub fn measured_calculation_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::MeasuredCalculation(source, group) = &object.kind else {
            return None;
        };
        if object.deleted || *source >= id {
            return None;
        }
        let measurement = match self.objects.get(source.checked_sub(1)?)?.kind {
            Kind::SegmentLength(_) => self.segment_length_cm(*source)?,
            Kind::PointDistance(_, _) => self.point_distance_cm(*source)?,
            Kind::PointLineDistance(_, _) => self.point_line_distance_cm(*source)?,
            Kind::PolygonPerimeter(_) => self.perimeter_cm(*source)?,
            Kind::CircleCircumference(_) => self.circle_circumference_cm(*source)?,
            Kind::CircleRadius(_) => self.circle_radius_cm(*source)?,
            Kind::ArcLength(_) => self.arc_length_cm(*source)?,
            Kind::MeasuredCalculation(_, _) => self.measured_calculation_cm(*source)?,
            Kind::LengthPairCalculation(_, _, _) => self.length_pair_calculation_cm(*source)?,
            Kind::ParameterCalculation(_, ref g) if g.unit() == NumericUnit::Length => {
                self.parameter_calculation_value(*source)?
            }
            _ => return None,
        };
        Some(if group.divides() {
            measurement / f64::from(group.factor())
        } else {
            measurement * f64::from(group.factor())
        })
    }
    pub fn parameter_calculation_value(&self, id: usize) -> Option<f64> {
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
            .map(|source| self.numeric_value(*source))
            .collect::<Option<Vec<_>>>()?;
        let units = ids
            .iter()
            .map(|source| self.objects[*source - 1].kind.numeric_unit())
            .collect::<Option<Vec<_>>>()?;
        group.value_for_sources(&values, &units)
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
            let source = &source;
            let name = match &source_object.kind {
                Kind::Parameter(parameter) => parameter.name().to_owned(),
                Kind::ParameterCalculation(_, _) => {
                    self.parameter_calculation_expression(*source)?
                }
                Kind::AngleMeasurement(a, b, c) => format!(
                    "∠{}{}{}",
                    self.coordinate_point_name(*a)?,
                    self.coordinate_point_name(*b)?,
                    self.coordinate_point_name(*c)?
                ),
                Kind::ArcAngle(_) => format!("arc angle {source}"),
                Kind::Calculation(g) => g.expression(),
                Kind::PointRatio(_, _, _) => format!("Ratio {source}"),
                Kind::SlopeMeasurement(_, _) => format!("Slope {source}"),
                Kind::LengthPairCalculation(_, _, _)
                | Kind::MeasuredCalculation(_, _)
                | Kind::SegmentLength(_)
                | Kind::PointDistance(_, _)
                | Kind::PointLineDistance(_, _)
                | Kind::PolygonPerimeter(_)
                | Kind::CircleCircumference(_)
                | Kind::CircleRadius(_)
                | Kind::ArcLength(_) => self.length_measurement_name(*source)?,
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
            names.push(name);
        }
        Some(group.expression_for_sources(&names.iter().map(String::as_str).collect::<Vec<_>>()))
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
            Kind::ArcLength(_) => return Some("Arc Length".into()),
            _ => return None,
        };
        Some(format!(
            "{}{}",
            self.coordinate_point_name(first)?,
            self.coordinate_point_name(second)?
        ))
    }
}
