use super::kind::{is_point_kind, is_segment_kind};
use super::{Document, Kind, LineMode};
use super::transform_model::{ElementType, NumericExpr, NativeTransformFlavor, Transform, TransformDefinition};
use crate::table_records::NumericUnit;

impl Document {
    pub fn add_calculated_angle_rotation(
        &mut self,
        source: usize,
        center: usize,
        angle: usize,
    ) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || self.position(center).is_none()
            || !self
                .objects
                .get(angle.saturating_sub(1))
                .is_some_and(|o| !o.deleted && o.kind.numeric_unit().is_some())
            || self.numeric_value(angle).is_none()
        {
            return Err(
                "Select a shape, rotation center, and a numeric angle"
                    .into(),
            );
        }
        let kind = match self
            .objects
            .get(source.saturating_sub(1))
            .filter(|o| !o.deleted)
            .map(|o| &o.kind)
        {
            Some(kind) if is_point_kind(kind) && self.position(source).is_some() => {
                Kind::CalculatedAngleRotatedPoint(source, center, angle)
            }
            Some(kind) if is_segment_kind(kind) && self.line_points(source).is_some() => {
                Kind::CalculatedAngleRotatedSegment(source, center, angle)
            }
            Some(kind)
                if kind.fixed_transform_straight_mode() == Some(LineMode::Ray)
                    && self.line_points(source).is_some() =>
            {
                Kind::CalculatedAngleRotatedRay(source, center, angle)
            }
            Some(kind)
                if kind.fixed_transform_straight_mode() == Some(LineMode::Line)
                    && self.line_points(source).is_some() =>
            {
                Kind::CalculatedAngleRotatedLine(source, center, angle)
            }
            Some(kind) if (kind.is_arc() && self.arc_points(source, 1).is_some())
                || ((kind.is_circle_interior() || kind.is_sector()) && self.filled_region_points(source).is_some()) => {
                let element = if kind.is_circle_interior() { ElementType::Disk }
                    else if kind.is_sector() { ElementType::Sector } else { ElementType::Arc };
                Kind::Transformed(TransformDefinition::new(source, element,
                    Transform::Rotate { center, angle: NumericExpr::ObjectValue { id: angle, unit: NumericUnit::Degrees } },
                    NativeTransformFlavor::NumericRotation)?)
            }
            Some(kind) if kind.is_circle() && self.circle_points(source).is_some() => {
                Kind::CalculatedAngleRotatedCircle(source, center, angle)
            }
            Some(kind) if kind.is_polygon() && self.polygon_points(source).is_some() => {
                Kind::CalculatedAngleRotatedPolygon(source, center, angle)
            }
            _ => {
                return Err(
                    "Select a point, segment, ray, line, circle, or polygon interior".into(),
                );
            }
        };
        self.checkpoint();
        Ok(self.append_object(kind))
    }

    pub fn add_calculated_ratio_dilation(
        &mut self,
        source: usize,
        center: usize,
        ratio: usize,
    ) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || self.position(center).is_none()
            || !self
                .objects
                .get(ratio.saturating_sub(1))
                .is_some_and(|o| !o.deleted && o.kind.numeric_unit().is_some())
            || self.numeric_value(ratio).is_none()
        {
            return Err("Select a shape, dilation center, and a numeric ratio".into());
        }
        let kind = match self
            .objects
            .get(source.saturating_sub(1))
            .filter(|o| !o.deleted)
            .map(|o| &o.kind)
        {
            Some(kind) if is_point_kind(kind) && self.position(source).is_some() => {
                Kind::CalculatedRatioScaledPoint(source, center, ratio)
            }
            Some(kind) if is_segment_kind(kind) && self.line_points(source).is_some() => {
                Kind::CalculatedRatioScaledSegment(source, center, ratio)
            }
            Some(kind)
                if kind.fixed_transform_straight_mode() == Some(LineMode::Ray)
                    && self.line_points(source).is_some() =>
            {
                Kind::CalculatedRatioScaledRay(source, center, ratio)
            }
            Some(kind)
                if kind.fixed_transform_straight_mode() == Some(LineMode::Line)
                    && self.line_points(source).is_some() =>
            {
                Kind::CalculatedRatioScaledLine(source, center, ratio)
            }
            Some(kind) if (kind.is_arc() && self.arc_points(source, 1).is_some())
                || ((kind.is_circle_interior() || kind.is_sector()) && self.filled_region_points(source).is_some()) => {
                let element = if kind.is_circle_interior() { ElementType::Disk }
                    else if kind.is_sector() { ElementType::Sector } else { ElementType::Arc };
                Kind::Transformed(TransformDefinition::new(source, element,
                    Transform::Dilate { center, ratio: NumericExpr::ObjectValue { id: ratio, unit: NumericUnit::Scalar } },
                    NativeTransformFlavor::NumericDilation)?)
            }
            Some(kind) if kind.is_circle() && self.circle_points(source).is_some() => {
                Kind::CalculatedRatioScaledCircle(source, center, ratio)
            }
            Some(kind) if kind.is_polygon() && self.polygon_points(source).is_some() => {
                Kind::CalculatedRatioScaledPolygon(source, center, ratio)
            }
            _ => {
                return Err(
                    "Select a point, segment, ray, line, circle, or polygon interior".into(),
                );
            }
        };
        self.checkpoint();
        Ok(self.append_object(kind))
    }
}
