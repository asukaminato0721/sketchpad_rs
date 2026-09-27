use super::calculations::calculation_source_ids;
use super::geometry::{
    marked_rotation_theta, ratio_of_points, reflect_point, rotate_point_about, scale_point_about,
};
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind, straight_kind_mode};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeTransformFlavor, NumericExpr, Transform,
    TransformDefinition,
};
use super::{Document, IterationDepth, Kind, Object, Point};
use crate::style_records;
use crate::table_records::NumericUnit;
use std::collections::BTreeSet;

fn point_iteration_definition(
    objects: &[Object],
    seed: usize,
    image: usize,
) -> Option<TransformDefinition> {
    let get = |id: usize| {
        id.checked_sub(1)
            .and_then(|at| objects.get(at))
            .filter(|object| !object.deleted)
    };
    let definition = get(image)?.kind.transform_definition()?;
    (definition.element() == ElementType::Point
        && definition.source() == seed
        && iteration_transform_inputs_supported(objects, &definition))
    .then_some(definition)
}

fn iteration_transform_inputs_supported(
    objects: &[Object],
    definition: &TransformDefinition,
) -> bool {
    let get = |id: usize| {
        id.checked_sub(1)
            .and_then(|at| objects.get(at))
            .filter(|object| !object.deleted)
    };
    if definition
        .native_literals()
        .iter()
        .any(|literal| !literal.value.is_finite())
    {
        return false;
    }
    let point = |id: usize| get(id).is_some_and(|object| is_point_kind(&object.kind));
    let numeric = |expression: &NumericExpr| match *expression {
        NumericExpr::Literal { .. } => true,
        NumericExpr::ObjectValue { id, unit } => get(id).is_some_and(|object| {
            if unit == NumericUnit::Length {
                object.kind.is_marked_distance_value()
            } else {
                object.kind.numeric_unit() == Some(unit)
            }
        }),
        NumericExpr::DirectedAngle {
            first,
            vertex,
            last,
        } => [first, vertex, last].into_iter().all(point),
        NumericExpr::DistanceRatio {
            origin,
            base,
            target,
        } => [origin, base, target].into_iter().all(point),
    };
    let supported = match definition.transform() {
        Transform::Translate(DisplacementExpr::Cartesian {
            horizontal,
            vertical,
        }) => numeric(horizontal) && numeric(vertical),
        Transform::Translate(DisplacementExpr::Polar { angle, distance }) => {
            numeric(angle) && numeric(distance)
        }
        Transform::Translate(DisplacementExpr::Vector { start, end }) => {
            point(*start) && point(*end)
        }
        Transform::Rotate { center, angle } => point(*center) && numeric(angle),
        Transform::Dilate { center, ratio } => point(*center) && numeric(ratio),
        Transform::Reflect { mirror } => {
            get(*mirror).is_some_and(|object| is_straight_kind(&object.kind))
        }
    };
    supported
}

pub(super) fn supported_point_iteration_mapping(
    objects: &[Object],
    seed: usize,
    image: usize,
) -> bool {
    point_iteration_definition(objects, seed, image).is_some()
}

/// The carried DAG uses the same typed transform inputs as ordinary geometry.
fn mapped_transform_supported(objects: &[Object], definition: &TransformDefinition) -> bool {
    let Some(source) = definition
        .source()
        .checked_sub(1)
        .and_then(|at| objects.get(at))
    else {
        return false;
    };
    let source_matches = match definition.element() {
        ElementType::Point => is_point_kind(&source.kind),
        ElementType::Segment => source.kind.is_segment(),
        ElementType::Ray => straight_kind_mode(&source.kind) == Some(super::LineMode::Ray),
        ElementType::Line => straight_kind_mode(&source.kind) == Some(super::LineMode::Line),
        ElementType::Circle => source.kind.is_circle(),
        ElementType::Arc => is_arc_kind(&source.kind),
        ElementType::Polygon => source.kind.is_polygon(),
    };
    !source.deleted && source_matches && iteration_transform_inputs_supported(objects, definition)
}

/// A supported value construction can be reevaluated in the substituted sample.
/// Actions, iteration outputs and sampled loci have separate runtime semantics.
fn mapped_value_construction(kind: &Kind) -> bool {
    !matches!(kind, Kind::ToolInput(_) | Kind::ToolPointOnPath(_))
        && (is_point_kind(kind)
            || is_straight_kind(kind)
            || kind.is_circle()
            || kind.is_polygon()
            || is_arc_kind(kind)
            || kind.numeric_unit().is_some()
            || kind.is_function()
            || matches!(kind, Kind::CoordinateSystem(..) | Kind::PointNumericLabel(..)))
}

pub(super) fn mapped_binding_kind(kind: &Kind) -> bool {
    matches!(kind,Kind::PointNumericLabel(..)) || is_point_kind(kind)
        || is_straight_kind(kind)
        || kind.is_circle()
        || kind.is_polygon()
        || is_arc_kind(kind)
}

pub(super) fn mapped_segment_binding(kind: &Kind) -> bool {
    matches!(kind, Kind::Segment(..))
        || kind
            .transform_definition()
            .is_some_and(|definition| definition.element() == ElementType::Segment)
}

pub(super) fn supported_point_parameter_iteration(
    objects: &[Object],
    seed: usize,
    parameter: usize,
    image: usize,
    calculation: usize,
    label: usize,
) -> bool {
    let get = |id: usize| {
        id.checked_sub(1)
            .and_then(|at| objects.get(at))
            .filter(|o| !o.deleted)
    };
    get(seed).is_some_and(|o|matches!(o.kind,Kind::Free(_)))
        && get(parameter).is_some_and(|o|matches!(o.kind,Kind::Parameter(_))
            && o.kind.numeric_unit() == Some(NumericUnit::Scalar))
        && supported_point_iteration_mapping(objects,seed,image)
        && get(calculation).is_some_and(|o|matches!(o.kind,Kind::ParameterCalculation(source,ref group) if calculation_source_ids(source, group).contains(&parameter) && group.unit()==NumericUnit::Scalar))
        && get(label).is_some_and(|o|matches!(o.kind,Kind::PointNumericLabel(host,value) if host==image && value==calculation))
}

pub(super) fn supported_branch_polygon_mapping(
    objects: &[Object],
    seed: usize,
    first: usize,
    second: usize,
    polygon: usize,
    segments: &[usize],
) -> bool {
    let get = |id: usize| {
        id.checked_sub(1)
            .and_then(|at| objects.get(at))
            .filter(|o| !o.deleted)
    };
    let translated = |id| {
        get(id)
            .and_then(|object| object.kind.transform_definition())
            .is_some_and(|definition| {
                definition.element() == ElementType::Point
                    && definition.source() == seed
                    && matches!(
                        definition.wire_flavor(),
                        NativeTransformFlavor::FixedPolar | NativeTransformFlavor::FixedCartesian
                    )
            })
    };
    get(seed).is_some_and(|o| matches!(o.kind, Kind::Free(_)))
        && first != second && translated(first) && translated(second)
        && get(polygon).is_some_and(|o| matches!(&o.kind, Kind::Polygon(vertices) if vertices.len() == 3 && vertices.iter().copied().collect::<BTreeSet<_>>() == BTreeSet::from([seed, first, second])))
        && segments.len() <= 3 && segments.iter().copied().collect::<BTreeSet<_>>().len() == segments.len()
        && segments.iter().all(|id| get(*id).is_some_and(|o| matches!(o.kind, Kind::Segment(a,b) if a != b && [seed,first,second].contains(&a) && [seed,first,second].contains(&b))))
}

/// Simultaneous point/scalar substitutions followed by reevaluation of the carried DAG.
pub(super) fn supported_mapped_point_iteration(
    objects: &[Object],
    seeds: &[usize],
    branches: &[Vec<usize>],
    carried: &[usize],
) -> bool {
    let Some(page) = seeds
        .first()
        .and_then(|id| id.checked_sub(1))
        .and_then(|at| objects.get(at))
        .map(|object| object.page)
    else {
        return false;
    };
    let get = |id: usize| {
        id.checked_sub(1)
            .and_then(|i| objects.get(i))
            .filter(|o| !o.deleted && o.page == page)
    };
    if !(1..=16).contains(&seeds.len())
        || !(1..=8).contains(&branches.len())
        || carried.is_empty()
        || seeds.iter().copied().collect::<BTreeSet<_>>().len() != seeds.len()
        || carried.iter().copied().collect::<BTreeSet<_>>().len() != carried.len()
        || !seeds.iter().all(|id| {
            get(*id).is_some_and(|o| {
                is_point_kind(&o.kind) || o.kind.numeric_unit() == Some(NumericUnit::Scalar)
            })
        })
        || branches.iter().any(|b| {
            b.len() != seeds.len()
                || !b
                    .iter()
                    .zip(seeds)
                    .all(|(id, seed)| match (get(*id), get(*seed)) {
                        (Some(image), Some(seed)) if is_point_kind(&seed.kind) => {
                            is_point_kind(&image.kind)
                        }
                        (Some(image), Some(_)) => {
                            image.kind.numeric_unit() == Some(NumericUnit::Scalar)
                        }
                        _ => false,
                    })
        })
    {
        return false;
    }
    if seeds.iter().any(|id| {
        get(*id).is_none_or(|object| {
            object
                .construction_dependencies()
                .iter()
                .any(|parent| *parent >= *id || get(*parent).is_none())
        })
    }) {
        return false;
    }
    let mut pending: Vec<_> = branches.iter().flatten().chain(carried).copied().collect();
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(object) = get(id) else {
            return false;
        };
        if object
            .construction_dependencies()
            .iter()
            .any(|parent| *parent >= id || get(*parent).is_none())
        {
            return false;
        }
        // Substitution replaces this construction's value, so its original
        // predecessors do not participate in subsequent generations.
        if seeds.contains(&id) {
            continue;
        }
        if let Some(definition) = object.kind.transform_definition() {
            if !mapped_transform_supported(objects, &definition) {
                return false;
            }
            pending.extend(definition.dependencies());
            continue;
        }
        if !mapped_value_construction(&object.kind) {
            return false;
        }
        pending.extend(object.construction_dependencies());
    }
    true
}

// State values are runtime substitutions, never additional native object kinds.
#[derive(Clone, Copy)]
pub(super) enum MappedValue {
    Point(Point),
    Scalar(f64),
}

impl MappedValue {
    pub(super) fn read(document: &Document, id: usize) -> Option<Self> {
        let object = document.objects.get(id.checked_sub(1)?)?;
        if is_point_kind(&object.kind) {
            let point = document.position(id)?;
            (point.x.is_finite() && point.y.is_finite()).then_some(Self::Point(point))
        } else {
            let value = document.unitless_numeric_value(id)?;
            value.is_finite().then_some(Self::Scalar(value))
        }
    }

    pub(super) fn substitute(self, document: &mut Document, id: usize) -> Option<()> {
        match self {
            Self::Point(point) => document.set_evaluation_point(id, point),
            Self::Scalar(value) => document.set_evaluation_scalar(id, value),
        }
    }

    fn key(self) -> (u8, u64, u64) {
        match self {
            Self::Point(point) => (0, point.x.to_bits(), point.y.to_bits()),
            Self::Scalar(value) => (1, value.to_bits(), 0),
        }
    }
}

#[derive(Clone)]
struct IterationMapping {
    seeds: Vec<usize>,
    branches: Vec<Vec<usize>>,
    carried: Vec<usize>,
    depth: IterationDepth,
    final_only: bool,
}

fn iteration_mapping(kind: &Kind) -> Option<IterationMapping> {
    let (seeds, branches, carried, depth, final_only) = match kind {
        Kind::MappedPointIteration {
            seeds,
            branches,
            carried,
            depth,
            final_only,
        } => (
            seeds.clone(),
            branches.clone(),
            carried.clone(),
            *depth,
            *final_only,
        ),
        Kind::PointIteration(seed, image, depth) => {
            (vec![*seed], vec![vec![*image]], vec![*image], *depth, false)
        }
        Kind::SegmentIteration(seed, image, host, depth)
        | Kind::CircleIteration(seed, image, host, depth) => (
            vec![*seed],
            vec![vec![*image]],
            vec![*image, *host],
            *depth,
            false,
        ),
        Kind::RayIteration(seed, image, host, depth) => (
            vec![*seed],
            vec![vec![*image]],
            vec![*image, *host],
            IterationDepth::Value(*depth),
            false,
        ),
        Kind::BranchingPolygonIteration {
            seed,
            first,
            second,
            polygon,
            segments,
            depth,
        } => {
            let mut carried = vec![*second, *first, *polygon];
            carried.extend(segments);
            (
                vec![*seed],
                vec![vec![*first], vec![*second]],
                carried,
                *depth,
                false,
            )
        }
        _ => return None,
    };
    Some(IterationMapping {
        seeds,
        branches,
        carried,
        depth,
        final_only,
    })
}

/// Canonicalize the legacy native layouts that already have supported adapters.
/// Wider carried graphs use the same mapped representation for one or many seeds.
fn mapped_iteration_kind(
    objects: &[Object],
    seeds: &[usize],
    branches: &[Vec<usize>],
    carried: &[usize],
    depth: IterationDepth,
    final_only: bool,
) -> Kind {
    if !final_only
        && let [seed, parameter] = seeds
        && let [branch] = branches
        && let [image, calculation] = branch.as_slice()
        && let [carried_value, carried_point, label] = carried
        && calculation == carried_value && image == carried_point
        && !matches!(depth, IterationDepth::Value(id) if id == *parameter)
        && supported_point_parameter_iteration(objects,*seed,*parameter,*image,*calculation,*label)
    {
        return Kind::PointParameterIteration(*seed,*parameter,*image,*calculation,*label,depth);
    }
    if !final_only && let [seed] = seeds {
        if let [branch] = branches
            && let [image] = branch.as_slice()
            && objects
                .get(seed.saturating_sub(1))
                .is_some_and(|o| matches!(o.kind, Kind::Free(_)))
            && supported_point_iteration_mapping(objects, *seed, *image)
        {
            match carried {
                [mapped] if mapped == image => return Kind::PointIteration(*seed, *image, depth),
                [mapped, host] if mapped == image => {
                    match objects.get(host.saturating_sub(1)).map(|o| &o.kind) {
                        Some(Kind::Segment(a, b)) if a == seed && b == image => {
                            return Kind::SegmentIteration(*seed, *image, *host, depth);
                        }
                        Some(Kind::Circle(a, b)) if a == seed && b == image => {
                            return Kind::CircleIteration(*seed, *image, *host, depth);
                        }
                        Some(Kind::Ray(a, b)) if a == seed && b == image => {
                            if let IterationDepth::Value(value) = depth {
                                return Kind::RayIteration(*seed, *image, *host, value);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if let [first_branch, second_branch] = branches
            && let ([first], [second]) = (first_branch.as_slice(), second_branch.as_slice())
            && let [carried_second, carried_first, polygon, segments @ ..] = carried
            && first == carried_first
            && second == carried_second
            && supported_branch_polygon_mapping(objects, *seed, *first, *second, *polygon, segments)
        {
            return Kind::BranchingPolygonIteration {
                seed: *seed,
                first: *first,
                second: *second,
                polygon: *polygon,
                segments: segments.to_vec(),
                depth,
            };
        }
    }
    Kind::MappedPointIteration {
        seeds: seeds.to_vec(),
        branches: branches.to_vec(),
        carried: carried.to_vec(),
        depth,
        final_only,
    }
}

impl Document {
    pub(super) fn iteration_binding_template(&self, image: usize, iteration: usize) -> &[u8] {
        let source = &self.objects[image - 1].kind;
        if matches!(
            self.objects[iteration - 1].kind,
            Kind::MappedPointIteration { .. }
        ) {
            return if matches!(source,Kind::PointNumericLabel(..)) {
                include_bytes!("../../assets/point_parameter_iteration_label_binding.bin")
            } else if source.is_polygon() {
                include_bytes!("../../assets/branch_polygon_iteration_polygon_binding.bin")
            } else if is_point_kind(source) {
                if matches!(
                    self.objects[iteration - 1].kind,
                    Kind::MappedPointIteration {
                        depth: IterationDepth::Value(_),
                        ..
                    }
                ) {
                    include_bytes!("../../assets/segment_value_depth_point_binding.bin")
                } else {
                    include_bytes!("../../assets/point_iteration_binding_group.bin")
                }
            } else {
                include_bytes!("../../assets/segment_iteration_binding_group.bin")
            };
        }
        let segment = self
            .objects
            .get(image - 1)
            .is_some_and(|object| mapped_segment_binding(&object.kind));
        let circle = matches!(
            self.objects.get(image - 1).map(|o| &o.kind),
            Some(Kind::Circle(_, _))
        );
        let ray = matches!(
            self.objects.get(image - 1).map(|object| &object.kind),
            Some(Kind::Ray(_, _))
        );
        let ray_iteration = matches!(
            self.objects.get(iteration - 1).map(|object| &object.kind),
            Some(Kind::RayIteration(_, _, _, _))
        );
        let numeric_iteration = matches!(
            self.objects.get(iteration - 1).map(|o| &o.kind),
            Some(Kind::PointParameterIteration(_, _, _, _, _, _))
        );
        let branch_polygon = matches!(
            self.objects.get(iteration - 1).map(|o| &o.kind),
            Some(Kind::BranchingPolygonIteration { .. } | Kind::MappedPointIteration { .. })
        );
        let segment_value_depth = matches!(
            self.objects.get(iteration - 1).map(|o| &o.kind),
            Some(
                Kind::MappedPointIteration {
                    depth: IterationDepth::Value(_),
                    ..
                } | Kind::SegmentIteration(_, _, _, IterationDepth::Value(_))
                    | Kind::CircleIteration(_, _, _, IterationDepth::Value(_))
                    | Kind::PointIteration(_, _, IterationDepth::Value(_))
            )
        );
        let group: &[u8] = if segment_value_depth {
            if segment || circle {
                include_bytes!("../../assets/segment_value_depth_segment_binding.bin")
            } else {
                include_bytes!("../../assets/segment_value_depth_point_binding.bin")
            }
        } else if branch_polygon
            && crate::document::kind::is_point_kind(&self.objects[image - 1].kind)
        {
            include_bytes!("../../assets/point_iteration_binding_group.bin")
        } else if branch_polygon {
            if segment {
                include_bytes!("../../assets/branch_polygon_iteration_segment_binding.bin")
            } else {
                include_bytes!("../../assets/branch_polygon_iteration_polygon_binding.bin")
            }
        } else if numeric_iteration
            && matches!(
                self.objects.get(image - 1).map(|o| &o.kind),
                Some(Kind::PointNumericLabel(_, _))
            )
        {
            include_bytes!("../../assets/point_parameter_iteration_label_binding.bin")
        } else if numeric_iteration {
            include_bytes!("../../assets/point_parameter_iteration_point_binding.bin")
        } else if ray {
            include_bytes!("../../assets/ray_iteration_binding_group.bin")
        } else if ray_iteration {
            include_bytes!("../../assets/ray_iteration_point_binding_group.bin")
        } else if segment || circle {
            include_bytes!("../../assets/segment_iteration_binding_group.bin")
        } else {
            include_bytes!("../../assets/point_iteration_binding_group.bin")
        };
        group
    }
}

impl Document {
    pub fn add_mapped_point_iteration(
        &mut self,
        seeds: &[usize],
        branches: &[Vec<usize>],
        carried: &[usize],
        depth: IterationDepth,
    ) -> Result<usize, String> {
        let bindings: Vec<_> = carried
            .iter()
            .copied()
            .filter(|id| {
                self.objects
                    .get(id.saturating_sub(1))
                    .is_some_and(|object| {
                        !is_point_kind(&object.kind) && mapped_binding_kind(&object.kind)
                    })
            })
            .collect();
        self.add_mapped_point_iteration_with_bindings(seeds, branches, carried, &bindings, depth)
    }

    /// Choose visible carried bindings explicitly: native carried references also include prerequisites.
    pub fn add_mapped_point_iteration_with_bindings(
        &mut self,
        seeds: &[usize],
        branches: &[Vec<usize>],
        carried: &[usize],
        bindings: &[usize],
        depth: IterationDepth,
    ) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || self.numeric_iteration_depth(depth).is_none()
            || !supported_mapped_point_iteration(&self.objects, seeds, branches, carried)
            || bindings.is_empty()
            || bindings.iter().copied().collect::<BTreeSet<_>>().len() != bindings.len()
            || bindings.iter().any(|id| {
                !carried.contains(id)
                    || !self
                        .objects
                        .get(id.saturating_sub(1))
                        .is_some_and(|object| mapped_binding_kind(&object.kind))
            })
        {
            return Err("Select supported simultaneous point/scalar mappings and carried geometry bindings; depth must be 1–128".into());
        }
        // Native reconstructs carried nodes in packet order. Keep the caller's
        // order among ready nodes, while placing carried predecessors first.
        let mut ordered = Vec::with_capacity(carried.len());
        let mut pending = carried.to_vec();
        while !pending.is_empty() {
            let index = pending
                .iter()
                .position(|id| {
                    self.objects[*id - 1]
                        .construction_dependencies()
                        .iter()
                        .all(|parent| !pending.contains(parent))
                })
                .ok_or("Carried constructions must form an acyclic graph")?;
            ordered.push(pending.remove(index));
        }
        let carried = ordered.as_slice();
        self.checkpoint();
        let kind = mapped_iteration_kind(&self.objects, seeds, branches, carried, depth, false);
        let id = self.append_object(kind);
        for source in bindings {
            let style = self.objects[*source - 1].visual_style;
            let binding = self.append_object(Kind::IterationBinding(*source, id));
            self.objects[binding - 1].visual_style = style;
        }
        Ok(id)
    }

    pub fn set_mapped_iteration_final_only(
        &mut self,
        id: usize,
        final_only: bool,
    ) -> Result<(), String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot edit iteration image levels in this sketch safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid iteration ID")?)
            .filter(|o| !o.deleted && o.page == self.active_page)
            .ok_or("Select an iteration on the active page")?;
        let mut mapping =
            iteration_mapping(&object.kind).ok_or("Select a supported mapped iteration")?;
        if mapping.final_only == final_only {
            return Ok(());
        }
        if !supported_mapped_point_iteration(
            &self.objects,
            &mapping.seeds,
            &mapping.branches,
            &mapping.carried,
        ) {
            return Err("Select a supported mapped iteration".into());
        }
        mapping.final_only = final_only;
        let mut candidate = self.clone();
        candidate.checkpoint();
        candidate.objects[id - 1].kind = mapped_iteration_kind(
            &candidate.objects,
            &mapping.seeds,
            &mapping.branches,
            &mapping.carried,
            mapping.depth,
            mapping.final_only,
        );
        candidate.verify_roundtrip()?;
        *self = candidate;
        Ok(())
    }

    fn mapped_segment_iteration_images(
        &self,
        source: usize,
        iteration: usize,
    ) -> Option<Vec<(Point, Point)>> {
        self.mapped_iteration_images(source, iteration, |sample, id| {
            sample.line_points(id).map(|(a, b, _)| (a, b))
        })
    }

    pub fn mapped_point_binding_images(&self, id: usize) -> Option<Vec<Point>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || !self
                .objects
                .get(source.checked_sub(1)?)
                .is_some_and(|o| is_point_kind(&o.kind))
        {
            return None;
        }
        match &self.objects.get(iteration.checked_sub(1)?)?.kind {
            Kind::PointParameterIteration(_, _, image, _, _, _)
            | Kind::PointIteration(_, image, _)
            | Kind::SegmentIteration(_, image, _, _)
            | Kind::CircleIteration(_, image, _, _)
            | Kind::RayIteration(_, image, _, _)
                if *image == source =>
            {
                self.point_iteration_images(iteration)
            }
            Kind::BranchingPolygonIteration { first, second, .. }
                if [*first, *second].contains(&source) =>
            {
                let point = self.position(source)?;
                Some(
                    self.branch_iteration_offsets(iteration)?
                        .into_iter()
                        .map(|(delta, _)| Point {
                            x: point.x + delta.x,
                            y: point.y + delta.y,
                        })
                        .collect(),
                )
            }
            _ => self.mapped_iteration_images(source, iteration, |sample, id| sample.position(id)),
        }
    }

    pub(super) fn mapped_external_inputs(
        &self,
        seeds: &[usize],
        branches: &[Vec<usize>],
        carried: &[usize],
    ) -> Option<Vec<usize>> {
        let carried: BTreeSet<_> = carried.iter().copied().collect();
        let seeds: BTreeSet<_> = seeds.iter().copied().collect();
        let mut pending: Vec<_> = branches
            .iter()
            .flatten()
            .copied()
            .chain(carried.iter().copied())
            .collect();
        let mut seen = BTreeSet::new();
        let mut external = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id) || seeds.contains(&id) {
                continue;
            }
            let object = self.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            if !carried.contains(&id) {
                // An external construction retains its original value, including
                // its complete predecessor environment, throughout this sequence.
                external.insert(id);
            } else {
                pending.extend(object.construction_dependencies());
            }
        }
        Some(external.into_iter().collect())
    }

    /// Reevaluate anchored numeric text in the same substituted scope as carried geometry.
    pub fn mapped_numeric_label_images(&self, id: usize) -> Option<Vec<(Point, String)>> {
        let object=self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source,iteration)=object.kind else { return None; };
        if object.deleted || !matches!(self.objects.get(source.checked_sub(1)?)?.kind,Kind::PointNumericLabel(..)) { return None; }
        if matches!(self.objects.get(iteration.checked_sub(1)?)?.kind,Kind::PointParameterIteration(_,_,_,_,label,_) if label==source) {
            return self.point_parameter_iteration_labels(iteration);
        }
        self.mapped_iteration_images(source,iteration,|sample,source| {
            sample.frozen_evaluation_document(source).unwrap_or(sample).point_numeric_label(source)
        })
    }
    fn mapped_iteration_images<T>(
        &self,
        source: usize,
        iteration: usize,
        mut evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<T>> {
        let object = self.objects.get(iteration.checked_sub(1)?)?;
        let Kind::MappedPointIteration {
            seeds,
            branches,
            carried,
            depth,
            final_only,
        } = &object.kind
        else {
            return None;
        };
        if object.deleted || !carried.contains(&source) {
            return None;
        }
        let depth = self.numeric_iteration_depth(*depth)?;
        let initial: Vec<_> = seeds
            .iter()
            .map(|id| MappedValue::read(self, *id))
            .collect::<Option<_>>()?;
        let mut level = vec![initial];
        let mut sample = self.clone();
        let external = self.mapped_external_inputs(seeds, branches, carried)?;
        sample.freeze_evaluation_inputs(self, &external)?;
        let mut result = Vec::new();
        let mut generated = 0;
        for step in 0..depth {
            let mut next = Vec::new();
            let mut seen = BTreeSet::new();
            for state in &level {
                sample.clear_evaluation_seeds();
                for (id, value) in seeds.iter().zip(state) {
                    value.substitute(&mut sample, *id)?;
                }
                for branch in branches {
                    let mapped: Vec<_> = branch
                        .iter()
                        .map(|id| MappedValue::read(&sample, *id))
                        .collect::<Option<_>>()?;
                    let key: Vec<_> = mapped.iter().map(|value| value.key()).collect();
                    if seen.insert(key) {
                        next.push(mapped);
                    }
                }
            }
            // Avoid exponential resource use on mappings with no coincident states.
            if next.len() > 20_000 || generated + next.len() > 100_000 {
                return None;
            }
            generated += next.len();
            if !*final_only || step + 1 == depth {
                for state in &next {
                    sample.clear_evaluation_seeds();
                    for (id, value) in seeds.iter().zip(state) {
                        value.substitute(&mut sample, *id)?;
                    }
                    // Undefined carried geometry hides only that image. Other
                    // bindings and earlier valid generations remain available.
                    if let Some(value) = evaluate(&sample, source) {
                        result.push(value);
                    }
                }
            }
            level = next;
        }
        Some(result)
    }
    pub fn add_point_iteration(
        &mut self,
        seed: usize,
        image: usize,
        depth: IterationDepth,
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add an iteration to this sketch safely".into());
        }
        if self.numeric_iteration_depth(depth).is_none()
            || !self
                .objects
                .get(seed.saturating_sub(1))
                .is_some_and(|o| !o.deleted && matches!(o.kind, Kind::Free(_)))
            || !supported_point_iteration_mapping(&self.objects, seed, image)
            || seed >= image
        {
            return Err(
                "Select a free point and its directly translated, rotated, dilated, or reflected image; depth must be 1–128".into(),
            );
        }
        self.checkpoint();
        let iteration = self.append_object(Kind::PointIteration(seed, image, depth));
        self.append_object(Kind::IterationBinding(image, iteration));
        Ok(iteration)
    }
    pub fn add_branching_polygon_iteration(
        &mut self,
        seed: usize,
        first: usize,
        second: usize,
        polygon: usize,
        segments: &[usize],
        depth: IterationDepth,
    ) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || self.numeric_iteration_depth(depth).is_none()
            || !supported_branch_polygon_mapping(
                &self.objects,
                seed,
                first,
                second,
                polygon,
                segments,
            )
        {
            return Err("Select a free seed, two translated images, their triangular interior, and optionally its sides and a scalar depth parameter; truncated depth must be 1–128".into());
        }
        self.checkpoint();
        let iteration = self.append_object(Kind::BranchingPolygonIteration {
            seed,
            first,
            second,
            polygon,
            segments: segments.to_vec(),
            depth,
        });
        for source in std::iter::once(polygon).chain(segments.iter().copied()) {
            let style = self.objects[source - 1].visual_style;
            let id = self.append_object(Kind::IterationBinding(source, iteration));
            self.objects[id - 1].visual_style = style;
        }
        Ok(iteration)
    }
    /// Translation offsets, with the number of branch paths having each pair of step counts.
    pub fn branch_iteration_offsets(&self, id: usize) -> Option<Vec<(Point, f64)>> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::BranchingPolygonIteration {
            seed,
            first,
            second,
            depth,
            ..
        } = o.kind
        else {
            return None;
        };
        if o.deleted {
            return None;
        }
        let depth = self.numeric_iteration_depth(depth)?;
        let seed = self.position(seed)?;
        let a = self.position(first)?;
        let b = self.position(second)?;
        let (a, b) = (
            Point {
                x: a.x - seed.x,
                y: a.y - seed.y,
            },
            Point {
                x: b.x - seed.x,
                y: b.y - seed.y,
            },
        );
        let mut result = Vec::new();
        for level in 1..=depth {
            let mut paths = 1.;
            for second_steps in 0..=level {
                let first_steps = level - second_steps;
                result.push((
                    Point {
                        x: a.x * f64::from(first_steps) + b.x * f64::from(second_steps),
                        y: a.y * f64::from(first_steps) + b.y * f64::from(second_steps),
                    },
                    paths,
                ));
                paths *= f64::from(level - second_steps) / f64::from(second_steps + 1);
            }
        }
        Some(result)
    }
    pub fn polygon_iteration_images(&self, id: usize) -> Option<Vec<(Vec<Point>, f64)>> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = o.kind else {
            return None;
        };
        if !o.deleted
            && self.objects.get(source.checked_sub(1)?)?.kind.is_polygon()
            && matches!(
                self.objects.get(iteration.checked_sub(1)?)?.kind,
                Kind::MappedPointIteration { .. }
            )
        {
            return self
                .mapped_iteration_images(source, iteration, |sample, id| sample.polygon_points(id))
                .map(|images| images.into_iter().map(|points| (points, 1.)).collect());
        }
        if o.deleted
            || !matches!(self.objects.get(iteration.checked_sub(1)?)?.kind,Kind::BranchingPolygonIteration { polygon,.. } if polygon==source)
        {
            return None;
        }
        let points = self.polygon_points(source)?;
        Some(
            self.branch_iteration_offsets(iteration)?
                .into_iter()
                .map(|(delta, paths)| {
                    (
                        points
                            .iter()
                            .map(|p| Point {
                                x: p.x + delta.x,
                                y: p.y + delta.y,
                            })
                            .collect(),
                        paths,
                    )
                })
                .collect(),
        )
    }
    pub fn branch_segment_iteration_images(&self, id: usize) -> Option<Vec<(Point, Point)>> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = o.kind else {
            return None;
        };
        if !o.deleted
            && matches!(self.objects.get(iteration.checked_sub(1)?)?.kind,
            Kind::SegmentIteration(_,_,segment,_) if segment == source)
        {
            return self.segment_iteration_images(iteration);
        }
        if !o.deleted
            && matches!(
                self.objects.get(iteration.checked_sub(1)?)?.kind,
                Kind::MappedPointIteration { .. }
            )
        {
            if !self.objects.get(source.checked_sub(1)?)?.kind.is_segment() {
                return None;
            }
            return self.mapped_segment_iteration_images(source, iteration);
        }
        if o.deleted
            || !matches!(&self.objects.get(iteration.checked_sub(1)?)?.kind,Kind::BranchingPolygonIteration { segments,.. } if segments.contains(&source))
        {
            return None;
        }
        let (a, b, _) = self.line_points(source)?;
        Some(
            self.branch_iteration_offsets(iteration)?
                .into_iter()
                .map(|(delta, _)| {
                    (
                        Point {
                            x: a.x + delta.x,
                            y: a.y + delta.y,
                        },
                        Point {
                            x: b.x + delta.x,
                            y: b.y + delta.y,
                        },
                    )
                })
                .collect(),
        )
    }
    pub fn add_point_parameter_iteration(
        &mut self,
        seed: usize,
        parameter: usize,
        image: usize,
        calculation: usize,
        label: usize,
        depth: IterationDepth,
    ) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || self.numeric_iteration_depth(depth).is_none()
            || matches!(depth, IterationDepth::Value(id) if id == parameter)
            || !supported_point_parameter_iteration(
                &self.objects,
                seed,
                parameter,
                image,
                calculation,
                label,
            )
        {
            return Err("Select a free point, parameter, transformed image, direct parameter calculation, and its attached numeric label; depth must be 1–128 or a distinct scalar parameter whose truncated value is in that range".into());
        }
        self.checkpoint();
        let iteration = self.append_object(Kind::PointParameterIteration(
            seed,
            parameter,
            image,
            calculation,
            label,
            depth,
        ));
        self.append_object(Kind::IterationBinding(image, iteration));
        self.append_object(Kind::IterationBinding(label, iteration));
        let values = self.append_object(Kind::IterationValues(iteration, calculation));
        self.objects[values - 1].visible = false;
        Ok(iteration)
    }
    pub(super) fn numeric_iteration_depth(&self, depth: IterationDepth) -> Option<u32> {
        match depth {
            IterationDepth::Fixed(value) => (1..=128).contains(&value).then_some(value),
            IterationDepth::Value(id) => self.iteration_depth_value(id),
        }
    }
    pub fn point_parameter_iteration_labels(&self, id: usize) -> Option<Vec<(Point, String)>> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PointParameterIteration(_, parameter, _, calculation, _, _) = o.kind else {
            return None;
        };
        if o.deleted {
            return None;
        }
        let Kind::Parameter(group) = &self.objects.get(parameter.checked_sub(1)?)?.kind else {
            return None;
        };
        let Kind::ParameterCalculation(first, formula) =
            &self.objects.get(calculation.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let sources = calculation_source_ids(*first, formula);
        let units =
            super::calculations::calculation_source_units(formula, &sources, &self.objects)?;
        let mut values = sources
            .iter()
            .zip(formula.source_kinds().ok()?)
            .map(|(source, kind)| {
                if kind == crate::table_records::CalculationSourceKind::Function {
                    Some(0.)
                } else {
                    self.numeric_value(*source)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        let mapped_inputs = sources
            .iter()
            .enumerate()
            .filter_map(|(index, source)| (*source == parameter).then_some(index))
            .collect::<Vec<_>>();
        if mapped_inputs.is_empty() {
            return None;
        }
        let mut value = group.value();
        // Bound functions and their numeric predecessors must see the mapped
        // parameter at each step, while the source document remains unchanged.
        let mut sample = self.clone();
        let mut mapped_parameter = group.clone();
        let mut next_value = || {
            if !mapped_parameter.set_evaluation_value(value) {
                return None;
            }
            sample.objects[parameter - 1].kind = Kind::Parameter(mapped_parameter.clone());
            for index in &mapped_inputs {
                values[*index] = value;
            }
            value = formula.value_for_sources_with_functions(
                &values,
                &units,
                &|index, argument, unit| {
                    sample.function_value_with_unit(*sources.get(index)?, argument, unit)
                },
            )?;
            Some(value)
        };
        next_value()?;
        self.point_iteration_images(id)?
            .into_iter()
            .map(|point| {
                let value = next_value()?;
                Some((point, formula.value_text(value)))
            })
            .collect()
    }
    pub fn add_segment_iteration(
        &mut self,
        seed: usize,
        image: usize,
        segment: usize,
        depth: IterationDepth,
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add an iteration to this sketch safely".into());
        }
        if self.numeric_iteration_depth(depth).is_none()
            || !self
                .objects
                .get(seed.saturating_sub(1))
                .is_some_and(|o| !o.deleted && matches!(o.kind, Kind::Free(_)))
            || !supported_point_iteration_mapping(&self.objects, seed, image)
            || !self
                .objects
                .get(segment.saturating_sub(1))
                .is_some_and(|o| {
                    !o.deleted && matches!(o.kind, Kind::Segment(a, b) if a == seed && b == image)
                })
            || !(seed < image && image < segment)
        {
            return Err("Select a free point, its directly translated, rotated, dilated, or reflected image, and their segment; depth must be 1–128".into());
        }
        self.checkpoint();
        let iteration = self.append_object(Kind::SegmentIteration(seed, image, segment, depth));
        self.append_object(Kind::IterationBinding(image, iteration));
        self.append_object(Kind::IterationBinding(segment, iteration));
        Ok(iteration)
    }
    pub fn add_circle_iteration(
        &mut self,
        seed: usize,
        image: usize,
        circle: usize,
        depth: IterationDepth,
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add an iteration to this sketch safely".into());
        }
        if self.numeric_iteration_depth(depth).is_none()
            || !self
                .objects
                .get(seed.saturating_sub(1))
                .is_some_and(|o| !o.deleted && matches!(o.kind, Kind::Free(_)))
            || !supported_point_iteration_mapping(&self.objects, seed, image)
            || !self.objects.get(circle.saturating_sub(1)).is_some_and(|o| {
                !o.deleted && matches!(o.kind, Kind::Circle(a, b) if a == seed && b == image)
            })
            || !(seed < image && image < circle)
        {
            return Err("Select a free point, its directly translated, rotated, dilated, or reflected image, and a circle centered on the seed through the image; depth must be 1–128".into());
        }
        self.checkpoint();
        let iteration = self.append_object(Kind::CircleIteration(seed, image, circle, depth));
        self.append_object(Kind::IterationBinding(image, iteration));
        let binding = self.append_object(Kind::IterationBinding(circle, iteration));
        self.objects[binding - 1].visual_style = style_records::iteration_binding_visual_style(
            &include_bytes!("../../assets/segment_iteration_binding_group.bin")[8..36],
            false,
        );
        Ok(iteration)
    }
    pub fn add_ray_iteration(
        &mut self,
        seed: usize,
        image: usize,
        ray: usize,
        parameter: usize,
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add an iteration to this sketch safely".into());
        }
        if self.iteration_depth_value(parameter).is_none()
            || !self
                .objects
                .get(seed.saturating_sub(1))
                .is_some_and(|o| !o.deleted && matches!(o.kind, Kind::Free(_)))
            || !supported_point_iteration_mapping(&self.objects, seed, image)
            || !self.objects.get(ray.saturating_sub(1)).is_some_and(|o| {
                !o.deleted && matches!(o.kind, Kind::Ray(a,b) if a == seed && b == image)
            })
            || !(seed < image && image < ray && parameter < self.objects.len() + 1)
        {
            return Err("Select a free point, its directly translated, rotated, dilated, or reflected image, their ray, and a scalar depth parameter/calculation from 1 to 128".into());
        }
        self.checkpoint();
        let iteration = self.append_object(Kind::RayIteration(seed, image, ray, parameter));
        self.append_object(Kind::IterationBinding(image, iteration));
        self.append_object(Kind::IterationBinding(ray, iteration));
        Ok(iteration)
    }
    fn iteration_image_at(&self, seed: usize, image: usize, current: Point) -> Option<Point> {
        // Substitute each direct seed reference. Other construction inputs are pinned.
        let point = |id: usize| {
            if id == seed {
                Some(current)
            } else {
                self.position(id)
            }
        };
        let translated = |dx: f64, dy: f64| Point {
            x: current.x + dx,
            y: current.y + dy,
        };
        let definition = point_iteration_definition(&self.objects, seed, image)?;
        // Numeric object values remain pinned to the original document. Only
        // direct point references use the current seed, including angle/ratio
        // construction inputs. A reflection mirror also remains pinned.
        let numeric = |expression: &NumericExpr| -> Option<f64> {
            let value = match *expression {
                NumericExpr::Literal {
                    magnitude, scale, ..
                } => Some(magnitude * scale),
                NumericExpr::ObjectValue { id, unit } => match unit {
                    NumericUnit::Length => self.marked_distance_value_pixels(id),
                    NumericUnit::Degrees => self.numeric_value(id),
                    NumericUnit::Scalar => self.unitless_numeric_value(id),
                    _ => None,
                },
                NumericExpr::DistanceRatio {
                    origin,
                    base,
                    target,
                } => ratio_of_points(point(origin)?, point(base)?, point(target)?),
                NumericExpr::DirectedAngle { .. } => None,
            }?;
            value.is_finite().then_some(value)
        };
        let angle_radians = |expression: &NumericExpr| match *expression {
            // Keep the original radians directly, avoiding a degrees/radians
            // roundtrip that would change existing orbit rounding.
            NumericExpr::DirectedAngle {
                first,
                vertex,
                last,
            } => marked_rotation_theta(point(first)?, point(vertex)?, point(last)?),
            _ => Some(numeric(expression)?.to_radians()),
        };
        let result = match definition.transform() {
            Transform::Translate(DisplacementExpr::Cartesian {
                horizontal,
                vertical,
            }) => translated(numeric(horizontal)?, -numeric(vertical)?),
            Transform::Translate(DisplacementExpr::Polar { angle, distance }) => {
                let theta = angle_radians(angle)?;
                let distance = numeric(distance)?;
                translated(distance * theta.cos(), -distance * theta.sin())
            }
            Transform::Translate(DisplacementExpr::Vector { start, end }) => {
                let (a, b) = (point(*start)?, point(*end)?);
                translated(b.x - a.x, b.y - a.y)
            }
            Transform::Rotate { center, angle } => {
                rotate_point_about(current, point(*center)?, angle_radians(angle)?)?
            }
            Transform::Dilate { center, ratio } => {
                scale_point_about(current, point(*center)?, numeric(ratio)?)?
            }
            Transform::Reflect { mirror } => {
                let (a, b, _) = self.line_points(*mirror)?;
                reflect_point(current, a, b)?
            }
        };
        (result.x.is_finite() && result.y.is_finite()).then_some(result)
    }
    fn iteration_orbit(&self, seed: usize, image: usize, depth: u32) -> Option<Vec<Point>> {
        if !supported_point_iteration_mapping(&self.objects, seed, image) {
            return None;
        }
        let mut orbit = vec![self.position(seed)?, self.position(image)?];
        if orbit
            .iter()
            .any(|point| !point.x.is_finite() || !point.y.is_finite())
        {
            return None;
        }
        for _ in 0..depth {
            orbit.push(self.iteration_image_at(seed, image, *orbit.last()?)?);
        }
        Some(orbit)
    }
    pub fn point_iteration_images(&self, id: usize) -> Option<Vec<Point>> {
        if let Some(images) = self.mapped_point_binding_images(id) {
            return Some(images);
        }
        let (seed, image, depth) = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::PointIteration(seed, image, depth) => {
                (seed, image, self.numeric_iteration_depth(depth)?)
            }
            Kind::SegmentIteration(seed, image, _, depth)
            | Kind::CircleIteration(seed, image, _, depth) => {
                (seed, image, self.numeric_iteration_depth(depth)?)
            }
            Kind::PointParameterIteration(seed, _, image, _, _, depth) => {
                (seed, image, self.numeric_iteration_depth(depth)?)
            }
            Kind::RayIteration(seed, image, _, parameter) => {
                (seed, image, self.iteration_depth_value(parameter)?)
            }
            _ => return None,
        };
        Some(
            self.iteration_orbit(seed, image, depth)?
                .into_iter()
                .skip(2)
                .collect(),
        )
    }

    pub(super) fn iteration_depth_value(&self, id: usize) -> Option<u32> {
        if !self
            .objects
            .get(id.checked_sub(1)?)?
            .kind
            .is_iteration_depth_value()
        {
            return None;
        }
        let value = self.unitless_numeric_value(id)?;
        let depth = value.trunc();
        (1. ..=128.).contains(&depth).then_some(depth as u32)
    }
    fn mapped_straight_binding_images(
        &self,
        id: usize,
        mode: super::LineMode,
    ) -> Option<Vec<(Point, Point)>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || straight_kind_mode(&self.objects.get(source.checked_sub(1)?)?.kind) != Some(mode)
        {
            return None;
        }
        self.mapped_iteration_images(source, iteration, |sample, source| {
            sample.line_points(source).map(|(a, b, _)| (a, b))
        })
    }
    pub fn line_iteration_images(&self, id: usize) -> Option<Vec<(Point, Point)>> {
        self.mapped_straight_binding_images(id, super::LineMode::Line)
    }
    pub fn arc_iteration_images(&self, id: usize, subdivisions: usize) -> Option<Vec<Vec<Point>>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted || !is_arc_kind(&self.objects.get(source.checked_sub(1)?)?.kind) {
            return None;
        }
        self.mapped_iteration_images(source, iteration, |sample, source| {
            sample.arc_points(source, subdivisions)
        })
    }
    pub fn ray_iteration_images(&self, id: usize) -> Option<Vec<(Point, Point)>> {
        if let Some(images) = self.mapped_straight_binding_images(id, super::LineMode::Ray) {
            return Some(images);
        }
        let Kind::RayIteration(seed, image, ray, parameter) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        if !matches!(self.objects.get(ray.checked_sub(1)?)?.kind, Kind::Ray(a,b) if a == seed && b == image)
        {
            return None;
        }
        let depth = self.iteration_depth_value(parameter)?;
        self.carried_line_iteration_images(seed, image, depth)
    }
    pub fn segment_iteration_images(&self, id: usize) -> Option<Vec<(Point, Point)>> {
        let Kind::SegmentIteration(seed, image, segment, depth) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        if !matches!(self.objects.get(segment - 1)?.kind, Kind::Segment(a, b) if a == seed && b == image)
        {
            return None;
        }
        let depth = self.numeric_iteration_depth(depth)?;
        self.carried_line_iteration_images(seed, image, depth)
    }
    pub fn circle_iteration_images(&self, id: usize) -> Option<Vec<(Point, f64)>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, parent) = object.kind else {
            return None;
        };
        let iteration = self.objects.get(parent.checked_sub(1)?)?;
        if !object.deleted
            && self.objects.get(source.checked_sub(1)?)?.kind.is_circle()
            && matches!(iteration.kind, Kind::MappedPointIteration { .. })
        {
            return self.mapped_iteration_images(source, parent, |sample, source| {
                sample.circle_points(source)
            });
        }
        let Kind::CircleIteration(seed, image, circle, depth) = iteration.kind else {
            return None;
        };
        if object.deleted
            || iteration.deleted
            || source != circle
            || !matches!(self.objects.get(circle.checked_sub(1)?)?.kind,Kind::Circle(a,b) if a==seed&&b==image)
        {
            return None;
        }
        Some(
            self.carried_line_iteration_images(seed, image, self.numeric_iteration_depth(depth)?)?
                .into_iter()
                .filter_map(|(a, b)| {
                    let radius = a.distance(b);
                    (radius.is_finite() && radius > 0.).then_some((a, radius))
                })
                .collect(),
        )
    }
    fn carried_line_iteration_images(
        &self,
        seed: usize,
        image: usize,
        depth: u32,
    ) -> Option<Vec<(Point, Point)>> {
        let orbit = self.iteration_orbit(seed, image, depth)?;
        Some(
            orbit[1..]
                .windows(2)
                .map(|pair| (pair[0], pair[1]))
                .collect(),
        )
    }
}
