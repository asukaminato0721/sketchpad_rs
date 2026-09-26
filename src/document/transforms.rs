use super::kind::{is_point_kind, is_segment_kind};
use super::{Document, Kind};
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
                .is_some_and(|o| !o.deleted && o.kind.numeric_unit() == Some(NumericUnit::Degrees))
            || self.numeric_value(angle).is_none()
        {
            return Err(
                "Select a shape, rotation center, and a degree-valued measurement or calculation"
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
            Some(kind) if kind.is_circle() && self.circle_points(source).is_some() => {
                Kind::CalculatedAngleRotatedCircle(source, center, angle)
            }
            Some(kind) if kind.is_polygon() && self.polygon_points(source).is_some() => {
                Kind::CalculatedAngleRotatedPolygon(source, center, angle)
            }
            _ => return Err("Select a point, segment, circle, or polygon interior".into()),
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
                .is_some_and(|o| !o.deleted && o.kind.numeric_unit() == Some(NumericUnit::Scalar))
            || self.numeric_value(ratio).is_none()
        {
            return Err("Select a shape, dilation center, and a scalar value".into());
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
            Some(kind) if kind.is_circle() && self.circle_points(source).is_some() => {
                Kind::CalculatedRatioScaledCircle(source, center, ratio)
            }
            Some(kind) if kind.is_polygon() && self.polygon_points(source).is_some() => {
                Kind::CalculatedRatioScaledPolygon(source, center, ratio)
            }
            _ => return Err("Select a point, segment, circle, or polygon interior".into()),
        };
        self.checkpoint();
        Ok(self.append_object(kind))
    }
}
