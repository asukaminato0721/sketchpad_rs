use super::{Editor, MarkedAngle, TranslationMode};
use sketchpad_rs::document::Kind;

impl Editor {
    pub(super) fn translate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Translation on this document cannot be saved safely".into();
            return;
        }
        if self.selection_order.len() != 1 {
            self.status = "Select one point, segment, or circle to translate".into();
            return;
        }
        let source = self.selection_order[0];
        let source_is_segment = matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|object| &object.kind),
            Some(Kind::Segment(_, _))
        );
        let source_is_circle = matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|object| &object.kind),
            Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _))
        );
        let source_is_polygon = matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|object| &object.kind),
            Some(kind) if matches!(kind, Kind::Polygon(_))
                || (self.translation_mode == TranslationMode::MarkedVector && kind.is_polygon())
        );
        if self.doc.position(source).is_none()
            && !source_is_segment
            && !source_is_circle
            && !(source_is_polygon
                && matches!(
                    self.translation_mode,
                    TranslationMode::FixedPolar
                        | TranslationMode::FixedRectangular
                        | TranslationMode::MarkedVector
                        | TranslationMode::MarkedDistance
                        | TranslationMode::RectangularMarked
                        | TranslationMode::RectangularHorizontalFixed
                        | TranslationMode::RectangularVerticalFixed
                ))
        {
            self.status = "Select one point, segment, or circle to translate".into();
            return;
        }
        let kind = match self.translation_mode {
            TranslationMode::FixedPolar => {
                if !self.translation_angle.is_finite() || !self.translation_distance.is_finite() {
                    self.status = "Translation values must be finite".into();
                    return;
                }
                if source_is_segment {
                    Kind::PolarTranslatedSegment(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                } else if source_is_circle {
                    Kind::PolarTranslatedCircle(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                } else if source_is_polygon {
                    Kind::PolarTranslatedPolygon(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                } else {
                    Kind::PolarTranslatedPoint(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                }
            }
            TranslationMode::FixedRectangular => {
                if !self.translation_horizontal.is_finite()
                    || !self.translation_vertical.is_finite()
                {
                    self.status = "Translation values must be finite".into();
                    return;
                }
                if source_is_segment {
                    Kind::FixedRectangularTranslatedSegment(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                } else if source_is_circle {
                    Kind::FixedRectangularTranslatedCircle(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                } else if source_is_polygon {
                    Kind::FixedRectangularTranslatedPolygon(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                } else {
                    Kind::FixedRectangularTranslatedPoint(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                }
            }
            TranslationMode::MarkedDistance => {
                let Some(distance) = self.marked_distance else {
                    self.status = "Mark a distance value first".into();
                    return;
                };
                if !self.translation_angle.is_finite()
                    || self
                        .doc
                        .objects
                        .get(distance.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.distance_value_pixels(distance).is_none()
                {
                    self.status = "Marked distance is unavailable".into();
                    return;
                }
                if source_is_segment {
                    Kind::MarkedDistanceTranslatedSegment(source, distance, self.translation_angle)
                } else if source_is_circle {
                    Kind::MarkedDistanceTranslatedCircle(source, distance, self.translation_angle)
                } else if source_is_polygon {
                    Kind::MarkedDistanceTranslatedPolygon(source, distance, self.translation_angle)
                } else {
                    Kind::MarkedDistanceTranslatedPoint(source, distance, self.translation_angle)
                }
            }
            TranslationMode::RectangularMarked => {
                let Some((horizontal, vertical)) = self.marked_rectangular_distances else {
                    self.status = "Mark two distance values first".into();
                    return;
                };
                if [horizontal, vertical].iter().any(|&id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                        || self.doc.distance_value_pixels(id).is_none()
                }) {
                    self.status = "Marked distances are unavailable".into();
                    return;
                }
                if source_is_segment {
                    Kind::RectangularMarkedTranslatedSegment(source, horizontal, vertical)
                } else if source_is_circle {
                    Kind::RectangularMarkedTranslatedCircle(source, horizontal, vertical)
                } else if source_is_polygon {
                    Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical)
                } else {
                    Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
                }
            }
            TranslationMode::RectangularHorizontalFixed
            | TranslationMode::RectangularVerticalFixed => {
                let Some((horizontal, vertical)) = self.marked_rectangular_distances else {
                    self.status = "Mark two distance values first".into();
                    return;
                };
                let fixed_horizontal =
                    self.translation_mode == TranslationMode::RectangularHorizontalFixed;
                let distance = if fixed_horizontal {
                    vertical
                } else {
                    horizontal
                };
                let fixed_cm = if fixed_horizontal {
                    self.translation_horizontal
                } else {
                    self.translation_vertical
                };
                if !fixed_cm.is_finite()
                    || self
                        .doc
                        .objects
                        .get(distance.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.distance_value_pixels(distance).is_none()
                {
                    self.status = "Mixed rectangular translation values are unavailable".into();
                    return;
                }
                if source_is_segment {
                    Kind::MixedRectangularTranslatedSegment(
                        source,
                        distance,
                        fixed_cm,
                        fixed_horizontal,
                    )
                } else if source_is_circle {
                    Kind::MixedRectangularTranslatedCircle(
                        source,
                        distance,
                        fixed_cm,
                        fixed_horizontal,
                    )
                } else if source_is_polygon {
                    Kind::MixedRectangularTranslatedPolygon(
                        source,
                        distance,
                        fixed_cm,
                        fixed_horizontal,
                    )
                } else {
                    Kind::MixedRectangularTranslatedPoint(
                        source,
                        distance,
                        fixed_cm,
                        fixed_horizontal,
                    )
                }
            }
            TranslationMode::MarkedVector => {
                let Some((start, end)) = self.marked_vector else {
                    self.status = "Mark two points as a vector first".into();
                    return;
                };
                if [start, end].iter().any(|&id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                        || self.doc.position(id).is_none()
                }) {
                    self.status = "Marked vector is unavailable".into();
                    return;
                }
                if source_is_segment {
                    Kind::VectorTranslatedSegment(source, start, end)
                } else if source_is_circle {
                    Kind::VectorTranslatedCircle(source, start, end)
                } else if source_is_polygon {
                    Kind::VectorTranslatedPolygon(source, start, end)
                } else {
                    Kind::VectorTranslatedPoint(source, start, end)
                }
            }
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    pub(super) fn reflect_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Reflection on this document cannot be saved safely".into();
            return;
        }
        let (source, mirror) = match self.selection_order.as_slice() {
            [source, mirror] => (*source, *mirror),
            [source] => match self.marked_mirror {
                Some(mirror) => (*source, mirror),
                None => {
                    self.status =
                        "Select a point, segment, circle, or interior and mark a mirror line"
                            .into();
                    return;
                }
            },
            _ => {
                self.status =
                    "Select a point, segment, circle, or interior, then a mirror line".into();
                return;
            }
        };
        if self
            .doc
            .objects
            .get(mirror - 1)
            .is_none_or(|o| o.deleted || o.page != self.doc.active_page)
        {
            self.status = "Marked mirror line is unavailable".into();
            return;
        }
        if self.doc.line_points(mirror).is_none() {
            self.status = "Select a point, segment, circle, or interior, then a mirror line".into();
            return;
        }
        let kind = if self.doc.position(source).is_some() {
            Kind::ReflectedPoint(source, mirror)
        } else if self.doc.circle_points(source).is_some() {
            Kind::ReflectedCircle(source, mirror)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(kind) if kind.is_segment()
        ) {
            Kind::ReflectedSegment(source, mirror)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Polygon(_))
        ) {
            Kind::ReflectedPolygon(source, mirror)
        } else {
            self.status = "Select a point, segment, circle, or interior, then a mirror line".into();
            return;
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    pub(super) fn mark_center_selected(&mut self) {
        let [id] = self.selection_order.as_slice() else {
            self.status = "Select one point to mark as center".into();
            return;
        };
        if self.doc.position(*id).is_none() {
            self.status = "Select one point to mark as center".into();
            return;
        }
        self.marked_center = Some(*id);
        self.status = format!("Marked point {id} as center");
    }
    pub(super) fn mark_mirror_selected(&mut self) {
        let [id] = self.selection_order.as_slice() else {
            self.status = "Select one straight object to mark as mirror".into();
            return;
        };
        if self.doc.line_points(*id).is_none() {
            self.status = "Select one straight object to mark as mirror".into();
            return;
        }
        self.marked_mirror = Some(*id);
        self.status = format!("Marked object {id} as mirror");
    }
    pub(super) fn mark_vector_selected(&mut self) {
        let points = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.position(id).is_some())
            .collect::<Vec<_>>();
        if points.len() < 2 {
            self.status = "Select two points to mark a vector".into();
            return;
        }
        let start = points[points.len() - 2];
        let end = points[points.len() - 1];
        self.marked_vector = Some((start, end));
        self.translation_mode = TranslationMode::MarkedVector;
        self.doc.selected.remove(&start);
        self.doc.selected.remove(&end);
        self.selection_order.retain(|&id| id != start && id != end);
        self.status = format!("Marked vector from point {start} to point {end}");
    }
    pub(super) fn mark_distance_selected(&mut self) {
        let values = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.distance_value_pixels(id).is_some())
            .collect::<Vec<_>>();
        match values.as_slice() {
            [id] => {
                self.marked_distance = Some(*id);
                self.translation_mode = TranslationMode::MarkedDistance;
                self.doc.selected.remove(id);
                self.selection_order.retain(|selected| selected != id);
                self.status = format!("Marked distance value {id}");
            }
            [horizontal, vertical] => {
                self.marked_rectangular_distances = Some((*horizontal, *vertical));
                self.translation_mode = TranslationMode::RectangularMarked;
                for id in [horizontal, vertical] {
                    self.doc.selected.remove(id);
                }
                self.selection_order
                    .retain(|id| id != horizontal && id != vertical);
                self.status = format!(
                    "Marked horizontal distance {horizontal} and vertical distance {vertical}"
                );
            }
            _ => self.status = "Select one or two distance measurements".into(),
        }
    }
    pub(super) fn mark_angle_selected(&mut self) {
        if let [id] = self.selection_order.as_slice()
            && self.doc.objects[*id - 1].kind.numeric_unit()
                == Some(sketchpad_rs::table_records::NumericUnit::Degrees)
            && self.doc.numeric_value(*id).is_some()
        {
            let id = *id;
            self.marked_angle = Some(MarkedAngle::Value(id));
            self.rotation_use_marked = true;
            self.doc.selected.remove(&id);
            self.selection_order.clear();
            self.status = format!("Marked angle value {id}");
            return;
        }
        let points = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.position(id).is_some())
            .collect::<Vec<_>>();
        if points.len() < 3 {
            self.status =
                "Select three points or a degree-valued measurement or calculation to mark an angle".into();
            return;
        }
        let angle = [
            points[points.len() - 3],
            points[points.len() - 2],
            points[points.len() - 1],
        ];
        if self
            .doc
            .angle_for_points(angle[0], angle[1], angle[2])
            .is_none()
        {
            self.status = "Marked angle needs distinct sides".into();
            return;
        }
        self.marked_angle = Some(MarkedAngle::Points(angle));
        self.rotation_use_marked = true;
        for &id in &angle {
            self.doc.selected.remove(&id);
        }
        self.selection_order.retain(|id| !angle.contains(id));
        self.status = format!("Marked angle {}–{}–{}", angle[0], angle[1], angle[2]);
    }
    pub(super) fn mark_ratio_selected(&mut self) {
        if let [id] = self.selection_order.as_slice()
            && self.doc.objects[*id - 1].kind.numeric_unit()
                == Some(sketchpad_rs::table_records::NumericUnit::Scalar)
            && self.doc.numeric_value(*id).is_some()
        {
            let id = *id;
            self.marked_ratio_value = Some(id);
            self.marked_ratio = None;
            self.dilation_use_marked = true;
            self.doc.selected.remove(&id);
            self.selection_order.clear();
            self.status = format!("Marked ratio value {id}");
            return;
        }
        let points = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.position(id).is_some())
            .collect::<Vec<_>>();
        if points.len() < 3 {
            self.status =
                "Select three collinear points or one scalar value to mark a ratio".into();
            return;
        }
        let ratio = [
            points[points.len() - 3],
            points[points.len() - 2],
            points[points.len() - 1],
        ];
        let [Some(origin), Some(base), Some(target)] = ratio.map(|id| self.doc.position(id)) else {
            self.status = "Selected ratio points are unavailable".into();
            return;
        };
        let (bx, by) = (base.x - origin.x, base.y - origin.y);
        let (cx, cy) = (target.x - origin.x, target.y - origin.y);
        let cross = bx * cy - by * cx;
        let tolerance = 1e-8 * bx.hypot(by) * cx.hypot(cy);
        if self
            .doc
            .ratio_for_points(ratio[0], ratio[1], ratio[2])
            .is_none()
            || cross.abs() > tolerance
        {
            self.status = "Marked ratio needs a nonzero base and collinear points".into();
            return;
        }
        self.marked_ratio = Some(ratio);
        self.marked_ratio_value = None;
        self.dilation_use_marked = true;
        for &id in &ratio {
            self.doc.selected.remove(&id);
        }
        self.selection_order.retain(|id| !ratio.contains(id));
        self.status = format!("Marked ratio {}–{}–{}", ratio[0], ratio[1], ratio[2]);
    }
    pub(super) fn dilate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Dilation on this document cannot be saved safely".into();
            return;
        }
        if !self.dilation_use_marked
            && (!self.dilation_factor.is_finite() || self.dilation_factor == 0.)
        {
            self.status =
                "Select a point, segment, or circle and a center point; use a nonzero ratio".into();
            return;
        }
        let (source, center) = match self.selection_order.as_slice() {
            [source, center] => (*source, *center),
            [source] => match self.marked_center {
                Some(center) => (*source, center),
                None => {
                    self.status = "Select a source and mark a center point".into();
                    return;
                }
            },
            _ => {
                self.status = "Select a point, segment, or circle, then a center point".into();
                return;
            }
        };
        if self
            .doc
            .objects
            .get(center - 1)
            .is_none_or(|o| o.deleted || o.page != self.doc.active_page)
        {
            self.status = "Marked center point is unavailable".into();
            return;
        }
        if self.doc.position(center).is_none() {
            self.status = "Select a point, segment, or circle, then a center point".into();
            return;
        }
        if self.dilation_use_marked
            && let Some(ratio) = self.marked_ratio_value
        {
            match self
                .doc
                .add_calculated_ratio_dilation(source, center, ratio)
            {
                Ok(id) => {
                    self.select_only(id);
                    self.status = format!("Dilated object {id}");
                }
                Err(error) => self.status = error,
            }
            return;
        }
        let kind = if self.dilation_use_marked {
            let Some([origin, base, target]) = self.marked_ratio else {
                self.status = "Mark a ratio first".into();
                return;
            };
            if [origin, base, target].iter().any(|&id| {
                self.doc
                    .objects
                    .get(id.saturating_sub(1))
                    .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.position(id).is_none()
            }) || self.doc.ratio_for_points(origin, base, target).is_none()
            {
                self.status = "Marked ratio is unavailable".into();
                return;
            }
            if self.doc.position(source).is_some() {
                Kind::RatioScaledPoint(source, center, origin, base, target)
            } else if self.doc.circle_points(source).is_some() {
                Kind::RatioScaledCircle(source, center, origin, base, target)
            } else if matches!(
                self.doc
                    .objects
                    .get(source.saturating_sub(1))
                    .map(|o| &o.kind),
                Some(Kind::Segment(_, _))
            ) {
                Kind::RatioScaledSegment(source, center, origin, base, target)
            } else if matches!(
                self.doc
                    .objects
                    .get(source.saturating_sub(1))
                    .map(|o| &o.kind),
                Some(Kind::Polygon(_))
            ) {
                Kind::RatioScaledPolygon(source, center, origin, base, target)
            } else {
                self.status =
                    "Select a point, segment, or circle to dilate by the marked ratio".into();
                return;
            }
        } else if self.doc.position(source).is_some() {
            Kind::ScaledPoint(source, center, self.dilation_factor)
        } else if self.doc.circle_points(source).is_some() {
            Kind::ScaledCircle(source, center, self.dilation_factor)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(kind) if kind.is_segment()
        ) {
            Kind::ScaledSegment(source, center, self.dilation_factor)
        } else if matches!(
            self.doc.objects.get(source.saturating_sub(1)).map(|o| &o.kind),
            Some(Kind::Ray(..))
        ) {
            Kind::ScaledRay(source, center, self.dilation_factor)
        } else if matches!(
            self.doc.objects.get(source.saturating_sub(1)).map(|o| &o.kind),
            Some(Kind::Line(..))
        ) {
            Kind::ScaledLine(source, center, self.dilation_factor)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Polygon(_))
        ) {
            Kind::ScaledPolygon(source, center, self.dilation_factor)
        } else {
            self.status = "Select a point, segment, or circle, then a center point".into();
            return;
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    pub(super) fn rotate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Rotation on this document cannot be saved safely".into();
            return;
        }
        if !self.rotation_use_marked && !self.rotation_angle.is_finite() {
            self.status = "Select a source and center point; use a finite angle".into();
            return;
        }
        let (source, center) = match self.selection_order.as_slice() {
            [source, center] => (*source, *center),
            [source] => match self.marked_center {
                Some(center) => (*source, center),
                None => {
                    self.status = "Select a source and mark a center point".into();
                    return;
                }
            },
            _ => {
                self.status = "Select a point, segment, or circle, then a center point".into();
                return;
            }
        };
        if self
            .doc
            .objects
            .get(center - 1)
            .is_none_or(|o| o.deleted || o.page != self.doc.active_page)
        {
            self.status = "Marked center point is unavailable".into();
            return;
        }
        let source_kind = self
            .doc
            .objects
            .get(source.saturating_sub(1))
            .map(|o| &o.kind);
        let source_point = self.doc.position(source).is_some();
        let source_segment = source_kind.is_some_and(|kind| matches!(kind, Kind::Segment(_, _))
            || ((!self.rotation_use_marked || matches!(self.marked_angle, Some(MarkedAngle::Value(_))))
                && kind.is_segment()))
            && self.doc.line_points(source).is_some();
        let source_circle = source_kind.is_some_and(|kind| matches!(kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _))
            || ((!self.rotation_use_marked || matches!(self.marked_angle, Some(MarkedAngle::Value(_))))
                && kind.is_circle()))
            && self.doc.circle_points(source).is_some();
        let source_ray = !self.rotation_use_marked && matches!(source_kind, Some(Kind::Ray(..)))
            && self.doc.line_points(source).is_some();
        let source_line = !self.rotation_use_marked && matches!(source_kind, Some(Kind::Line(..)))
            && self.doc.line_points(source).is_some();
        let source_polygon = source_kind.is_some_and(|kind| matches!(kind, Kind::Polygon(_))
            || (self.rotation_use_marked && matches!(self.marked_angle, Some(MarkedAngle::Value(_)))
                && kind.is_polygon()))
            && self.doc.polygon_points(source).is_some();
        if !(source_point || source_segment || source_circle || source_polygon || source_ray || source_line)
            || self.doc.position(center).is_none()
        {
            self.status = "Select a point, segment, ray, line, circle, or interior and a center point".into();
            return;
        }
        if self.rotation_use_marked
            && let Some(MarkedAngle::Value(angle)) = self.marked_angle
        {
            match self
                .doc
                .add_calculated_angle_rotation(source, center, angle)
            {
                Ok(id) => {
                    self.select_only(id);
                    self.status = format!("Rotated object {id}");
                }
                Err(error) => self.status = error,
            }
            return;
        }
        let kind = if self.rotation_use_marked {
            let Some(MarkedAngle::Points([first, vertex, last])) = self.marked_angle else {
                self.status = "Mark an angle first".into();
                return;
            };
            if [first, vertex, last].iter().any(|&id| {
                self.doc
                    .objects
                    .get(id.saturating_sub(1))
                    .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.position(id).is_none()
            }) || self.doc.angle_for_points(first, vertex, last).is_none()
            {
                self.status = "Marked angle is unavailable".into();
                return;
            }
            if source_point {
                Kind::AngleRotatedPoint(source, center, first, vertex, last)
            } else if source_segment {
                Kind::AngleRotatedSegment(source, center, first, vertex, last)
            } else if source_polygon {
                Kind::AngleRotatedPolygon(source, center, first, vertex, last)
            } else {
                Kind::AngleRotatedCircle(source, center, first, vertex, last)
            }
        } else {
            if source_point {
                Kind::RotatedPoint(source, center, self.rotation_angle)
            } else if source_segment {
                Kind::RotatedSegment(source, center, self.rotation_angle)
            } else if source_ray {
                Kind::RotatedRay(source, center, self.rotation_angle)
            } else if source_line {
                Kind::RotatedLine(source, center, self.rotation_angle)
            } else if source_polygon {
                Kind::RotatedPolygon(source, center, self.rotation_angle)
            } else {
                Kind::RotatedCircle(source, center, self.rotation_angle)
            }
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
}
