use super::calculations::calculation_source_ids;
use super::iteration::{
    supported_branch_polygon_mapping, supported_mapped_point_iteration, supported_point_iteration_mapping,
    supported_point_parameter_iteration,
};
use super::kind::{
    is_arc_kind, expected_locus_refs, expected_parameter_locus_refs, is_point_kind, is_segment_kind, is_straight_kind, literal_plot_path_accepts, value_plot_path_accepts, polygon_vertex_count, straight_kind_mode,
};
use super::records::records;
use super::{Document, IterationDepth, Kind, LineMode, Object};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::image_records::ImageFile;
use crate::table_records::{
    CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup, NumericUnit,
    ParameterCalculationGroup, ParameterGroup, TableGroup,
};
use crate::value_equation_records::ValueEquationKind;
use crate::{plot_points_records, text_object};
use std::collections::BTreeSet;

impl Document {
    pub fn can_save(&self) -> Result<(), String> {
        // An untouched source document can be copied byte for byte even when
        // its geometry cannot yet be evaluated by this port. Revalidate once
        // anything has changed before serializing an edited document.
        if !self.dirty {
            return Ok(());
        }
        if self.page_count
            > records(&self.original)?
                .iter()
                .filter(|record| record.kind == 0x03e8)
                .count()
        {
            self.encode_added_pages()?;
            return Ok(());
        }
        if self.page_count > 1 {
            if self.objects.iter().any(|object| object.deleted) {
                self.encode_multi_page_deletions()?;
            } else if self.has_pending_multi_page_additions() {
                self.encode_multi_page_additions()?;
            } else {
                self.encode_multi_page_state()?;
            }
            return Ok(());
        }
        if self.image_read_only && self.dirty {
            return Err("Cropped or multi-resource picture content is read-only".into());
        }
        if self.snap_points.len() != self.page_count
            || self.grid_dotted.len() != self.page_count
            || self.grid_form.len() != self.page_count
        {
            return Err("Invalid page grid state".into());
        }
        if self.objects.len() < self.source_count {
            return Err("Removing saved objects is not yet supported by the GSP writer".into());
        }
        if self.objects.iter().any(|o| o.deleted) && (!self.safe_structure || self.page_count != 1)
        {
            return Err("Deleting objects in this document cannot be saved safely".into());
        }
        let first_deleted = self.objects.iter().position(|object| object.deleted);
        let last_plot_related = self.objects.iter().rposition(|o| {
            !o.deleted
                && matches!(
                    o.kind,
                    Kind::FunctionDefinition(_)
                        | Kind::DerivativeFunction(_, _)
                        | Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::ParametricPlot(_, _, _, _, _, _, _)
                        | Kind::PathLiteralPoint(_, _)
                        | Kind::PathValuePoint(_, _)
                        | Kind::PlottedPoint(_, _, _, _)
                        | Kind::NumericCoordinatePoint(_, _, _)
                        | Kind::ParameterLocus(_, _, _, _)
                )
        });
        let only_independent_objects_deleted = self
            .objects
            .iter()
            .filter(|object| object.deleted)
            .all(|object| {
                matches!(object.kind, Kind::Free(_) | Kind::PlainText { .. })
                    && object.label.is_none()
            });
        if first_deleted.is_some_and(|first| last_plot_related.is_some_and(|last| first <= last))
            && self.objects[first_deleted.unwrap()..]
                .iter()
                .any(|object| !object.deleted)
            && !only_independent_objects_deleted
        {
            return Err(
                "Deleting an object before a surviving plot or function is not supported".into(),
            );
        }
        if self
            .objects
            .iter()
            .any(|o| o.deleted && matches!(o.kind, Kind::Image { .. }))
        {
            return Err("Removing a picture resource is not yet supported".into());
        }
        if self
            .objects
            .iter()
            .filter(|o| matches!(o.kind, Kind::Image { .. }))
            .count()
            > 1
        {
            return Err("Only one independent picture can be written".into());
        }
        for (i, o) in self.objects.iter().enumerate() {
            if o.id != i + 1 {
                return Err("Invalid object ID".into());
            }
            if o.deleted {
                continue;
            }
            if let Kind::Free(point) = o.kind
                && (!point.x.is_finite() || !point.y.is_finite())
            {
                return Err(format!("Non-finite coordinates in object {}", o.id));
            }
            if let Kind::PlainText { text, rect } = &o.kind {
                if let Some(template) = &o.rich_text_override {
                    template.encode_group(text, *rect)?;
                } else {
                    text_object::encode_plain_text_group(text, *rect)?;
                }
            }
            match &o.kind {
                Kind::CalculatedRatioScaledPoint(source, center, ratio)
                | Kind::CalculatedRatioScaledSegment(source, center, ratio)
                | Kind::CalculatedRatioScaledCircle(source, center, ratio)
                | Kind::CalculatedRatioScaledPolygon(source, center, ratio) => {
                    let source_valid = self.objects.get(source.saturating_sub(1)).is_some_and(|p| !p.deleted && match o.kind {
                        Kind::CalculatedRatioScaledPoint(..) => is_point_kind(&p.kind),
                        Kind::CalculatedRatioScaledSegment(..) => is_segment_kind(&p.kind),
                        Kind::CalculatedRatioScaledCircle(..) => p.kind.is_circle(),
                        _ => p.kind.is_polygon() && polygon_vertex_count(&self.objects, *source).is_some(),
                    });
                    if [*source, *center, *ratio].iter().any(|id| *id == 0 || *id >= o.id) || !source_valid
                        || !self.objects.get(center.saturating_sub(1)).is_some_and(|p| !p.deleted && is_point_kind(&p.kind))
                        || !self.objects.get(ratio.saturating_sub(1)).is_some_and(|p| !p.deleted && p.kind.numeric_unit() == Some(NumericUnit::Scalar)) {
                        return Err("Invalid numeric-ratio dilation parents".into());
                    }
                    if i >= self.source_count && (self.unitless_numeric_value(*ratio).is_none() || self.position(*center).is_none()) {
                        return Err("Undefined numeric-ratio dilation".into());
                    }
                }
                Kind::CalculatedAngleRotatedPoint(source, center, angle)
                | Kind::CalculatedAngleRotatedSegment(source, center, angle)
                | Kind::CalculatedAngleRotatedCircle(source, center, angle)
                | Kind::CalculatedAngleRotatedPolygon(source, center, angle) => {
                    let source_valid = self.objects.get(source.saturating_sub(1)).is_some_and(|p| !p.deleted && match o.kind {
                        Kind::CalculatedAngleRotatedPoint(..) => is_point_kind(&p.kind),
                        Kind::CalculatedAngleRotatedSegment(..) => is_segment_kind(&p.kind),
                        Kind::CalculatedAngleRotatedCircle(..) => p.kind.is_circle(),
                        _ => p.kind.is_polygon() && polygon_vertex_count(&self.objects, *source).is_some(),
                    });
                    if [*source, *center, *angle].iter().any(|id| *id == 0 || *id >= o.id) || !source_valid
                        || !self.objects.get(center.saturating_sub(1)).is_some_and(|p| !p.deleted && is_point_kind(&p.kind))
                        || !self.objects.get(angle.saturating_sub(1)).is_some_and(|p| !p.deleted && p.kind.numeric_unit() == Some(NumericUnit::Degrees)) {
                        return Err("Invalid numeric-angle rotation parents".into());
                    }
                    if i >= self.source_count && (self.numeric_value(*angle).is_none() || self.position(*center).is_none()
                        || match o.kind { Kind::CalculatedAngleRotatedPoint(..) => self.position(o.id).is_none(), Kind::CalculatedAngleRotatedSegment(..) => self.line_points(o.id).is_none(), Kind::CalculatedAngleRotatedCircle(..) => self.circle_points(o.id).is_none(), _ => self.polygon_points(o.id).is_none() }) {
                        return Err("Undefined numeric-angle rotation".into());
                    }
                }
                Kind::RayIteration(seed,image,ray,value)=>{
                    if o.kind.dependencies().iter().any(|id|*id==0||*id>=o.id) || !self.objects.get(value.saturating_sub(1)).is_some_and(|o|!o.deleted&&o.kind.is_iteration_depth_value())
                        || !self.objects.get(seed.saturating_sub(1)).is_some_and(|o|!o.deleted&&matches!(o.kind,Kind::Free(_))) || !supported_point_iteration_mapping(&self.objects,*seed,*image)
                        || !self.objects.get(ray.saturating_sub(1)).is_some_and(|o|!o.deleted&&matches!(o.kind,Kind::Ray(a,b) if a==*seed&&b==*image)) {return Err("Invalid carried ray iteration mapping or depth".into());}
                }
                Kind::MappedPointIteration { seeds, branches, carried, depth, .. } => {
                    let valid_depth = match depth {
                        IterationDepth::Fixed(n) => (1..=128).contains(n),
                        IterationDepth::Value(id) => *id > 0 && *id < o.id && self.objects.get(id - 1).is_some_and(|p| !p.deleted && p.kind.is_iteration_depth_value()),
                    };
                    if !valid_depth || !supported_mapped_point_iteration(&self.objects, seeds, branches, carried) {
                        return Err(format!("Invalid mapped point iteration {}", o.id));
                    }
                }
                Kind::IterationBinding(image, iteration)
                    if self.objects.get(iteration.saturating_sub(1)).is_some_and(|parent| matches!(parent.kind, Kind::MappedPointIteration { .. })) => {
                    let Some(parent) = self.objects.get(iteration.saturating_sub(1)) else { unreachable!() };
                    let Kind::MappedPointIteration { carried, .. } = &parent.kind else { unreachable!() };
                    if *image == 0 || *image >= *iteration || *iteration >= o.id || parent.deleted || parent.page != o.page
                        || !carried.contains(image)
                        || !self.objects.get(image.saturating_sub(1)).is_some_and(|source| !source.deleted && source.page == o.page
                            && (is_point_kind(&source.kind) || matches!(source.kind, Kind::Segment(..) | Kind::VectorTranslatedSegment(..)))) {
                        return Err("Invalid mapped iteration binding".into());
                    }
                }
                Kind::PointIteration(seed,image,depth)=>{
                    let valid_depth=match depth {IterationDepth::Fixed(n)=>(1..=128).contains(n),IterationDepth::Value(id)=>*id>0 && *id<o.id && self.objects.get(id-1).is_some_and(|p|!p.deleted&&p.kind.is_iteration_depth_value())};
                    if !valid_depth || *seed==0 || *image==0 || *seed>=*image || *image>=o.id || !self.objects.get(seed-1).is_some_and(|p|!p.deleted&&matches!(p.kind,Kind::Free(_))) || !supported_point_iteration_mapping(&self.objects,*seed,*image) {return Err("Invalid point iteration mapping or depth".into());}
                }
                Kind::CircleIteration(seed,image,circle,depth)=>{
                    let valid_depth=match depth {IterationDepth::Fixed(n)=>(1..=128).contains(n),IterationDepth::Value(id)=>*id>0&&*id<o.id&&self.objects.get(id-1).is_some_and(|p|!p.deleted&&p.kind.is_iteration_depth_value())};
                    if !valid_depth || o.kind.dependencies().iter().any(|id|*id==0||*id>=o.id) || !self.objects.get(seed.saturating_sub(1)).is_some_and(|p|!p.deleted&&matches!(p.kind,Kind::Free(_))) || !supported_point_iteration_mapping(&self.objects,*seed,*image)
                        || !self.objects.get(circle.saturating_sub(1)).is_some_and(|p|!p.deleted&&matches!(p.kind,Kind::Circle(a,b) if a==*seed&&b==*image)) {return Err("Invalid carried circle iteration".into());}
                }
                Kind::SegmentIteration(seed,image,segment,depth)=>{
                    let valid_depth=match depth {IterationDepth::Fixed(value)=>(1..=128).contains(value),IterationDepth::Value(id)=>*id>0&&*id<o.id&&self.objects.get(id-1).is_some_and(|p|!p.deleted&&p.kind.is_iteration_depth_value())};
                    if !valid_depth || o.kind.dependencies().iter().any(|id|*id==0||*id>=o.id) || !supported_point_iteration_mapping(&self.objects,*seed,*image)
                        || !self.objects.get(segment.saturating_sub(1)).is_some_and(|p|!p.deleted&&matches!(p.kind,Kind::Segment(a,b) if a==*seed&&b==*image)) {return Err("Invalid carried segment iteration".into());}
                }
                Kind::BranchingPolygonIteration { seed,first,second,polygon,segments,depth } => {
                    let valid_depth = match depth { IterationDepth::Fixed(value) => (1..=128).contains(value), IterationDepth::Value(id) => *id>0 && *id<o.id && self.objects.get(id-1).is_some_and(|p| !p.deleted && p.kind.is_iteration_depth_value()) };
                    if !valid_depth || o.kind.dependencies().iter().any(|id| *id==0 || *id>=o.id)
                        || !supported_branch_polygon_mapping(&self.objects,*seed,*first,*second,*polygon,segments) {return Err("Invalid branching polygon iteration".into());}
                }
                Kind::PointParameterIteration(seed, parameter, image, calculation, label, depth) => {
                    let valid_depth = match *depth {
                        IterationDepth::Fixed(value) => (1..=128).contains(&value),
                        IterationDepth::Value(id) => id > 0 && id < o.id && id != *parameter && self.objects.get(id - 1).is_some_and(|parent| !parent.deleted && parent.kind.is_iteration_depth_value()),
                    };
                    if !valid_depth
                        || [*seed, *parameter, *image, *calculation, *label].iter().any(|id| *id == 0 || *id >= o.id)
                        || !supported_point_parameter_iteration(&self.objects, *seed, *parameter, *image, *calculation, *label)
                    {
                        return Err("Invalid point and parameter iteration mapping".into());
                    }
                }
                Kind::IterationValues(iteration, calculation) => {
                    if *iteration == 0 || *iteration >= o.id || *calculation == 0 || *calculation >= o.id
                        || !self.objects.get(iteration - 1).is_some_and(|parent| !parent.deleted && matches!(parent.kind, Kind::PointParameterIteration(_, _, _, value, _, _) if value == *calculation))
                    {
                        return Err("Invalid iteration value series parents".into());
                    }
                }
                Kind::PointNumericLabel(point, value) => {
                    if [*point, *value].iter().any(|&id| {
                        id == 0
                            || id >= o.id
                            || !self
                                .objects
                                .get(id - 1)
                                .is_some_and(|parent| !parent.deleted)
                    }) || !is_point_kind(&self.objects[*point - 1].kind)
                        || !matches!(
                            self.objects[*value - 1].kind,
                            Kind::Parameter(_)
                                | Kind::ParameterCalculation(_, _)
                                | Kind::Calculation(_)
                        )
                    {
                        return Err("Invalid attached numeric label parents".into());
                    }
                }
                Kind::Parameter(group) => {
                    ParameterGroup::parse(group.as_bytes())?;
                }
                Kind::Calculation(group) => {
                    CalculationGroup::parse(group.as_bytes())?;
                }
                Kind::MeasuredCalculation(source, group) => {
                    MeasuredCalculationGroup::parse(group.as_bytes())?;
                    if group.source_id() as usize != *source
                        || *source >= o.id
                        || !self
                            .objects
                            .get(source.saturating_sub(1))
                            .is_some_and(|object| {
                                !object.deleted && object.kind.is_distance_value()
                            })
                    {
                        return Err("Unsupported measured calculation source".into());
                    }
                }
                Kind::LengthPairCalculation(first, second, group) => {
                    LengthPairCalculationGroup::parse(group.as_bytes())?;
                    if group.sources() != [*first as u32, *second as u32]
                        || [*first, *second].iter().any(|&id| {
                            id >= o.id
                                || !self
                                    .objects
                                    .get(id.saturating_sub(1))
                                    .is_some_and(|parent| {
                                        !parent.deleted && parent.kind.is_distance_value()
                                    })
                        })
                    {
                        return Err("Invalid two-length calculation sources".into());
                    }
                }
                Kind::ParameterCalculation(source,group)=>{
                    ParameterCalculationGroup::parse(group.as_bytes())?;
                    let sources=calculation_source_ids(*source,group);
                    let units=sources.iter().map(|id|self.objects.get(id.saturating_sub(1)).filter(|p|!p.deleted && *id<o.id).and_then(|p|p.kind.numeric_unit())).collect::<Option<Vec<_>>>().ok_or("Invalid calculation sources")?;
                    if group.units_for_sources(&units)?!=group.unit() { return Err("Invalid calculation units".into()); }
                }
                Kind::ValueTable(source, group) => {
                    TableGroup::parse(group.as_bytes())?;
                    if group.source_id() as usize != *source
                        || !self.objects.get(source.saturating_sub(1)).is_some_and(|o| {
                            match &o.kind {
                                Kind::Parameter(parameter) if parameter.unit() == NumericUnit::Scalar => match group.named_label_value() {
                                    Some((name, _)) => name == parameter.name(),
                                    None => parameter.name() == "t[1]",
                                },
                                _ => false,
                            }
                        })
                    {
                        return Err("Unsupported table source".into());
                    }
                }
                Kind::ValueTablePair(first, second, group) => {
                    TableGroup::parse_pair(group.as_bytes())?;
                    if group.pair_source_ids() != Some([*first as u32, *second as u32])
                        || !matches!(self.objects.get(second - 1).map(|o| &o.kind), Some(Kind::Parameter(parameter)) if parameter.name() == "t[1]" && parameter.unit() == NumericUnit::Scalar)
                        || !matches!(self.objects.get(first - 1).map(|o| &o.kind), Some(Kind::Parameter(parameter)) if parameter.name() == "t[2]" && parameter.unit() == NumericUnit::Scalar)
                    {
                        return Err("Unsupported two-column table source".into());
                    }
                }
                _ => {}
            }
            if let Kind::Image { png, transform } = &o.kind {
                ImageFile::png_size(png)?;
                if transform.iter().any(|value| !value.is_finite()) {
                    return Err(format!("Non-finite picture transform in object {}", o.id));
                }
                if o.id <= self.source_count {
                    let source = ImageFile::parse(&self.original)?;
                    if source.png() != png {
                        return Err("Replacing a saved picture's PNG is not supported".into());
                    }
                }
            }
            if o.label.is_some() && !is_point_kind(&o.kind) && o.kind.numeric_unit().is_none() && !matches!(o.kind, Kind::ValueEquation {..} | Kind::CoordinateMeasurement {..}) {
                return Err(format!("Invalid object label in object {}", o.id));
            }
            if matches!(o.kind, Kind::PointOnSegment(_, t) | Kind::PointOnArc(_, t) | Kind::PointOnPolygon(_, _, t) if !t.is_finite() || !(0.0..=1.0).contains(&t))
                || matches!(o.kind, Kind::PointOnRay(_, t) if !t.is_finite() || t < 0.)
                || matches!(o.kind, Kind::PointOnLine(_, t) if !t.is_finite())
                || matches!(o.kind, Kind::PointOnCircle(_, x, y) if !x.is_finite() || !y.is_finite() || (x*x+y*y-1.).abs() > 1e-6)
                || matches!(o.kind, Kind::PolarTranslatedPoint(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::PolarTranslatedSegment(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::PolarTranslatedCircle(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::PolarTranslatedPolygon(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::FixedRectangularTranslatedPoint(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::FixedRectangularTranslatedSegment(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::FixedRectangularTranslatedCircle(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::FixedRectangularTranslatedPolygon(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::MarkedDistanceTranslatedPoint(_, _, angle) if !angle.is_finite())
                || matches!(o.kind, Kind::MarkedDistanceTranslatedSegment(_, _, angle) | Kind::MarkedDistanceTranslatedCircle(_, _, angle) | Kind::MarkedDistanceTranslatedPolygon(_, _, angle) if !angle.is_finite())
                || matches!(o.kind, Kind::MixedRectangularTranslatedPoint(_, _, fixed_cm, _) | Kind::MixedRectangularTranslatedSegment(_, _, fixed_cm, _) | Kind::MixedRectangularTranslatedCircle(_, _, fixed_cm, _) | Kind::MixedRectangularTranslatedPolygon(_, _, fixed_cm, _) if !fixed_cm.is_finite())
                || matches!(o.kind, Kind::ScaledPoint(_, _, factor) | Kind::ScaledCircle(_, _, factor) | Kind::ScaledSegment(_, _, factor) | Kind::ScaledPolygon(_, _, factor) if !factor.is_finite() || factor == 0.)
                || matches!(o.kind, Kind::ScaledRay(_, _, factor) | Kind::ScaledLine(_, _, factor) if !factor.is_finite())
                || matches!(o.kind, Kind::RotatedPoint(_, _, angle) | Kind::RotatedSegment(_, _, angle) | Kind::RotatedRay(_, _, angle) | Kind::RotatedLine(_, _, angle) | Kind::RotatedCircle(_, _, angle) | Kind::RotatedPolygon(_, _, angle) if !angle.is_finite())
            {
                return Err(format!("Invalid point constraint in object {}", o.id));
            }
            let refs = o.kind.dependencies();
            if refs
                .iter()
                .any(|&r| r == 0 || r >= o.id || self.objects[r - 1].deleted)
            {
                return Err(format!("Invalid dependency in object {}", o.id));
            }
            if let Kind::HideShowButton { targets, label, .. } = &o.kind {
                if self.page_count != 1
                    || targets.is_empty()
                    || targets
                        .iter()
                        .any(|&target| self.objects[target - 1].page != o.page)
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Hide/Show button {}", o.id));
                }
            }
            if let Kind::AnimationButton {
                targets,
                settings,
                label,
                ..
            } = &o.kind
            {
                if self.page_count != 1
                    || targets.is_empty()
                    || targets.len() != settings.len()
                    || targets
                        .iter()
                        .any(|&target| self.objects[target - 1].page != o.page)
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Animation button {}", o.id));
                }
            }
            if let Kind::AnimationButton { settings, .. } = &o.kind {
                for setting in settings {
                    setting
                        .validate()
                        .map_err(|error| format!("Invalid Animation button {}: {error}", o.id))?;
                }
            }
            if let Kind::MovementButton {
                source,
                destination,
                label,
                ..
            } = &o.kind
            {
                if self.page_count != 1
                    || *source == *destination
                    || *source == 0
                    || *destination == 0
                    || *source > self.objects.len()
                    || *destination > self.objects.len()
                    || self.objects[*source - 1].page != o.page
                    || self.objects[*destination - 1].page != o.page
                    || !matches!(self.objects[*source - 1].kind, Kind::Free(_))
                    || self.position(*destination).is_none()
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Movement button {}", o.id));
                }
            }
            if let Kind::ScrollButton { target, label, .. } = &o.kind {
                if self.page_count != 1
                    || *target == 0
                    || *target > self.objects.len()
                    || self.objects[*target - 1].page != o.page
                    || !is_point_kind(&self.objects[*target - 1].kind)
                    || self.position(*target).is_none()
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Scroll button {}", o.id));
                }
            }
            if let Kind::PresentationButton { targets, label, .. } = &o.kind {
                if self.page_count != 1
                    || targets.is_empty()
                    || targets.iter().enumerate().any(|(index, &target)| {
                        target == 0
                            || target >= o.id
                            || targets[..index].contains(&target)
                            || self.objects[target - 1].page != o.page
                            || !self.objects[target - 1].kind.is_action_button()
                    })
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Presentation button {}", o.id));
                }
            }
            if let Kind::LinkUrlButton { url, label, .. } = &o.kind {
                if self.page_count != 1
                    || url.is_empty()
                    || url.len() > u16::MAX as usize
                    || label.len() > u16::MAX as usize
                    || url.as_bytes().contains(&0)
                    || label.as_bytes().contains(&0)
                {
                    return Err(format!("Invalid Link URL button {}", o.id));
                }
            }
            if let Kind::Midpoint(a, b) = o.kind {
                let Some(host) = o.midpoint_host else {
                    return Err(format!("Midpoint {} has no segment host", o.id));
                };
                if host == 0
                    || host >= o.id
                    || self.objects[host - 1].deleted
                    || !matches!(self.objects[host - 1].kind, Kind::Segment(x, y) if (x == a && y == b) || (x == b && y == a))
                {
                    return Err(format!("Invalid segment host in midpoint {}", o.id));
                }
            }
            if let Kind::ParameterLocus(ref refs, min, max, samples) = o.kind {
                if !min.is_finite() || !max.is_finite() || min >= max
                    || !(2..=4096).contains(&samples)
                    || refs.first().zip(refs.get(1)).and_then(|(&target, &driver)| expected_parameter_locus_refs(&self.objects, driver, target)).as_ref() != Some(refs)
                {
                    return Err(format!("Invalid parameter locus {}", o.id));
                }
            }
            if let Kind::Locus(ref refs, samples) = o.kind {
                if !(2..=10_000).contains(&samples)
                    || refs
                        .first()
                        .zip(refs.get(2))
                        .and_then(|(&target, &driver)| {
                            expected_locus_refs(&self.objects, driver, target)
                        })
                        .as_ref()
                        != Some(refs)
                    || self.locus_segments(o.id, 2).is_none()
                {
                    return Err(format!("Invalid locus {}", o.id));
                }
            }
            if i >= self.source_count
                && let Kind::Polygon(vertices) = &o.kind
            {
                if vertices.len() < 3
                    || vertices.iter().collect::<BTreeSet<_>>().len() != vertices.len()
                    || self.polygon_points(o.id).is_none()
                {
                    return Err(format!("Invalid polygon {}", o.id));
                }
            }
            if matches!(o.kind, Kind::SegmentLength(_))
                && self
                    .segment_length_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid segment in length measurement {}", o.id));
            }
            if matches!(o.kind, Kind::PointDistance(_, _))
                && self
                    .point_distance_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid points in distance measurement {}", o.id));
            }
            if matches!(o.kind, Kind::PointLineDistance(_, _))
                && self.point_line_distance_cm(o.id).is_none()
            {
                return Err(format!(
                    "Invalid host in point-to-line distance measurement {}",
                    o.id
                ));
            }
            if matches!(o.kind, Kind::PointRatio(_, _, _)) && self.ratio_value(o.id).is_none() {
                return Err(format!("Invalid points in ratio measurement {}", o.id));
            }
            if matches!(o.kind, Kind::GraphHorizontalUnit(_, step) | Kind::GraphVerticalUnit(_, step) if !step.is_finite() || step <= 0.0)
                || matches!(o.kind, Kind::DistanceVerticalUnit(_, step, _) if !step.is_finite() || step <= 0.0)
                || matches!(o.kind, Kind::GraphAxis(_, _, axis) if !matches!(axis, 0 | 1 | 0x1_0001))
                || matches!(o.kind, Kind::CircleAxis(_, axis) if !matches!(axis, 0 | 1))
                || matches!(o.kind, Kind::DistanceAxis(_, _, axis) if !matches!(axis, 0 | 1))
            {
                return Err(format!("Invalid graph object {}", o.id));
            }
            if matches!(
                o.kind,
                Kind::GraphHorizontalUnit(_, _)
                    | Kind::GraphVerticalUnit(_, _)
                    | Kind::DistanceVerticalUnit(_, _, _)
            ) && self
                .position(o.id)
                .is_none_or(|p| !p.x.is_finite() || !p.y.is_finite())
            {
                return Err(format!("Undefined graph point {}", o.id));
            }
            if matches!(
                o.kind,
                Kind::GraphAxis(_, _, _) | Kind::CircleAxis(_, _) | Kind::DistanceAxis(_, _, _)
            ) && self.line_points(o.id).is_none()
            {
                return Err(format!("Undefined graph axis {}", o.id));
            }
            if matches!(o.kind, Kind::CoordinateSystem(_, _)) && self.graph_frame(o.id).is_none() {
                return Err(format!("Invalid coordinate system {}", o.id));
            }
            if let Kind::DerivativeFunction(source, _) = o.kind
                && (source >= o.id || self.function_expression(source).is_none())
            {
                return Err(format!("Invalid derivative source in object {}", o.id));
            }
            if let Kind::FunctionPlot(definition, frame, min, max, samples) = o.kind {
                if !matches!(
                    self.objects
                        .get(definition.saturating_sub(1))
                        .map(|o| &o.kind),
                    Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _))
                ) || self.graph_frame(frame).is_none()
                    || !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || !(2..=4096).contains(&samples)
                {
                    return Err(format!("Invalid function plot {}", o.id));
                }
            }
            if let Kind::ParametricPlot(x, y, frame, min, max, samples, _) = o.kind {
                if x == y
                    || [x, y].into_iter().any(|id| {
                        !matches!(
                            self.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                            Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _))
                        )
                    })
                    || self.graph_frame(frame).is_none()
                    || !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || !(2..=4096).contains(&samples)
                {
                    return Err(format!("Invalid parametric plot {}", o.id));
                }
            }
            if let Kind::PlottedPoint(source, frame, mode, stored) = o.kind {
                if self
                    .objects
                    .get(source.saturating_sub(1))
                    .is_none_or(|anchor| anchor.page != o.page)
                    || self.graph_frame(frame).is_none()
                    || stored.iter().any(|value| !value.is_finite())
                    || self.position(o.id).is_none()
                {
                    return Err(format!("Invalid plotted point {}", o.id));
                }
                if o.id > self.source_count {
                    let group = plot_points_records::encode_group(
                        mode,
                        [source as u32, frame as u32],
                        stored,
                    )?;
                    if plot_points_records::decode_group(&group)?.stored != stored {
                        return Err("Cannot encode this Plot Points value".into());
                    }
                }
            }
            if let Kind::PathValuePoint(source, path) = o.kind {
                if !self.objects.get(source.saturating_sub(1)).is_some_and(|parent| !parent.deleted && parent.page == o.page && source < o.id && parent.kind.numeric_unit() == Some(NumericUnit::Scalar))
                    || !self.objects.get(path.saturating_sub(1)).is_some_and(|host| !host.deleted && host.page == o.page && path < o.id && self.numeric_value(source).is_some_and(|value| value_plot_path_accepts(&host.kind, value)))
                    || self.position(o.id).is_none()
                {
                    return Err(format!("Invalid path value point {}", o.id));
                }
            }
            if let Kind::PathLiteralPoint(path, value) = o.kind {
                if !self
                    .objects
                    .get(path.saturating_sub(1))
                    .is_some_and(|host| !host.deleted && host.page == o.page && path < o.id && literal_plot_path_accepts(&host.kind, value))
                    || self.position(o.id).is_none()
                {
                    return Err(format!("Invalid literal path point {}", o.id));
                }
            }
            if let Kind::NumericCoordinatePoint(x, y, frame) = o.kind {
                if ![x, y].iter().all(|&id| self.objects.get(id.saturating_sub(1)).is_some_and(|source| !source.deleted && source.page == o.page && id < o.id && source.kind.numeric_unit().is_some()))
                    || self.objects.get(frame.saturating_sub(1)).is_none_or(|source| source.deleted || source.page != o.page || frame >= o.id || !matches!(source.kind, Kind::CoordinateSystem(_, _)))
                    || self.graph_frame(frame).is_none()
                {
                    return Err(format!("Invalid numeric coordinate point {}", o.id));
                }
            }
            if let Kind::SlopeMeasurement(line, frame) = o.kind {
                if !self
                    .objects
                    .get(line.saturating_sub(1))
                    .is_some_and(|o| is_straight_kind(&o.kind))
                    || self.line_points(line).is_none()
                    || self.graph_frame(frame).is_none()
                {
                    return Err(format!("Invalid slope measurement {}", o.id));
                }
            }
            if let Kind::CoordinateMeasurement {
                kind,
                first,
                second,
                frame,
            } = o.kind
            {
                let point_ok = |id: usize| {
                    self.objects
                        .get(id.saturating_sub(1))
                        .is_some_and(|object| {
                            !object.deleted && object.page == o.page && is_point_kind(&object.kind)
                        })
                };
                if !point_ok(first)
                    || second.is_some_and(|id| !point_ok(id))
                    || (kind == CoordinateMeasureKind::CoordinateDistance) != second.is_some()
                    || !matches!(
                        self.objects
                            .get(frame.saturating_sub(1))
                            .map(|object| &object.kind),
                        Some(Kind::CoordinateSystem(_, _))
                    )
                    || self.coordinate_measure_value(o.id).is_none()
                {
                    return Err(format!("Invalid coordinate measurement {}", o.id));
                }
            }
            if let Kind::ValueEquation {
                kind,
                first,
                second,
            } = o.kind
            {
                let hosts_match = match kind {
                    ValueEquationKind::PointValue => {
                        self.objects.get(first.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                            && self.objects.get(second.saturating_sub(1)).is_some_and(|object| is_straight_kind(&object.kind) || object.kind.is_circle() || is_arc_kind(&object.kind)
                                || matches!(self.objects[first-1].kind, Kind::PointOnPolygon(host, _, _) if host == second))
                    }
                    ValueEquationKind::LineEquation => {
                        self.objects
                            .get(first.saturating_sub(1))
                            .is_some_and(|o| o.kind.is_line())
                            && matches!(
                                self.objects
                                    .get(second.saturating_sub(1))
                                    .map(|object| &object.kind),
                                Some(Kind::CoordinateSystem(_, _))
                            )
                    }
                    ValueEquationKind::CircleEquation => {
                        self.objects
                            .get(first.saturating_sub(1))
                            .is_some_and(|o| o.kind.is_circle())
                            && matches!(
                                self.objects
                                    .get(second.saturating_sub(1))
                                    .map(|object| &object.kind),
                                Some(Kind::CoordinateSystem(_, _))
                            )
                    }
                };
                if !hosts_match || self.value_equation_value(o.id).is_none() {
                    return Err(format!("Invalid Value/Equation measurement {}", o.id));
                }
            }
            if matches!(o.kind, Kind::PolygonPerimeter(_))
                && self
                    .perimeter_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid polygon in perimeter measurement {}", o.id));
            }
            if matches!(o.kind, Kind::CircleCircumference(_))
                && self
                    .circle_circumference_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!(
                    "Invalid circle in circumference measurement {}",
                    o.id
                ));
            }
            if matches!(o.kind, Kind::CircleRadius(_))
                && self
                    .circle_radius_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid circle in radius measurement {}", o.id));
            }
            if matches!(o.kind, Kind::CircleArea(_))
                && self
                    .circle_area_cm2(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid circle in area measurement {}", o.id));
            }
            if matches!(o.kind, Kind::PolygonArea(_))
                && self
                    .polygon_area_cm2(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid polygon in area measurement {}", o.id));
            }
            if matches!(o.kind, Kind::ArcAngle(_))
                && self
                    .arc_angle_degrees(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid arc in angle measurement {}", o.id));
            }
            if matches!(o.kind, Kind::ArcLength(_))
                && self
                    .arc_length_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid arc in length measurement {}", o.id));
            }
            if matches!(o.kind, Kind::AngleMeasurement(_, _, _))
                && self
                    .angle_degrees(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid points in angle measurement {}", o.id));
            }
            if let Kind::PointOnPolygon(host, edge, _) = o.kind {
                if !self.objects.get(host.saturating_sub(1)).is_some_and(|object| !object.deleted && object.page == o.page && host < o.id)
                    || !polygon_vertex_count(&self.objects, host).is_some_and(|count| edge < count) {
                    return Err(format!("Invalid polygon boundary constraint {}", o.id));
                }
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnSegment(host, _) if self.objects.get(host.saturating_sub(1)).and_then(|o| straight_kind_mode(&o.kind)) != Some(LineMode::Segment))
            {
                return Err(format!("Invalid segment host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnRay(host, _) if self.objects.get(host.saturating_sub(1)).and_then(|o| straight_kind_mode(&o.kind)) != Some(LineMode::Ray))
            {
                return Err(format!("Invalid ray host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnLine(host, _) if self.objects.get(host.saturating_sub(1)).and_then(|o| straight_kind_mode(&o.kind)) != Some(LineMode::Line))
            {
                return Err(format!("Invalid line host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnCircle(host, _, _) if self.circle_points(host).is_none())
            {
                return Err(format!("Invalid circle host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnArc(host, _) if self.arc_geometry(host).is_none())
            {
                return Err(format!("Invalid arc host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedPoint(origin, _, _, _) if self.position(origin).is_none())
            {
                return Err(format!("Invalid source point in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedSegment(source, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid source segment in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedCircle(source, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid source circle in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::FixedRectangularTranslatedPoint(source, _, _, _, _)
                    if self.position(source).is_none()
                    || !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind)))
            {
                return Err(format!(
                    "Invalid source point in rectangular translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::FixedRectangularTranslatedSegment(source, _, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || self.line_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid source segment in rectangular translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::FixedRectangularTranslatedCircle(source, _, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid source circle in rectangular translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedPoint(source, distance, _)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value()))
            {
                return Err(format!("Invalid marked distance translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedSegment(source, distance, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value()))
            {
                return Err(format!(
                    "Invalid marked-distance segment translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedCircle(source, distance, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value()))
            {
                return Err(format!(
                    "Invalid marked-distance circle translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MixedRectangularTranslatedPoint(source, distance, _, _)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value()))
            {
                return Err(format!("Invalid mixed rectangular translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MixedRectangularTranslatedSegment(source, distance, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value()))
            {
                return Err(format!("Invalid mixed rectangular translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MixedRectangularTranslatedCircle(source, distance, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value()))
            {
                return Err(format!("Invalid mixed rectangular translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
                if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                || [horizontal, vertical].iter().any(|&id| {
                    !self.objects.get(id.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value())
                }))
            {
                return Err(format!("Invalid rectangular marked translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RectangularMarkedTranslatedSegment(source, horizontal, vertical)
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                || [horizontal, vertical].iter().any(|&id| {
                    !self.objects.get(id.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value())
                }))
            {
                return Err(format!("Invalid rectangular marked translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RectangularMarkedTranslatedCircle(source, horizontal, vertical)
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                || [horizontal, vertical].iter().any(|&id| {
                    !self.objects.get(id.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value())
                }))
            {
                return Err(format!("Invalid rectangular marked translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedPoint(source, start, end)
                if [source, start, end].iter().any(|&id| {
                    self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                        object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid point in vector translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedSegment(source, start, end)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| !object.deleted && is_segment_kind(&object.kind))
                    || [start, end].iter().any(|&id| self.objects.get(id.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()))
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment in vector translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedCircle(source, start, end)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| !object.deleted && object.kind.is_circle())
                    || [start, end].iter().any(|&id| self.objects.get(id.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()))
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle in vector translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedPolygon(source, _, _, _)
                    | Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!("Invalid interior in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedPolygon(source, distance, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [distance].iter().any(|&id| !self.objects.get(id.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value())))
            {
                return Err(format!(
                    "Invalid interior or distance in translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [horizontal, vertical].iter().any(|&id| !self.objects.get(id.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value())))
            {
                return Err(format!(
                    "Invalid interior or distances in translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MixedRectangularTranslatedPolygon(source, distance, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [distance].iter().any(|&id| !self.objects.get(id.saturating_sub(1)).is_some_and(|object| object.kind.is_distance_value())))
            {
                return Err(format!(
                    "Invalid interior or distance in mixed translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedPoint(source, mirror)
                    if self.position(source).is_none() || self.line_points(mirror).is_none()
                    || !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind)))
            {
                return Err(format!("Invalid point or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedCircle(source, mirror)
                    if self.circle_points(source).is_none() || self.line_points(mirror).is_none()
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedSegment(source, mirror)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|o| is_segment_kind(&o.kind))
                    || self.line_points(mirror).is_none() || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedPolygon(source, mirror)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.line_points(mirror).is_none() || self.polygon_points(o.id).is_none())
            {
                return Err(format!("Invalid interior or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedPolygon(source, start, end)
                    if polygon_vertex_count(&self.objects, source).is_none()
                    || self.position(start).is_none() || self.position(end).is_none()
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior or vector in translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedPolygon(source, center, _) | Kind::ScaledPolygon(source, center, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.position(center).is_none() || self.polygon_points(o.id).is_none())
            {
                return Err(format!("Invalid interior or center in transform {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedPolygon(source, center, first, vertex, last)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [center, first, vertex, last].iter().any(|&id| self.position(id).is_none())
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior in marked-angle rotation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledPolygon(source, center, origin, base, target)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [center, origin, base, target].iter().any(|&id| self.position(id).is_none())
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior in marked-ratio dilation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledPoint(source, center, _)
                    if self.position(source).is_none() || self.position(center).is_none()
                    || !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind))
                    || !matches!(self.objects.get(center.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind)))
            {
                return Err(format!("Invalid point or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedPoint(source, center, _)
                    if self.position(source).is_none() || self.position(center).is_none()
                    || !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind))
                    || !matches!(self.objects.get(center.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind)))
            {
                return Err(format!("Invalid point or center in rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedSegment(source, center, _)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_segment_kind(&object.kind))
                    || self.line_points(o.id).is_none()
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(center).is_none()))
            {
                return Err(format!("Invalid segment or center in rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedCircle(source, center, _)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|object| object.kind.is_circle())
                    || self.circle_points(o.id).is_none()
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(center).is_none()))
            {
                return Err(format!("Invalid circle or center in rotation {}", o.id));
            }
            if i >= self.source_count
                && let Kind::ScaledRay(source, center, factor) | Kind::ScaledLine(source, center, factor) = o.kind
            {
                let expected = if matches!(o.kind, Kind::ScaledRay(..)) { LineMode::Ray } else { LineMode::Line };
                let source_matches = self.objects.get(source.saturating_sub(1)).is_some_and(|object|
                    !object.deleted && matches!((&o.kind, &object.kind),
                        (Kind::ScaledRay(..), Kind::Ray(..)) | (Kind::ScaledLine(..), Kind::Line(..))));
                if !source_matches
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(center).is_none())
                    || (factor != 0. && self.line_points(o.id).is_none_or(|(_, _, mode)| mode != expected))
                {
                    return Err(format!("Invalid ray or line source/center in dilation {}", o.id));
                }
            }
            if i >= self.source_count
                && let Kind::RotatedRay(source, center, _) | Kind::RotatedLine(source, center, _) = o.kind
            {
                let expected = if matches!(o.kind, Kind::RotatedRay(..)) { LineMode::Ray } else { LineMode::Line };
                let source_matches = self.objects.get(source.saturating_sub(1)).is_some_and(|object|
                    !object.deleted && matches!((&o.kind, &object.kind),
                        (Kind::RotatedRay(..), Kind::Ray(..)) | (Kind::RotatedLine(..), Kind::Line(..))));
                if !source_matches
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(center).is_none())
                    || self.line_points(o.id).is_none_or(|(_, _, mode)| mode != expected)
                {
                    return Err(format!("Invalid ray or line source/center in rotation {}", o.id));
                }
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedPoint(source, center, first, vertex, last)
                    if [source, center, first, vertex, last].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid point in marked-angle rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedSegment(source, center, first, vertex, last)
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                || self.line_points(o.id).is_none()
                || [center, first, vertex, last].iter().any(|&id| {
                    self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                        object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid segment in marked-angle rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedCircle(source, center, first, vertex, last)
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                || self.circle_points(o.id).is_none()
                || [center, first, vertex, last].iter().any(|&id| {
                    self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                        object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid circle in marked-angle rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledPoint(source, center, origin, base, target)
                    if [source, center, origin, base, target].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                        })
                    }) || self.ratio_for_points(origin, base, target).is_none())
            {
                return Err(format!("Invalid point in marked-ratio dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledCircle(source, center, _)
                    if self.circle_points(source).is_none() || self.position(center).is_none() || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledSegment(source, center, _)
                    if !self.objects.get(source.saturating_sub(1)).is_some_and(|o| is_segment_kind(&o.kind))
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|o| o.deleted || !is_point_kind(&o.kind) || self.position(center).is_none())
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledCircle(source, center, origin, base, target)
                    if self.circle_points(source).is_none()
                    || self.position(center).is_none()
                    || [center, origin, base, target].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                        })
                    })
                    || self.ratio_for_points(origin, base, target).is_none()
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle in marked-ratio dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledSegment(source, center, origin, base, target)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Segment(_, _)))
                    || [center, origin, base, target].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                        })
                    }) || self.ratio_for_points(origin, base, target).is_none()
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment in marked-ratio dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(
                    o.kind,
                    Kind::PolarTranslatedPoint(_, _, _, _)
                        | Kind::FixedRectangularTranslatedPoint(_, _, _, _, _)
                        | Kind::VectorTranslatedPoint(_, _, _)
                        | Kind::ReflectedPoint(_, _)
                        | Kind::ScaledPoint(_, _, _)
                        | Kind::RotatedPoint(_, _, _)
                        | Kind::CalculatedAngleRotatedPoint(_, _, _)
                        | Kind::CalculatedRatioScaledPoint(_, _, _)
                        | Kind::AngleRotatedPoint(_, _, _, _, _)
                        | Kind::RatioScaledPoint(_, _, _, _, _)
                )
                && !self
                    .position(o.id)
                    .is_some_and(|p| p.x.is_finite() && p.y.is_finite())
            {
                return Err(format!("Undefined transformed point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Segment(a,b)|Kind::Line(a,b)|Kind::Ray(a,b)|Kind::Circle(a,b)|Kind::Midpoint(a,b)
                if self.position(a).is_none()||self.position(b).is_none())
            {
                return Err(format!("Invalid point dependency in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Parallel(_,line)|Kind::Perpendicular(_,line) if self.line_points(line).is_none())
            {
                return Err(format!("Invalid source line in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Parallel(point,_)|Kind::Perpendicular(point,_) if self.position(point).is_none())
            {
                return Err(format!("Invalid anchor point in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::CircleBySegment(center,segment)
                    if self.position(center).is_none()
                    || !matches!(self.objects.get(segment.saturating_sub(1)).map(|o| &o.kind),Some(Kind::Segment(_, _))))
            {
                return Err(format!(
                    "Invalid center or radius segment in circle {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Intersection(a,b) if self.line_points(a).is_none() || self.line_points(b).is_none())
            {
                return Err(format!("Invalid straight objects in intersection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Bisector(a,b,c)
                    if self.position(a).is_none() || self.position(b).is_none() || self.position(c).is_none())
            {
                return Err(format!("Invalid points in angle bisector {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::ThreePointArc(a,b,c)
                    if self.position(a).is_none() || self.position(b).is_none() || self.position(c).is_none())
            {
                return Err(format!("Invalid points in three-point arc {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::ArcOnCircle(circle,start,end)
                    if self.circle_points(circle).is_none()
                    || !matches!(self.objects.get(start.saturating_sub(1)).map(|o| &o.kind), Some(Kind::PointOnCircle(host, _, _)) if *host == circle)
                    || !matches!(self.objects.get(end.saturating_sub(1)).map(|o| &o.kind), Some(Kind::PointOnCircle(host, _, _)) if *host == circle))
            {
                return Err(format!("Invalid circle or endpoints in arc {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::LineCircleIntersection(a,b,variant)
                    if variant > 1 || self.line_points(a).is_none() || self.circle_points(b).is_none())
            {
                return Err(format!("Invalid line and circle in intersection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::CircleCircleIntersection(a,b,variant)
                    if variant > 1 || self.circle_points(a).is_none() || self.circle_points(b).is_none())
            {
                return Err(format!("Invalid circles in intersection {}", o.id));
            }
            if matches!(o.kind, Kind::Unsupported(_)) && i >= self.source_count {
                return Err("Unsupported new object".into());
            }
        }
        if self.objects.len() > self.source_count && !self.safe_structure {
            return Err("This sketch has unsupported object records. Adding objects could change their references; save refused.".into());
        }
        Ok(())
    }
    pub(super) fn validate_reparsed(&self, reparsed: &Self) -> Result<(), String> {
        let survivors = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect();
        self.validate_reparsed_in_order(reparsed, survivors, false)
    }
    pub(super) fn validate_reparsed_in_order(
        &self,
        reparsed: &Self,
        survivors: Vec<&Object>,
        ignore_active_page_frame: bool,
    ) -> Result<(), String> {
        let mut ids = vec![0; self.objects.len() + 1];
        for (index, object) in survivors.iter().enumerate() {
            ids[object.id] = index + 1;
        }
        let expected_kinds = survivors
            .iter()
            .map(|object| {
                let kind = object.kind.remap_checked(&ids)?;
                let visibility = |bytes: &[u8]| {
                    let mut bytes = bytes.to_vec();
                    bytes[10] = (bytes[10] & !1) | u8::from(!object.visible);
                    bytes
                };
                Ok(match kind {
                    Kind::Parameter(group) => {
                        Kind::Parameter(ParameterGroup::parse(&visibility(group.as_bytes()))?)
                    }
                    Kind::Calculation(group) => {
                        Kind::Calculation(CalculationGroup::parse(&visibility(group.as_bytes()))?)
                    }
                    Kind::ParameterCalculation(source, group) => {
                        Kind::ParameterCalculation(source, group.with_visibility(object.visible)?)
                    }
                    Kind::MeasuredCalculation(source, group) => Kind::MeasuredCalculation(
                        source,
                        MeasuredCalculationGroup::parse(&visibility(group.as_bytes()))?,
                    ),
                    Kind::LengthPairCalculation(first, second, group) => {
                        Kind::LengthPairCalculation(
                            first,
                            second,
                            LengthPairCalculationGroup::parse(&visibility(group.as_bytes()))?,
                        )
                    }
                    Kind::ValueTable(source, group) => {
                        Kind::ValueTable(source, TableGroup::parse(&visibility(group.as_bytes()))?)
                    }
                    Kind::ValueTablePair(first, second, group) => Kind::ValueTablePair(
                        first,
                        second,
                        TableGroup::parse_pair(&visibility(group.as_bytes()))?,
                    ),
                    other => other,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        if reparsed.partial_view && self.safe_structure
            || reparsed.snap_points != self.snap_points
            || reparsed.grid_dotted != self.grid_dotted
            || reparsed.grid_form != self.grid_form
            || reparsed.page_names != self.page_names
            || reparsed.angle_unit_radians != self.angle_unit_radians
            || reparsed.page_count != self.page_count
            || reparsed.marked_coordinate_system
                != self
                    .marked_coordinate_system
                    .iter()
                    .map(|id| id.and_then(|id| (ids[id] != 0).then_some(ids[id])))
                    .collect::<Vec<_>>()
            || reparsed.frame_styles
                != self
                    .frame_styles
                    .iter()
                    .filter_map(|(id, style)| {
                        let remapped = ids.get(*id).copied().unwrap_or(0);
                        (remapped != 0).then_some((remapped, *style))
                    })
                    .collect()
            || !ignore_active_page_frame
                && reparsed.active_coordinate_system()
                    != self.active_coordinate_system().map(|id| ids[id])
            || reparsed.objects.len() != survivors.len()
            || reparsed
                .objects
                .iter()
                .zip(survivors.iter().zip(&expected_kinds))
                .any(|(saved, (current, expected_kind))| {
                    saved.id != ids[current.id]
                        || saved.page != current.page
                        || saved.kind != *expected_kind
                        || saved.visible != current.visible
                        || saved.label != current.label
                        || saved.visual_style != current.visual_style
                        || saved.interior_order != current.interior_order
                        || saved.midpoint_host != current.midpoint_host.map(|host| ids[host])
                        || current.function_anchor.is_some()
                            && saved.function_anchor != current.function_anchor
                        || current.function_name.is_some()
                            && saved.function_name != current.function_name
                        || current.measurement_anchor.is_some()
                            && saved.measurement_anchor != current.measurement_anchor
                        || current.measurement_precision.is_some()
                            && saved.measurement_precision != current.measurement_precision
                })
        {
            return Err("Serialized sketch does not preserve the edited objects".into());
        }
        Ok(())
    }
    pub(super) fn validate_reparsed_document(&self, reparsed: &Self) -> Result<(), String> {
        if self.has_pending_multi_page_additions() {
            let mut survivors: Vec<_> = self
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            survivors.sort_by_key(|object| (object.page, object.id > self.source_count, object.id));
            self.validate_reparsed_in_order(reparsed, survivors, true)
        } else if self.page_count > 1 {
            let survivors = self
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            self.validate_reparsed_in_order(reparsed, survivors, true)
        } else {
            self.validate_reparsed(reparsed)
        }
    }
}
