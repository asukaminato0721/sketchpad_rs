use super::kind::{
    is_arc_kind, is_point_kind, is_straight_kind, is_transformed_plot_path_kind,
    is_transformed_plot_ray_kind, plot_path_accepts,
};
use super::transform_evaluator::TransformGeometry;
use super::transform_model::{ElementType, NativeTransformFlavor, NumericExpr, Transform};
use super::{Document, Kind, LineMode, Point};
use crate::graph::GridForm;
use crate::plot_points_records::PlotPointsMode;
use std::collections::BTreeSet;

pub(super) fn reflect_point(point: Point, a: Point, b: Point) -> Option<Point> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length_sq = dx * dx + dy * dy;
    if !length_sq.is_finite() || length_sq <= 1e-18 {
        return None;
    }
    let t = ((point.x - a.x) * dx + (point.y - a.y) * dy) / length_sq;
    let projected = Point {
        x: a.x + t * dx,
        y: a.y + t * dy,
    };
    let reflected = Point {
        x: 2. * projected.x - point.x,
        y: 2. * projected.y - point.y,
    };
    (reflected.x.is_finite() && reflected.y.is_finite()).then_some(reflected)
}

pub(super) fn scale_point_about(source: Point, center: Point, factor: f64) -> Option<Point> {
    let result = Point {
        x: center.x + factor * (source.x - center.x),
        y: center.y + factor * (source.y - center.y),
    };
    (result.x.is_finite() && result.y.is_finite()).then_some(result)
}

pub(super) fn ratio_of_points(a: Point, b: Point, c: Point) -> Option<f64> {
    let (bx, by) = (b.x - a.x, b.y - a.y);
    let denominator = bx.mul_add(bx, by * by);
    if !denominator.is_finite() || denominator <= 0.0 {
        return None;
    }
    let value = ((c.x - a.x) * bx + (c.y - a.y) * by) / denominator;
    value.is_finite().then_some(value)
}

pub(super) fn marked_rotation_theta(first: Point, vertex: Point, last: Point) -> Option<f64> {
    let (ax, ay) = (first.x - vertex.x, first.y - vertex.y);
    let (cx, cy) = (last.x - vertex.x, last.y - vertex.y);
    if ax.hypot(ay) == 0. || cx.hypot(cy) == 0. {
        return None;
    }
    Some((-cy).atan2(cx) - (-ay).atan2(ax))
}

pub(super) fn rotate_point_about(source: Point, center: Point, theta: f64) -> Option<Point> {
    let dx = source.x - center.x;
    let dy = source.y - center.y;
    let rotated = Point {
        x: center.x + dx * theta.cos() + dy * theta.sin(),
        y: center.y - dx * theta.sin() + dy * theta.cos(),
    };
    (rotated.x.is_finite() && rotated.y.is_finite()).then_some(rotated)
}

pub(super) fn intersect_straight(
    first: (Point, Point, LineMode),
    second: (Point, Point, LineMode),
) -> Option<Point> {
    let (a, b, first_mode) = first;
    let (c, d, second_mode) = second;
    let r = Point {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let s = Point {
        x: d.x - c.x,
        y: d.y - c.y,
    };
    let (r_len, s_len) = (r.x.hypot(r.y), s.x.hypot(s.y));
    if r_len < 1e-12 || s_len < 1e-12 {
        return None;
    }
    let cross = |p: Point, q: Point| p.x * q.y - p.y * q.x;
    let denominator = cross(r, s);
    if denominator.abs() <= r_len * s_len * 1e-12 {
        return None;
    }
    let delta = Point {
        x: c.x - a.x,
        y: c.y - a.y,
    };
    let t = cross(delta, s) / denominator;
    let u = cross(delta, r) / denominator;
    let contains = |mode: LineMode, parameter: f64| match mode {
        LineMode::Segment => parameter >= -1e-10 && parameter <= 1. + 1e-10,
        LineMode::Ray => parameter >= -1e-10,
        LineMode::Line => true,
    };
    if !contains(first_mode, t) || !contains(second_mode, u) {
        return None;
    }
    let result = Point {
        x: a.x + r.x * t,
        y: a.y + r.y * t,
    };
    (result.x.is_finite() && result.y.is_finite()).then_some(result)
}
fn line_circle_intersection(
    line: (Point, Point, LineMode),
    circle: (Point, f64),
    variant: usize,
) -> Option<Point> {
    let (a, b, mode) = line;
    let (center, radius) = circle;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length_squared = dx * dx + dy * dy;
    if length_squared <= 1e-18 || radius <= 1e-9 {
        return None;
    }
    let (fx, fy) = (a.x - center.x, a.y - center.y);
    let linear = 2. * (fx * dx + fy * dy);
    let constant = fx * fx + fy * fy - radius * radius;
    let discriminant = linear * linear - 4. * length_squared * constant;
    if discriminant < -1e-9 {
        return None;
    }
    let root = discriminant.max(0.).sqrt();
    // Native code 12 (variant 0) is the larger line parameter; code 11
    // (variant 1) is the smaller. Keep the chosen branch undefined when it
    // falls outside a segment/ray instead of substituting the other root.
    let t = if variant == 0 {
        (-linear + root) / (2. * length_squared)
    } else {
        (-linear - root) / (2. * length_squared)
    };
    let contains = match mode {
        LineMode::Segment => (-1e-10..=1. + 1e-10).contains(&t),
        LineMode::Ray => t >= -1e-10,
        LineMode::Line => true,
    };
    if !contains {
        return None;
    }
    let result = Point {
        x: a.x + dx * t,
        y: a.y + dy * t,
    };
    (result.x.is_finite() && result.y.is_finite()).then_some(result)
}
fn circle_circle_intersection(
    first: (Point, f64),
    second: (Point, f64),
    variant: usize,
) -> Option<Point> {
    let (a, first_radius) = first;
    let (b, second_radius) = second;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let distance = dx.hypot(dy);
    if first_radius <= 1e-9
        || second_radius <= 1e-9
        || distance <= 1e-9
        || distance > first_radius + second_radius + 1e-9
        || distance < (first_radius - second_radius).abs() - 1e-9
    {
        return None;
    }
    let along = (first_radius * first_radius - second_radius * second_radius + distance * distance)
        / (2. * distance);
    let height_squared = first_radius * first_radius - along * along;
    if height_squared < -1e-9 {
        return None;
    }
    let height = height_squared.max(0.).sqrt();
    let (ux, uy) = (dx / distance, dy / distance);
    let base = Point {
        x: a.x + along * ux,
        y: a.y + along * uy,
    };
    let results = [
        Point {
            x: base.x - height * uy,
            y: base.y + height * ux,
        },
        Point {
            x: base.x + height * uy,
            y: base.y - height * ux,
        },
    ];
    results
        .get(variant)
        .copied()
        .filter(|p| p.x.is_finite() && p.y.is_finite())
}
fn three_point_arc_geometry(
    start: Point,
    mid: Point,
    end: Point,
) -> Option<(Point, f64, f64, f64)> {
    let determinant =
        2. * (start.x * (mid.y - end.y) + mid.x * (end.y - start.y) + end.x * (start.y - mid.y));
    if determinant.abs() <= 1e-9 {
        return None;
    }
    let square = |point: Point| point.x * point.x + point.y * point.y;
    let center = Point {
        x: (square(start) * (mid.y - end.y)
            + square(mid) * (end.y - start.y)
            + square(end) * (start.y - mid.y))
            / determinant,
        y: (square(start) * (end.x - mid.x)
            + square(mid) * (start.x - end.x)
            + square(end) * (mid.x - start.x))
            / determinant,
    };
    let radius = center.distance(start);
    if !radius.is_finite() || radius <= 1e-9 {
        return None;
    }
    let angle = |p: Point| (p.y - center.y).atan2(p.x - center.x);
    let (first, middle, last) = (angle(start), angle(mid), angle(end));
    let ccw = (last - first).rem_euclid(std::f64::consts::TAU);
    let mid_ccw = (middle - first).rem_euclid(std::f64::consts::TAU);
    let sweep = if mid_ccw <= ccw + 1e-9 {
        ccw
    } else {
        -(first - last).rem_euclid(std::f64::consts::TAU)
    };
    Some((center, radius, first, sweep))
}

impl Document {
    /// Native code 95/123 retain negative ray values as undefined points.
    pub(super) fn undefined_transformed_ray_plot(&self, path: usize, value: f64) -> bool {
        value.is_finite()
            && value < 0.0
            && self
                .objects
                .get(path.saturating_sub(1))
                .is_some_and(|host| !host.deleted && is_transformed_plot_ray_kind(&host.kind))
            && self
                .line_points(path)
                .is_some_and(|(start, end, _)| start != end)
    }

    pub(super) fn plot_arc_host_available(&self, path: usize) -> bool {
        if let Some(original) = self.frozen_evaluation_document(path) {
            return original.plot_arc_host_available(path);
        }
        let Some(host) = path
            .checked_sub(1)
            .and_then(|index| self.objects.get(index))
        else {
            return false;
        };
        if host.deleted {
            return false;
        }
        fn dependencies_available(
            document: &Document,
            id: usize,
            page: usize,
            visiting: &mut BTreeSet<usize>,
        ) -> bool {
            let Some(object) = id
                .checked_sub(1)
                .and_then(|index| document.objects.get(index))
            else {
                return false;
            };
            if object.deleted
                || object.page != page
                || matches!(object.kind, Kind::Unsupported(_))
                || !visiting.insert(id)
            {
                return false;
            }
            let available = object
                .kind
                .dependencies()
                .into_iter()
                .all(|parent| dependencies_available(document, parent, page, visiting));
            visiting.remove(&id);
            available
        }
        if !host
            .kind
            .dependencies()
            .into_iter()
            .all(|id| dependencies_available(self, id, host.page, &mut BTreeSet::new()))
        {
            return false;
        }
        let point_available = |id: usize| {
            id.checked_sub(1)
                .and_then(|index| self.objects.get(index))
                .is_some_and(|point| {
                    id < path
                        && !point.deleted
                        && point.page == host.page
                        && is_point_kind(&point.kind)
                        && self.position(id).is_some_and(|position| {
                            position.x.is_finite() && position.y.is_finite()
                        })
                })
        };
        match host.kind {
            Kind::Colorized(ref definition) if definition.element() == super::transform_model::ElementType::Arc =>
                self.plot_arc_host_available(definition.source()),
            Kind::Transformed(ref definition) if definition.element() == super::transform_model::ElementType::Arc =>
                self.plot_arc_host_available(definition.source()),
            Kind::ThreePointArc(a, b, c) | Kind::CenterArc(a, b, c) => {
                [a, b, c].into_iter().all(point_available)
            }
            Kind::ArcOnCircle(circle, start, end) => {
                self.objects
                    .get(circle.saturating_sub(1))
                    .is_some_and(|object| {
                        circle < path
                            && !object.deleted
                            && object.page == host.page
                            && object.kind.is_circle()
                    })
                    && self.circle_points(circle).is_some()
                    && [start, end].into_iter().all(|id| {
                        point_available(id)
                            && matches!(self.objects[id - 1].kind,
                            Kind::PointOnCircle(source, _, _) if source == circle)
                    })
            }
            _ => false,
        }
    }

    /// Native three-point arcs with three distinct collinear positions form a
    /// segment only when the middle vertex is strictly between the endpoints.
    pub fn linear_three_point_arc(&self, path: usize) -> Option<(Point, Point)> {
        if let Some(Kind::Colorized(definition)) = self.objects.get(path.checked_sub(1)?).map(|object| &object.kind) {
            return self.linear_three_point_arc(definition.source());
        }
        if let Some(original) = self.frozen_evaluation_document(path) {
            return original.linear_three_point_arc(path);
        }
        if self.objects.get(path.checked_sub(1)?)?.kind.transform_definition()
            .is_some_and(|definition| definition.element() == super::transform_model::ElementType::Arc) {
            return match self.evaluate_transform(path).ok()? {
                TransformGeometry::Straight { start, end, mode: LineMode::Segment } => Some((start, end)),
                _ => None,
            };
        }
        let Kind::ThreePointArc(a, b, c) = self.objects.get(path.checked_sub(1)?)?.kind else {
            return None;
        };
        if !self.plot_arc_host_available(path) {
            return None;
        }
        let (a, b, c) = (self.position(a)?, self.position(b)?, self.position(c)?);
        let (dx, dy) = (c.x - a.x, c.y - a.y);
        let determinant = (b.x - a.x) * dy - (b.y - a.y) * dx;
        let length_squared = dx * dx + dy * dy;
        let projection = (b.x - a.x) * dx + (b.y - a.y) * dy;
        (determinant == 0.0
            && length_squared.is_finite()
            && length_squared > 0.0
            && projection > 0.0
            && projection < length_squared)
            .then_some((a, c))
    }

    pub(super) fn undefined_arc_plot(&self, path: usize, value: f64) -> bool {
        if let Some(original) = self.frozen_evaluation_document(path) {
            return original.undefined_arc_plot(path, value);
        }
        if !value.is_finite() || !self.plot_arc_host_available(path) {
            return false;
        }
        if !(0.0..=1.0).contains(&value) {
            return true;
        }
        match self.objects[path - 1].kind {
            Kind::ThreePointArc(a, b, c) => {
                let (a, b, c) = (
                    self.position(a).unwrap(),
                    self.position(b).unwrap(),
                    self.position(c).unwrap(),
                );
                if a == b || b == c || a == c {
                    return true;
                }
                let (dx, dy) = (c.x - a.x, c.y - a.y);
                let determinant = (b.x - a.x) * dy - (b.y - a.y) * dx;
                let length_squared = dx * dx + dy * dy;
                let projection = (b.x - a.x) * dx + (b.y - a.y) * dy;
                determinant == 0.0
                    && length_squared.is_finite()
                    && length_squared > 0.0
                    && projection.is_finite()
                    && (projection <= 0.0 || projection >= length_squared)
            }
            Kind::ArcOnCircle(_, start, end) => self.position(start) == self.position(end),
            Kind::CenterArc(..) => self.arc_geometry(path).is_none(),
            Kind::Transformed(_) if self.objects[path - 1].kind.is_arc() =>
                self.arc_geometry(path).is_none() && self.linear_three_point_arc(path).is_none(),
            _ => false,
        }
    }

    pub(super) fn plotted_arc_position(&self, path: usize, value: f64) -> Option<Point> {
        if !value.is_finite()
            || !(0.0..=1.0).contains(&value)
            || !self.plot_arc_host_available(path)
            || self.undefined_arc_plot(path, value)
        {
            return None;
        }
        if let Some((start, end)) = self.linear_three_point_arc(path) {
            return Some(Point {
                x: start.x + (end.x - start.x) * value,
                y: start.y + (end.y - start.y) * value,
            });
        }
        let geometry = self.arc_geometry(path)?;
        if geometry.3 == 0.0 {
            return None;
        }
        let point = Self::sample_arc(geometry, value);
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }

    pub(super) fn undefined_native_path_plot(&self, path: usize, value: f64) -> bool {
        self.undefined_transformed_ray_plot(path, value) || self.undefined_arc_plot(path, value)
    }

    pub(super) fn undefined_three_point_arc(&self, path: usize) -> bool {
        if let Some(Kind::Colorized(definition)) = path.checked_sub(1).and_then(|index| self.objects.get(index)).map(|object| &object.kind) {
            return self.undefined_three_point_arc(definition.source());
        }
        self.objects
            .get(path.saturating_sub(1))
            .is_some_and(|host| matches!(host.kind, Kind::ThreePointArc(..)))
            && self.undefined_arc_plot(path, 0.5)
    }

    pub(super) fn undefined_center_arc(&self, path: usize) -> bool {
        if let Some(Kind::Colorized(definition)) = path.checked_sub(1).and_then(|index| self.objects.get(index)).map(|object| &object.kind) {
            return self.undefined_center_arc(definition.source());
        }
        self.objects
            .get(path.saturating_sub(1))
            .is_some_and(|host| matches!(host.kind, Kind::CenterArc(..)))
            && self.undefined_arc_plot(path, 0.5)
    }

    pub(super) fn undefined_transformed_arc(&self, path: usize) -> bool {
        if let Some(Kind::Colorized(definition)) = path.checked_sub(1).and_then(|index| self.objects.get(index)).map(|object| &object.kind) {
            return self.undefined_transformed_arc(definition.source());
        }
        self.objects.get(path.saturating_sub(1)).is_some_and(|host|
            host.kind.is_arc() && host.kind.transform_definition().is_some())
            && self.undefined_arc_plot(path, 0.5)
    }

    pub fn position(&self, id: usize) -> Option<Point> {
        fn go(d: &Document, id: usize, seen: &mut BTreeSet<usize>) -> Option<Point> {
            if let Some(value) = d.evaluation_point(id) {
                return value;
            }
            fn on_path(
                d: &Document,
                path: usize,
                t: f64,
                seen: &mut BTreeSet<usize>,
            ) -> Option<Point> {
                let kind = &d.objects.get(path.checked_sub(1)?)?.kind;
                if !plot_path_accepts(kind, t) {
                    return None;
                }
                if is_arc_kind(kind) {
                    return d.plotted_arc_position(path, t);
                }
                if is_transformed_plot_path_kind(kind) {
                    let (start, end, _) = d.line_points(path)?;
                    let point = Point {
                        x: start.x + (end.x - start.x) * t,
                        y: start.y + (end.y - start.y) * t,
                    };
                    return (point.x.is_finite() && point.y.is_finite()).then_some(point);
                }
                if kind.is_circle() {
                    let (center, radius) = d.circle_points(path)?;
                    let angle = t * std::f64::consts::TAU;
                    let point = Point {
                        x: center.x + radius * angle.cos(),
                        y: center.y - radius * angle.sin(),
                    };
                    return (point.x.is_finite() && point.y.is_finite()).then_some(point);
                }
                if matches!(
                    kind,
                    Kind::CircleAxis(_, 0 | 1) | Kind::DistanceAxis(_, _, 0 | 1)
                ) {
                    let (start, end, _) = d.line_points(path)?;
                    let point = Point {
                        x: start.x + (end.x - start.x) * t,
                        y: start.y + (end.y - start.y) * t,
                    };
                    return (point.x.is_finite() && point.y.is_finite()).then_some(point);
                }
                let (start, end) = match kind {
                    Kind::GraphAxis(a, b, 0 | 1 | 0x1_0001)
                    | Kind::Segment(a, b)
                    | Kind::Line(a, b)
                    | Kind::Ray(a, b) => (*a, *b),
                    _ => return None,
                };
                let (start, end) = if d.evaluation_context.is_some() {
                    let (start, end, _) = d.line_points(path)?;
                    (start, end)
                } else {
                    (go(d, start, seen)?, go(d, end, seen)?)
                };
                let point = Point {
                    x: start.x + (end.x - start.x) * t,
                    y: start.y + (end.y - start.y) * t,
                };
                (point.x.is_finite() && point.y.is_finite()).then_some(point)
            }
            if !seen.insert(id) {
                return None;
            }
            let object = d.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            if object.kind.transform_definition().is_some() {
                let point = match d.evaluate_transform(id).ok()? {
                    TransformGeometry::Point(point) => Some(point),
                    _ => None,
                };
                seen.remove(&id);
                return point;
            }
            let p = match &object.kind {
                Kind::Colorized(definition) if definition.element() == super::transform_model::ElementType::Point =>
                    go(d, definition.source(), seen),
                Kind::Free(p) => Some(*p),
                Kind::GraphHorizontalUnit(origin, step) => {
                    let origin = go(d, *origin, seen)?;
                    Some(Point {
                        x: origin.x + step,
                        y: origin.y,
                    })
                }
                Kind::GraphVerticalUnit(horizontal_unit, step) => {
                    let Kind::GraphHorizontalUnit(origin, horizontal_step) =
                        d.objects.get(horizontal_unit.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    let origin = if d.evaluation_context.is_some() {
                        let unit = go(d, *horizontal_unit, seen)?;
                        Point {
                            x: unit.x - horizontal_step,
                            y: unit.y,
                        }
                    } else {
                        go(d, origin, seen)?
                    };
                    let form = d
                        .coordinate_system_for_object(id)
                        .and_then(|frame| d.frame_styles.get(&frame).map(|style| style.0))
                        .unwrap_or(d.grid_form[object.page]);
                    let step = if form == crate::graph::GridForm::Rectangular {
                        *step
                    } else {
                        horizontal_step
                    };
                    Some(Point {
                        x: origin.x,
                        y: origin.y - step,
                    })
                }
                Kind::DistanceVerticalUnit(x_axis, step, independent) => {
                    let Kind::DistanceAxis(origin, distance, 1) =
                        d.objects.get(x_axis.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    let (origin, axis_length) = if d.evaluation_context.is_some() {
                        let (origin, unit, _) = d.line_points(*x_axis)?;
                        (origin, Some(origin.distance(unit)))
                    } else {
                        (go(d, origin, seen)?, None)
                    };
                    let length = if *independent {
                        *step
                    } else if let Some(length) = axis_length {
                        length
                    } else {
                        d.defining_distance_pixels(distance)?
                    };
                    Some(Point {
                        x: origin.x,
                        y: origin.y - length,
                    })
                }
                Kind::PathLiteralPoint(path, t) => {
                    if d.objects.get(path.checked_sub(1)?)?.kind.is_polygon() {
                        d.point_at_polygon_parameter(*path, *t)
                    } else {
                        on_path(d, *path, *t, seen)
                    }
                }
                Kind::PathValuePoint(source, path) => {
                    let value = d.numeric_value(*source)?;
                    if d.objects.get(path.checked_sub(1)?)?.kind.is_polygon() {
                        d.point_at_polygon_parameter(*path, value)
                    } else {
                        on_path(d, *path, value, seen)
                    }
                }
                Kind::CoordinatePoint(definition) => d.evaluate_coordinate_point(definition),
                Kind::NumericCoordinatePoint(x, y, frame) => {
                    let frame = d.graph_frame(*frame)?;
                    let x = d.numeric_value(*x)?;
                    let y = d.numeric_value(*y)?;
                    let point = frame.world(x, y)?;
                    Some(Point {
                        x: point.x,
                        y: point.y,
                    })
                }
                Kind::PlottedPoint(frame, mode, stored) => {
                    let frame = d.graph_frame(*frame)?;
                    let [x, y] = match mode {
                        PlotPointsMode::Cartesian => *stored,
                        PlotPointsMode::Polar => {
                            [stored[0] * stored[1].cos(), stored[0] * stored[1].sin()]
                        }
                    };
                    let point = frame.world(x, y)?;
                    Some(Point {
                        x: point.x,
                        y: point.y,
                    })
                }
                Kind::MappedElement(_) => d.mapped_element_point(id),
                Kind::PointOnSegment(host, t)
                | Kind::PointOnRay(host, t)
                | Kind::PointOnLine(host, t) => {
                    let (a, b, _) = d.point_constraint_line_points(*host)?;
                    Some(Point {
                        x: a.x + (b.x - a.x) * t,
                        y: a.y + (b.y - a.y) * t,
                    })
                }
                Kind::PointOnCircle(host, unit_x, unit_y) => {
                    let (center, radius) = d.circle_points(*host)?;
                    Some(Point {
                        x: center.x + radius * unit_x,
                        y: center.y - radius * unit_y,
                    })
                }
                Kind::PointOnArc(host, t) => d.point_on_arc(*host, *t),
                Kind::PointOnPolygon(host, edge, t) => d.point_on_polygon(*host, *edge, *t),
                Kind::Midpoint(a, b) => {
                    let (a, b) = if let Some(host) = object.midpoint_host {
                        let (a, b, mode) = d.line_points(host)?;
                        if mode != LineMode::Segment {
                            return None;
                        }
                        (a, b)
                    } else {
                        (go(d, *a, seen)?, go(d, *b, seen)?)
                    };
                    Some(Point {
                        x: (a.x + b.x) / 2.,
                        y: (a.y + b.y) / 2.,
                    })
                }
                Kind::SegmentMidpoint(host) => {
                    let (a, b, mode) = d.line_points(*host)?;
                    (mode == LineMode::Segment).then_some(Point {
                        x: (a.x + b.x) / 2.,
                        y: (a.y + b.y) / 2.,
                    })
                }
                Kind::Intersection(a, b) => {
                    let first = d.line_points(*a)?;
                    let second = d.line_points(*b)?;
                    intersect_straight(first, second)
                }
                Kind::LineCircleIntersection(a, b, variant) => {
                    line_circle_intersection(d.line_points(*a)?, d.circle_points(*b)?, *variant)
                }
                Kind::CircleCircleIntersection(a, b, variant) => {
                    circle_circle_intersection(d.circle_points(*a)?, d.circle_points(*b)?, *variant)
                }
                _ => None,
            };
            seen.remove(&id);
            p
        }
        go(self, id, &mut BTreeSet::new())
    }
    pub fn polygon_points(&self, id: usize) -> Option<Vec<Point>> {
        if let Some(value) = self.evaluation_polygon(id) {
            return value;
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        if object.kind.transform_definition().is_some() {
            return match self.evaluate_transform(id).ok()? {
                TransformGeometry::Polygon(vertices) => Some(vertices),
                _ => None,
            };
        }
        match &object.kind {
            Kind::Colorized(definition) if definition.element() == super::transform_model::ElementType::Polygon =>
                self.polygon_points(definition.source()),
            Kind::Polygon(vertices) => vertices
                .iter()
                .map(|&vertex| self.position(vertex))
                .collect(),
            _ => None,
        }
    }
    pub(super) fn straight_transform_has_suppressed_ancestor(&self, mut id: usize) -> bool {
        let mut seen = BTreeSet::new();
        while seen.insert(id) {
            let Some(definition) = self
                .objects
                .get(id.saturating_sub(1))
                .and_then(|object| object.kind.transform_definition())
            else {
                return false;
            };
            if !matches!(definition.element(), ElementType::Ray | ElementType::Line) {
                return false;
            }
            match (definition.wire_flavor(), definition.transform()) {
                (
                    NativeTransformFlavor::FixedDilation,
                    Transform::Dilate {
                        ratio: NumericExpr::Literal { magnitude, .. },
                        ..
                    },
                ) if *magnitude == 0. => return true,
                (
                    NativeTransformFlavor::NumericDilation,
                    Transform::Dilate {
                        ratio: NumericExpr::ObjectValue { id: ratio, .. },
                        ..
                    },
                ) => {
                    if self
                        .unitless_numeric_value(*ratio)
                        .is_none_or(|value| value == 0.)
                    {
                        return true;
                    }
                }
                (
                    NativeTransformFlavor::NumericRotation,
                    Transform::Rotate {
                        angle: NumericExpr::ObjectValue { id: angle, .. },
                        ..
                    },
                ) => {
                    if self.numeric_value(*angle).is_none() {
                        return true;
                    }
                }
                (
                    NativeTransformFlavor::FixedDilation | NativeTransformFlavor::FixedRotation,
                    _,
                ) => {}
                _ => return false,
            }
            id = definition.source();
        }
        false
    }

    pub fn line_points(&self, id: usize) -> Option<(Point, Point, LineMode)> {
        fn go(
            d: &Document,
            id: usize,
            seen: &mut BTreeSet<usize>,
        ) -> Option<(Point, Point, LineMode)> {
            if let Some(value) = d.evaluation_straight(id) {
                return value;
            }
            if !seen.insert(id) {
                return None;
            }
            let object = d.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            if object.kind.transform_definition().is_some() {
                if object.kind.is_arc() { return None; }
                let straight = match d.evaluate_transform(id).ok()? {
                    TransformGeometry::Straight { start, end, mode } => Some((start, end, mode)),
                    _ => None,
                };
                seen.remove(&id);
                return straight;
            }
            let result = match object.kind {
                Kind::Colorized(ref definition) if matches!(definition.element(), super::transform_model::ElementType::Segment | super::transform_model::ElementType::Ray | super::transform_model::ElementType::Line) =>
                    go(d, definition.source(), seen),
                Kind::Segment(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Segment)),
                Kind::GraphAxis(a, b, _) => Some((d.position(a)?, d.position(b)?, LineMode::Line)),
                Kind::CircleAxis(circle, axis @ (0 | 1)) => {
                    let (center, radius) = d.circle_points(circle)?;
                    let unit = if axis == 1 {
                        Point {
                            x: center.x + radius,
                            y: center.y,
                        }
                    } else {
                        Point {
                            x: center.x,
                            y: center.y - radius,
                        }
                    };
                    Some((center, unit, LineMode::Line))
                }
                Kind::DistanceAxis(origin, distance, axis @ (0 | 1)) => {
                    let center = d.position(origin)?;
                    let length = d.defining_distance_pixels(distance)?;
                    let unit = if axis == 1 {
                        Point {
                            x: center.x + length,
                            y: center.y,
                        }
                    } else {
                        Point {
                            x: center.x,
                            y: center.y - length,
                        }
                    };
                    Some((center, unit, LineMode::Line))
                }
                Kind::Line(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Line)),
                Kind::Ray(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Ray)),
                Kind::Parallel(p, line) | Kind::Perpendicular(p, line) => {
                    let origin = d.position(p)?;
                    let (a, b, _) = go(d, line, seen)?;
                    let mut direction = Point {
                        x: b.x - a.x,
                        y: b.y - a.y,
                    };
                    if matches!(d.objects[id - 1].kind, Kind::Perpendicular(_, _)) {
                        direction = Point {
                            x: -direction.y,
                            y: direction.x,
                        };
                    }
                    // Native derived-line parameters are centered on the
                    // construction point; .5 is the point itself.
                    Some((
                        Point {
                            x: origin.x - direction.x * 0.5,
                            y: origin.y - direction.y * 0.5,
                        },
                        Point {
                            x: origin.x + direction.x * 0.5,
                            y: origin.y + direction.y * 0.5,
                        },
                        LineMode::Line,
                    ))
                }
                Kind::Bisector(start, vertex, end) => {
                    let (a, b, c) = (d.position(start)?, d.position(vertex)?, d.position(end)?);
                    let normalize = |p: Point| {
                        let len = p.x.hypot(p.y);
                        (len > 1e-9).then_some(Point {
                            x: p.x / len,
                            y: p.y / len,
                        })
                    };
                    let first = normalize(Point {
                        x: a.x - b.x,
                        y: a.y - b.y,
                    })?;
                    let second = normalize(Point {
                        x: c.x - b.x,
                        y: c.y - b.y,
                    })?;
                    let sum = Point {
                        x: first.x + second.x,
                        y: first.y + second.y,
                    };
                    let direction = normalize(sum).unwrap_or(Point {
                        x: -first.y,
                        y: first.x,
                    });
                    Some((
                        b,
                        Point {
                            x: b.x + direction.x,
                            y: b.y + direction.y,
                        },
                        LineMode::Ray,
                    ))
                }
                _ => None,
            };
            seen.remove(&id);
            result
        }
        go(self, id, &mut BTreeSet::new())
    }
    pub fn circle_points(&self, id: usize) -> Option<(Point, f64)> {
        if let Some(value) = self.evaluation_circle(id) {
            return value;
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        if let Kind::Colorized(definition) = &object.kind {
            return (definition.element() == super::transform_model::ElementType::Circle)
                .then(|| self.circle_points(definition.source())).flatten();
        }
        if object.kind.transform_definition().is_some() {
            return match self.evaluate_transform(id).ok()? {
                TransformGeometry::Circle { center, radius } => Some((center, radius)),
                _ => None,
            };
        }
        let (center_id, radius) = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Circle(center, radius_point) => {
                let center_position = self.position(center)?;
                (
                    center,
                    center_position.distance(self.position(radius_point)?),
                )
            }
            Kind::CircleBySegment(center, radius_source) => {
                let source = &self.objects.get(radius_source.checked_sub(1)?)?.kind;
                let radius = if source.is_segment() {
                    let (a, b, LineMode::Segment) = self.line_points(radius_source)? else { return None; };
                    a.distance(b)
                } else {
                    self.distance_value_pixels(radius_source)?
                };
                (center, radius)
            }
            _ => return None,
        };
        let center = self.position(center_id)?;
        (radius.is_finite() && radius > 0.).then_some((center, radius))
    }
    pub(super) fn arc_geometry(&self, id: usize) -> Option<(Point, f64, f64, f64, bool)> {
        if let Some(value) = self.evaluation_arc(id) {
            return value;
        }
        if self
            .objects
            .get(id.checked_sub(1)?)
            .is_some_and(|host| matches!(host.kind, Kind::ThreePointArc(..)))
            && !self.plot_arc_host_available(id)
        {
            return None;
        }
        let geometry = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Colorized(ref definition) if definition.element() == super::transform_model::ElementType::Arc =>
                return self.arc_geometry(definition.source()),
            Kind::Transformed(_) if is_arc_kind(&self.objects[id - 1].kind) => {
                match self.evaluate_transform(id).ok()? {
                    TransformGeometry::Arc { center, radius, start, sweep, clockwise_y } =>
                        (center, radius, start, sweep, clockwise_y),
                    _ => return None,
                }
            }
            Kind::ThreePointArc(a, b, c) => {
                let (center, radius, start, sweep) = three_point_arc_geometry(
                    self.position(a)?,
                    self.position(b)?,
                    self.position(c)?,
                )?;
                (center, radius, start, sweep, false)
            }
            Kind::CenterArc(center, start, end) => {
                let center = self.position(center)?;
                let start_point = self.position(start)?;
                let end_point = self.position(end)?;
                let radius = center.distance(start_point);
                let end_radius = center.distance(end_point);
                if !radius.is_finite()
                    || radius <= 0.
                    || !end_radius.is_finite()
                    || end_radius <= 0.
                {
                    return None;
                }
                let a = (center.y - start_point.y).atan2(start_point.x - center.x);
                let b = (center.y - end_point.y).atan2(end_point.x - center.x);
                let sweep = (b - a).rem_euclid(std::f64::consts::TAU);
                if !sweep.is_finite() || sweep == 0. {
                    return None;
                }
                (center, radius, a, sweep, true)
            }
            Kind::ArcOnCircle(circle, start, end) => {
                let (center, radius) = self.circle_points(circle)?;
                let start_point = self.position(start)?;
                let end_point = self.position(end)?;
                let a = (center.y - start_point.y).atan2(start_point.x - center.x);
                let b = (center.y - end_point.y).atan2(end_point.x - center.x);
                let sweep = (b - a).rem_euclid(std::f64::consts::TAU);
                (center, radius, a, sweep, true)
            }
            _ => return None,
        };
        Some(geometry)
    }
    pub fn point_on_arc(&self, id: usize, t: f64) -> Option<Point> {
        if !t.is_finite() {
            return None;
        }
        if let Some((start, end)) = self.linear_three_point_arc(id) {
            let t = t.clamp(0., 1.);
            return Some(Point {
                x: start.x + (end.x - start.x) * t,
                y: start.y + (end.y - start.y) * t,
            });
        }
        Some(Self::sample_arc(self.arc_geometry(id)?, t))
    }
    fn sample_arc(
        (center, radius, start, sweep, clockwise_y): (Point, f64, f64, f64, bool),
        t: f64,
    ) -> Point {
        let angle = start + sweep * t.clamp(0., 1.);
        Point {
            x: center.x + radius * angle.cos(),
            y: center.y
                + if clockwise_y {
                    -radius * angle.sin()
                } else {
                    radius * angle.sin()
                },
        }
    }
    pub(super) fn arc_parameter(&self, id: usize, p: Point) -> Option<f64> {
        if let Some((start, end)) = self.linear_three_point_arc(id) {
            let (dx, dy) = (end.x - start.x, end.y - start.y);
            let parameter = ((p.x - start.x) * dx + (p.y - start.y) * dy) / (dx * dx + dy * dy);
            return parameter.is_finite().then_some(parameter.clamp(0., 1.));
        }
        let (center, _, start, sweep, clockwise_y) = self.arc_geometry(id)?;
        if sweep.abs() < 1e-9 {
            return None;
        }
        let angle = (if clockwise_y {
            center.y - p.y
        } else {
            p.y - center.y
        })
        .atan2(p.x - center.x);
        let progress = if sweep >= 0. {
            (angle - start).rem_euclid(std::f64::consts::TAU)
        } else {
            (start - angle).rem_euclid(std::f64::consts::TAU)
        };
        if progress <= sweep.abs() {
            Some(progress / sweep.abs())
        } else {
            let first = self.point_on_arc(id, 0.)?;
            let last = self.point_on_arc(id, 1.)?;
            Some(if p.distance(first) <= p.distance(last) {
                0.
            } else {
                1.
            })
        }
    }
    pub fn arc_points(&self, id: usize, subdivisions: usize) -> Option<Vec<Point>> {
        if self
            .objects
            .get(id.saturating_sub(1))
            .is_some_and(|object| matches!(object.kind, Kind::ArcOnCircle(..)))
            && self.undefined_arc_plot(id, 0.5)
        {
            return None;
        }
        if let Some((start, end)) = self.linear_three_point_arc(id) {
            return Some(vec![start, end]);
        }
        let geometry = self.arc_geometry(id)?;
        let count = subdivisions.max(2);
        Some(
            (0..=count)
                .map(|step| Self::sample_arc(geometry, step as f64 / count as f64))
                .collect(),
        )
    }
    pub fn move_point(&mut self, id: usize, p: Point) -> bool {
        if !p.x.is_finite() || !p.y.is_finite() {
            return false;
        }
        let kind = match self.objects.get(id.saturating_sub(1)).map(|o| &o.kind) {
            Some(Kind::Free(_)) => Kind::Free(self.snap_point(p)),
            Some(Kind::GraphVerticalUnit(horizontal_unit, _))
                if self.grid_form[self.active_page] == GridForm::Rectangular =>
            {
                let horizontal_unit = *horizontal_unit;
                let Some(Kind::GraphHorizontalUnit(origin, _)) = horizontal_unit
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|object| &object.kind)
                else {
                    return false;
                };
                let Some(origin) = self.position(*origin) else {
                    return false;
                };
                let step = origin.y - p.y;
                if !step.is_finite() || step <= 0.0 {
                    return false;
                }
                Kind::GraphVerticalUnit(horizontal_unit, step)
            }
            Some(Kind::DistanceVerticalUnit(x_axis, _, true)) => {
                let x_axis = *x_axis;
                let Some(Kind::DistanceAxis(origin, _, 1)) = x_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|object| &object.kind)
                else {
                    return false;
                };
                let Some(origin) = self.position(*origin) else {
                    return false;
                };
                let step = origin.y - p.y;
                if !step.is_finite() || step <= 0.0 {
                    return false;
                }
                Kind::DistanceVerticalUnit(x_axis, step, true)
            }
            Some(Kind::PointOnSegment(host, _))
            | Some(Kind::PointOnRay(host, _))
            | Some(Kind::PointOnLine(host, _)) => {
                let host = *host;
                let Some((a, b, mode)) = self.point_constraint_line_points(host) else {
                    return false;
                };
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let length_sq = dx * dx + dy * dy;
                if length_sq < 1e-18 {
                    return false;
                }
                let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq;
                match mode {
                    LineMode::Segment => Kind::PointOnSegment(host, t.clamp(0., 1.)),
                    LineMode::Ray => Kind::PointOnRay(host, t.max(0.)),
                    LineMode::Line => Kind::PointOnLine(host, t),
                }
            }
            Some(Kind::PointOnCircle(host, _, _)) => {
                let host = *host;
                let Some((center, _)) = self.circle_points(host) else {
                    return false;
                };
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                let length = dx.hypot(dy);
                if length < 1e-9 {
                    return false;
                }
                Kind::PointOnCircle(host, dx / length, -dy / length)
            }
            Some(Kind::PointOnPolygon(host, _, _)) => {
                let host = *host;
                let Some((edge, t)) = self.polygon_boundary_projection(host, p) else {
                    return false;
                };
                Kind::PointOnPolygon(host, edge, t)
            }
            Some(Kind::PointOnArc(host, _)) => {
                let host = *host;
                let Some(t) = self.arc_parameter(host, p) else {
                    return false;
                };
                Kind::PointOnArc(host, t)
            }
            _ => return false,
        };
        self.objects[id - 1].kind = kind;
        self.last_moved_object = Some(id);
        self.dirty = true;
        true
    }
    pub fn point_on_polygon(&self, host: usize, edge: usize, t: f64) -> Option<Point> {
        let vertices = self.polygon_points(host)?;
        if !t.is_finite() || !(0.0..=1.0).contains(&t) || edge >= vertices.len() {
            return None;
        }
        let a = vertices[edge];
        let b = vertices[(edge + 1) % vertices.len()];
        Some(Point {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        })
    }
    pub fn polygon_boundary_projection(&self, host: usize, p: Point) -> Option<(usize, f64)> {
        if !p.x.is_finite() || !p.y.is_finite() {
            return None;
        }
        let vertices = self.polygon_points(host)?;
        let mut closest: Option<(f64, usize, f64)> = None;
        for edge in 0..vertices.len() {
            let a = vertices[edge];
            let b = vertices[(edge + 1) % vertices.len()];
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let length_sq = dx * dx + dy * dy;
            if length_sq <= 1e-18 {
                continue;
            }
            let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq).clamp(0., 1.);
            let distance = (p.x - a.x - t * dx).powi(2) + (p.y - a.y - t * dy).powi(2);
            if closest.is_none_or(|(best, _, _)| distance < best) {
                closest = Some((distance, edge, t));
            }
        }
        closest.map(|(_, edge, t)| (edge, t))
    }
    /// Fraction of the polygon perimeter traversed from its first vertex.
    pub fn polygon_boundary_parameter(&self, host: usize, edge: usize, t: f64) -> Option<f64> {
        let vertices = self.polygon_points(host)?;
        if edge >= vertices.len() || !t.is_finite() {
            return None;
        }
        let lengths = vertices
            .iter()
            .zip(vertices.iter().cycle().skip(1))
            .take(vertices.len())
            .map(|(a, b)| (b.x - a.x).hypot(b.y - a.y))
            .collect::<Vec<_>>();
        let perimeter: f64 = lengths.iter().sum();
        (perimeter > 1e-9)
            .then(|| (lengths[..edge].iter().sum::<f64>() + lengths[edge] * t) / perimeter)
    }
    pub fn point_at_polygon_parameter(&self, host: usize, parameter: f64) -> Option<Point> {
        if !parameter.is_finite() {
            return None;
        }
        let vertices = self.polygon_points(host)?;
        let lengths = vertices
            .iter()
            .zip(vertices.iter().cycle().skip(1))
            .take(vertices.len())
            .map(|(a, b)| (b.x - a.x).hypot(b.y - a.y))
            .collect::<Vec<_>>();
        let perimeter: f64 = lengths.iter().sum();
        if perimeter <= 1e-9 {
            return None;
        }
        let mut remaining = parameter.rem_euclid(1.) * perimeter;
        for (edge, length) in lengths.iter().copied().enumerate() {
            if length <= 1e-9 {
                continue;
            }
            if remaining <= length {
                return self.point_on_polygon(host, edge, remaining / length);
            }
            remaining -= length;
        }
        vertices.first().copied()
    }
    /// Native axis constraints parameterize a fixed 200-pixel span centered
    /// on the origin. Coordinate unit vectors still govern equations and plots.
    pub(super) fn point_constraint_line_points(&self, host: usize) -> Option<(Point, Point, LineMode)> {
        let (center, horizontal) = match &self.objects.get(host.checked_sub(1)?)?.kind {
            Kind::GraphAxis(origin, _, axis @ (0 | 1 | 0x1_0001))
                | Kind::DistanceAxis(origin, _, axis @ (0 | 1)) => (self.position(*origin)?, *axis != 0),
            Kind::CircleAxis(circle, axis @ (0 | 1)) => (self.circle_points(*circle)?.0, *axis != 0),
            _ => return self.line_points(host),
        };
        let (dx, dy) = if horizontal { (100., 0.) } else { (0., 100.) };
        Some((Point { x: center.x-dx, y: center.y-dy },
              Point { x: center.x+dx, y: center.y+dy }, LineMode::Line))
    }
    pub fn constrained_point(&self, host: usize, p: Point) -> Option<Kind> {
        match self.objects.get(host.checked_sub(1)?)?.kind {
            ref kind if is_straight_kind(kind) => {
                let (a, b, mode) = self.point_constraint_line_points(host)?;
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let length_sq = dx * dx + dy * dy;
                if length_sq < 1e-18 {
                    return None;
                }
                let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq;
                Some(match mode {
                    LineMode::Segment => Kind::PointOnSegment(host, t.clamp(0., 1.)),
                    LineMode::Ray => Kind::PointOnRay(host, t.max(0.)),
                    LineMode::Line => Kind::PointOnLine(host, t),
                })
            }
            ref kind if kind.is_circle() => {
                let (center, _) = self.circle_points(host)?;
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                let length = dx.hypot(dy);
                (length > 1e-9).then_some(Kind::PointOnCircle(host, dx / length, -dy / length))
            }
            ref kind if kind.is_polygon() => self
                .polygon_boundary_projection(host, p)
                .map(|(edge, t)| Kind::PointOnPolygon(host, edge, t)),
            ref kind if kind.is_arc() => self
                .arc_parameter(host, p)
                .map(|t| Kind::PointOnArc(host, t)),
            _ => None,
        }
    }
}
