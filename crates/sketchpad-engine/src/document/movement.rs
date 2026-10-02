use super::kind::is_point_kind;
use super::{Document, Kind, Point};
use crate::graph::GridForm;
use crate::table_records::NumericUnit;

/// The live value consumed by movement controls, independent of its construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovementValue {
    Point(Point),
    Numeric { value: f64, unit: NumericUnit },
}

impl MovementValue {
    pub fn interpolate(self, destination: Self, fraction: f64) -> Option<Self> {
        if !fraction.is_finite() {
            return None;
        }
        let fraction = fraction.clamp(0., 1.);
        self.distance(destination)?;
        if fraction == 0. {
            return Some(self);
        }
        if fraction == 1. {
            return Some(match (self, destination) {
                (Self::Numeric { unit, .. }, Self::Numeric { value, .. }) => {
                    Self::Numeric { value, unit }
                }
                _ => destination,
            });
        }
        match (self, destination) {
            (Self::Point(a), Self::Point(b)) => Some(Self::Point(Point {
                x: a.x + (b.x - a.x) * fraction,
                y: a.y + (b.y - a.y) * fraction,
            })),
            (Self::Numeric { value: a, unit }, Self::Numeric { value: b, .. }) => {
                Some(Self::Numeric {
                    value: a + (b - a) * fraction,
                    unit,
                })
            }
            _ => None,
        }
    }

    pub fn distance(self, destination: Self) -> Option<f64> {
        match (self, destination) {
            (Self::Point(a), Self::Point(b)) => Some(a.distance(b)),
            (Self::Numeric { value: a, .. }, Self::Numeric { value: b, .. }) => Some((a - b).abs()),
            _ => None,
        }
    }
}

impl Document {
    /// Validates stored roles without requiring currently defined values or an
    /// editable active page. Persistence uses this rather than runtime guards.
    pub(super) fn movement_pair_valid(&self, source: usize, destination: usize) -> bool {
        if source == destination {
            return false;
        }
        let Some(moving) = source.checked_sub(1).and_then(|at| self.objects.get(at)) else {
            return false;
        };
        let Some(target) = destination
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
        else {
            return false;
        };
        if moving.deleted || target.deleted || moving.page != target.page {
            return false;
        }
        for object in [moving, target] {
            if object.kind.dependencies().into_iter().any(|parent| {
                parent >= object.id
                    || parent
                        .checked_sub(1)
                        .and_then(|at| self.objects.get(at))
                        .is_none_or(|parent| parent.deleted || parent.page != object.page)
            }) {
                return false;
            }
        }
        match moving.kind {
            Kind::Parameter(_) => target.kind.numeric_unit().is_some(),
            Kind::Free(_)
            | Kind::PointOnSegment(..)
            | Kind::PointOnRay(..)
            | Kind::PointOnLine(..)
            | Kind::PointOnCircle(..)
            | Kind::PointOnArc(..)
            | Kind::PointOnCurve(..)
            | Kind::PointOnSector(..)
            | Kind::PointOnCircularSegment(..)
            | Kind::PointOnPolygon(..)
            | Kind::GraphHorizontalUnit(..) => is_point_kind(&target.kind),
            Kind::GraphVerticalUnit(..) => {
                self.grid_form.get(moving.page) == Some(&GridForm::Rectangular)
                    && is_point_kind(&target.kind)
            }
            Kind::DistanceVerticalUnit(_, _, independent) => {
                independent && is_point_kind(&target.kind)
            }
            _ => false,
        }
    }

    /// Whether this object exposes an editable movement control. Numeric controls
    /// also retain the parameter editor's page and document safety checks.
    pub fn can_move_movement_source(&self, id: usize) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|at| self.objects.get(at)) else {
            return false;
        };
        if object.deleted {
            return false;
        }
        let supported = match object.kind {
            Kind::Free(_)
            | Kind::PointOnSegment(..)
            | Kind::PointOnRay(..)
            | Kind::PointOnLine(..)
            | Kind::PointOnCircle(..)
            | Kind::PointOnArc(..)
            | Kind::PointOnCurve(..)
            | Kind::PointOnSector(..)
            | Kind::PointOnCircularSegment(..)
            | Kind::PointOnPolygon(..)
            | Kind::GraphHorizontalUnit(..) => true,
            Kind::Parameter(_) => {
                object.page == self.active_page
                    && !(self.page_count == 1 && (self.partial_view || !self.safe_structure))
                    && !(self.page_count > 1
                        && (self.image_read_only
                            || self.source_count != self.objects.len()
                            || id > self.source_count
                            || self.objects.iter().any(|object| {
                                object.deleted || matches!(object.kind, Kind::Unsupported(_))
                            })))
            }
            Kind::GraphVerticalUnit(..) => {
                self.grid_form.get(object.page) == Some(&GridForm::Rectangular)
            }
            Kind::DistanceVerticalUnit(_, _, independent) => independent,
            _ => false,
        };
        supported && self.movement_target_value(id).is_some()
    }

    pub fn movement_source_value(&self, id: usize) -> Option<MovementValue> {
        self.can_move_movement_source(id)
            .then(|| self.movement_target_value(id))
            .flatten()
    }

    /// Reads the evaluated target, including derived points and measurements.
    pub fn movement_target_value(&self, id: usize) -> Option<MovementValue> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        if let Some(point) = self.position(id) {
            return (point.x.is_finite() && point.y.is_finite())
                .then_some(MovementValue::Point(point));
        }
        let unit = object.kind.numeric_unit()?;
        let value = self.numeric_value(id)?;
        value
            .is_finite()
            .then_some(MovementValue::Numeric { value, unit })
    }

    pub fn movement_pair_supported(&self, source: usize, destination: usize) -> bool {
        if source == destination {
            return false;
        }
        let Some(moving) = source.checked_sub(1).and_then(|at| self.objects.get(at)) else {
            return false;
        };
        let Some(target) = destination
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
        else {
            return false;
        };
        moving.page == target.page
            && self
                .movement_source_value(source)
                .zip(self.movement_target_value(destination))
                .is_some_and(|(a, b)| a.distance(b).is_some())
    }

    /// Applies one frame without creating a history checkpoint. The caller
    /// checkpoints the complete action once before applying its controls.
    pub fn set_movement_value(&mut self, id: usize, value: MovementValue) -> bool {
        if !self.can_move_movement_source(id) {
            return false;
        }
        match value {
            MovementValue::Numeric { value, .. } => {
                // Native copies the target magnitude and retains the source's
                // own units, even when the target is a different quantity.
                if !value.is_finite() || !matches!(self.objects[id - 1].kind, Kind::Parameter(_)) {
                    return false;
                }
                self.update_parameter_value(id, value, false).is_ok()
            }
            MovementValue::Point(point) => {
                if !point.x.is_finite() || !point.y.is_finite() {
                    return false;
                }
                if let Kind::GraphHorizontalUnit(origin, _) = self.objects[id - 1].kind {
                    let Some(origin_point) = self.position(origin) else {
                        return false;
                    };
                    let step = point.x - origin_point.x;
                    if !step.is_finite() || step <= 0. {
                        return false;
                    }
                    self.objects.edit(id - 1).kind = Kind::GraphHorizontalUnit(origin, step);
                    self.last_moved_object = Some(id);
                    self.dirty = true;
                    true
                } else {
                    self.move_point(id, point)
                }
            }
        }
    }
}
