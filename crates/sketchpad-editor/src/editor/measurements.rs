use super::{CircleMeasure, Editor};
use sketchpad_engine::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_engine::document::{Kind, LineMode};
use sketchpad_engine::value_equation_records::ValueEquationKind;

impl Editor {
    pub(super) fn attach_numeric_label_selected(&mut self) {
        let point = self
            .doc
            .selected
            .iter()
            .copied()
            .find(|id| self.doc.position(*id).is_some());
        let value = self
            .doc
            .selected
            .iter()
            .copied()
            .find(|id| self.doc.numeric_value(*id).is_some());
        let result = if self.doc.selected.len() == 2 {
            point
                .zip(value)
                .ok_or_else(|| "Select one point and one numeric value".to_string())
                .and_then(|(point, value)| self.doc.add_point_numeric_label(point, value))
        } else {
            Err("Select one point and one numeric value".into())
        };
        match result {
            Ok(id) => {
                self.select_only(id);
                self.status = format!("Created attached numeric label {id}");
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn measure_length(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [segment] = self.selection_order.as_slice() else {
            self.status = "Select one segment to measure".into();
            return;
        };
        let segment = *segment;
        if !matches!(
            self.doc.line_points(segment),
            Some((_, _, LineMode::Segment))
        ) {
            self.status = "Select one segment to measure".into();
            return;
        }
        if self.doc.objects[segment - 1].page != self.doc.active_page {
            self.status = "Select a segment on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::SegmentLength(segment));
        self.select_only(id);
        self.status = format!("Measured segment {segment}");
    }
    fn coordinate_selection(&self) -> (Option<usize>, Vec<usize>, bool) {
        let mut selected_origin = None;
        let mut selected_distances = Vec::new();
        let mut valid_distance_selection = true;
        for &id in &self.selection_order {
            match self
                .doc
                .objects
                .get(id.saturating_sub(1))
                .map(|object| &object.kind)
            {
                Some(kind) if kind.is_defining_distance() => {
                    selected_distances.push(id);
                }
                Some(_) if selected_origin.is_none() && self.doc.position(id).is_some() => {
                    selected_origin = Some(id);
                }
                _ => valid_distance_selection = false,
            }
        }
        (
            selected_origin,
            selected_distances,
            valid_distance_selection,
        )
    }
    pub(super) fn define_coordinate_system_label(&self) -> &'static str {
        match self.selection_order.as_slice() {
            [circle] if self.doc.circle_points(*circle).is_some() => "Define Unit Circle",
            [origin] if self.doc.position(*origin).is_some() => "Define Origin",
            _ => {
                let (_, distances, valid) = self.coordinate_selection();
                if valid {
                    match distances.len() {
                        1 => "Define Unit Distance",
                        2 => "Define Unit Distances",
                        _ => "Define Coordinate System",
                    }
                } else {
                    "Define Coordinate System"
                }
            }
        }
    }
    pub(super) fn define_coordinate_system(&mut self) {
        let (selected_origin, selected_distances, valid_distance_selection) =
            self.coordinate_selection();
        let result = match self.selection_order.as_slice() {
            [circle] if self.doc.circle_points(*circle).is_some() => {
                self.doc.add_coordinate_system_from_circle(*circle)
            }
            [origin] if self.doc.position(*origin).is_some() => {
                self.doc.add_coordinate_system_at_point(*origin)
            }
            _ if valid_distance_selection && matches!(selected_distances.len(), 1 | 2) => {
                self.doc.add_coordinate_system_from_distances(
                    selected_origin,
                    &selected_distances,
                    self.center,
                )
            }
            _ => self.doc.add_default_coordinate_system(self.center),
        };
        match result {
            Ok(frame) => {
                self.select_only(frame);
                self.status = format!("Defined coordinate system {frame}");
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn mark_coordinate_system(&mut self) {
        let [selected] = self.selection_order.as_slice() else {
            self.status = "Select one coordinate system axis, origin, or unit point".into();
            return;
        };
        match self.doc.mark_coordinate_system(*selected) {
            Ok(frame) => self.status = format!("Marked coordinate system {frame}"),
            Err(error) => self.status = error,
        }
    }
    pub(super) fn measure_slope(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [line] = self.selection_order.as_slice() else {
            self.status = "Select one straight object to measure".into();
            return;
        };
        let line = *line;
        if self.doc.line_points(line).is_none() {
            self.status = "Select one straight object to measure".into();
            return;
        }
        let frame = match self.doc.active_coordinate_system() {
            Some(frame) => frame,
            None => match self.doc.add_default_coordinate_system(self.center) {
                Ok(frame) => frame,
                Err(error) => {
                    self.status = error;
                    return;
                }
            },
        };
        let id = self.doc.add(Kind::SlopeMeasurement(line, frame));
        self.select_only(id);
        self.status = format!("Measured slope of object {line}");
    }

    pub(super) fn measure_value_equation(&mut self, equation: bool) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let (first, kind, second) = match self.selection_order.as_slice() {
            [first, second] if !equation => {
                let point_polygon = |point: usize, polygon: usize| {
                    self.doc.position(point).is_some()
                        && self
                            .doc
                            .objects
                            .get(polygon.saturating_sub(1))
                            .is_some_and(|o| o.kind.is_polygon())
                };
                if point_polygon(*first, *second) {
                    (*first, ValueEquationKind::PointValue, *second)
                } else if point_polygon(*second, *first) {
                    (*second, ValueEquationKind::PointValue, *first)
                } else {
                    self.status = "Select a point and a polygon for Value of Point".into();
                    return;
                }
            }
            [first] => {
                let second = match self
                    .doc
                    .objects
                    .get(first.saturating_sub(1))
                    .map(|o| &o.kind)
                {
                    Some(
                        Kind::PointOnSegment(host, _)
                        | Kind::PointOnRay(host, _)
                        | Kind::PointOnLine(host, _)
                        | Kind::PointOnArc(host, _)
                        | Kind::PointOnCurve(host, _)
                        | Kind::PointOnCircle(host, _, _)
                        | Kind::PointOnPolygon(host, _, _),
                    ) if !equation => (ValueEquationKind::PointValue, *host),
                    Some(Kind::PathLiteralPoint(host, _) | Kind::PathValuePoint(_, host))
                        if !equation
                            && self
                                .doc
                                .objects
                                .get(host.saturating_sub(1))
                                .is_some_and(|o| o.kind.is_polygon()) =>
                    {
                        (ValueEquationKind::PointValue, *host)
                    }
                    Some(kind) if equation && kind.is_line() => {
                        (ValueEquationKind::LineEquation, 0)
                    }
                    Some(kind) if equation && kind.is_circle() => {
                        (ValueEquationKind::CircleEquation, 0)
                    }
                    _ => {
                        self.status = if equation {
                            "Select one line or circle for Equation"
                        } else {
                            "Select a constrained point, or a point and a polygon"
                        }
                        .into();
                        return;
                    }
                };
                (*first, second.0, second.1)
            }
            _ => {
                self.status = "Select a constrained point, or a point and a polygon".into();
                return;
            }
        };
        let second = if equation {
            match self.doc.active_coordinate_system() {
                Some(frame) => frame,
                None => match self.doc.add_default_coordinate_system(self.center) {
                    Ok(frame) => frame,
                    Err(error) => {
                        self.status = error;
                        return;
                    }
                },
            }
        } else {
            second
        };
        let id = self.doc.add(Kind::ValueEquation {
            kind,
            first,
            second,
        });
        if self.doc.value_equation_value(id).is_none() {
            self.doc.undo();
            self.status = "The selected geometry has no supported value or equation".into();
            return;
        }
        self.select_only(id);
        self.status = format!("Measured {kind:?} of object {first}");
    }
    pub(super) fn measure_distance(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [a, b] = self.selection_order.as_slice() else {
            self.status = "Select two points, or a point and a straight object".into();
            return;
        };
        let (a, b) = (*a, *b);
        let kind = match (self.doc.position(a), self.doc.position(b)) {
            (Some(_), Some(_)) => Kind::PointDistance(a, b),
            (Some(_), None) if self.doc.line_points(b).is_some() => Kind::PointLineDistance(a, b),
            (None, Some(_)) if self.doc.line_points(a).is_some() => Kind::PointLineDistance(b, a),
            _ => {
                self.status = "Select two points, or a point and a straight object".into();
                return;
            }
        };
        if [a, b]
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select objects on the current page".into();
            return;
        }
        let id = self.doc.add(kind);
        self.select_only(id);
        self.status = format!("Measured distance between {a} and {b}");
    }

    pub(super) fn measure_coordinate(&mut self, kind: CoordinateMeasureKind) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let selected = self.selection_order.as_slice();
        let (first, second) = match (kind, selected) {
            (CoordinateMeasureKind::CoordinateDistance, [a, b]) => (*a, Some(*b)),
            (
                CoordinateMeasureKind::Coordinates
                | CoordinateMeasureKind::Abscissa
                | CoordinateMeasureKind::Ordinate,
                [a],
            ) => (*a, None),
            _ => {
                self.status = if kind == CoordinateMeasureKind::CoordinateDistance {
                    "Select two points to measure"
                } else {
                    "Select one point to measure"
                }
                .into();
                return;
            }
        };
        if self.doc.position(first).is_none()
            || second.is_some_and(|id| self.doc.position(id).is_none())
        {
            self.status = "Select point objects to measure".into();
            return;
        }
        let frame = match self.doc.active_coordinate_system() {
            Some(frame) => frame,
            None => match self.doc.add_default_coordinate_system(self.center) {
                Ok(frame) => frame,
                Err(error) => {
                    self.status = error;
                    return;
                }
            },
        };
        let id = self.doc.add(Kind::CoordinateMeasurement {
            kind,
            first,
            second,
            frame,
        });
        self.select_only(id);
        self.status = format!("Measured {kind:?} for point {first}");
    }
    pub(super) fn measure_ratio(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [a, b, c] = self.selection_order.as_slice() else {
            self.status = "Select three collinear points in origin, base, target order".into();
            return;
        };
        let (a, b, c) = (*a, *b, *c);
        let (Some(origin), Some(base), Some(target)) = (
            self.doc.position(a),
            self.doc.position(b),
            self.doc.position(c),
        ) else {
            self.status = "Select three points to measure".into();
            return;
        };
        let (bx, by) = (base.x - origin.x, base.y - origin.y);
        let (cx, cy) = (target.x - origin.x, target.y - origin.y);
        let cross = bx * cy - by * cx;
        let tolerance = 1e-8 * bx.hypot(by) * cx.hypot(cy);
        if self.doc.ratio_for_points(a, b, c).is_none() || cross.abs() > tolerance {
            self.status =
                "Ratio construction needs three collinear points and a nonzero base".into();
            return;
        }
        if [a, b, c]
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select points on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::PointRatio(a, b, c));
        self.select_only(id);
        self.status = format!("Measured ratio from points {a}, {b}, {c}");
    }
    pub(super) fn measure_perimeter(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [polygon] = self.selection_order.as_slice() else {
            self.status = "Select one polygon or sector interior to measure".into();
            return;
        };
        let polygon = *polygon;
        if self.doc.polygon_perimeter_cm(polygon).is_none() {
            self.status = "Select one polygon or sector interior to measure".into();
            return;
        }
        if self.doc.objects[polygon - 1].page != self.doc.active_page {
            self.status = "Select an interior on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::PolygonPerimeter(polygon));
        self.select_only(id);
        self.status = format!("Measured perimeter of interior {polygon}");
    }
    pub(super) fn measure_angle(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [a, b, c] = self.selection_order.as_slice() else {
            self.status = "Select three points in angle order".into();
            return;
        };
        let (a, b, c) = (*a, *b, *c);
        if self.doc.angle_for_points(a, b, c).is_none() {
            self.status = "The angle needs two nonzero rays".into();
            return;
        }
        if [a, b, c]
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select points on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::AngleMeasurement(a, b, c));
        self.select_only(id);
        self.status = format!("Measured angle {a}-{b}-{c}");
    }
    pub(super) fn measure_circle(&mut self, measurement: CircleMeasure) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [host] = self.selection_order.as_slice() else {
            self.status = "Select one circle, polygon, or sector interior to measure".into();
            return;
        };
        let host = *host;
        if self
            .doc
            .objects
            .get(host.saturating_sub(1))
            .is_none_or(|object| object.page != self.doc.active_page)
        {
            self.status = "Select an object on the current page".into();
            return;
        }
        let source = self
            .doc
            .objects
            .get(host.saturating_sub(1))
            .map(|o| &o.kind);
        let is_circle = self.doc.circle_points(host).is_some();
        let (kind, name) = match (measurement, source) {
            (CircleMeasure::Circumference, Some(_)) if is_circle => {
                (Kind::CircleCircumference(host), "circumference")
            }
            (CircleMeasure::Radius, Some(_)) if is_circle => (Kind::CircleRadius(host), "radius"),
            (CircleMeasure::Area, Some(_)) if is_circle => (Kind::CircleArea(host), "area"),
            (CircleMeasure::Area, Some(kind))
                if kind.is_polygon() || matches!(kind, Kind::SectorBoundary(_)) =>
            {
                (Kind::PolygonArea(host), "area")
            }
            _ => {
                self.status = "Select one circle, polygon, or sector interior to measure".into();
                return;
            }
        };
        let id = self.doc.add(kind);
        self.select_only(id);
        self.status = format!("Measured {name} of object {host}");
    }
    pub(super) fn measure_arc_angle(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [arc] = self.selection_order.as_slice() else {
            self.status = "Select one arc to measure".into();
            return;
        };
        let arc = *arc;
        if !self
            .doc
            .objects
            .get(arc.saturating_sub(1))
            .is_some_and(|o| o.kind.is_arc())
        {
            self.status = "Select one arc to measure".into();
            return;
        }
        if self.doc.objects[arc - 1].page != self.doc.active_page {
            self.status = "Select an arc on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::ArcAngle(arc));
        self.select_only(id);
        self.status = format!("Measured angle of arc {arc}");
    }
    pub(super) fn measure_arc_length(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [arc] = self.selection_order.as_slice() else {
            self.status = "Select one arc to measure".into();
            return;
        };
        let arc = *arc;
        if !self
            .doc
            .objects
            .get(arc.saturating_sub(1))
            .is_some_and(|o| o.kind.is_arc() || matches!(o.kind, Kind::SectorBoundary(_)))
        {
            self.status = "Select one arc to measure".into();
            return;
        }
        if self.doc.objects[arc - 1].page != self.doc.active_page {
            self.status = "Select an arc on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::ArcLength(arc));
        self.select_only(id);
        self.status = format!("Measured length of arc {arc}");
    }
}
