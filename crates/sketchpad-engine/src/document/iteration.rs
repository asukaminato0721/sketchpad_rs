use super::calculations::calculation_source_ids;
use super::geometry::{
    marked_rotation_theta, ratio_of_points, reflect_point, rotate_point_about, scale_point_about,
};
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind, straight_kind_mode};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeTransformFlavor, NumericExpr, Transform,
    TransformDefinition,
};
use super::{Document, EmbeddedToolInputKind, IterationDepth, Kind, Object, Point};
use crate::style_records;
use crate::style_records::RgbColor;
use crate::table_records::NumericUnit;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const MAX_MAPPED_ITERATION_DEPTH: u32 = 4096;

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
    let segment = |id: usize| get(id).is_some_and(|object| object.kind.is_segment());
    let numeric = |expression: &NumericExpr| match *expression {
        NumericExpr::Literal { .. } => true,
        NumericExpr::ObjectValue { id, unit } => {
            get(id).is_some_and(|object| object.kind.is_transform_operand(unit))
        }
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
        NumericExpr::SegmentRatio {
            numerator,
            denominator,
        } => [numerator, denominator].into_iter().all(segment),
    };

    match definition.transform() {
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
    }
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
        ElementType::Sector => source.kind.is_sector(),
        ElementType::CircularSegment => source.kind.is_circular_segment(),
        ElementType::Disk => source.kind.is_circle_interior(),
        ElementType::Image => false,
    };
    !source.deleted && source_matches && iteration_transform_inputs_supported(objects, definition)
}

/// A supported value construction can be reevaluated in the substituted sample.
/// Actions are capture-only dependencies and never binding kinds. Iteration
/// outputs and sampled loci have separate runtime semantics.
fn mapped_reference_value(kind: &Kind) -> bool {
    let text = match kind {
        Kind::PointReferencedText { text, .. } | Kind::ReferencedText { text, .. } => text,
        _ => return false,
    };
    // Names are stable presentation metadata while values share the mapped
    // live/frozen resolver. Other live roles have separate scope contracts.
    let roles = text.referenced_roles();
    roles.iter().all(|(_, role)| {
        matches!(
            role,
            super::referenced_text::ReferenceRole::Name
                | super::referenced_text::ReferenceRole::Value
        )
    }) && roles
        .iter()
        .any(|(_, role)| *role == super::referenced_text::ReferenceRole::Value)
}

/// Unbound captions can remain in the carried construction list without being
/// sampled as iteration images. Their names and expressions are presentation
/// metadata; any live values still use the proved mapped resolver above.
fn mapped_reference_construction(kind: &Kind) -> bool {
    let Kind::ReferencedText { text, .. } = kind else {
        return mapped_reference_value(kind);
    };
    text.referenced_roles().iter().all(|(_, role)| {
        matches!(
            role,
            super::referenced_text::ReferenceRole::Name
                | super::referenced_text::ReferenceRole::Value
                | super::referenced_text::ReferenceRole::Expression
        )
    })
}

fn mapped_value_construction(kind: &Kind) -> bool {
    !matches!(kind, Kind::ToolInput(_) | Kind::ToolPointOnPath(_))
        && (is_point_kind(kind)
            || is_straight_kind(kind)
            || kind.is_circle()
            || kind.is_filled_region()
            || is_arc_kind(kind)
            || kind.numeric_unit().is_some()
            || kind.is_function()
            || mapped_reference_construction(kind)
            || matches!(kind, Kind::Colorized(definition) if definition.element() == ElementType::Disk)
            || matches!(
                kind,
                Kind::CoordinateSystem(..)
                    | Kind::CoordinateMeasurement {
                        kind: crate::coordinate_measure_records::CoordinateMeasureKind::Coordinates,
                        ..
                    }
                    | Kind::PointNumericLabel(..)
                    | Kind::PlainText { .. }
                    | Kind::Image { .. }
                    | Kind::HideShowButton { .. }
                    | Kind::AnimationButton { .. }
                    | Kind::SoundButton { .. }
                    | Kind::MovementButton { .. }
                    | Kind::Locus(..)
                    | Kind::ParameterLocus(..)
                    | Kind::SampledMapped(..)
                    | Kind::FunctionPlot(..)
                    | Kind::MappedPointIteration { .. }
            ))
}

pub(super) fn mapped_binding_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::PointNumericLabel(..)
            | Kind::Image { .. }
            | Kind::Locus(..)
            | Kind::ParameterLocus(..)
            | Kind::SampledMapped(..)
            | Kind::FunctionPlot(..)
    ) || (matches!(kind, Kind::PointReferencedText { .. }) && mapped_reference_value(kind))
        || is_point_kind(kind)
        || is_straight_kind(kind)
        || kind.is_circle()
        || kind.is_filled_region()
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
    supported_mapped_point_iteration_impl(objects, seeds, branches, carried, false)
}

pub(super) fn supported_tool_mapped_point_iteration(
    objects: &[Object],
    seeds: &[usize],
    branches: &[Vec<usize>],
    carried: &[usize],
) -> bool {
    supported_mapped_point_iteration_impl(objects, seeds, branches, carried, true)
}

fn supported_mapped_point_iteration_impl(
    objects: &[Object],
    seeds: &[usize],
    branches: &[Vec<usize>],
    carried: &[usize],
    allow_tool_placeholders: bool,
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
    if !(1..=128).contains(&seeds.len())
        || !(1..=16).contains(&branches.len())
        || carried.is_empty()
        || seeds.iter().copied().collect::<BTreeSet<_>>().len() != seeds.len()
        || carried.iter().copied().collect::<BTreeSet<_>>().len() != carried.len()
        || !seeds.iter().all(|id| {
            get(*id).is_some_and(|o| {
                is_point_kind(&o.kind)
                    || o.kind.numeric_unit().is_some()
                    || allow_tool_placeholders
                        && matches!(o.kind, Kind::ToolInput(EmbeddedToolInputKind::Numeric))
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
                        (Some(image), Some(seed))
                            if allow_tool_placeholders
                                && matches!(
                                    seed.kind,
                                    Kind::ToolInput(EmbeddedToolInputKind::Numeric)
                                ) =>
                        {
                            image.kind.numeric_unit().is_some()
                                || matches!(
                                    image.kind,
                                    Kind::ToolInput(EmbeddedToolInputKind::Numeric)
                                )
                        }
                        (Some(image), Some(seed)) => {
                            match (seed.kind.numeric_unit(), image.kind.numeric_unit()) {
                                (Some(seed), Some(image)) => {
                                    // Native treats scalar and angular mapping
                                    // states as the same stored magnitude.
                                    seed == image
                                        || matches!(
                                            (seed, image),
                                            (NumericUnit::Scalar, NumericUnit::Degrees)
                                                | (NumericUnit::Degrees, NumericUnit::Scalar)
                                        )
                                }
                                _ => false,
                            }
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
    let Some(captured_structures) = super::nested_iteration_scope::captured_structures(
        objects,
        page,
        seeds,
        branches,
        carried,
        allow_tool_placeholders,
    ) else {
        return false;
    };
    let branch_ids = branches.iter().flatten().copied().collect::<BTreeSet<_>>();
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
        // Invocation materializes a tool path point as an ordinary constrained
        // point. It can therefore participate in the mapped DAG as a seed,
        // branch image, or carried construction with its host dependency.
        if allow_tool_placeholders && matches!(object.kind, Kind::ToolPointOnPath(_)) {
            pending.extend(object.construction_dependencies());
            continue;
        }
        // Tool placeholders outside the substituted construction are frozen at
        // invocation time. They may feed the carried DAG, but may not themselves
        // be a seed, branch, or carried output.
        if !carried.contains(&id)
            && !branch_ids.contains(&id)
            && (matches!(
                object.kind,
                Kind::ToolInput(EmbeddedToolInputKind::Polygon) | Kind::ToolPointOnPath(_)
            ) || allow_tool_placeholders && matches!(object.kind, Kind::ToolInput(_)))
        {
            pending.extend(object.construction_dependencies());
            continue;
        }
        if let Some(definition) = object.kind.transform_definition() {
            if !mapped_transform_supported(objects, &definition) {
                return false;
            }
            pending.extend(definition.dependencies());
            continue;
        }
        if matches!(object.kind, Kind::Locus(..) | Kind::ParameterLocus(..))
            && !super::locus::is_mapped_stroke_locus(objects, &object.kind)
            && !super::locus::is_point_locus(objects, &object.kind)
            && !super::locus::is_mapped_polygon_locus(objects, &object.kind)
        {
            return false;
        }
        if !mapped_value_construction(&object.kind) && !captured_structures.contains(&id) {
            return false;
        }
        pending.extend(object.construction_dependencies());
    }
    true
}

/// Code-92 value tables may accompany either the specialized point/parameter
/// iteration or a general mapped iteration.  Native stores only carried
/// numeric values and functions in these tables.
pub(super) fn supported_iteration_values(
    objects: &[Object],
    page: usize,
    id: usize,
    iteration: usize,
    values: &[usize],
) -> bool {
    if iteration == 0
        || iteration >= id
        || values.is_empty()
        || values.iter().copied().collect::<BTreeSet<_>>().len() != values.len()
    {
        return false;
    }
    let Some(parent) = objects.get(iteration - 1) else {
        return false;
    };
    if parent.deleted || parent.page != page {
        return false;
    }
    if values.iter().any(|value| {
        *value == 0
            || *value >= id
            || objects.get(value - 1).is_none_or(|object| {
                object.deleted
                    || object.page != page
                    || (object.kind.numeric_unit().is_none() && !object.kind.is_function())
            })
    }) {
        return false;
    }
    match &parent.kind {
        Kind::PointParameterIteration(_, _, _, value, _, _) => {
            values.len() == 1 && values[0] == *value
        }
        Kind::MappedPointIteration { carried, .. } => {
            values.iter().all(|value| carried.contains(value))
        }
        _ => false,
    }
}

// State values are runtime substitutions, never additional native object kinds.
#[derive(Clone, Copy)]
pub(super) enum MappedValue {
    Point(Point),
    Number(f64),
}

impl MappedValue {
    pub(super) fn read(document: &Document, id: usize) -> Option<Self> {
        let object = document.objects.get(id.checked_sub(1)?)?;
        if is_point_kind(&object.kind) {
            let point = document.position(id)?;
            (point.x.is_finite() && point.y.is_finite()).then_some(Self::Point(point))
        } else {
            let value = document.numeric_value(id)?;
            value.is_finite().then_some(Self::Number(value))
        }
    }

    pub(super) fn substitute(self, document: &mut Document, id: usize) -> Option<()> {
        match self {
            Self::Point(point) => document.set_evaluation_point(id, point),
            Self::Number(value) => document.set_evaluation_number(id, value),
        }
    }

    fn key(self) -> (u8, u64, u64) {
        match self {
            Self::Point(point) => (0, point.x.to_bits(), point.y.to_bits()),
            Self::Number(value) => (1, value.to_bits(), 0),
        }
    }
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
        && calculation == carried_value
        && image == carried_point
        && !matches!(depth, IterationDepth::Value(id) if id == *parameter)
        && supported_point_parameter_iteration(
            objects,
            *seed,
            *parameter,
            *image,
            *calculation,
            *label,
        )
    {
        return Kind::PointParameterIteration(
            *seed,
            *parameter,
            *image,
            *calculation,
            *label,
            depth,
        );
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
            return if matches!(source, Kind::SampledMapped(..)) {
                include_bytes!("../../../../assets/sampled_iteration_binding_group.bin")
            } else if matches!(source, Kind::Locus(..) | Kind::ParameterLocus(..)) {
                include_bytes!("../../../../assets/locus_iteration_binding_group.bin")
            } else if matches!(
                source,
                Kind::PointNumericLabel(..) | Kind::PointReferencedText { .. }
            ) {
                include_bytes!("../../../../assets/point_parameter_iteration_label_binding.bin")
            } else if source.is_filled_region() {
                include_bytes!("../../../../assets/branch_polygon_iteration_polygon_binding.bin")
            } else if is_point_kind(source) {
                if matches!(
                    self.objects[iteration - 1].kind,
                    Kind::MappedPointIteration {
                        depth: IterationDepth::Value(_),
                        ..
                    }
                ) {
                    include_bytes!("../../../../assets/segment_value_depth_point_binding.bin")
                } else {
                    include_bytes!("../../../../assets/point_iteration_binding_group.bin")
                }
            } else {
                include_bytes!("../../../../assets/segment_iteration_binding_group.bin")
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
                include_bytes!("../../../../assets/segment_value_depth_segment_binding.bin")
            } else {
                include_bytes!("../../../../assets/segment_value_depth_point_binding.bin")
            }
        } else if branch_polygon
            && crate::document::kind::is_point_kind(&self.objects[image - 1].kind)
        {
            include_bytes!("../../../../assets/point_iteration_binding_group.bin")
        } else if branch_polygon {
            if segment {
                include_bytes!("../../../../assets/branch_polygon_iteration_segment_binding.bin")
            } else {
                include_bytes!("../../../../assets/branch_polygon_iteration_polygon_binding.bin")
            }
        } else if numeric_iteration
            && matches!(
                self.objects.get(image - 1).map(|o| &o.kind),
                Some(Kind::PointNumericLabel(_, _))
            )
        {
            include_bytes!("../../../../assets/point_parameter_iteration_label_binding.bin")
        } else if numeric_iteration {
            include_bytes!("../../../../assets/point_parameter_iteration_point_binding.bin")
        } else if ray {
            include_bytes!("../../../../assets/ray_iteration_binding_group.bin")
        } else if ray_iteration {
            include_bytes!("../../../../assets/ray_iteration_point_binding_group.bin")
        } else if segment || circle {
            include_bytes!("../../../../assets/segment_iteration_binding_group.bin")
        } else {
            include_bytes!("../../../../assets/point_iteration_binding_group.bin")
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
            self.objects.edit(binding - 1).visual_style = style;
        }
        Ok(id)
    }

    fn iteration_root_id(&self, id: usize) -> Option<usize> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let root = match object.kind {
            Kind::IterationBinding(_, parent) => parent,
            _ if object.kind.iteration_depth().is_some() => id,
            _ => return None,
        };
        let root_object = self.objects.get(root.checked_sub(1)?)?;
        (!object.deleted
            && !root_object.deleted
            && object.page == self.active_page
            && root_object.page == self.active_page
            && root_object.kind.iteration_depth().is_some())
        .then_some(root)
    }

    /// Root ID, saved depth source, final-only mode, and the fixed-depth limit.
    pub fn iteration_properties(&self, id: usize) -> Option<(usize, IterationDepth, bool, u32)> {
        let root = self.iteration_root_id(id)?;
        let object = &self.objects[root - 1];
        let depth = object.kind.iteration_depth()?;
        let maximum = if matches!(object.kind, Kind::MappedPointIteration { .. }) {
            MAX_MAPPED_ITERATION_DEPTH
        } else {
            128
        };
        Some((root, depth, object.iteration_final_only, maximum))
    }

    /// Apply Native's Iteration panel without changing a live depth dependency.
    pub fn set_iteration_properties(
        &mut self,
        id: usize,
        fixed_depth: Option<u32>,
        final_only: bool,
    ) -> Result<bool, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot edit iteration properties in this sketch safely".into());
        }
        let root = self
            .iteration_root_id(id)
            .ok_or("Select an iteration image on the active page")?;
        let (_, depth, saved_final_only, maximum) = self
            .iteration_properties(root)
            .ok_or("Select a supported iteration")?;
        let requested_depth = match (depth, fixed_depth) {
            (IterationDepth::Fixed(current), None) => current,
            (IterationDepth::Fixed(_), Some(value)) if (1..=maximum).contains(&value) => value,
            (IterationDepth::Fixed(_), Some(_)) => {
                return Err(format!("Iteration count must be 1–{maximum}"));
            }
            (IterationDepth::Value(_), None) => 0,
            (IterationDepth::Value(_), Some(_)) => {
                return Err(
                    "A live iteration count must be edited through its source value".into(),
                );
            }
        };
        if saved_final_only == final_only
            && !matches!(depth, IterationDepth::Fixed(current) if current != requested_depth)
        {
            return Ok(false);
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        let mut object = candidate.objects.edit(root - 1);
        object.iteration_final_only = final_only;
        match &mut object.kind {
            Kind::PointParameterIteration(_, _, _, _, _, depth)
            | Kind::PointIteration(_, _, depth)
            | Kind::SegmentIteration(_, _, _, depth)
            | Kind::CircleIteration(_, _, _, depth) => {
                if matches!(depth, IterationDepth::Fixed(_)) {
                    *depth = IterationDepth::Fixed(requested_depth);
                }
            }
            Kind::BranchingPolygonIteration { depth, .. }
            | Kind::MappedPointIteration { depth, .. } => {
                if matches!(depth, IterationDepth::Fixed(_)) {
                    *depth = IterationDepth::Fixed(requested_depth);
                }
            }
            Kind::RayIteration(..) => {}
            _ => return Err("Select a supported iteration".into()),
        }
        if let Kind::MappedPointIteration {
            final_only: mapped_final_only,
            ..
        } = &mut object.kind
        {
            *mapped_final_only = final_only;
        }
        drop(object);
        candidate.verify_roundtrip()?;
        *self = candidate;
        Ok(true)
    }

    pub fn set_mapped_iteration_final_only(
        &mut self,
        id: usize,
        final_only: bool,
    ) -> Result<(), String> {
        self.set_iteration_properties(id, None, final_only)
            .map(|_| ())
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
            if matches!(object.kind, Kind::IterationBinding(..)) {
                // Bindings are structural adapters. The descriptor reached
                // through them chooses a live or frozen entry environment.
                pending.extend(object.construction_dependencies());
            } else if matches!(object.kind, Kind::FunctionPlot(..)) {
                // A plot is a recipe over a function and a coordinate frame,
                // not a cached curve value. Native mapping freezes each omitted
                // input independently, even when the plot itself is omitted.
                pending.extend(object.construction_dependencies());
            } else if !carried.contains(&id) {
                // An external construction retains its original value, including
                // its complete predecessor environment, throughout this sequence.
                external.insert(id);
            } else {
                pending.extend(object.construction_dependencies());
            }
        }
        Some(external.into_iter().collect())
    }

    /// Reevaluate anchored text in the same substituted scope as carried geometry.
    pub fn mapped_numeric_label_images(&self, id: usize) -> Option<Vec<(Point, String)>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || !matches!(
                self.objects.get(source.checked_sub(1)?)?.kind,
                Kind::PointNumericLabel(..) | Kind::PointReferencedText { .. }
            )
        {
            return None;
        }
        if matches!(self.objects.get(iteration.checked_sub(1)?)?.kind,Kind::PointParameterIteration(_,_,_,_,label,_) if label==source)
        {
            return self.point_parameter_iteration_labels(iteration);
        }
        self.mapped_iteration_images(source, iteration, |sample, source| {
            sample
                .frozen_evaluation_document(source)
                .unwrap_or(sample)
                .point_numeric_label(source)
        })
    }

    /// Reevaluate all visible labels attached to each mapped root in one pass.
    /// A large simultaneous substitution can expose many bindings, and replaying
    /// the identical state sequence once per label makes painting quadratic.
    pub fn visible_mapped_numeric_label_images(&self) -> BTreeMap<usize, Vec<(Point, String)>> {
        let mut groups: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
        for binding in &self.objects {
            let Kind::IterationBinding(source, iteration) = binding.kind else {
                continue;
            };
            if !self.is_displayed(binding.id)
                || !matches!(
                    self.objects.get(source.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::PointNumericLabel(..) | Kind::PointReferencedText { .. })
                )
                || !matches!(
                    self.objects
                        .get(iteration.saturating_sub(1))
                        .map(|o| &o.kind),
                    Some(Kind::MappedPointIteration { .. })
                )
            {
                continue;
            }
            groups
                .entry(iteration)
                .or_default()
                .push((binding.id, source));
        }

        let mut result = BTreeMap::new();
        for (iteration, bindings) in groups {
            let Some(primary) = bindings.first().map(|(_, source)| *source) else {
                continue;
            };
            let Some(states) = self.mapped_iteration_images(primary, iteration, |sample, _| {
                Some(
                    bindings
                        .iter()
                        .map(|(_, source)| {
                            sample
                                .frozen_evaluation_document(*source)
                                .unwrap_or(sample)
                                .point_numeric_label(*source)
                        })
                        .collect::<Vec<_>>(),
                )
            }) else {
                continue;
            };
            for (column, (binding, _)) in bindings.iter().enumerate() {
                result.insert(
                    *binding,
                    states
                        .iter()
                        .filter_map(|state| state.get(column).cloned().flatten())
                        .collect(),
                );
            }
        }
        result
    }

    fn iteration_value_cell(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.kind.is_function() {
            self.function_display_text(id)
        } else {
            self.numeric_value_text(id)
        }
    }

    /// Resolve a visible code-92 iteration table, including the initial row.
    /// The generation index is retained for branching and final-image-only
    /// mappings, where row count alone cannot recover the native first column.
    pub fn iteration_value_table(&self, id: usize) -> Option<Vec<Vec<String>>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationValues(iteration, values) = &object.kind else {
            return None;
        };
        if object.deleted
            || !matches!(
                self.objects.get(iteration.checked_sub(1)?)?.kind,
                Kind::MappedPointIteration { .. }
            )
        {
            return None;
        }
        let mut headers = vec!["n".to_owned()];
        headers.extend(
            values
                .iter()
                .map(|value| self.calculation_source_name(*value))
                .collect::<Option<Vec<_>>>()?,
        );
        let mut columns = Vec::with_capacity(values.len());
        for value in values {
            let mut column = vec![(
                0,
                self.iteration_value_cell(*value)
                    .unwrap_or_else(|| "undefined".into()),
            )];
            if let Some(outputs) = self.mapped_iteration_outputs_with_steps(
                *value,
                *iteration,
                false,
                false,
                |sample, source| sample.iteration_value_cell(source),
            ) {
                column.extend(
                    outputs
                        .into_iter()
                        .map(|(step, value)| (step, value.unwrap_or_else(|| "undefined".into()))),
                );
            }
            columns.push(column);
        }
        let steps = columns
            .iter()
            .max_by_key(|column| column.len())?
            .iter()
            .map(|row| row.0)
            .collect::<Vec<_>>();
        for column in &mut columns {
            if column.len() == 1 && steps.len() > 1 {
                column.extend(
                    steps[1..]
                        .iter()
                        .copied()
                        .map(|step| (step, "undefined".into())),
                );
            } else if column.iter().map(|row| row.0).ne(steps.iter().copied()) {
                return None;
            }
        }
        let mut rows = vec![headers];
        for (row, step) in steps.into_iter().enumerate() {
            let mut cells = vec![step.to_string()];
            cells.extend(columns.iter().map(|column| column[row].1.clone()));
            rows.push(cells);
        }
        Some(rows)
    }
    /// Preserve mathematical structure while reevaluating each anchored caption
    /// in its own iteration environment.
    pub fn point_math_labels(
        &self,
        id: usize,
    ) -> Option<Vec<(Point, Vec<super::referenced_text::ReferencedNode>)>> {
        if let Kind::PointReferencedText { point, .. } = &self.objects.get(id.checked_sub(1)?)?.kind
        {
            return Some(vec![(self.position(*point)?, self.text_math_nodes(id)?)]);
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || !matches!(
                self.objects.get(source.checked_sub(1)?)?.kind,
                Kind::PointReferencedText { .. }
            )
        {
            return None;
        }
        self.text_math_nodes(source)?;
        self.mapped_iteration_images(source, iteration, |sample, source| {
            let sample = sample.frozen_evaluation_document(source).unwrap_or(sample);
            let Kind::PointReferencedText { point, .. } = sample.objects.get(source - 1)?.kind
            else {
                return None;
            };
            Some((sample.position(point)?, sample.text_math_nodes(source)?))
        })
    }

    pub(super) fn mapped_terminal_point(&self, source: usize, iteration: usize) -> Option<Point> {
        let outputs =
            self.mapped_scoped_outputs(source, iteration, true, |sample, id| sample.position(id))?;
        (outputs.len() == 1).then(|| outputs[0]).flatten()
    }
    /// Replay a stored mapping in its caller or frozen entry environment.
    /// Preserve undefined members so collection consumers can retain gaps.
    pub(super) fn mapped_scoped_outputs<T>(
        &self,
        source: usize,
        iteration: usize,
        terminal_only: bool,
        evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<Option<T>>> {
        self.frozen_evaluation_document(iteration)
            .unwrap_or(self)
            .mapped_iteration_outputs(source, iteration, terminal_only, true, evaluate)
    }
    fn mapped_iteration_images<T>(
        &self,
        source: usize,
        iteration: usize,
        evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<T>> {
        Some(
            self.mapped_iteration_outputs(source, iteration, false, false, evaluate)?
                .into_iter()
                .flatten()
                .collect(),
        )
    }
    fn mapped_iteration_outputs<T>(
        &self,
        source: usize,
        iteration: usize,
        terminal_only: bool,
        retain_parent: bool,
        mut evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<Option<T>>> {
        Some(
            self.mapped_iteration_outputs_with_steps(
                source,
                iteration,
                terminal_only,
                retain_parent,
                |sample, id| evaluate(sample, id),
            )?
            .into_iter()
            .map(|(_, value)| value)
            .collect(),
        )
    }
    fn mapped_iteration_outputs_with_steps<T>(
        &self,
        source: usize,
        iteration: usize,
        terminal_only: bool,
        retain_parent: bool,
        mut evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<(u32, Option<T>)>> {
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
        let mapped_value_depth = |id, minimum: f64| {
            let raw = self.unitless_numeric_value(id)?;
            if !raw.is_finite() || raw < 0. {
                return None;
            }
            let value = raw.trunc();
            (minimum..=f64::from(MAX_MAPPED_ITERATION_DEPTH))
                .contains(&value)
                .then_some(value as u32)
        };
        let depth = if terminal_only {
            match *depth {
                IterationDepth::Fixed(value) => (1..=MAX_MAPPED_ITERATION_DEPTH)
                    .contains(&value)
                    .then_some(value)?,
                IterationDepth::Value(id) => mapped_value_depth(id, 0.)?,
            }
        } else {
            match *depth {
                IterationDepth::Fixed(_) => self.mapped_iteration_depth(*depth)?,
                IterationDepth::Value(id) => mapped_value_depth(id, 1.)?,
            }
        };
        let initial: Vec<_> = seeds
            .iter()
            .map(|id| MappedValue::read(self, *id))
            .collect::<Option<_>>()?;
        let mut level = vec![initial];
        let mut sample = self.clone();
        let external = self.mapped_external_inputs(seeds, branches, carried)?;
        let mut evaluation_order = branches
            .iter()
            .flatten()
            .chain(carried)
            .copied()
            .filter(|id| !seeds.contains(id))
            .collect::<Vec<_>>();
        evaluation_order.sort_unstable();
        evaluation_order.dedup();
        if retain_parent {
            sample.freeze_scoped_evaluation_inputs(self, &external)?;
            sample.inherit_evaluation_seeds(self)?;
        } else {
            sample.freeze_evaluation_inputs(self, &external)?;
        }
        if depth == 0 {
            return Some(vec![(0, evaluate(&sample, source))]);
        }
        let mut result = Vec::new();
        let mut generated = 0;
        for step in 0..depth {
            let mut next = Vec::new();
            let mut seen = BTreeSet::new();
            for state in &level {
                sample.clear_evaluation_results();
                if retain_parent {
                    sample.clear_evaluation_seed_ids(seeds);
                } else {
                    sample.clear_evaluation_seeds();
                }
                for (id, value) in seeds.iter().zip(state) {
                    value.substitute(&mut sample, *id)?;
                }
                sample.cache_evaluation_values(&evaluation_order)?;
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
            if (!terminal_only && !*final_only) || step + 1 == depth {
                for state in &next {
                    sample.clear_evaluation_results();
                    if retain_parent {
                        sample.clear_evaluation_seed_ids(seeds);
                    } else {
                        sample.clear_evaluation_seeds();
                    }
                    for (id, value) in seeds.iter().zip(state) {
                        value.substitute(&mut sample, *id)?;
                    }
                    sample.cache_evaluation_values(&evaluation_order)?;
                    // Keep terminal undefinedness. The ordinary image reader
                    // filters each undefined image while preserving valid levels.
                    result.push((step + 1, evaluate(&sample, source)));
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
            self.objects.edit(id - 1).visual_style = style;
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
        let levels = if o.iteration_final_only {
            depth..=depth
        } else {
            1..=depth
        };
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
        for level in levels {
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
    /// Analytic disk samples; polygon vertices remain presentation-only outlines.
    pub fn circle_interior_iteration_images(&self, id: usize) -> Option<Vec<(Point, f64)>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || !self
                .objects
                .get(source.checked_sub(1)?)?
                .kind
                .is_circle_interior()
        {
            return None;
        }
        self.mapped_iteration_images(source, iteration, |sample, id| {
            sample.circle_interior_geometry(id)
        })
    }

    /// Per-generation analytic disks and colors from one carried color binding.
    pub fn colorized_disk_iteration_images(
        &self,
        id: usize,
    ) -> Option<Vec<(Point, f64, RgbColor)>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || !matches!(self.objects.get(source.checked_sub(1)?)?.kind,
                Kind::Colorized(ref definition) if definition.element() == ElementType::Disk)
            || !matches!(
                self.objects.get(iteration.checked_sub(1)?)?.kind,
                Kind::MappedPointIteration { .. }
            )
        {
            return None;
        }
        self.mapped_iteration_images(source, iteration, |sample, id| {
            let (center, radius) = sample.circle_interior_geometry(id)?;
            let color = sample.colorized_color(id)?;
            Some((center, radius, color))
        })
    }

    /// Filled polygons and their colors evaluated in the same mapped state.
    pub fn colorized_polygon_iteration_images(
        &self,
        id: usize,
    ) -> Option<Vec<(Vec<Point>, RgbColor)>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        if object.deleted
            || !matches!(self.objects.get(source.checked_sub(1)?)?.kind,
                Kind::Colorized(ref definition) if matches!(definition.element(), ElementType::Polygon | ElementType::Sector))
            || !matches!(
                self.objects.get(iteration.checked_sub(1)?)?.kind,
                Kind::MappedPointIteration { .. }
            )
        {
            return None;
        }
        self.mapped_iteration_images(source, iteration, |sample, id| {
            Some((
                sample.filled_region_points(id)?,
                sample.colorized_color(id)?,
            ))
        })
    }

    pub fn polygon_iteration_images(&self, id: usize) -> Option<Vec<(Vec<Point>, f64)>> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = o.kind else {
            return None;
        };
        let source_kind = &self.objects.get(source.checked_sub(1)?)?.kind;
        if !o.deleted
            && source_kind.is_filled_region()
            && !matches!(source_kind, Kind::Colorized(definition) if definition.element() == ElementType::Disk)
            && matches!(
                self.objects.get(iteration.checked_sub(1)?)?.kind,
                Kind::MappedPointIteration { .. }
            )
        {
            return self
                .mapped_iteration_images(source, iteration, |sample, id| {
                    sample.filled_region_points(id)
                })
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
        let values = self.append_object(Kind::IterationValues(iteration, vec![calculation]));
        self.objects.edit(values - 1).visible = false;
        Ok(iteration)
    }
    pub(super) fn numeric_iteration_depth(&self, depth: IterationDepth) -> Option<u32> {
        match depth {
            IterationDepth::Fixed(value) => (1..=128).contains(&value).then_some(value),
            IterationDepth::Value(id) => self.iteration_depth_value(id),
        }
    }

    pub(super) fn mapped_iteration_depth(&self, depth: IterationDepth) -> Option<u32> {
        match depth {
            IterationDepth::Fixed(value) => (1..=MAX_MAPPED_ITERATION_DEPTH)
                .contains(&value)
                .then_some(value),
            IterationDepth::Value(id) => {
                let value = self.unitless_numeric_value(id)?.trunc();
                (1. ..=f64::from(MAX_MAPPED_ITERATION_DEPTH))
                    .contains(&value)
                    .then_some(value as u32)
            }
        }
    }
    pub fn point_parameter_iteration_labels(&self, id: usize) -> Option<Vec<(Point, String)>> {
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PointParameterIteration(_, parameter, _, calculation, _, depth) = o.kind else {
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
            sample.objects.edit(parameter - 1).kind = Kind::Parameter(mapped_parameter.clone());
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
        if o.iteration_final_only {
            for _ in 1..self.numeric_iteration_depth(depth)? {
                next_value()?;
            }
        }
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
        self.objects.edit(binding - 1).visual_style = style_records::iteration_binding_visual_style(
            &include_bytes!("../../../../assets/segment_iteration_binding_group.bin")[8..36],
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
                    NumericUnit::Degrees | NumericUnit::Scalar => self.numeric_value(id),
                    _ => None,
                },
                NumericExpr::DistanceRatio {
                    origin,
                    base,
                    target,
                } => ratio_of_points(point(origin)?, point(base)?, point(target)?),
                NumericExpr::SegmentRatio {
                    numerator,
                    denominator,
                } => self.ratio_for_segments(numerator, denominator),
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
        let object = self.objects.get(id.checked_sub(1)?)?;
        let (seed, image, depth) = match object.kind {
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
        let images = self
            .iteration_orbit(seed, image, depth)?
            .into_iter()
            .skip(2);
        Some(if object.iteration_final_only {
            images.last().into_iter().collect()
        } else {
            images.collect()
        })
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
