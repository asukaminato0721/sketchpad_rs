use super::geometry::ratio_of_points;
use super::kind::{is_point_kind, is_straight_kind};
use super::{Document, Kind, LineMode, Point};
use crate::coordinate_measure_records::{
    CoordinateBasis, CoordinateMeasureKind, CoordinateMeasureValue, ScreenPoint,
};
use crate::graph::GraphPoint;
use crate::table_records::NumericUnit;
use crate::value_equation_records::{CartesianFrame, EquationValue, ValueEquationKind};
use crate::{coordinate_measure_records, value_equation_records};
use std::collections::{BTreeMap, BTreeSet};

impl Document {
    pub fn add_point_numeric_label(&mut self, point: usize, value: usize) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add a numeric label in this sketch safely".into());
        }
        if !self
            .objects
            .get(point.checked_sub(1).ok_or("Invalid point ID")?)
            .is_some_and(|o| !o.deleted && is_point_kind(&o.kind))
            || self.numeric_value(value).is_none()
        {
            return Err("Select a point and a supported numeric value".into());
        }
        self.checkpoint();
        Ok(self.append_object(Kind::PointNumericLabel(point, value)))
    }
    pub fn unitless_numeric_value(&self, id: usize) -> Option<f64> {
        if self.objects.get(id.checked_sub(1)?)?.kind.numeric_unit()? != NumericUnit::Scalar {
            return None;
        }
        self.numeric_value(id)
    }
    pub fn numeric_value(&self, id: usize) -> Option<f64> {
        if let Some(value) = self.evaluation_numeric(id) {
            return value;
        }
        let o = self.objects.get(id.checked_sub(1)?)?;
        if o.deleted {
            return None;
        }
        match &o.kind {
            Kind::Parameter(group) => Some(group.value()),
            Kind::ParameterCalculation(_, _) => self.parameter_calculation_value(id),
            Kind::Calculation(group) => group.evaluated_value(),
            Kind::PointRatio(_, _, _) => self.ratio_value(id),
            Kind::SlopeMeasurement(_, _) => self.slope_value(id),
            Kind::AngleMeasurement(_, _, _) => self.angle_degrees(id),
            Kind::ArcAngle(_) => self.arc_angle_degrees(id),
            Kind::SegmentLength(_) => self.segment_length_cm(id),
            Kind::PointDistance(_, _) => self.point_distance_cm(id),
            Kind::PointLineDistance(_, _) => self.point_line_distance_cm(id),
            Kind::PolygonPerimeter(_) => self.perimeter_cm(id),
            Kind::CircleCircumference(_) => self.circle_circumference_cm(id),
            Kind::CircleRadius(_) => self.circle_radius_cm(id),
            Kind::ArcLength(_) => self.arc_length_cm(id),
            Kind::CircleArea(_) => self.circle_area_cm2(id),
            Kind::PolygonArea(_) => self.polygon_area_cm2(id),
            Kind::MeasuredCalculation(_, _) => self.measured_calculation_cm(id),
            Kind::LengthPairCalculation(_, _, _) => self.length_pair_calculation_value(id),
            Kind::CoordinateMeasurement { .. } => match self.coordinate_measure_value(id)? {
                CoordinateMeasureValue::Scalar(v) => Some(v),
                _ => None,
            },
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                ..
            } => match self.value_equation_value(id)? {
                EquationValue::PointValue(v) => Some(v),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn measurement_value_text(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        Some(crate::table_records::format_numeric_value(
            self.numeric_value(id)?,
            usize::from(object.measurement_precision?),
        ))
    }
    pub fn measurement_text(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let name = match object.kind {
            Kind::SegmentLength(_) => "Length",
            Kind::PointDistance(_, _) | Kind::PointLineDistance(_, _) => "Distance",
            Kind::PointRatio(_, _, _) => "Ratio",
            Kind::PolygonPerimeter(_) => "Perimeter",
            Kind::CircleCircumference(_) => "Circumference",
            Kind::CircleRadius(_) => "Radius",
            Kind::CircleArea(_) | Kind::PolygonArea(_) => "Area",
            Kind::ArcAngle(_) => "Arc Angle",
            Kind::ArcLength(_) => "Arc Length",
            Kind::AngleMeasurement(_, _, _) => "Angle",
            _ => return None,
        };
        let value = self.measurement_value_text(id)?;
        let unit = object.kind.numeric_unit()?.suffix();
        Some(format!("{name} = {value}{unit}"))
    }
    /// Move a numeric measurement caption without altering its geometry or value.
    pub fn move_measurement_caption(
        &mut self,
        id: usize,
        anchor: (i16, i16),
    ) -> Result<(), String> {
        self.move_measurement_caption_inner(id, anchor, true)
    }
    pub fn drag_measurement_caption(
        &mut self,
        id: usize,
        anchor: (i16, i16),
    ) -> Result<(), String> {
        self.move_measurement_caption_inner(id, anchor, false)
    }
    fn move_measurement_caption_inner(
        &mut self,
        id: usize,
        anchor: (i16, i16),
        checkpoint: bool,
    ) -> Result<(), String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid measurement ID")?)
            .ok_or("Invalid measurement ID")?;
        if object.deleted
            || object.page != self.active_page
            || (object.measurement_anchor.is_none() && !object.kind.has_measurement_precision())
        {
            return Err("Select a numeric measurement caption on the current page".into());
        }
        if self.image_read_only
            || (self.page_count == 1 && !self.safe_structure)
            || (self.page_count > 1
                && (self.source_count != self.objects.len()
                    || self.objects.iter().any(|object| object.deleted)))
            || self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
        {
            return Err("Cannot move a measurement caption in this sketch safely".into());
        }
        if object.measurement_anchor == Some(anchor) {
            return Ok(());
        }
        let mut candidate = self.clone();
        candidate.objects[id - 1].measurement_anchor = Some(anchor);
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
    pub fn set_measurement_precision(
        &mut self,
        ids: &[usize],
        precision: u16,
    ) -> Result<usize, String> {
        if precision > 5 || ids.is_empty() {
            return Err("Select measurements and a decimal precision from 0 to 5".into());
        }
        if self.page_count == 1 && (self.partial_view || !self.safe_structure) {
            return Err("Cannot edit measurement precision in this sketch safely".into());
        }
        let mut changed = BTreeSet::new();
        for id in ids {
            let object = self
                .objects
                .get(id.checked_sub(1).ok_or("Invalid measurement ID")?)
                .ok_or("Invalid measurement ID")?;
            if object.deleted
                || object.page != self.active_page
                || !object.kind.has_measurement_precision()
            {
                return Err("Select measurements on the current page".into());
            }
            if object.measurement_precision != Some(precision) {
                changed.insert(*id);
            }
        }
        if !changed.is_empty() {
            self.checkpoint();
            for id in &changed {
                self.objects[*id - 1].measurement_precision = Some(precision);
            }
        }
        Ok(changed.len())
    }
    /// Change saved display precision atomically without rounding numeric inputs.
    pub fn set_numeric_precision(
        &mut self,
        ids: &[usize],
        precision: u16,
    ) -> Result<usize, String> {
        if precision > 5 || ids.is_empty() {
            return Err(
                "Select measurements or calculations and a decimal precision from 0 to 5".into(),
            );
        }
        if self.page_count == 1 && (self.partial_view || !self.safe_structure) {
            return Err("Cannot edit numeric precision in this sketch safely".into());
        }
        let mut changes = BTreeMap::new();
        for id in ids {
            let object = self
                .objects
                .get(id.checked_sub(1).ok_or("Invalid numeric ID")?)
                .ok_or("Invalid numeric ID")?;
            if object.deleted || object.page != self.active_page {
                return Err("Select measurements or calculations on the current page".into());
            }
            if object.kind.has_measurement_precision() {
                if object.measurement_precision != Some(precision) {
                    changes.insert(*id, None);
                }
                continue;
            }
            if self.page_count != 1 {
                return Err(
                    "Calculation precision editing requires a supported single-page sketch".into(),
                );
            }
            let mut kind = object.kind.clone();
            let current = match &mut kind {
                Kind::Calculation(group) => {
                    let current = group.precision();
                    group.set_precision(precision)?;
                    current
                }
                Kind::ParameterCalculation(_, group) => {
                    let current = group.precision();
                    group.set_precision(precision)?;
                    current
                }
                Kind::MeasuredCalculation(_, group) => {
                    let current = group.precision();
                    group.set_precision(precision)?;
                    current
                }
                Kind::LengthPairCalculation(_, _, group) => {
                    let current = group.precision();
                    group.set_precision(precision)?;
                    current
                }
                _ => return Err("Select measurements or calculations on the current page".into()),
            };
            if current != usize::from(precision) {
                changes.insert(*id, Some(kind));
            }
        }
        if !changes.is_empty() {
            self.checkpoint();
            for (id, kind) in &changes {
                if let Some(kind) = kind {
                    self.objects[*id - 1].kind = kind.clone();
                } else {
                    self.objects[*id - 1].measurement_precision = Some(precision);
                }
            }
        }
        Ok(changes.len())
    }
    /// Format current numeric values, including scoped substitutions, using the
    /// source's own precision and quantity suffix.
    pub fn numeric_value_text(&self, id: usize) -> Option<String> {
        let object=self.objects.get(id.checked_sub(1)?)?;
        let value=self.numeric_value(id)?;
        if value.is_nan(){return Some("undefined".into())}
        let precision=match &object.kind {
            Kind::Parameter(group)=>group.precision(),
            Kind::ParameterCalculation(_,group)=>group.precision(),
            Kind::Calculation(group)=>group.precision(),
            Kind::MeasuredCalculation(_,group)=>group.precision(),
            Kind::LengthPairCalculation(_,_,group)=>group.precision(),
            _=>usize::from(object.measurement_precision?),
        };
        let number=if value==f64::INFINITY {"∞".into()}
            else if value==f64::NEG_INFINITY {"−∞".into()}
            else {crate::table_records::format_numeric_value(value,precision)};
        Some(format!("{number}{}",object.kind.numeric_unit()?.suffix()))
    }
    pub fn point_numeric_label(&self, id: usize) -> Option<(Point, String)> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PointNumericLabel(point, source) = o.kind else {
            return None;
        };
        if o.deleted {
            return None;
        }
        let position = self.position(point)?;
        let Some(_) = self.numeric_value(source) else {
            return Some((position, "undefined".into()));
        };
        let text = self.numeric_value_text(source)?;
        Some((position, text))
    }
    pub fn slope_value(&self, id: usize) -> Option<f64> {
        let Kind::SlopeMeasurement(line, frame) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (a, b, _) = self.line_points(line)?;
        self.graph_frame(frame)?
            .slope(GraphPoint { x: a.x, y: a.y }, GraphPoint { x: b.x, y: b.y })
    }
    pub fn coordinate_measure_value(&self, id: usize) -> Option<CoordinateMeasureValue> {
        let Kind::CoordinateMeasurement {
            kind,
            first,
            second,
            frame,
        } = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let graph_frame = self.graph_frame(frame)?;
        let to_screen = |point: Point| ScreenPoint {
            x: point.x,
            y: point.y,
        };
        let basis = CoordinateBasis {
            origin: to_screen(Point {
                x: graph_frame.origin.x,
                y: graph_frame.origin.y,
            }),
            x_unit: to_screen(Point {
                x: graph_frame.x_unit.x,
                y: graph_frame.x_unit.y,
            }),
            y_unit: to_screen(Point {
                x: graph_frame.y_unit.x,
                y: graph_frame.y_unit.y,
            }),
        };
        coordinate_measure_records::evaluate(
            kind,
            basis,
            to_screen(self.position(first)?),
            second.and_then(|id| self.position(id)).map(to_screen),
        )
    }
    pub fn coordinate_measure_text(&self, id: usize) -> Option<String> {
        let Kind::CoordinateMeasurement { kind, first, .. } =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let name = self.coordinate_point_name(first)?;
        let value = self.coordinate_measure_value(id)?;
        match (kind, value) {
            (CoordinateMeasureKind::Coordinates, CoordinateMeasureValue::Pair(p)) => {
                Some(format!("{name}: ({:.2}, {:.2})", p.x, p.y))
            }
            (CoordinateMeasureKind::Abscissa, CoordinateMeasureValue::Scalar(_)) => {
                Some(format!("x_{name} = {}", self.measurement_value_text(id)?))
            }
            (CoordinateMeasureKind::Ordinate, CoordinateMeasureValue::Scalar(_)) => {
                Some(format!("y_{name} = {}", self.measurement_value_text(id)?))
            }
            (CoordinateMeasureKind::CoordinateDistance, CoordinateMeasureValue::Scalar(_)) => Some(
                format!("Coordinate Distance = {}", self.measurement_value_text(id)?),
            ),
            _ => None,
        }
    }
    pub fn value_equation_value(&self, id: usize) -> Option<EquationValue> {
        let Kind::ValueEquation {
            kind,
            first,
            second,
        } = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let screen = |point: Point| value_equation_records::ScreenPoint {
            x: point.x,
            y: point.y,
        };
        match kind {
            ValueEquationKind::PointValue => {
                let point = screen(self.position(first)?);
                match self.objects.get(second - 1)?.kind {
                    ref kind if is_straight_kind(kind) => {
                        let (a, b, _) = self.point_constraint_line_points(second)?;
                        value_equation_records::segment_point_value(point, screen(a), screen(b))
                    }
                    ref kind if kind.is_circle() => {
                        if let Kind::PointOnCircle(host, x, y) = self.objects.get(first - 1)?.kind {
                            if host == second {
                                return value_equation_records::circle_point_value(x, y);
                            }
                        }
                        let (center, _) = self.circle_points(second)?;
                        let dx = point.x - center.x;
                        let dy = center.y - point.y;
                        let length = dx.hypot(dy);
                        if !length.is_finite() || length == 0. {
                            return None;
                        }
                        value_equation_records::circle_point_value(dx / length, dy / length)
                    }
                    Kind::ThreePointArc(_, _, _)
                    | Kind::ArcOnCircle(_, _, _)
                    | Kind::CenterArc(_, _, _) => {
                        if let Kind::PointOnArc(host, t) = self.objects.get(first - 1)?.kind {
                            if host == second {
                                return t.is_finite().then_some(EquationValue::PointValue(t));
                            }
                        }
                        self.arc_parameter(
                            second,
                            Point {
                                x: point.x,
                                y: point.y,
                            },
                        )
                        .map(EquationValue::PointValue)
                    }
                    ref kind if kind.is_polygon() => {
                        if let Kind::PointOnPolygon(host, edge, t) = self.objects.get(first - 1)?.kind
                            && host == second
                        {
                            return self
                                .polygon_boundary_parameter(host, edge, t)
                                .map(EquationValue::PointValue);
                        }
                        // Native projects independent points to the closest edge,
                        // then measures progress along the complete listed perimeter.
                        let (edge, t) = self.polygon_boundary_projection(
                            second,
                            Point {
                                x: point.x,
                                y: point.y,
                            },
                        )?;
                        self.polygon_boundary_parameter(second, edge, t)
                            .map(EquationValue::PointValue)
                    }
                    _ => None,
                }
            }
            ValueEquationKind::LineEquation | ValueEquationKind::CircleEquation => {
                let graph = self.graph_frame(second)?;
                let frame = CartesianFrame {
                    origin: value_equation_records::ScreenPoint {
                        x: graph.origin.x,
                        y: graph.origin.y,
                    },
                    x_unit: value_equation_records::ScreenPoint {
                        x: graph.x_unit.x,
                        y: graph.x_unit.y,
                    },
                    y_unit: value_equation_records::ScreenPoint {
                        x: graph.y_unit.x,
                        y: graph.y_unit.y,
                    },
                };
                match kind {
                    ValueEquationKind::LineEquation => {
                        let (a, b, _) = self.line_points(first)?;
                        value_equation_records::line_equation(frame, screen(a), screen(b))
                    }
                    ValueEquationKind::CircleEquation => {
                        let (center, radius) = self.circle_points(first)?;
                        value_equation_records::circle_equation(
                            frame,
                            screen(center),
                            screen(Point {
                                x: center.x + radius,
                                y: center.y,
                            }),
                        )
                    }
                    ValueEquationKind::PointValue => unreachable!(),
                }
            }
        }
    }
    pub fn value_equation_text(&self, id: usize) -> Option<String> {
        let value = self.value_equation_value(id)?;
        let signed = |number: f64| {
            let number = if number == 0. { 0. } else { number };
            if number < 0. {
                format!("−{:.2}", -number)
            } else {
                format!("{number:.2}")
            }
        };
        let shifted = |number: f64| {
            if number < 0. {
                format!("+ {:.2}", -number)
            } else {
                format!("− {number:.2}")
            }
        };
        Some(match value {
            EquationValue::PointValue(_) => {
                format!("Value of Point = {}", self.measurement_value_text(id)?)
            }
            EquationValue::Line { slope, intercept } => {
                let sign = if intercept < 0. { "−" } else { "+" };
                if slope == 0. {
                    format!("y = {}", signed(intercept))
                } else {
                    format!("y = {}x {sign} {:.2}", signed(slope), intercept.abs())
                }
            }
            EquationValue::VerticalLine { x } => format!("x = {}", signed(x)),
            EquationValue::Circle { center, radius } => format!(
                "(x {})² + (y {})² = {radius:.2}²",
                shifted(center.x),
                shifted(center.y)
            ),
        })
    }
    pub(super) fn coordinate_point_name(&self, first: usize) -> Option<String> {
        Some(
            self.objects
                .get(first - 1)?
                .label
                .as_ref()
                .and_then(|label| label.name_utf8().ok())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    let ordinal = self
                        .objects
                        .iter()
                        .take(first)
                        .filter(|object| is_point_kind(&object.kind))
                        .count();
                    if (1..=26).contains(&ordinal) {
                        ((b'A' + ordinal as u8 - 1) as char).to_string()
                    } else {
                        format!("Point {first}")
                    }
                }),
        )
    }
    pub fn segment_length_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::SegmentLength(segment) = object.kind else {
            return None;
        };
        let (a, b, LineMode::Segment) = self.line_points(segment)? else {
            return None;
        };
        Some(a.distance(b) / 37.79527559055118)
    }
    /// Native marked translations accept signed lengths and unitless centimeters.
    pub fn marked_distance_value_pixels(&self, id: usize) -> Option<f64> {
        let kind = &self.objects.get(id.checked_sub(1)?)?.kind;
        if !kind.is_marked_distance_value() {
            return None;
        }
        let pixels = self.numeric_value(id)? * 37.79527559055118;
        pixels.is_finite().then_some(pixels)
    }
    pub fn distance_value_pixels(&self, id: usize) -> Option<f64> {
        if !self
            .objects
            .get(id.checked_sub(1)?)?
            .kind
            .is_distance_value()
        {
            return None;
        }
        let cm = self.numeric_value(id)?;
        let pixels = cm * 37.79527559055118;
        (pixels.is_finite() && pixels >= 0.).then_some(pixels)
    }
    pub fn point_line_distance_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PointLineDistance(point, host) = object.kind else {
            return None;
        };
        let point = self.position(point)?;
        let (a, b, _) = self.line_points(host)?;
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length = dx.hypot(dy);
        if !length.is_finite() || length == 0. {
            return None;
        }
        let distance = ((point.x - a.x) * (dy / length) - (point.y - a.y) * (dx / length)).abs()
            / 37.79527559055118;
        distance.is_finite().then_some(distance)
    }

    pub fn point_distance_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PointDistance(a, b) = object.kind else {
            return None;
        };
        Some(self.position(a)?.distance(self.position(b)?) / 37.79527559055118)
    }
    pub fn ratio_for_points(&self, a: usize, b: usize, c: usize) -> Option<f64> {
        ratio_of_points(self.position(a)?, self.position(b)?, self.position(c)?)
    }
    pub fn ratio_value(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PointRatio(a, b, c) = object.kind else {
            return None;
        };
        self.ratio_for_points(a, b, c)
    }
    pub fn polygon_perimeter_cm(&self, id: usize) -> Option<f64> {
        let points = self.polygon_points(id)?;
        if points.len() < 3 {
            return None;
        }
        let perimeter = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .map(|(a, b)| a.distance(*b))
            .sum::<f64>();
        perimeter
            .is_finite()
            .then_some(perimeter / 37.79527559055118)
    }
    pub fn perimeter_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PolygonPerimeter(polygon) = object.kind else {
            return None;
        };
        self.polygon_perimeter_cm(polygon)
    }
    pub fn circle_circumference_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleCircumference(circle) = object.kind else {
            return None;
        };
        let (_, radius) = self.circle_points(circle)?;
        Some(std::f64::consts::TAU * radius / 37.79527559055118)
    }
    pub fn circle_radius_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleRadius(circle) = object.kind else {
            return None;
        };
        let (_, radius) = self.circle_points(circle)?;
        Some(radius / 37.79527559055118)
    }
    pub fn circle_area_cm2(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleArea(circle) = object.kind else {
            return None;
        };
        let (_, radius) = self.circle_points(circle)?;
        let cm = radius / 37.79527559055118;
        Some(std::f64::consts::PI * cm * cm)
    }
    pub fn polygon_area_cm2(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PolygonArea(polygon) = object.kind else {
            return None;
        };
        let points = self.polygon_points(polygon)?;
        if points.len() < 3 {
            return None;
        }
        let origin = points[0];
        let twice_area = points
            .iter()
            .skip(1)
            .zip(points.iter().skip(2))
            .map(|(a, b)| (a.x - origin.x) * (b.y - origin.y) - (a.y - origin.y) * (b.x - origin.x))
            .sum::<f64>();
        let value = twice_area.abs() / (2. * 37.79527559055118_f64.powi(2));
        value.is_finite().then_some(value)
    }
    pub fn arc_angle_degrees(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::ArcAngle(arc) = object.kind else {
            return None;
        };
        if self.linear_three_point_arc(arc).is_some() {
            return Some(0.);
        }
        Some(self.arc_geometry(arc)?.3.abs().to_degrees())
    }
    pub fn arc_length_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::ArcLength(arc) = object.kind else {
            return None;
        };
        if let Some((start, end)) = self.linear_three_point_arc(arc) {
            return Some(start.distance(end) / 37.79527559055118);
        }
        let (_, radius, _, sweep, _) = self.arc_geometry(arc)?;
        Some(radius * sweep.abs() / 37.79527559055118)
    }
    pub fn angle_degrees(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::AngleMeasurement(a, vertex, c) = object.kind else {
            return None;
        };
        self.angle_for_points(a, vertex, c)
    }
    pub fn angle_for_points(&self, a: usize, vertex: usize, c: usize) -> Option<f64> {
        let a = self.position(a)?;
        let vertex = self.position(vertex)?;
        let c = self.position(c)?;
        let (ax, ay) = (a.x - vertex.x, a.y - vertex.y);
        let (cx, cy) = (c.x - vertex.x, c.y - vertex.y);
        let (al, cl) = (ax.hypot(ay), cx.hypot(cy));
        if !al.is_finite() || !cl.is_finite() || al == 0. || cl == 0. {
            return None;
        }
        let (ax, ay, cx, cy) = (ax / al, ay / al, cx / cl, cy / cl);
        Some(
            (ax * cy - ay * cx)
                .abs()
                .atan2(ax * cx + ay * cy)
                .to_degrees(),
        )
    }
}
