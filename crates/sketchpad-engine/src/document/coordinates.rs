use super::kind::{is_coordinate_unit_kind, is_point_kind};
use super::{Document, Kind, Point};
use crate::graph::{CoordinateFrame, GraphPoint, GridForm};

impl Document {
    pub fn active_coordinate_system(&self) -> Option<usize> {
        self.active_coordinate_system_for_page(self.active_page)
    }
    fn active_coordinate_system_for_page(&self, page: usize) -> Option<usize> {
        if let Some(id) = self.marked_coordinate_system.get(page).copied().flatten()
            && self.objects.get(id.checked_sub(1)?).is_some_and(|object| {
                object.page == page
                    && !object.deleted
                    && matches!(object.kind, Kind::CoordinateSystem(_, _))
            })
        {
            return Some(id);
        }
        self.objects
            .iter()
            .rev()
            .find(|o| {
                o.page == page && !o.deleted && matches!(o.kind, Kind::CoordinateSystem(_, _))
            })
            .map(|o| o.id)
    }
    pub fn coordinate_system_for_object(&self, id: usize) -> Option<usize> {
        let page = self.objects.get(id.checked_sub(1)?)?.page;
        self.objects.iter().rev().find_map(|frame| {
            if frame.page != page || frame.deleted {
                return None;
            }
            let Kind::CoordinateSystem(x_axis, y_axis) = frame.kind else {
                return None;
            };
            match (
                self.objects.get(x_axis.checked_sub(1)?)?.kind.clone(),
                self.objects.get(y_axis.checked_sub(1)?)?.kind.clone(),
            ) {
                (Kind::GraphAxis(origin, x_unit, 1 | 0x1_0001), Kind::GraphAxis(_, y_unit, 0)) => {
                    [frame.id, x_axis, y_axis, origin, x_unit, y_unit]
                        .contains(&id)
                        .then_some(frame.id)
                }
                (Kind::CircleAxis(circle, 1), Kind::CircleAxis(other, 0)) if circle == other => {
                    [frame.id, x_axis, y_axis, circle]
                        .contains(&id)
                        .then_some(frame.id)
                }
                (Kind::DistanceAxis(origin, distance, 1), Kind::DistanceAxis(other, second, 0))
                    if origin == other =>
                {
                    [frame.id, x_axis, y_axis, origin, distance, second]
                        .contains(&id)
                        .then_some(frame.id)
                }
                (Kind::DistanceAxis(origin, distance, 1), Kind::GraphAxis(other, unit, 0))
                    if origin == other
                        && matches!(
                            self.objects.get(unit.checked_sub(1)?)?.kind,
                            Kind::DistanceVerticalUnit(host, _, _) if host == x_axis
                        ) =>
                {
                    [frame.id, x_axis, y_axis, origin, distance, unit]
                        .contains(&id)
                        .then_some(frame.id)
                }
                _ => None,
            }
        })
    }
    pub fn mark_coordinate_system(&mut self, selected: usize) -> Result<usize, String> {
        let frame = self
            .coordinate_system_for_object(selected)
            .ok_or("Select a coordinate system axis, origin, or unit point")?;
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot mark a coordinate system in this sketch".into());
        }
        if self.active_coordinate_system() != Some(frame) {
            self.checkpoint();
            self.marked_coordinate_system[self.active_page] = Some(frame);
            if let Some((form, dotted)) = self.frame_styles.get(&frame).copied() {
                self.grid_form[self.active_page] = form;
                self.grid_dotted[self.active_page] = dotted;
            }
        }
        Ok(frame)
    }
    pub(super) fn active_x_axis(&self, page: usize) -> Option<usize> {
        let frame = self.objects.get(
            self.active_coordinate_system_for_page(page)?
                .checked_sub(1)?,
        )?;
        let Kind::CoordinateSystem(x_axis, _) = frame.kind else {
            return None;
        };
        Some(x_axis)
    }
    pub fn add_default_coordinate_system(&mut self, origin: Point) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || self.active_coordinate_system().is_some() {
            return Err("Cannot define another coordinate system in this sketch".into());
        }
        if !origin.x.is_finite() || !origin.y.is_finite() {
            return Err("Coordinate-system origin must be finite".into());
        }
        let step: f64 = 37.79527559055118;
        self.checkpoint();
        let center = self.append_object(Kind::Free(origin));
        let x_unit = self.append_object(Kind::GraphHorizontalUnit(center, step));
        let x_axis = self.append_object(Kind::GraphAxis(center, x_unit, 1));
        let y_unit = self.append_object(Kind::GraphVerticalUnit(x_unit, step));
        let y_axis = self.append_object(Kind::GraphAxis(center, y_unit, 0));
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (GridForm::Square, false));
        self.snap_points[self.active_page] = false;
        self.grid_dotted[self.active_page] = false;
        self.grid_form[self.active_page] = GridForm::Square;
        Ok(frame)
    }
    pub fn add_coordinate_system_at_point(&mut self, origin: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot define a coordinate system in this sketch".into());
        }
        let point = self.position(origin).ok_or("Select one origin point")?;
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err("Coordinate-system origin must be finite".into());
        }
        let step: f64 = 37.79527559055118;
        self.checkpoint();
        let x_unit = self.append_object(Kind::GraphHorizontalUnit(origin, step));
        let x_axis = self.append_object(Kind::GraphAxis(origin, x_unit, 1));
        let y_unit = self.append_object(Kind::GraphVerticalUnit(x_unit, step));
        let y_axis = self.append_object(Kind::GraphAxis(origin, y_unit, 0));
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (GridForm::Square, false));
        self.grid_form[self.active_page] = GridForm::Square;
        self.grid_dotted[self.active_page] = false;
        Ok(frame)
    }
    pub fn add_coordinate_system_from_circle(&mut self, circle: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot define a coordinate system in this sketch".into());
        }
        if !self
            .objects
            .get(circle.saturating_sub(1))
            .is_some_and(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_circle()
            })
        {
            return Err("Select one circle".into());
        }
        let (_, radius) = self.circle_points(circle).ok_or("Invalid unit circle")?;
        if !radius.is_finite() || radius <= 0.0 {
            return Err("Invalid unit circle".into());
        }
        self.checkpoint();
        let x_axis = self.append_object(Kind::CircleAxis(circle, 1));
        let y_axis = self.append_object(Kind::CircleAxis(circle, 0));
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (GridForm::Square, false));
        self.grid_form[self.active_page] = GridForm::Square;
        self.grid_dotted[self.active_page] = false;
        Ok(frame)
    }
    pub fn add_coordinate_system_from_distances(
        &mut self,
        origin: Option<usize>,
        distances: &[usize],
        default_origin: Point,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot define a coordinate system in this sketch".into());
        }
        if !matches!(distances.len(), 1 | 2)
            || distances[0] == *distances.last().unwrap() && distances.len() == 2
            || distances.iter().any(|&id| {
                self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                    id == 0
                        || object.deleted
                        || object.page != self.active_page
                        || !is_coordinate_unit_kind(&object.kind)
                }) || self.defining_distance_pixels(id).is_none()
            })
        {
            return Err("Select one or two nonzero defining distances".into());
        }
        if let Some(id) = origin {
            if self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                id == 0
                    || object.deleted
                    || object.page != self.active_page
                    || !is_point_kind(&object.kind)
            }) {
                return Err("Select one origin point".into());
            }
        } else if !default_origin.x.is_finite() || !default_origin.y.is_finite() {
            return Err("Coordinate-system origin must be finite".into());
        }
        self.checkpoint();
        let origin = origin.unwrap_or_else(|| self.append_object(Kind::Free(default_origin)));
        let x_axis = self.append_object(Kind::DistanceAxis(origin, distances[0], 1));
        let (y_axis, form) = if distances.len() == 1 {
            let unit =
                self.append_object(Kind::DistanceVerticalUnit(x_axis, 37.79527559055118, false));
            (
                self.append_object(Kind::GraphAxis(origin, unit, 0)),
                GridForm::Square,
            )
        } else {
            (
                self.append_object(Kind::DistanceAxis(origin, distances[1], 0)),
                GridForm::Rectangular,
            )
        };
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (form, false));
        self.grid_form[self.active_page] = form;
        self.grid_dotted[self.active_page] = false;
        Ok(frame)
    }
    pub fn set_grid_visible(&mut self, visible: bool) {
        if let Some(id) = self.active_coordinate_system()
            && self.objects[id - 1].visible != visible
        {
            self.checkpoint();
            self.objects[id - 1].visible = visible;
        }
    }
    pub fn grid_visible(&self) -> bool {
        self.grid_enabled()
    }
    pub fn grid_enabled(&self) -> bool {
        self.active_coordinate_system()
            .is_some_and(|id| self.objects[id - 1].visible)
    }
    pub fn set_snap_points(&mut self, enabled: bool) {
        if self.active_coordinate_system().is_some()
            && self.snap_points[self.active_page] != enabled
        {
            self.checkpoint();
            self.snap_points[self.active_page] = enabled;
        }
    }
    pub fn snap_point(&self, point: Point) -> Point {
        if !self.snap_points[self.active_page] {
            return point;
        }
        let Some(frame) = self
            .active_coordinate_system()
            .and_then(|id| self.graph_frame(id))
        else {
            return point;
        };
        let Some(graph_point) = frame.coordinates(GraphPoint {
            x: point.x,
            y: point.y,
        }) else {
            return point;
        };
        let (x, y) = if self.grid_form[self.active_page] == GridForm::Polar {
            let radius = graph_point.x.hypot(graph_point.y).round();
            let sector =
                (graph_point.y.atan2(graph_point.x) / (std::f64::consts::TAU / 24.)).round();
            let angle = sector * std::f64::consts::TAU / 24.;
            (radius * angle.cos(), radius * angle.sin())
        } else {
            (graph_point.x.round(), graph_point.y.round())
        };
        frame.world(x, y).map_or(point, |point| Point {
            x: point.x,
            y: point.y,
        })
    }
    pub fn set_grid_dotted(&mut self, dotted: bool) {
        if let Some(frame) = self.active_coordinate_system()
            && self.grid_dotted[self.active_page] != dotted
        {
            self.checkpoint();
            self.grid_dotted[self.active_page] = dotted;
            self.frame_styles
                .insert(frame, (self.grid_form[self.active_page], dotted));
        }
    }
    pub fn set_grid_form(&mut self, form: GridForm) -> Result<(), String> {
        let frame = self
            .active_coordinate_system()
            .ok_or("No coordinate system")?;
        if self.grid_form[self.active_page] == form {
            return Ok(());
        }
        let Kind::CoordinateSystem(x_axis, y_axis) = self.objects[frame - 1].kind else {
            return Err("Invalid coordinate system".into());
        };
        if matches!(
            self.objects.get(x_axis - 1).map(|object| &object.kind),
            Some(Kind::CircleAxis(_, 1))
        ) {
            if form == GridForm::Rectangular {
                return Err("A unit circle cannot define a rectangular grid".into());
            }
            self.checkpoint();
            self.grid_form[self.active_page] = form;
            self.frame_styles
                .insert(frame, (form, self.grid_dotted[self.active_page]));
            return Ok(());
        }
        if let Some(Kind::DistanceAxis(_, distance, 1)) =
            self.objects.get(x_axis - 1).map(|object| &object.kind)
        {
            let unit = match self.objects.get(y_axis - 1).map(|object| &object.kind) {
                Some(Kind::DistanceAxis(_, _, 0)) if form != GridForm::Square => None,
                Some(Kind::GraphAxis(_, unit, 0))
                    if matches!(
                        self.objects.get(unit.saturating_sub(1)).map(|object| &object.kind),
                        Some(Kind::DistanceVerticalUnit(host, _, _)) if *host == x_axis
                    ) =>
                {
                    Some(*unit)
                }
                _ => return Err("This defining-distance frame cannot use that grid form".into()),
            };
            let unit_change = unit.and_then(|unit| {
                let Kind::DistanceVerticalUnit(_, step, independent) = self.objects[unit - 1].kind
                else {
                    return None;
                };
                let next = match form {
                    GridForm::Square => (step, false),
                    GridForm::Rectangular if !independent => {
                        (self.defining_distance_pixels(*distance)?, true)
                    }
                    GridForm::Rectangular | GridForm::Polar => (step, independent),
                };
                Some((unit, next))
            });
            if unit.is_some() && unit_change.is_none() {
                return Err("Invalid defining-distance unit".into());
            }
            self.checkpoint();
            if let Some((unit, (step, independent))) = unit_change {
                self.objects[unit - 1].kind = Kind::DistanceVerticalUnit(x_axis, step, independent);
            }
            self.grid_form[self.active_page] = form;
            self.frame_styles
                .insert(frame, (form, self.grid_dotted[self.active_page]));
            return Ok(());
        }
        let Some(Kind::GraphAxis(_, y_unit, 0)) = y_axis
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
            .map(|object| &object.kind)
        else {
            return Err("Invalid vertical axis".into());
        };
        let Some(Kind::GraphVerticalUnit(x_unit, _)) = y_unit
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
            .map(|object| &object.kind)
        else {
            return Err("Unsupported vertical unit".into());
        };
        let Some(Kind::GraphHorizontalUnit(_, x_step)) = x_unit
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
            .map(|object| &object.kind)
        else {
            return Err("Unsupported horizontal unit".into());
        };
        let (y_unit, x_unit, x_step) = (*y_unit, *x_unit, *x_step);
        let y_step = if form == GridForm::Rectangular {
            f64::from_bits(x_step.to_bits().saturating_sub(1))
        } else {
            x_step
        };
        if !y_step.is_finite() || y_step <= 0.0 {
            return Err("Invalid grid spacing".into());
        }
        self.checkpoint();
        self.objects[y_unit - 1].kind = Kind::GraphVerticalUnit(x_unit, y_step);
        self.grid_form[self.active_page] = form;
        self.frame_styles
            .insert(frame, (form, self.grid_dotted[self.active_page]));
        Ok(())
    }
    pub fn axis_trigonometric(&self) -> bool {
        self.active_x_axis(self.active_page)
            .and_then(|id| self.objects.get(id.checked_sub(1)?))
            .is_some_and(|object| matches!(object.kind, Kind::GraphAxis(_, _, 0x1_0001)))
    }
    pub fn set_axis_trigonometric(&mut self, enabled: bool) -> Result<(), String> {
        let x_axis = self
            .active_x_axis(self.active_page)
            .ok_or("No coordinate system")?;
        let axis_index = x_axis.checked_sub(1).ok_or("Invalid horizontal axis")?;
        let Kind::GraphAxis(origin, unit, axis) = self
            .objects
            .get(axis_index)
            .ok_or("Invalid horizontal axis")?
            .kind
        else {
            return Err("Invalid horizontal axis".into());
        };
        if !matches!(axis, 1 | 0x1_0001) {
            return Err("Unsupported horizontal axis".into());
        }
        if self.axis_trigonometric() != enabled {
            self.checkpoint();
            self.objects[axis_index].kind =
                Kind::GraphAxis(origin, unit, if enabled { 0x1_0001 } else { 1 });
        }
        Ok(())
    }
    pub fn graph_frame(&self, id: usize) -> Option<CoordinateFrame> {
        if let Some(value) = self.evaluation_frame(id) {
            return value;
        }
        let Kind::CoordinateSystem(x_axis, y_axis) = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let valid = match (
            &self.objects.get(x_axis.checked_sub(1)?)?.kind,
            &self.objects.get(y_axis.checked_sub(1)?)?.kind,
        ) {
            (Kind::GraphAxis(origin, _, 1 | 0x1_0001), Kind::GraphAxis(other, _, 0)) => {
                origin == other
            }
            (Kind::CircleAxis(circle, 1), Kind::CircleAxis(other, 0)) => circle == other,
            (Kind::DistanceAxis(origin, _, 1), Kind::DistanceAxis(other, _, 0)) => origin == other,
            (Kind::DistanceAxis(origin, _, 1), Kind::GraphAxis(other, unit, 0)) => {
                origin == other
                    && matches!(self.objects.get(unit.checked_sub(1)?)?.kind,
                    Kind::DistanceVerticalUnit(host, _, _) if host == x_axis)
            }
            _ => false,
        };
        if !valid {
            return None;
        }
        let (origin, x_unit, _) = self.line_points(x_axis)?;
        let (_, y_unit, _) = self.line_points(y_axis)?;
        let map = |p: Point| GraphPoint { x: p.x, y: p.y };
        Some(CoordinateFrame {
            origin: map(origin),
            x_unit: map(x_unit),
            y_unit: map(y_unit),
        })
    }

    /// Stored coordinate geometry remains valid when a live origin or unit is
    /// undefined. Native hides its grid and dependent plots until it recovers.
    pub(super) fn coordinate_geometry_undefined(&self, id: usize) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|i| self.objects.get(i)) else {
            return false;
        };
        if object.deleted {
            return false;
        }
        let parent = |source: usize| {
            source
                .checked_sub(1)
                .and_then(|i| self.objects.get(i))
                .filter(|o| source < id && !o.deleted && o.page == object.page)
        };
        match object.kind {
            Kind::GraphHorizontalUnit(origin, step) => {
                step.is_finite()
                    && step > 0.
                    && parent(origin).is_some_and(|origin| {
                        self.position(origin.id).is_none()
                            && match &origin.kind {
                                Kind::IterationPointAlias(binding) => {
                                    super::iteration_endpoint::binding_source(
                                        &self.objects,
                                        *binding,
                                        origin.page,
                                        origin.id,
                                    )
                                    .is_some()
                                }
                                Kind::Transformed(definition) => {
                                    definition.element()
                                        == super::transform_model::ElementType::Point
                                        && super::transform_validation::definition_supported(
                                            self, definition,
                                        )
                                }
                                Kind::GraphHorizontalUnit(..)
                                | Kind::GraphVerticalUnit(..)
                                | Kind::DistanceVerticalUnit(..) => {
                                    self.coordinate_geometry_undefined(origin.id)
                                }
                                _ => false,
                            }
                    })
            }
            Kind::GraphVerticalUnit(horizontal, step) => {
                step.is_finite()
                    && step > 0.
                    && parent(horizontal).is_some_and(|horizontal| {
                        matches!(horizontal.kind, Kind::GraphHorizontalUnit(..))
                            && self.coordinate_geometry_undefined(horizontal.id)
                    })
            }
            Kind::DistanceAxis(origin, distance, 0 | 1) => {
                parent(origin).is_some_and(|o| is_point_kind(&o.kind))
                    && parent(distance).is_some_and(|o| is_coordinate_unit_kind(&o.kind))
                    && self.defining_distance_pixels(distance).is_none()
            }
            Kind::DistanceVerticalUnit(axis, step, false) => {
                step.is_finite()
                    && step > 0.
                    && parent(axis).is_some_and(|o| matches!(o.kind, Kind::DistanceAxis(_, _, 1)))
                    && self.coordinate_geometry_undefined(axis)
            }
            Kind::GraphAxis(origin, unit, 0) => {
                parent(origin).is_some_and(|o| is_point_kind(&o.kind))
                    && parent(unit).is_some_and(|o| match o.kind {
                        Kind::DistanceVerticalUnit(axis, _, false) => {
                            self.objects.get(axis.saturating_sub(1)).is_some_and(
                                |o| matches!(o.kind, Kind::DistanceAxis(a, _, 1) if a == origin),
                            ) && self.coordinate_geometry_undefined(unit)
                        }
                        _ => false,
                    })
            }
            Kind::CoordinateSystem(x, y) => {
                match (parent(x).map(|o| &o.kind), parent(y).map(|o| &o.kind)) {
                    (Some(Kind::DistanceAxis(a, _, 1)), Some(Kind::DistanceAxis(b, _, 0))) if a == b =>
                        self.coordinate_geometry_undefined(x) || self.coordinate_geometry_undefined(y),
                    (Some(Kind::DistanceAxis(origin, _, 1)), Some(Kind::GraphAxis(other, unit, 0)))
                        if origin == other && self.objects.get(unit.saturating_sub(1)).is_some_and(|o|
                            matches!(o.kind, Kind::DistanceVerticalUnit(axis, _, false) if axis == x)) =>
                        self.coordinate_geometry_undefined(x),
                    _ => false,
                }
            }
            _ => false,
        }
    }
    pub(super) fn defining_distance_pixels(&self, id: usize) -> Option<f64> {
        let kind = &self.objects.get(id.checked_sub(1)?)?.kind;
        let value = if kind.is_segment() {
            let (start, end, _) = self.line_points(id)?;
            start.distance(end)
        } else if is_coordinate_unit_kind(kind) {
            // Stored axes use the numeric magnitude as centimeters, including
            // angular and higher dimensional sources; units do not rescale it.
            self.numeric_value(id)? * 37.79527559055118
        } else {
            return None;
        };
        (value.is_finite() && value > 0.).then_some(value)
    }
}
