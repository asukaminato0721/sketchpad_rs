//! Compile stable point constructions into a shared, topologically ordered tape.
//! Values and matrices are bound once per batch, never retained across frames.
use super::geometry::{marked_rotation_theta, ratio_of_points};
use super::iteration::MappedValue;
use super::sac_evaluation::Identity;
use super::transform_model::{DisplacementExpr, ElementType, NumericExpr, Transform};
use super::{Document, Kind, Point};
use crate::table_records::{CalculationSourceKind, CalculationTape, NumericUnit};
use crate::value_equation_records::{self, EquationValue, ScreenPoint, ValueEquationKind};
use lru::LruCache;
use rustc_hash::{FxHashMap, FxHashSet};
use sketchpad_ir::affine::{AffineMatrix, reflect_point, rotate_point_about, scale_point_about};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

type Key = (Vec<usize>, Vec<usize>);
type Cache = LruCache<Key, Arc<Compiled>>;
#[derive(Debug)]
struct Compiled {
    structure: Identity,
    guards: Vec<(usize, Identity)>,
    program: Option<Arc<PointProgram>>,
}
#[derive(Clone, Debug)]
pub(super) struct PointPrograms(Arc<Mutex<Cache>>);
impl Default for PointPrograms {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(128).unwrap(),
        ))))
    }
}

#[derive(Debug)]
enum Op {
    Seed(usize),
    External,
    Midpoint(usize, usize),
    Matrix(usize),
    Vector(usize, usize, usize),
    Rotate(usize, usize, Operand),
    Dilate(usize, usize, Operand),
    Reflect(usize, usize, usize),
    Copy(usize),
    Cartesian(usize, Operand, Operand),
    Polar(usize, Operand, Operand),
    Number(NumberOp),
    Mapping {
        inputs: Vec<usize>,
        external: Arc<[usize]>,
        program: Arc<PointProgram>,
    },
}

#[derive(Clone, Debug)]
enum NumberOp {
    Seed(usize),
    External,
    Constant(Option<f64>),
    Projection {
        point: usize,
        a: usize,
        b: usize,
        path: usize,
        clamp: bool,
        frozen: Option<Option<(Point, Point)>>,
    },
    Calculation {
        sources: Arc<[usize]>,
        code: Arc<CalculationTape>,
    },
}

/// Scalar operands from numeric and point nodes in the same batch. Keep the native
/// geometry helpers and angle conversions shared with the ordinary evaluator.
#[derive(Clone, Copy, Debug)]
enum Operand {
    Literal(f64),
    Number(usize, f64),
    Ratio(usize, usize, usize),
    Angle(usize, usize, usize),
}
impl Operand {
    fn supports(expr: &NumericExpr) -> bool {
        matches!(
            expr,
            NumericExpr::Literal { .. }
                | NumericExpr::ObjectValue { .. }
                | NumericExpr::DistanceRatio { .. }
                | NumericExpr::DirectedAngle { .. }
        )
    }

    fn value(
        self,
        point: impl Fn(usize) -> Option<Point>,
        number: impl Fn(usize) -> Option<f64>,
    ) -> Option<f64> {
        let value = match self {
            Self::Literal(value) => value,
            Self::Number(slot, scale) => number(slot)? * scale,
            Self::Ratio(a, b, c) => ratio_of_points(point(a)?, point(b)?, point(c)?)?,
            // The ordinary evaluator converts to degrees before consuming an
            // angle. Retain that round trip even when the consumer is rotation.
            Self::Angle(a, b, c) => {
                marked_rotation_theta(point(a)?, point(b)?, point(c)?)?.to_degrees()
            }
        };
        value.is_finite().then_some(value)
    }
}

fn supports_dynamic_transform(transform: &Transform) -> bool {
    match transform {
        Transform::Translate(DisplacementExpr::Vector { .. }) => true,
        Transform::Translate(DisplacementExpr::Cartesian {
            horizontal,
            vertical,
        }) => Operand::supports(horizontal) && Operand::supports(vertical),
        Transform::Translate(DisplacementExpr::Polar { angle, distance }) => {
            Operand::supports(angle) && Operand::supports(distance)
        }
        Transform::Rotate { angle, .. } => Operand::supports(angle),
        Transform::Dilate { ratio, .. } => Operand::supports(ratio),
        Transform::Reflect { .. } => true,
    }
}
#[derive(Debug)]
struct Node {
    id: usize,
    op: Op,
}
#[derive(Debug)]
pub(super) struct PointProgram {
    nodes: Vec<Node>,
    outputs: Vec<usize>,
}

#[derive(Clone)]
enum BoundOp {
    Seed(usize),
    Constant(Option<Point>),
    Midpoint(usize, usize),
    Matrix(usize, Option<AffineMatrix>),
    Vector(usize, usize, usize),
    Rotate(usize, usize, Operand),
    Dilate(usize, usize, Operand),
    Reflect(usize, usize, usize),
    Copy(usize),
    Cartesian(usize, Operand, Operand),
    Polar(usize, Operand, Operand),
    Number(NumberOp),
    Mapping(Vec<usize>, Arc<PointBatch>),
}
pub(super) struct PointBatch {
    ops: Vec<BoundOp>,
    outputs: Vec<usize>,
    radians: bool,
}

#[derive(Default)]
struct PointWorkspace<const LANES: usize> {
    values: Vec<[Option<Point>; LANES]>,
    numbers: Vec<[Option<f64>; LANES]>,
    expression: Vec<f64>,
}

/// A bound single-seed evaluator. All instruction and expression storage stays
/// local to this curve evaluation and is reused between refinement parameters.
pub(super) struct PointEvaluator {
    batch: PointBatch,
    workspace: PointWorkspace<1>,
    points: [Vec<Option<Point>>; 1],
    numbers: [Vec<Option<f64>>; 1],
}
impl PointEvaluator {
    pub(super) fn point(&mut self, value: Option<Point>) -> Option<Point> {
        self.points[0][0] = value;
        self.evaluate()
    }
    pub(super) fn number(&mut self, value: f64) -> Option<Point> {
        self.numbers[0][0] = Some(value);
        self.evaluate()
    }
    fn evaluate(&mut self) -> Option<Point> {
        let mut result = None;
        self.batch.run_tiles(
            &self.points,
            &self.numbers,
            &mut self.workspace,
            |_, point, _| result = point,
        );
        result
    }
}

impl PointProgram {
    pub(super) fn get(doc: &Document, seeds: &[usize], roots: &[usize]) -> Option<Arc<Self>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Validation);
        // Reject unsupported direct varying operands before building a slice.
        for &root in roots {
            if let Kind::Transformed(definition) = &doc.objects.get(root.checked_sub(1)?)?.kind
                && !seeds.contains(&root)
                && !supports_dynamic_transform(definition.transform())
                && definition
                    .transform()
                    .references()
                    .iter()
                    .any(|id| seeds.contains(id))
            {
                return None;
            }
        }
        let key = (seeds.to_vec(), roots.to_vec());
        let cache = &doc.sampling_plans.points.0;
        let cached = cache.lock().unwrap().get(&key).cloned();
        if let Some(compiled) = cached.filter(|p| {
            p.structure == doc.objects.sac_validation_identity()
                && p.guards
                    .iter()
                    .all(|(id, revision)| *revision == doc.objects.sac_identity(Some(id - 1)))
        }) {
            return compiled.program.clone();
        }
        let mut guards = Vec::new();
        let program = Self::compile(doc, seeds, roots, &mut guards, 0).map(Arc::new);
        cache.lock().unwrap().put(
            key,
            Arc::new(Compiled {
                structure: doc.objects.sac_validation_identity(),
                guards,
                program: program.clone(),
            }),
        );
        program
    }

    fn compile(
        doc: &Document,
        seeds: &[usize],
        roots: &[usize],
        guards: &mut Vec<(usize, Identity)>,
        nesting: usize,
    ) -> Option<Self> {
        if nesting >= 16 {
            return None;
        }
        let graph = doc.objects.dependency_graph();
        let mut all = roots.to_vec();
        all.extend(seeds);
        let slice = graph.slice(&all).ok()?;
        let update = slice.update(seeds);
        let affected: FxHashSet<_> = update.order().iter().chain(seeds).copied().collect();
        struct Builder<'a> {
            doc: &'a Document,
            seeds: &'a [usize],
            affected: &'a FxHashSet<usize>,
            slots: FxHashMap<usize, usize>,
            number_slots: FxHashMap<usize, usize>,
            nodes: Vec<Node>,
            guards: &'a mut Vec<(usize, Identity)>,
            depth: usize,
            nesting: usize,
        }
        impl Builder<'_> {
            fn value(&mut self, id: usize) -> Option<usize> {
                if self.doc.objects.get(id.checked_sub(1)?)?.kind.is_point() {
                    self.point(id)
                } else {
                    self.number(id)
                }
            }

            fn number(&mut self, id: usize) -> Option<usize> {
                if let Some(&slot) = self.number_slots.get(&id) {
                    return Some(slot);
                }
                if self.depth >= 256 || self.nodes.len() >= 4096 {
                    return None;
                }
                self.depth += 1;
                let result = self.number_inner(id);
                self.depth -= 1;
                result
            }
            fn number_inner(&mut self, id: usize) -> Option<usize> {
                let object = self.doc.objects.get(id.checked_sub(1)?)?;
                if object.deleted {
                    return None;
                }
                object.kind.numeric_unit()?;
                // Numeric units affect expression validation and length conversion.
                self.guards
                    .push((id, self.doc.objects.sac_identity(Some(id - 1))));
                let op = if let Some(index) = self.seeds.iter().position(|&seed| seed == id) {
                    NumberOp::Seed(index)
                } else if !self.affected.contains(&id) {
                    NumberOp::External
                } else if let Kind::ValueEquation {
                    kind: ValueEquationKind::PointValue,
                    first,
                    second,
                } = object.kind
                {
                    // Stored curve/arc parameters have different semantics. Lower only
                    // the ordinary straight projection, retaining the segment clamp.
                    let first_kind = &self.doc.objects.get(first.checked_sub(1)?)?.kind;
                    if !matches!(
                        first_kind,
                        Kind::Free(_)
                            | Kind::PointOnSegment(..)
                            | Kind::PointOnLine(..)
                            | Kind::PointOnRay(..)
                            | Kind::Transformed(_)
                            | Kind::Midpoint(..)
                    ) {
                        return None;
                    }
                    self.guards
                        .push((first, self.doc.objects.sac_identity(Some(first - 1))));
                    let clamp = matches!(first_kind, Kind::PointOnSegment(..))
                        && matches!(
                            self.doc.objects.get(second.checked_sub(1)?)?.kind,
                            Kind::Segment(..)
                        );
                    let (a, b) = self.straight(second)?;
                    NumberOp::Projection {
                        point: self.point(first)?,
                        a,
                        b,
                        path: second,
                        clamp,
                        frozen: None,
                    }
                } else {
                    let Kind::ParameterCalculation(first, group) = &object.kind else {
                        return None;
                    };
                    let ids = super::calculations::calculation_source_ids(*first, group);
                    if ids.iter().any(|&source| source >= id)
                        || group
                            .source_kinds()
                            .ok()?
                            .contains(&CalculationSourceKind::Function)
                    {
                        return None;
                    }
                    let units = super::calculations::calculation_source_units(
                        group,
                        &ids,
                        &self.doc.objects,
                    )?;
                    let code = group.batch_program(&units)?;
                    let sources = ids
                        .iter()
                        .map(|&id| self.number(id))
                        .collect::<Option<Vec<_>>>()?
                        .into();
                    NumberOp::Calculation { sources, code }
                };
                let slot = self.nodes.len();
                self.nodes.push(Node {
                    id,
                    op: Op::Number(op),
                });
                self.number_slots.insert(id, slot);
                Some(slot)
            }

            fn straight(&mut self, id: usize) -> Option<(usize, usize)> {
                let object = self.doc.objects.get(id.checked_sub(1)?)?;
                if object.deleted {
                    return None;
                }
                self.guards
                    .push((id, self.doc.objects.sac_identity(Some(id - 1))));
                let (a, b) = match object.kind {
                    Kind::Segment(a, b) | Kind::Ray(a, b) | Kind::Line(a, b) => (a, b),
                    _ => return None,
                };
                Some((self.point(a)?, self.point(b)?))
            }

            fn operand(&mut self, expr: &NumericExpr) -> Option<Operand> {
                Some(match *expr {
                    NumericExpr::Literal {
                        magnitude, scale, ..
                    } => Operand::Literal(magnitude * scale),
                    NumericExpr::DistanceRatio {
                        origin,
                        base,
                        target,
                    } => {
                        Operand::Ratio(self.point(origin)?, self.point(base)?, self.point(target)?)
                    }
                    NumericExpr::DirectedAngle {
                        first,
                        vertex,
                        last,
                    } => Operand::Angle(self.point(first)?, self.point(vertex)?, self.point(last)?),
                    NumericExpr::ObjectValue { id, unit } => {
                        if !self
                            .doc
                            .objects
                            .get(id.checked_sub(1)?)?
                            .kind
                            .is_transform_operand(unit)
                        {
                            return None;
                        }
                        let scale = match unit {
                            NumericUnit::Scalar | NumericUnit::Degrees => 1.,
                            NumericUnit::Length => 37.79527559055118,
                            _ => return None,
                        };
                        Operand::Number(self.number(id)?, scale)
                    }
                    _ => return None,
                })
            }

            fn point(&mut self, id: usize) -> Option<usize> {
                if let Some(&slot) = self.slots.get(&id) {
                    return Some(slot);
                }
                if self.nodes.len() >= 4096 || self.depth >= 256 {
                    return None;
                }
                self.depth += 1;
                let result = self.point_inner(id);
                self.depth -= 1;
                result
            }
            fn point_inner(&mut self, id: usize) -> Option<usize> {
                let object = self.doc.objects.get(id.checked_sub(1)?)?;
                if object.deleted {
                    return None;
                }
                let op = if let Some(index) = self.seeds.iter().position(|&seed| seed == id) {
                    Op::Seed(index)
                } else if !self.affected.contains(&id) {
                    Op::External
                } else {
                    self.guards
                        .push((id, self.doc.objects.sac_identity(Some(id - 1))));
                    match &object.kind {
                        Kind::MappedElement(d) => {
                            let external = self.doc.mapped_external_inputs(
                                &[d.seed()],
                                &[vec![d.replacement()]],
                                d.carried(),
                            )?;
                            let mut seeds = vec![d.seed()];
                            let mut inputs = vec![self.point(d.replacement())?];
                            for &id in external.iter() {
                                if self.affected.contains(&id) {
                                    seeds.push(id);
                                    // Whole curves/frames require their own frozen value.
                                    // Keep those mappings on the ordinary evaluator.
                                    inputs.push(self.value(id)?);
                                }
                            }
                            let program = PointProgram::compile(
                                self.doc,
                                &seeds,
                                &[d.output()],
                                self.guards,
                                self.nesting + 1,
                            )?;
                            Op::Mapping {
                                inputs,
                                external,
                                program: Arc::new(program),
                            }
                        }
                        Kind::Midpoint(a, b) if object.midpoint_host.is_none() => {
                            Op::Midpoint(self.point(*a)?, self.point(*b)?)
                        }
                        Kind::Colorized(d) if d.element() == ElementType::Point => {
                            Op::Copy(self.point(d.source())?)
                        }
                        Kind::Transformed(d) if d.element() == ElementType::Point => {
                            let source = self.point(d.source())?;
                            if let Transform::Translate(DisplacementExpr::Vector { start, end }) =
                                d.transform()
                            {
                                Op::Vector(source, self.point(*start)?, self.point(*end)?)
                            } else if d
                                .transform()
                                .references()
                                .iter()
                                .all(|id| !self.affected.contains(id))
                            {
                                Op::Matrix(source)
                            } else {
                                match d.transform() {
                                    Transform::Rotate { center, angle } => Op::Rotate(
                                        source,
                                        self.point(*center)?,
                                        self.operand(angle)?,
                                    ),
                                    Transform::Dilate { center, ratio } => Op::Dilate(
                                        source,
                                        self.point(*center)?,
                                        self.operand(ratio)?,
                                    ),
                                    Transform::Reflect { mirror } => {
                                        let (a, b) = self.straight(*mirror)?;
                                        Op::Reflect(source, a, b)
                                    }
                                    Transform::Translate(DisplacementExpr::Cartesian {
                                        horizontal,
                                        vertical,
                                    }) => Op::Cartesian(
                                        source,
                                        self.operand(horizontal)?,
                                        self.operand(vertical)?,
                                    ),
                                    Transform::Translate(DisplacementExpr::Polar {
                                        angle,
                                        distance,
                                    }) => Op::Polar(
                                        source,
                                        self.operand(angle)?,
                                        self.operand(distance)?,
                                    ),
                                    _ => return None,
                                }
                            }
                        }
                        _ => return None,
                    }
                };
                let slot = self.nodes.len();
                self.nodes.push(Node { id, op });
                self.slots.insert(id, slot);
                Some(slot)
            }
        }
        let mut builder = Builder {
            doc,
            seeds,
            affected: &affected,
            slots: Default::default(),
            number_slots: Default::default(),
            nodes: Vec::new(),
            guards,
            depth: 0,
            nesting,
        };
        let outputs = roots
            .iter()
            .map(|&id| builder.value(id))
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            nodes: builder.nodes,
            outputs,
        })
    }

    pub(super) fn bind(&self, doc: &Document) -> Option<PointBatch> {
        let mut ops = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            // Observe every live instruction even when the tape came from cache.
            let object = doc.objects.get(node.id - 1)?;
            if object.deleted {
                return None;
            }
            let op = if let Op::Number(ref number) = node.op {
                BoundOp::Number(match number {
                    NumberOp::Seed(index) => NumberOp::Seed(*index),
                    _ if doc.evaluation_numeric(node.id).is_some() => {
                        NumberOp::Constant(doc.evaluation_numeric(node.id)?)
                    }
                    NumberOp::External => NumberOp::Constant(doc.numeric_value(node.id)),
                    NumberOp::Projection {
                        point,
                        a,
                        b,
                        path,
                        clamp,
                        ..
                    } => NumberOp::Projection {
                        point: *point,
                        a: *a,
                        b: *b,
                        path: *path,
                        clamp: *clamp,
                        frozen: doc
                            .evaluation_straight(*path)
                            .map(|line| line.map(|(a, b, _)| (a, b))),
                    },
                    other => other.clone(),
                })
            } else if let Op::Seed(index) = node.op {
                BoundOp::Seed(index)
            } else if let Some(value) = doc.evaluation_point(node.id) {
                BoundOp::Constant(value)
            } else {
                match node.op {
                    Op::Seed(_) | Op::Number(_) => unreachable!(),
                    Op::Mapping {
                        ref inputs,
                        ref external,
                        ref program,
                    } => {
                        super::sac_evaluation::observe(
                            super::sac_evaluation::Input::MappedElementValidity(node.id),
                        );
                        if !doc.objects.mapped_element_validity(node.id) {
                            return None;
                        }
                        let mut sample = doc.clone_for_evaluation();
                        sample.freeze_scoped_evaluation_inputs(doc, external)?;
                        BoundOp::Mapping(inputs.clone(), Arc::new(program.bind(&sample)?))
                    }
                    Op::External => BoundOp::Constant(doc.position(node.id)),
                    Op::Midpoint(a, b) => BoundOp::Midpoint(a, b),
                    Op::Copy(a) => BoundOp::Copy(a),
                    Op::Matrix(source) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        let d = object.kind.transform_definition()?;
                        BoundOp::Matrix(
                            source,
                            doc.transform_operation_matrix(&d).ok().map(|v| v.0),
                        )
                    }
                    Op::Cartesian(source, a, b) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        BoundOp::Cartesian(source, a, b)
                    }
                    Op::Polar(source, a, b) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        BoundOp::Polar(source, a, b)
                    }
                    Op::Vector(source, a, b) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        BoundOp::Vector(source, a, b)
                    }
                    Op::Rotate(source, center, angle) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        BoundOp::Rotate(source, center, angle)
                    }
                    Op::Dilate(source, center, ratio) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        BoundOp::Dilate(source, center, ratio)
                    }
                    Op::Reflect(source, a, b) => {
                        doc.validate_transform_dependencies(node.id).ok()?;
                        let d = object.kind.transform_definition()?;
                        let Transform::Reflect { mirror } = d.transform() else {
                            return None;
                        };
                        // An enclosing mapped scope may replace the whole mirror.
                        if doc.evaluation_straight(*mirror).is_some() {
                            BoundOp::Matrix(
                                source,
                                doc.transform_operation_matrix(&d).ok().map(|v| v.0),
                            )
                        } else {
                            BoundOp::Reflect(source, a, b)
                        }
                    }
                }
            };
            ops.push(op);
        }
        Some(PointBatch {
            ops,
            outputs: self.outputs.clone(),
            radians: doc.angle_unit_radians,
        })
    }
}

impl PointBatch {
    pub(super) fn into_evaluator(self) -> PointEvaluator {
        debug_assert_eq!(self.outputs.len(), 1);
        PointEvaluator {
            batch: self,
            workspace: PointWorkspace::default(),
            points: [vec![None]],
            numbers: [vec![None]],
        }
    }
    /// Each output keeps its undefined lanes. Shared ancestors execute once per
    /// lane, with no document queries, dependency tracking or hashes in the loop.
    pub(super) fn evaluate(&self, inputs: &[Vec<Option<Point>>]) -> Vec<Vec<Option<Point>>> {
        self.evaluate_inputs(inputs, &[], |point, _| point)
    }
    pub(super) fn evaluate_numbers(&self, inputs: &[Vec<Option<f64>>]) -> Vec<Vec<Option<Point>>> {
        self.evaluate_inputs(&[], inputs, |point, _| point)
    }
    pub(super) fn evaluate_mapped(
        &self,
        inputs: &[Vec<Option<MappedValue>>],
    ) -> Vec<Vec<Option<MappedValue>>> {
        let mut outputs = self.evaluate_values(inputs);
        for value in outputs.iter_mut().flatten() {
            *value = value.filter(|value| match value {
                MappedValue::Point(p) => p.x.is_finite() && p.y.is_finite(),
                MappedValue::Number(n) => n.is_finite(),
            });
        }
        outputs
    }
    pub(super) fn evaluate_values(
        &self,
        inputs: &[Vec<Option<MappedValue>>],
    ) -> Vec<Vec<Option<MappedValue>>> {
        let points: Vec<Vec<_>> = inputs
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|value| match value {
                        Some(MappedValue::Point(point)) => Some(*point),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        let numbers: Vec<Vec<_>> = inputs
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|value| match value {
                        Some(MappedValue::Number(number)) => Some(*number),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        self.evaluate_inputs(&points, &numbers, |point, number| {
            point
                .map(MappedValue::Point)
                .or_else(|| number.map(MappedValue::Number))
        })
    }

    fn evaluate_inputs<T>(
        &self,
        inputs: &[Vec<Option<Point>>],
        numeric_inputs: &[Vec<Option<f64>>],
        output_value: impl Fn(Option<Point>, Option<f64>) -> T,
    ) -> Vec<Vec<T>> {
        let count = inputs
            .first()
            .map(Vec::len)
            .or_else(|| numeric_inputs.first().map(Vec::len))
            .unwrap_or(0);
        if count == 1 {
            self.evaluate_tiles::<T, 1>(inputs, numeric_inputs, output_value)
        } else {
            self.evaluate_tiles::<T, 64>(inputs, numeric_inputs, output_value)
        }
    }
    fn evaluate_tiles<T, const LANES: usize>(
        &self,
        inputs: &[Vec<Option<Point>>],
        numeric_inputs: &[Vec<Option<f64>>],
        output_value: impl Fn(Option<Point>, Option<f64>) -> T,
    ) -> Vec<Vec<T>> {
        let count = inputs
            .first()
            .map(Vec::len)
            .or_else(|| numeric_inputs.first().map(Vec::len))
            .unwrap_or(0);
        let mut outputs: Vec<Vec<T>> = self
            .outputs
            .iter()
            .map(|_| Vec::with_capacity(count))
            .collect();
        self.run_tiles::<LANES>(
            inputs,
            numeric_inputs,
            &mut PointWorkspace::default(),
            |index, point, number| {
                outputs[index].push(output_value(point, number));
            },
        );
        outputs
    }
    fn run_tiles<const LANES: usize>(
        &self,
        inputs: &[Vec<Option<Point>>],
        numeric_inputs: &[Vec<Option<f64>>],
        workspace: &mut PointWorkspace<LANES>,
        mut emit: impl FnMut(usize, Option<Point>, Option<f64>),
    ) {
        let count = inputs
            .first()
            .map(Vec::len)
            .or_else(|| numeric_inputs.first().map(Vec::len))
            .unwrap_or(0);
        assert!(inputs.iter().all(|input| input.len() == count));
        assert!(numeric_inputs.iter().all(|input| input.len() == count));
        let PointWorkspace {
            values,
            numbers,
            expression: scratch,
        } = workspace;
        values.resize(self.ops.len(), [None; LANES]);
        let numeric_slots = self
            .ops
            .iter()
            .rposition(|op| matches!(op, BoundOp::Number(_)))
            .map_or(0, |i| i + 1);
        numbers.resize(numeric_slots, [None; LANES]);
        for offset in (0..count).step_by(LANES) {
            let length = (count - offset).min(LANES);
            for (slot, op) in self.ops.iter().enumerate() {
                if let BoundOp::Number(op) = op {
                    for index in 0..length {
                        numbers[slot][index] = match op {
                            NumberOp::Seed(seed) => {
                                numeric_inputs[*seed][offset + index].filter(|v| v.is_finite())
                            }
                            NumberOp::Constant(value) => *value,
                            NumberOp::Projection {
                                point,
                                a,
                                b,
                                clamp,
                                frozen,
                                ..
                            } => {
                                let screen = |p: Point| ScreenPoint { x: p.x, y: p.y };
                                (|| {
                                    let (a, b) = match frozen {
                                        Some(line) => (*line)?,
                                        None => (values[*a][index]?, values[*b][index]?),
                                    };
                                    let EquationValue::PointValue(value) =
                                        value_equation_records::segment_point_value(
                                            screen(values[*point][index]?),
                                            screen(a),
                                            screen(b),
                                        )?
                                    else {
                                        return None;
                                    };
                                    Some(if *clamp { value.clamp(0., 1.) } else { value })
                                })()
                            }
                            NumberOp::Calculation { sources, code } => {
                                // Validate every source, including unused native reference slots.
                                if sources.iter().any(|&source| {
                                    numbers[source][index].is_none_or(|v| !v.is_finite())
                                }) {
                                    None
                                } else {
                                    code.evaluate(
                                        |source| numbers[sources[source]][index],
                                        self.radians,
                                        scratch,
                                    )
                                    .filter(|v| v.is_finite())
                                }
                            }
                            NumberOp::External => unreachable!(),
                        };
                    }
                    continue;
                }
                if let BoundOp::Mapping(inputs, batch) = op {
                    let columns: Vec<Vec<_>> = inputs
                        .iter()
                        .map(|&slot| {
                            (0..length)
                                .map(|index| {
                                    if matches!(self.ops[slot], BoundOp::Number(_)) {
                                        numbers[slot][index].map(MappedValue::Number)
                                    } else {
                                        values[slot][index].map(MappedValue::Point)
                                    }
                                })
                                .collect()
                        })
                        .collect();
                    let mapped = batch.evaluate_values(&columns);
                    for (index, value) in mapped[0].iter().enumerate() {
                        // An undefined replacement invalidates the mapping even
                        // when its output does not depend on the inner seed.
                        values[slot][index] = match (columns[0][index], value) {
                            (Some(MappedValue::Point(p)), Some(MappedValue::Point(value)))
                                if p.x.is_finite() && p.y.is_finite() =>
                            {
                                Some(*value)
                            }
                            _ => None,
                        };
                    }
                    continue;
                }
                let lane = |index: usize| -> Option<Point> {
                    match *op {
                        BoundOp::Number(_) | BoundOp::Mapping(..) => unreachable!(),
                        BoundOp::Cartesian(source, a, b) | BoundOp::Polar(source, a, b) => {
                            let source = values[source][index]?;
                            let a =
                                a.value(|slot| values[slot][index], |slot| numbers[slot][index])?;
                            let b =
                                b.value(|slot| values[slot][index], |slot| numbers[slot][index])?;
                            let (dx, dy) = if matches!(op, BoundOp::Polar(..)) {
                                let theta = a.to_radians();
                                (b * theta.cos(), -b * theta.sin())
                            } else {
                                (a, -b)
                            };
                            let point = Point {
                                x: source.x + dx,
                                y: source.y + dy,
                            };
                            (point.x.is_finite() && point.y.is_finite()).then_some(point)
                        }
                        BoundOp::Seed(seed) => inputs[seed][offset + index],
                        BoundOp::Constant(value) => value,
                        BoundOp::Copy(a) => values[a][index],
                        BoundOp::Midpoint(a, b) => {
                            let (a, b) = (values[a][index]?, values[b][index]?);
                            Some(Point {
                                x: (a.x + b.x) / 2.,
                                y: (a.y + b.y) / 2.,
                            })
                        }
                        BoundOp::Matrix(a, matrix) => matrix?.apply(values[a][index]?),
                        BoundOp::Rotate(source, center, angle) => rotate_point_about(
                            values[source][index]?,
                            values[center][index]?,
                            angle
                                .value(|slot| values[slot][index], |slot| numbers[slot][index])?
                                .to_radians(),
                        ),
                        BoundOp::Dilate(source, center, ratio) => scale_point_about(
                            values[source][index]?,
                            values[center][index]?,
                            ratio.value(|slot| values[slot][index], |slot| numbers[slot][index])?,
                        ),
                        BoundOp::Reflect(source, a, b) => reflect_point(
                            values[source][index]?,
                            values[a][index]?,
                            values[b][index]?,
                        ),
                        BoundOp::Vector(source, a, b) => {
                            let (p, a, b) =
                                (values[source][index]?, values[a][index]?, values[b][index]?);
                            let p = Point {
                                x: p.x + (b.x - a.x),
                                y: p.y + (b.y - a.y),
                            };
                            (p.x.is_finite() && p.y.is_finite()).then_some(p)
                        }
                    }
                };
                let mut result = [None; LANES];
                for (index, point) in result[..length].iter_mut().enumerate() {
                    *point = lane(index);
                }
                values[slot] = result;
            }
            for (output, &slot) in self.outputs.iter().enumerate() {
                for index in 0..length {
                    if matches!(self.ops[slot], BoundOp::Number(_)) {
                        emit(output, None, numbers[slot][index]);
                    } else {
                        emit(output, values[slot][index], None);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_mappings_freeze_omitted_constructions_in_each_parent_lane() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 8., y: -6. }));
        let driver = doc.add(Kind::Free(Point { x: 2., y: 4. }));
        // This omitted construction depends on both the inner seed and the
        // outer driver. It must use the parent seed, before substitution.
        let external = doc.add(Kind::Midpoint(seed, driver));
        let output = doc.add(Kind::Midpoint(seed, external));
        let mapped = doc
            .add_mapped_element(output, seed, driver, &[output])
            .unwrap();
        let twice = doc
            .add_mapped_element(mapped, driver, external, &[mapped])
            .unwrap();
        let inputs: Vec<_> = (0..131)
            .map(|i| {
                (i != 63).then_some(Point {
                    x: i as f64 / 3.,
                    y: -(i as f64) / 7.,
                })
            })
            .collect();
        let reference = |doc: &Document, id: usize| {
            let Kind::MappedElement(d) = &doc.objects[id - 1].kind else {
                panic!()
            };
            let point = doc.position(d.replacement())?;
            if !point.x.is_finite() || !point.y.is_finite() {
                return None;
            }
            let external =
                doc.mapped_external_inputs(&[d.seed()], &[vec![d.replacement()]], d.carried())?;
            let mut sample = doc.clone_for_evaluation();
            sample.freeze_scoped_evaluation_inputs(doc, &external)?;
            sample.set_evaluation_point(d.seed(), point)?;
            sample.position(d.output())
        };
        for revision in 0..2 {
            if revision == 1 {
                doc.objects.edit(external - 1).kind = Kind::Midpoint(driver, driver);
            }
            let mut parent = doc.evaluated().0;
            parent
                .set_evaluation_point(seed, Point { x: 19., y: -17. })
                .unwrap();
            let batch = PointProgram::get(&parent, &[driver], &[mapped, twice])
                .unwrap()
                .bind(&parent)
                .unwrap();
            let columns = batch.evaluate(std::slice::from_ref(&inputs));
            for (lane, value) in inputs.iter().enumerate() {
                for (column, id) in [mapped, twice].into_iter().enumerate() {
                    let expected = value.and_then(|value| {
                        let mut sample = parent.clone_for_evaluation();
                        sample.set_evaluation_point(driver, value)?;
                        reference(&sample, id)
                    });
                    exact(columns[column][lane], expected);
                }
            }
        }
    }

    #[test]
    fn mixed_state_tape_matches_scoped_branch_replay_across_tiles() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let fixed = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        let number = doc.add_named_parameter("n", 1., (0, 0)).unwrap();
        let next_number = doc
            .add_parameter_calculation(number, "n+1", (0, 0))
            .unwrap();
        let midpoint = doc.add(Kind::Midpoint(seed, fixed));
        let scaled = doc.add(Kind::CalculatedRatioScaledPoint(midpoint, fixed, number));
        let mut sample = doc.clone_for_evaluation();
        sample.freeze_evaluation_inputs(&doc, &[fixed]).unwrap();
        let roots = [midpoint, next_number, scaled, number];
        let batch = PointProgram::get(&sample, &[seed, number], &roots)
            .unwrap()
            .bind(&sample)
            .unwrap();
        let points: Vec<_> = (0..137)
            .map(|i| {
                Some(MappedValue::Point(Point {
                    x: i as f64,
                    y: i as f64 / 7.,
                }))
            })
            .collect();
        let numbers: Vec<_> = (0..137)
            .map(|i| Some(MappedValue::Number(i as f64 / 3.)))
            .collect();
        let actual = batch.evaluate_mapped(&[points.clone(), numbers.clone()]);
        for lane in 0..points.len() {
            sample.clear_evaluation_results();
            points[lane].unwrap().substitute(&mut sample, seed).unwrap();
            numbers[lane]
                .unwrap()
                .substitute(&mut sample, number)
                .unwrap();
            for (column, &root) in roots.iter().enumerate() {
                match (actual[column][lane], MappedValue::read(&sample, root)) {
                    (Some(MappedValue::Point(a)), Some(MappedValue::Point(b))) => {
                        exact(Some(a), Some(b))
                    }
                    (Some(MappedValue::Number(a)), Some(MappedValue::Number(b))) => {
                        assert_eq!(a.to_bits(), b.to_bits())
                    }
                    (None, None) => (),
                    _ => panic!("different value types"),
                }
            }
        }
        let invalid =
            batch.evaluate_mapped(&[vec![None], vec![Some(MappedValue::Number(f64::NAN))]]);
        assert!(invalid.iter().all(|column| column[0].is_none()));
    }

    fn exact(a: Option<Point>, b: Option<Point>) {
        assert_eq!(
            a.map(|p| (p.x.to_bits(), p.y.to_bits())),
            b.map(|p| (p.x.to_bits(), p.y.to_bits()))
        );
    }

    #[test]
    fn measured_point_values_feed_numeric_chains_and_keep_segment_clamps() {
        use crate::table_records::ParameterCalculationGroup;
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: -1.7, y: 2.3 }));
        let b = doc.add(Kind::Free(Point { x: 3.1, y: -9.7 }));
        let path = doc.add(Kind::Segment(a, b));
        let seed = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let value = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: seed,
            second: path,
        });
        let group =
            ParameterCalculationGroup::from_expression(value as u32, "n", "sin(n)+n^2", (0, 0))
                .unwrap();
        let calculated = doc.add(Kind::ParameterCalculation(value, group));
        let target = doc.add(Kind::CalculatedRatioScaledPoint(b, a, calculated));
        let lanes: Vec<_> = (0..130)
            .map(|i| {
                Some(Point {
                    x: i as f64 / 7.,
                    y: i as f64 / -3.,
                })
            })
            .collect();
        for kind in [
            Kind::Free(Point { x: 1., y: 2. }),
            Kind::PointOnSegment(path, 0.5),
        ] {
            doc.objects.edit(seed - 1).kind = kind;
            let program = PointProgram::get(&doc, &[seed], &[target]).unwrap();
            let outputs = program
                .bind(&doc)
                .unwrap()
                .evaluate(std::slice::from_ref(&lanes))
                .remove(0);
            for (index, value) in lanes.iter().enumerate() {
                let mut sample = doc.clone_for_evaluation();
                sample.ensure_evaluation_context(&doc).unwrap();
                sample.set_evaluation_point(seed, value.unwrap()).unwrap();
                exact(outputs[index], sample.position(target));
            }
        }
        let mut frozen = doc.clone_for_evaluation();
        frozen.freeze_evaluation_inputs(&doc, &[path]).unwrap();
        frozen.objects.edit(b - 1).kind = Kind::Free(Point { x: 30., y: 40. });
        let program = PointProgram::get(&frozen, &[seed], &[target]).unwrap();
        let output = program
            .bind(&frozen)
            .unwrap()
            .evaluate(std::slice::from_ref(&lanes))
            .remove(0);
        for (index, value) in lanes.iter().enumerate() {
            let mut sample = frozen.clone_for_evaluation();
            sample.set_evaluation_point(seed, value.unwrap()).unwrap();
            exact(output[index], sample.position(target));
        }
    }

    #[test]
    fn numeric_chains_match_scoped_evaluation_and_rebind_edits() {
        use crate::table_records::{ParameterCalculationGroup, ParameterGroup};
        let mut doc = Document::new();
        let driver = doc.add(Kind::Parameter(ParameterGroup::named("n", 0.5).unwrap()));
        let constant = doc.add(Kind::Parameter(ParameterGroup::named("k", 2.).unwrap()));
        let group = ParameterCalculationGroup::from_sources(
            &[driver as u32, constant as u32],
            "sin(v1)+v1^2/v2",
            (0, 0),
            &[NumericUnit::Scalar, NumericUnit::Scalar],
        )
        .unwrap();
        let value = doc.add(Kind::ParameterCalculation(driver, group));
        let group = ParameterCalculationGroup::from_expression(
            value as u32,
            "n",
            "sqrt(abs(n))+ln(abs(n))",
            (0, 0),
        )
        .unwrap();
        let second = doc.add(Kind::ParameterCalculation(value, group));
        let center = doc.add(Kind::Free(Point { x: -1.7, y: 2.3 }));
        let source = doc.add(Kind::Free(Point { x: 3.1, y: -9.7 }));
        let target = doc.add(Kind::CalculatedRatioScaledPoint(source, center, second));
        let lanes: Vec<_> = (0..130).map(|i| Some((i as f64 - 65.) / 17.)).collect();
        let old = doc.evaluated();
        for radians in [false, true] {
            doc.angle_unit_radians = radians;
            for constant_value in [2., 0., -3.] {
                doc.objects.edit(constant - 1).kind =
                    Kind::Parameter(ParameterGroup::named("k", constant_value).unwrap());
                let program = PointProgram::get(&doc, &[driver], &[target]).unwrap();
                let outputs = program
                    .bind(&doc)
                    .unwrap()
                    .evaluate_numbers(std::slice::from_ref(&lanes))
                    .remove(0);
                for (index, value) in lanes.iter().enumerate() {
                    let mut sample = doc.clone_for_evaluation();
                    sample.ensure_evaluation_context(&doc).unwrap();
                    sample
                        .set_evaluation_number(driver, value.unwrap())
                        .unwrap();
                    exact(outputs[index], sample.position(target));
                }
            }
        }
        let program = PointProgram::get(&old, &[driver], &[target]).unwrap();
        let output = program
            .bind(&old)
            .unwrap()
            .evaluate_numbers(&[vec![Some(0.5)]])
            .remove(0);
        exact(output[0], old.position(target));
        doc.objects.edit(value - 1).deleted = true;
        assert!(PointProgram::get(&doc, &[driver], &[target]).is_none());
    }

    #[test]
    fn moving_mirrors_preserve_degeneracy_frozen_overrides_and_kind_edits() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: -1., y: 2. }));
        let end_point = Point { x: 7., y: -3. };
        let end = doc.add(Kind::Free(end_point));
        let source = doc.add(Kind::Free(Point { x: 5., y: 9. }));
        let mirror = doc.add(Kind::Line(seed, end));
        let reflected = doc.add(Kind::ReflectedPoint(source, mirror));
        let inputs = vec![Some(end_point), Some(Point { x: -9.1, y: -0. })];
        let frozen_ids = [mirror];
        for frozen in [false, true] {
            let mut parent = doc.clone_for_evaluation();
            parent
                .freeze_scoped_evaluation_inputs(&doc, if frozen { &frozen_ids } else { &[] })
                .unwrap();
            let program = PointProgram::get(&parent, &[seed], &[reflected]).unwrap();
            let output = program
                .bind(&parent)
                .unwrap()
                .evaluate(std::slice::from_ref(&inputs))
                .remove(0);
            assert_eq!(output[0].is_some(), frozen);
            for (index, point) in inputs.iter().enumerate() {
                let mut sample = parent.clone_for_evaluation();
                sample.set_evaluation_point(seed, point.unwrap()).unwrap();
                exact(output[index], sample.position(reflected));
            }
        }
        // Same dependency IDs, different construction: the cached lowering of
        // a plain mirror must not survive this edit.
        doc.objects.edit(mirror - 1).kind = Kind::Perpendicular(seed, end);
        assert!(PointProgram::get(&doc, &[seed], &[reflected]).is_none());
    }

    #[test]
    fn moving_centers_ratios_and_angles_match_recursive_math_and_degeneracy() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let base = doc.add(Kind::Free(Point { x: 1.7, y: -2.9 }));
        let source = doc.add(Kind::Free(Point { x: -8.1, y: 7.3 }));
        let scaled = doc.add(Kind::RatioScaledPoint(source, seed, origin, base, seed));
        let angled = doc.add(Kind::AngleRotatedPoint(scaled, seed, base, origin, seed));
        let fixed_scale = doc.add(Kind::ScaledPoint(source, seed, -0.75));
        let fixed_angle = doc.add(Kind::RotatedPoint(fixed_scale, seed, 17.3));
        let roots = [seed, scaled, angled, fixed_scale, fixed_angle];
        let program = PointProgram::get(&doc, &[seed], &roots).unwrap();
        let lanes: Vec<_> = (0..130)
            .map(|i| {
                if i == 65 {
                    None
                } else {
                    Some(Point {
                        x: (i as f64 - 64.) / 7.,
                        y: (i as f64 - 64.) / -3.,
                    })
                }
            })
            .collect();
        for base_point in [Point { x: 1.7, y: -2.9 }, Point { x: 0., y: 0. }] {
            doc.objects.edit(base - 1).kind = Kind::Free(base_point);
            let reused = PointProgram::get(&doc, &[seed], &roots).unwrap();
            assert!(Arc::ptr_eq(&program, &reused));
            let outputs = reused
                .bind(&doc)
                .unwrap()
                .evaluate(std::slice::from_ref(&lanes));
            for (lane, value) in lanes.iter().enumerate() {
                let mut sample = doc.clone_for_evaluation();
                sample.ensure_evaluation_context(&doc).unwrap();
                if let Some(value) = value {
                    sample.set_evaluation_point(seed, *value).unwrap();
                }
                for (output, &root) in outputs.iter().zip(&roots) {
                    exact(output[lane], value.and_then(|_| sample.position(root)));
                }
            }
        }
    }

    #[test]
    fn batches_share_ancestors_preserve_rounding_and_rebind_live_inputs() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let center = doc.add(Kind::Free(Point { x: 10., y: -3. }));
        let middle = doc.add(Kind::Midpoint(seed, center));
        let rotated = doc.add(Kind::RotatedPoint(middle, center, 37.));
        let roots = [middle, rotated];
        let first = PointProgram::get(&doc, &[seed], &roots).unwrap();
        assert_eq!(first.nodes.len(), 4);
        let old = doc.evaluated();
        let lanes = vec![
            Some(Point { x: -0., y: 0. }),
            Some(Point { x: 9.1, y: -73. }),
            None,
        ];
        for x in [10., 19.] {
            doc.objects.edit(center - 1).kind = Kind::Free(Point { x, y: -3. });
            let program = PointProgram::get(&doc, &[seed], &roots).unwrap();
            assert!(Arc::ptr_eq(&first, &program));
            let outputs = program
                .bind(&doc)
                .unwrap()
                .evaluate(std::slice::from_ref(&lanes));
            for (lane, seed_value) in lanes.iter().enumerate() {
                let mut sample = doc.clone_for_evaluation();
                sample.ensure_evaluation_context(&doc).unwrap();
                if let Some(point) = seed_value {
                    sample.set_evaluation_point(seed, *point).unwrap();
                }
                for (slot, &root) in roots.iter().enumerate() {
                    exact(
                        outputs[slot][lane],
                        seed_value.and_then(|_| sample.position(root)),
                    );
                }
            }
        }
        let old_program = PointProgram::get(&old, &[seed], &roots).unwrap();
        assert!(Arc::ptr_eq(&first, &old_program));
        assert_ne!(
            old_program
                .bind(&old)
                .unwrap()
                .evaluate(std::slice::from_ref(&lanes)),
            first
                .bind(&doc)
                .unwrap()
                .evaluate(std::slice::from_ref(&lanes))
        );
        // The dependency edges are unchanged, but operation constants changed.
        doc.objects.edit(rotated - 1).kind = Kind::RotatedPoint(middle, center, 61.);
        let changed = PointProgram::get(&doc, &[seed], &roots).unwrap();
        assert!(!Arc::ptr_eq(&first, &changed));
        doc.objects.edit(middle - 1).kind = Kind::Midpoint(rotated, seed);
        assert!(PointProgram::get(&doc, &[seed], &roots).is_none());
    }

    #[test]
    fn undefined_transform_and_deleted_ancestors_do_not_poison_other_outputs() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let mirror = doc.add(Kind::Line(a, b));
        let reflected = doc.add(Kind::ReflectedPoint(seed, mirror));
        let roots = [seed, reflected];
        let program = PointProgram::get(&doc, &[seed], &roots).unwrap();
        let input = vec![Some(Point { x: 2., y: 3. })];
        let values = program
            .bind(&doc)
            .unwrap()
            .evaluate(std::slice::from_ref(&input));
        assert_eq!(values, vec![input.clone(), vec![None]]);
        doc.objects.edit(b - 1).kind = Kind::Free(Point { x: 1., y: 0. });
        let reused = PointProgram::get(&doc, &[seed], &roots).unwrap();
        assert!(Arc::ptr_eq(&program, &reused));
        assert!(reused.bind(&doc).unwrap().evaluate(&[input])[1][0].is_some());
        doc.objects.edit(a - 1).deleted = true;
        let program = PointProgram::get(&doc, &[seed], &roots).unwrap();
        assert!(program.bind(&doc).is_none());
    }
}
