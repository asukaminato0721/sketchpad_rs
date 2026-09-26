use super::kind::{is_straight_kind, is_transformed_plot_path_kind, is_transformed_plot_ray_kind, plot_path_accepts};
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
    let mut parameters = [
        (-linear - root) / (2. * length_squared),
        (-linear + root) / (2. * length_squared),
    ]
    .into_iter()
    .filter(|t| match mode {
        LineMode::Segment => *t >= -1e-10 && *t <= 1. + 1e-10,
        LineMode::Ray => *t >= -1e-10,
        LineMode::Line => true,
    })
    .collect::<Vec<_>>();
    parameters.sort_by(f64::total_cmp);
    let t = *parameters.get(variant.min(parameters.len().checked_sub(1)?))?;
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
    let mut results = [
        Point {
            x: base.x - height * uy,
            y: base.y + height * ux,
        },
        Point {
            x: base.x + height * uy,
            y: base.y - height * ux,
        },
    ];
    results.sort_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)));
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
        value.is_finite() && value < 0.0
            && self.objects.get(path.saturating_sub(1)).is_some_and(|host| !host.deleted && is_transformed_plot_ray_kind(&host.kind))
            && self.line_points(path).is_some_and(|(start, end, _)| start != end)
    }

    pub fn position(&self, id: usize) -> Option<Point> {
        fn go(d: &Document, id: usize, seen: &mut BTreeSet<usize>) -> Option<Point> {
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
                if is_transformed_plot_path_kind(kind) {
                    let (start, end, _) = d.line_points(path)?;
                    let point = Point { x: start.x + (end.x - start.x) * t, y: start.y + (end.y - start.y) * t };
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
                let start = go(d, start, seen)?;
                let end = go(d, end, seen)?;
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
            let p = match &object.kind {
                Kind::Free(p) => Some(*p),
                Kind::GraphHorizontalUnit(origin, step) => {
                    let origin = go(d, *origin, seen)?;
                    Some(Point {
                        x: origin.x + step,
                        y: origin.y,
                    })
                }
                Kind::GraphVerticalUnit(horizontal_unit, step) => {
                    let Kind::GraphHorizontalUnit(origin, _) =
                        d.objects.get(horizontal_unit.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    let origin = go(d, origin, seen)?;
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
                    let origin = go(d, origin, seen)?;
                    let length = if *independent {
                        *step
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
                },
                Kind::PathValuePoint(source, path) => {
                    let value = d.numeric_value(*source)?;
                    if d.objects.get(path.checked_sub(1)?)?.kind.is_polygon() {
                        d.point_at_polygon_parameter(*path, value)
                    } else {
                        on_path(d, *path, value, seen)
                    }
                }
                Kind::NumericCoordinatePoint(x, y, frame) => {
                    let frame = d.graph_frame(*frame)?;
                    let x = d.numeric_value(*x)?;
                    let y = d.numeric_value(*y)?;
                    let point = frame.world(x, y)?;
                    Some(Point { x: point.x, y: point.y })
                }
                Kind::PlottedPoint(_, frame, mode, stored) => {
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
                Kind::PointOnSegment(host, t)
                | Kind::PointOnRay(host, t)
                | Kind::PointOnLine(host, t) => {
                    let (a, b, _) = d.line_points(*host)?;
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
                Kind::PolarTranslatedPoint(origin, angle, units_to_raw, distance) => {
                    let source = go(d, *origin, seen)?;
                    let radians = angle.to_radians();
                    let step = units_to_raw * distance;
                    Some(Point {
                        x: source.x + step * radians.cos(),
                        y: source.y - step * radians.sin(),
                    })
                }
                Kind::FixedRectangularTranslatedPoint(
                    source,
                    x_units,
                    x_distance,
                    y_units,
                    y_distance,
                ) => {
                    let source = go(d, *source, seen)?;
                    Some(Point {
                        x: source.x + x_units * x_distance,
                        y: source.y - y_units * y_distance,
                    })
                }
                Kind::MarkedDistanceTranslatedPoint(source, distance, angle) => {
                    let source = go(d, *source, seen)?;
                    let step = d.distance_value_pixels(*distance)?;
                    let radians = angle.to_radians();
                    Some(Point {
                        x: source.x + step * radians.cos(),
                        y: source.y - step * radians.sin(),
                    })
                }
                Kind::MixedRectangularTranslatedPoint(
                    source,
                    distance,
                    fixed_cm,
                    fixed_horizontal,
                ) => {
                    let source = go(d, *source, seen)?;
                    let fixed_pixels = fixed_cm * 37.79527559055118;
                    let marked_pixels = d.distance_value_pixels(*distance)?;
                    Some(Point {
                        x: source.x
                            + if *fixed_horizontal {
                                fixed_pixels
                            } else {
                                marked_pixels
                            },
                        y: source.y
                            - if *fixed_horizontal {
                                marked_pixels
                            } else {
                                fixed_pixels
                            },
                    })
                }
                Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical) => {
                    let source = go(d, *source, seen)?;
                    Some(Point {
                        x: source.x + d.distance_value_pixels(*horizontal)?,
                        y: source.y - d.distance_value_pixels(*vertical)?,
                    })
                }
                Kind::VectorTranslatedPoint(source, start, end) => {
                    let source = go(d, *source, seen)?;
                    let start = go(d, *start, seen)?;
                    let end = go(d, *end, seen)?;
                    Some(Point {
                        x: source.x + end.x - start.x,
                        y: source.y + end.y - start.y,
                    })
                }
                Kind::ReflectedPoint(source, mirror) => {
                    let source = go(d, *source, seen)?;
                    let (a, b, _) = d.line_points(*mirror)?;
                    reflect_point(source, a, b)
                }
                Kind::ScaledPoint(source, center, factor) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    Some(Point {
                        x: center.x + factor * (source.x - center.x),
                        y: center.y + factor * (source.y - center.y),
                    })
                }
                Kind::RatioScaledPoint(source, center, origin, base, target) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    let factor = d.ratio_for_points(*origin, *base, *target)?;
                    Some(Point {
                        x: center.x + factor * (source.x - center.x),
                        y: center.y + factor * (source.y - center.y),
                    })
                }
                Kind::CalculatedRatioScaledPoint(source, center, ratio) => {
                    let source = go(d, *source, seen)?;
                    scale_point_about(
                        source,
                        go(d, *center, seen)?,
                        d.unitless_numeric_value(*ratio)?,
                    )
                }
                Kind::CalculatedAngleRotatedPoint(source, center, angle) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    rotate_point_about(source, center, d.numeric_value(*angle)?.to_radians())
                }
                Kind::RotatedPoint(source, center, angle) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    let theta = angle.to_radians();
                    let dx = source.x - center.x;
                    let dy = source.y - center.y;
                    Some(Point {
                        x: center.x + dx * theta.cos() + dy * theta.sin(),
                        y: center.y - dx * theta.sin() + dy * theta.cos(),
                    })
                }
                Kind::AngleRotatedPoint(source, center, first, vertex, last) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    let first = go(d, *first, seen)?;
                    let vertex = go(d, *vertex, seen)?;
                    let last = go(d, *last, seen)?;
                    rotate_point_about(source, center, marked_rotation_theta(first, vertex, last)?)
                }
                Kind::Midpoint(a, b) => {
                    let a = go(d, *a, seen)?;
                    let b = go(d, *b, seen)?;
                    Some(Point {
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
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        match &object.kind {
            Kind::Polygon(vertices) => vertices
                .iter()
                .map(|&vertex| self.position(vertex))
                .collect(),
            Kind::ReflectedPolygon(source, mirror) => {
                let points = self.polygon_points(*source)?;
                let (a, b, _) = self.line_points(*mirror)?;
                points
                    .into_iter()
                    .map(|point| reflect_point(point, a, b))
                    .collect()
            }
            Kind::VectorTranslatedPolygon(source, start, end) => {
                let start = self.position(*start)?;
                let end = self.position(*end)?;
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::PolarTranslatedPolygon(source, angle, units, distance) => {
                let radians = angle.to_radians();
                let dx = units * distance * radians.cos();
                let dy = -units * distance * radians.sin();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::FixedRectangularTranslatedPolygon(source, xu, xd, yu, yd) => {
                let dx = xu * xd;
                let dy = -yu * yd;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::MarkedDistanceTranslatedPolygon(source, distance, angle) => {
                let step = self.distance_value_pixels(*distance)?;
                let radians = angle.to_radians();
                let dx = step * radians.cos();
                let dy = -step * radians.sin();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical) => {
                let dx = self.distance_value_pixels(*horizontal)?;
                let dy = -self.distance_value_pixels(*vertical)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::MixedRectangularTranslatedPolygon(
                source,
                distance,
                fixed_cm,
                fixed_horizontal,
            ) => {
                let marked = self.distance_value_pixels(*distance)?;
                let fixed = fixed_cm * 37.79527559055118;
                let (dx, dy) = if *fixed_horizontal {
                    (fixed, -marked)
                } else {
                    (marked, -fixed)
                };
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::RotatedPolygon(source, center, angle) => {
                let center = self.position(*center)?;
                let theta = angle.to_radians();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| rotate_point_about(point, center, theta))
                    .collect()
            }
            Kind::CalculatedRatioScaledPolygon(source, center, ratio) => {
                let center = self.position(*center)?;
                let factor = self.unitless_numeric_value(*ratio)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| scale_point_about(point, center, factor))
                    .collect()
            }
            Kind::CalculatedAngleRotatedPolygon(source, center, angle) => {
                let center = self.position(*center)?;
                let theta = self.numeric_value(*angle)?.to_radians();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| rotate_point_about(point, center, theta))
                    .collect()
            }
            Kind::AngleRotatedPolygon(source, center, first, vertex, last) => {
                let center = self.position(*center)?;
                let theta = marked_rotation_theta(
                    self.position(*first)?,
                    self.position(*vertex)?,
                    self.position(*last)?,
                )?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| rotate_point_about(point, center, theta))
                    .collect()
            }
            Kind::ScaledPolygon(source, center, factor) => {
                let center = self.position(*center)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = center.x + factor * (point.x - center.x);
                        let y = center.y + factor * (point.y - center.y);
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::RatioScaledPolygon(source, center, origin, base, target) => {
                let center = self.position(*center)?;
                let factor = self.ratio_for_points(*origin, *base, *target)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = center.x + factor * (point.x - center.x);
                        let y = center.y + factor * (point.y - center.y);
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            _ => None,
        }
    }
    pub(super) fn fixed_straight_has_zero_factor(&self, mut id: usize) -> bool {
        let mut seen = BTreeSet::new();
        while seen.insert(id) {
            match self.objects.get(id.saturating_sub(1)).map(|object| &object.kind) {
                Some(Kind::ScaledRay(_, _, factor) | Kind::ScaledLine(_, _, factor)) if *factor == 0. => return true,
                Some(Kind::ScaledRay(source, _, _) | Kind::ScaledLine(source, _, _)
                    | Kind::RotatedRay(source, _, _) | Kind::RotatedLine(source, _, _)) => id = *source,
                _ => return false,
            }
        }
        false
    }

    pub fn line_points(&self, id: usize) -> Option<(Point, Point, LineMode)> {
        fn go(
            d: &Document,
            id: usize,
            seen: &mut BTreeSet<usize>,
        ) -> Option<(Point, Point, LineMode)> {
            if !seen.insert(id) {
                return None;
            }
            let object = d.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            let result = match object.kind {
                Kind::Segment(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Segment)),
                Kind::PolarTranslatedSegment(source, angle, units, distance) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let step = units * distance;
                    let radians = angle.to_radians();
                    let dx = step * radians.cos();
                    let dy = -step * radians.sin();
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::FixedRectangularTranslatedSegment(
                    source,
                    x_units,
                    x_distance,
                    y_units,
                    y_distance,
                ) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let dx = x_units * x_distance;
                    let dy = -y_units * y_distance;
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::RectangularMarkedTranslatedSegment(source, horizontal, vertical) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let dx = d.distance_value_pixels(horizontal)?;
                    let dy = -d.distance_value_pixels(vertical)?;
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::MixedRectangularTranslatedSegment(source, distance, fixed, horizontal) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let dx = if horizontal {
                        fixed * 37.79527559055118
                    } else {
                        d.distance_value_pixels(distance)?
                    };
                    let dy = if horizontal {
                        -d.distance_value_pixels(distance)?
                    } else {
                        -fixed * 37.79527559055118
                    };
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::VectorTranslatedSegment(source, start, end) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let start = d.position(start)?;
                    let end = d.position(end)?;
                    let dx = end.x - start.x;
                    let dy = end.y - start.y;
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::MarkedDistanceTranslatedSegment(source, distance, angle) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let step = d.distance_value_pixels(distance)?;
                    let radians = angle.to_radians();
                    let dx = step * radians.cos();
                    let dy = -step * radians.sin();
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::CalculatedRatioScaledSegment(source, center, ratio) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let factor = d.unitless_numeric_value(ratio)?;
                    Some((
                        scale_point_about(a, center, factor)?,
                        scale_point_about(b, center, factor)?,
                        mode,
                    ))
                }
                Kind::CalculatedAngleRotatedSegment(source, center, angle) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let theta = d.numeric_value(angle)?.to_radians();
                    Some((
                        rotate_point_about(a, center, theta)?,
                        rotate_point_about(b, center, theta)?,
                        mode,
                    ))
                }
                Kind::AngleRotatedSegment(source, center, first, vertex, last) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let theta = marked_rotation_theta(
                        d.position(first)?,
                        d.position(vertex)?,
                        d.position(last)?,
                    )?;
                    Some((
                        rotate_point_about(a, center, theta)?,
                        rotate_point_about(b, center, theta)?,
                        mode,
                    ))
                }
                Kind::RotatedSegment(source, center, angle)
                | Kind::RotatedRay(source, center, angle)
                | Kind::RotatedLine(source, center, angle) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let theta = angle.to_radians();
                    Some((
                        rotate_point_about(a, center, theta)?,
                        rotate_point_about(b, center, theta)?,
                        mode,
                    ))
                }
                Kind::ScaledSegment(source, center, factor)
                | Kind::ScaledRay(source, center, factor)
                | Kind::ScaledLine(source, center, factor) => {
                    if factor == 0.
                        && matches!(object.kind, Kind::ScaledRay(..) | Kind::ScaledLine(..))
                    {
                        return None;
                    }
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let scale = |point: Point| Point {
                        x: center.x + factor * (point.x - center.x),
                        y: center.y + factor * (point.y - center.y),
                    };
                    let (a, b) = (scale(a), scale(b));
                    (a.x.is_finite() && a.y.is_finite() && b.x.is_finite() && b.y.is_finite())
                        .then_some((a, b, mode))
                }
                Kind::RatioScaledSegment(source, center, origin, base, target) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let factor = d.ratio_for_points(origin, base, target)?;
                    let scale = |point: Point| Point {
                        x: center.x + factor * (point.x - center.x),
                        y: center.y + factor * (point.y - center.y),
                    };
                    let (a, b) = (scale(a), scale(b));
                    (a.x.is_finite() && a.y.is_finite() && b.x.is_finite() && b.y.is_finite())
                        .then_some((a, b, mode))
                }
                Kind::ReflectedSegment(source, mirror) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let (mirror_a, mirror_b, _) = go(d, mirror, seen)?;
                    Some((
                        reflect_point(a, mirror_a, mirror_b)?,
                        reflect_point(b, mirror_a, mirror_b)?,
                        mode,
                    ))
                }
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
                    Some((
                        origin,
                        Point {
                            x: origin.x + direction.x,
                            y: origin.y + direction.y,
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
        if self.objects.get(id.checked_sub(1)?)?.deleted {
            return None;
        }
        if let Kind::PolarTranslatedCircle(source, angle, units, distance) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let step = units * distance;
            let radians = angle.to_radians();
            let moved = Point {
                x: center.x + step * radians.cos(),
                y: center.y - step * radians.sin(),
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::FixedRectangularTranslatedCircle(
            source,
            x_units,
            x_distance,
            y_units,
            y_distance,
        ) = self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let moved = Point {
                x: center.x + x_units * x_distance,
                y: center.y - y_units * y_distance,
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::RectangularMarkedTranslatedCircle(source, horizontal, vertical) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let dx = self.distance_value_pixels(horizontal)?;
            let dy = -self.distance_value_pixels(vertical)?;
            let moved = Point {
                x: center.x + dx,
                y: center.y + dy,
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::MixedRectangularTranslatedCircle(source, distance, fixed, horizontal) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let dx = if horizontal {
                fixed * 37.79527559055118
            } else {
                self.distance_value_pixels(distance)?
            };
            let dy = if horizontal {
                -self.distance_value_pixels(distance)?
            } else {
                -fixed * 37.79527559055118
            };
            let moved = Point {
                x: center.x + dx,
                y: center.y + dy,
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::VectorTranslatedCircle(source, start, end) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let start = self.position(start)?;
            let end = self.position(end)?;
            let moved = Point {
                x: center.x + end.x - start.x,
                y: center.y + end.y - start.y,
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::MarkedDistanceTranslatedCircle(source, distance, angle) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let step = self.distance_value_pixels(distance)?;
            let radians = angle.to_radians();
            let moved = Point {
                x: center.x + step * radians.cos(),
                y: center.y - step * radians.sin(),
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::CalculatedRatioScaledCircle(source, center, ratio) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let factor = self.unitless_numeric_value(ratio)?;
            let scaled = scale_point_about(source_center, center, factor)?;
            let radius = radius * factor.abs();
            return (radius.is_finite() && radius > 0.).then_some((scaled, radius));
        }
        if let Kind::CalculatedAngleRotatedCircle(source, center, angle) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            return Some((
                rotate_point_about(
                    source_center,
                    center,
                    self.numeric_value(angle)?.to_radians(),
                )?,
                radius,
            ));
        }
        if let Kind::AngleRotatedCircle(source, center, first, vertex, last) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let theta = marked_rotation_theta(
                self.position(first)?,
                self.position(vertex)?,
                self.position(last)?,
            )?;
            return Some((rotate_point_about(source_center, center, theta)?, radius));
        }
        if let Kind::RotatedCircle(source, center, angle) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            return Some((
                rotate_point_about(source_center, center, angle.to_radians())?,
                radius,
            ));
        }
        if let Kind::ReflectedCircle(source, mirror) = self.objects.get(id.checked_sub(1)?)?.kind {
            let (center, radius) = self.circle_points(source)?;
            let (a, b, _) = self.line_points(mirror)?;
            return Some((reflect_point(center, a, b)?, radius));
        }
        if let Kind::ScaledCircle(source, center, factor) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let scaled = Point {
                x: center.x + factor * (source_center.x - center.x),
                y: center.y + factor * (source_center.y - center.y),
            };
            let radius = radius * factor.abs();
            return (scaled.x.is_finite()
                && scaled.y.is_finite()
                && radius.is_finite()
                && radius > 0.)
                .then_some((scaled, radius));
        }
        if let Kind::RatioScaledCircle(source, center, origin, base, target) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let factor = self.ratio_for_points(origin, base, target)?;
            let scaled = Point {
                x: center.x + factor * (source_center.x - center.x),
                y: center.y + factor * (source_center.y - center.y),
            };
            let radius = radius * factor.abs();
            return (scaled.x.is_finite()
                && scaled.y.is_finite()
                && radius.is_finite()
                && radius > 0.)
                .then_some((scaled, radius));
        }
        let (center_id, radius) = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Circle(center, radius_point) => {
                let center_position = self.position(center)?;
                (
                    center,
                    center_position.distance(self.position(radius_point)?),
                )
            }
            Kind::CircleBySegment(center, segment) => {
                let Kind::Segment(a, b) = self.objects.get(segment.checked_sub(1)?)?.kind else {
                    return None;
                };
                (center, self.position(a)?.distance(self.position(b)?))
            }
            _ => return None,
        };
        let center = self.position(center_id)?;
        (radius.is_finite() && radius > 0.).then_some((center, radius))
    }
    pub(super) fn arc_geometry(&self, id: usize) -> Option<(Point, f64, f64, f64, bool)> {
        let geometry = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::ThreePointArc(a, b, c) => {
                let (center, radius, start, sweep) = three_point_arc_geometry(
                    self.position(a)?,
                    self.position(b)?,
                    self.position(c)?,
                )?;
                (center, radius, start, sweep, false)
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
                let Some((a, b, mode)) = self.line_points(host) else {
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
    pub fn constrained_point(&self, host: usize, p: Point) -> Option<Kind> {
        match self.objects.get(host.checked_sub(1)?)?.kind {
            ref kind if is_straight_kind(kind) => {
                let (a, b, mode) = self.line_points(host)?;
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
            Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _) => self
                .arc_parameter(host, p)
                .map(|t| Kind::PointOnArc(host, t)),
            _ => None,
        }
    }
}
