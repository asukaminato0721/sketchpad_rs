use super::{Editor, is_circle_construction_host};
use sketchpad_engine::document::transform_model::{ElementType, NativeTransformFlavor};
use sketchpad_engine::document::{IterationDepth, Kind, Point};

impl Editor {
    pub(super) fn pair(&mut self, mode: u8) {
        if self.doc.partial_view && !(self.doc.page_count > 1 && matches!(mode, 0..=4)) {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        if ids.len() != 2 {
            self.status = "Select exactly two points".into();
            return;
        }
        let (a, b) = (ids[0], ids[1]);
        if a == b
            || self.doc.objects[a - 1].page != self.doc.active_page
            || self.doc.objects[b - 1].page != self.doc.active_page
            || self.doc.page_count > 1
                && self
                    .doc
                    .position(a)
                    .unwrap()
                    .distance(self.doc.position(b).unwrap())
                    <= 1e-9
        {
            self.status = "Select two distinct points on the current page".into();
            return;
        }
        let kind = match mode {
            0 => Kind::Segment(a, b),
            1 => Kind::Circle(a, b),
            3 => Kind::Line(a, b),
            4 => Kind::Ray(a, b),
            _ => {
                if !self
                    .doc
                    .objects
                    .iter()
                    .any(|o| matches!(o.kind,Kind::Segment(x,y) if (x==a&&y==b)||(x==b&&y==a)))
                {
                    self.doc.add(Kind::Segment(a, b));
                }
                Kind::Midpoint(a, b)
            }
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    pub(super) fn derived_line(&mut self, perpendicular: bool) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let points: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        let lines: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.line_points(*id).is_some())
            .collect();
        if points.len() != 1 || lines.len() != 1 || self.doc.selected.len() != 2 {
            self.status = "Select one point and one straight object".into();
            return;
        }
        if self.doc.objects[points[0] - 1].page != self.doc.active_page
            || self.doc.objects[lines[0] - 1].page != self.doc.active_page
        {
            self.status = "Select objects on the current page".into();
            return;
        }
        let kind = if perpendicular {
            Kind::Perpendicular(points[0], lines[0])
        } else {
            Kind::Parallel(points[0], lines[0])
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    pub(super) fn circle_by_segment(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let selected = &self.selection_order;
        let centers: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        let segments: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| {
                matches!(
                    self.doc.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::Segment(_, _))
                )
            })
            .collect();
        if selected.len() != 2 || centers.len() != 1 || segments.len() != 1 {
            self.status = "Select one center point and one radius segment".into();
            return;
        }
        if self.doc.objects[centers[0] - 1].page != self.doc.active_page
            || self.doc.objects[segments[0] - 1].page != self.doc.active_page
        {
            self.status = "Select objects on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::CircleBySegment(centers[0], segments[0]));
        self.select_only(id);
    }
    pub(super) fn polygon_interior(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let vertices = self.selection_order.clone();
        if vertices.len() < 3 || vertices.iter().any(|&id| self.doc.position(id).is_none()) {
            self.status = "Select at least three points in polygon order".into();
            return;
        }
        if vertices
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select points on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::Polygon(vertices));
        self.select_only(id);
        self.status = format!("Constructed polygon interior {id}");
    }
    pub(super) fn circle_interior(&mut self) {
        let [circle] = self.selection_order.as_slice() else {
            self.status = "Select one circle for the circle interior".into();
            return;
        };
        match self.doc.add_circle_interior(*circle) {
            Ok(id) => {
                self.select_only(id);
                self.status = format!("Constructed circle interior {id}");
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn sector_boundary(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let [arc] = self.selection_order.as_slice() else {
            self.status = "Select one arc for the sector interior".into(); return;
        };
        match self.doc.add_sector_boundary(*arc) {
            Ok(id) => self.select_only(id),
            Err(error) => self.status = error,
        }
    }
    pub(super) fn construct_locus(&mut self) {
        let ids = self.selection_order.clone();
        if ids.len() != 2 {
            self.status = "Select a moving point or parameter and a dependent object".into();
            return;
        }
        let (driver, target) = if matches!(
            self.doc.objects[ids[0] - 1].kind,
            Kind::PointOnSegment(_, _)
                | Kind::PointOnCircle(_, _, _)
                | Kind::PointOnArc(_, _)
                | Kind::PointOnCurve(_, _)
                | Kind::PointOnPolygon(_, _, _)
                | Kind::PointOnSector(_, _)
                | Kind::Parameter(_)
        ) {
            (ids[0], ids[1])
        } else {
            (ids[1], ids[0])
        };
        match self.doc.construct_locus(driver, target) {
            Ok(id) => {
                self.select_only(id);
                self.status = format!("Constructed locus {id}");
            }
            Err(error) => self.status = error,
        }
    }
    fn iterate_geometry_selection(&mut self, depth: u32) -> bool {
        if self.selection_order.len() < 3 {
            return false;
        }
        let mut mappings = self
            .selection_order
            .iter()
            .filter_map(|&id| {
                let definition = self
                    .doc
                    .objects
                    .get(id.checked_sub(1)?)?
                    .kind
                    .transform_definition()?;
                (definition.element() == ElementType::Point
                    && self.selection_order.contains(&definition.source()))
                .then_some((definition.source(), id))
            })
            .collect::<Vec<_>>();
        for &image in &self.selection_order {
            let object = &self.doc.objects[image - 1];
            if !object.kind.is_iteration_depth_value() {
                continue;
            }
            for seed in object.kind.dependencies() {
                if self.selection_order.contains(&seed)
                    && matches!(self.doc.objects[seed - 1].kind, Kind::Parameter(_))
                    && self.doc.objects[seed - 1].kind.is_iteration_depth_value()
                {
                    mappings.push((seed, image));
                }
            }
        }
        let [(seed, image)] = mappings.as_slice() else {
            return false;
        };
        let (seed, image) = (*seed, *image);
        let geometry = |kind: &Kind| {
            kind.is_segment()
                || kind.is_circle()
                || kind.is_polygon()
                || matches!(
                    kind,
                    Kind::Ray(..)
                        | Kind::Line(..)
                        | Kind::Bisector(..)
                        | Kind::Parallel(..)
                        | Kind::Perpendicular(..)
                        | Kind::CenterArc(..)
                        | Kind::ThreePointArc(..)
                        | Kind::ArcOnCircle(..)
                        | Kind::Locus(..)
                )
                || kind
                    .transform_definition()
                    .is_some_and(|definition| definition.element() != ElementType::Point)
        };
        let bindings = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| {
                self.doc
                    .objects
                    .get(id - 1)
                    .is_some_and(|object| geometry(&object.kind))
            })
            .collect::<Vec<_>>();
        if bindings.is_empty() {
            return false;
        }
        let values = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.objects[id - 1].kind.is_iteration_depth_value())
            .filter(|id| ![seed, image].contains(id))
            .collect::<Vec<_>>();
        if values.len() > 1
            || self.selection_order.iter().any(|id| {
                ![seed, image].contains(id) && !bindings.contains(id) && !values.contains(id)
            })
        {
            return false;
        }
        let broad = !matches!(self.doc.objects[seed - 1].kind, Kind::Free(_))
            || bindings.len() > 1
            || bindings
                .iter()
                .any(|&id| match self.doc.objects[id - 1].kind {
                    Kind::Segment(..) | Kind::Circle(..) => false,
                    Kind::Ray(..) => values.is_empty(),
                    _ => true,
                });
        if !broad {
            return false;
        }
        let mut dependent = vec![false; self.doc.objects.len() + 1];
        dependent[seed] = true;
        for object in &self.doc.objects {
            if object.id != seed {
                dependent[object.id] = self.doc.dependencies_of(object.id).is_some_and(|parents| {
                    parents
                        .into_iter()
                        .any(|parent| dependent.get(parent).copied().unwrap_or(false))
                });
            }
        }
        let mut carried = std::collections::BTreeSet::new();
        let mut pending = bindings.clone();
        pending.push(image);
        while let Some(id) = pending.pop() {
            if id == seed || carried.contains(&id) {
                continue;
            }
            if dependent[id] || bindings.contains(&id) || id == image {
                carried.insert(id);
                pending.extend(self.doc.dependencies_of(id).unwrap_or_default());
            }
        }
        let depth = values
            .first()
            .copied()
            .map_or(IterationDepth::Fixed(depth), IterationDepth::Value);
        match self.doc.add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![image]],
            &carried.into_iter().collect::<Vec<_>>(),
            &bindings,
            depth,
        ) {
            Ok(id) => {
                self.select_only(id);
                self.status = format!("Created iteration {id}");
            }
            Err(error) => self.status = error,
        }
        true
    }
    pub(super) fn iterate_selected(&mut self, depth: u32) {
        if self.iterate_geometry_selection(depth) {
            return;
        }
        if !(2..=8).contains(&self.selection_order.len()) {
            self.status =
                "Select a free point and transformed image; optionally add their segment or circle, ray with a depth parameter, or parameter mapping with a numeric label".into();
            return;
        }
        let seed = self
            .selection_order
            .iter()
            .copied()
            .find(|id| matches!(self.doc.objects[*id - 1].kind, Kind::Free(_)));
        let image = self.selection_order.iter().copied().find(|id| {
            self.doc.objects[*id - 1]
                .kind
                .transform_definition()
                .is_some_and(|definition| definition.element() == ElementType::Point)
        });
        let (Some(seed), Some(image)) = (seed, image) else {
            self.status = "Select a free point and its directly translated, rotated, dilated, or reflected image".into();
            return;
        };
        let selected_polygon = self
            .selection_order
            .iter()
            .copied()
            .find(|id| matches!(self.doc.objects[*id - 1].kind, Kind::Polygon(_)));
        let result = if let Some(polygon) = selected_polygon {
            let images = self
                .selection_order
                .iter()
                .copied()
                .filter(|id| {
                    self.doc.objects[*id - 1]
                        .kind
                        .transform_definition()
                        .is_some_and(|definition| {
                            definition.element() == ElementType::Point
                                && definition.source() == seed
                                && matches!(
                                    definition.wire_flavor(),
                                    NativeTransformFlavor::FixedPolar
                                        | NativeTransformFlavor::FixedCartesian
                                )
                        })
                })
                .collect::<Vec<_>>();
            let segments = self
                .selection_order
                .iter()
                .copied()
                .filter(|id| matches!(self.doc.objects[*id - 1].kind, Kind::Segment(_, _)))
                .collect::<Vec<_>>();
            let depth_parameter = self
                .selection_order
                .iter()
                .copied()
                .find(|id| matches!(self.doc.objects[*id - 1].kind, Kind::Parameter(_)));
            let depth = depth_parameter.map_or(IterationDepth::Fixed(depth), IterationDepth::Value);
            if images.len() == 2 {
                self.doc.add_branching_polygon_iteration(
                    seed, images[0], images[1], polygon, &segments, depth,
                )
            } else {
                Err("Select two directly translated images for the branching triangle".into())
            }
        } else if self.selection_order.len() >= 5 {
            let calculation =
                self.selection_order
                    .iter()
                    .find_map(|id| match self.doc.objects[*id - 1].kind {
                        Kind::PointNumericLabel(_, value)
                            if self.selection_order.contains(&value) =>
                        {
                            Some(value)
                        }
                        _ => None,
                    });
            let parameter = calculation.and_then(|id| {
                let sources = self.doc.objects[id - 1].kind.dependencies();
                self.selection_order.iter().copied().find(|source| {
                    sources.contains(source)
                        && matches!(self.doc.objects[*source - 1].kind, Kind::Parameter(_))
                })
            });
            let label = self.selection_order.iter().copied().find(|id| {
                matches!(
                    self.doc.objects[*id - 1].kind,
                    Kind::PointNumericLabel(_, _)
                )
            });
            match (parameter, calculation, label) {
                (Some(parameter), Some(calculation), Some(label)) => {
                    let depth = if self.selection_order.len() == 6 {
                        self.selection_order
                            .iter()
                            .copied()
                            .find(|id| {
                                *id != parameter
                                    && *id != calculation
                                    && self.doc.objects[*id - 1].kind.is_iteration_depth_value()
                                    && self.doc.unitless_numeric_value(*id).is_some()
                            })
                            .map(IterationDepth::Value)
                    } else {
                        Some(IterationDepth::Fixed(depth))
                    };
                    match depth {
                        Some(depth) => self.doc.add_point_parameter_iteration(
                            seed,
                            parameter,
                            image,
                            calculation,
                            label,
                            depth,
                        ),
                        None => Err("Select a distinct scalar depth parameter along with the numeric mapping".into()),
                    }
                }
                _ => Err(
                    "Select the point mapping, parameter mapping, and attached numeric label"
                        .into(),
                ),
            }
        } else if self.selection_order.len() == 4 {
            let carried = self.selection_order.iter().copied().find(|id| {
                matches!(
                    self.doc.objects[*id - 1].kind,
                    Kind::Segment(_, _) | Kind::Ray(_, _) | Kind::Circle(_, _)
                )
            });
            let value = self.selection_order.iter().copied().find(|id| {
                self.doc.objects[*id - 1].kind.is_iteration_depth_value()
                    && self.doc.unitless_numeric_value(*id).is_some()
            });
            match (carried,value) {
                (Some(object),Some(value))=>match self.doc.objects[object-1].kind {
                    Kind::Segment(_,_)=>self.doc.add_segment_iteration(seed, image, object, IterationDepth::Value(value)),
                    Kind::Ray(_,_)=>self.doc.add_ray_iteration(seed,image,object,value),
                    Kind::Circle(_,_)=>self.doc.add_circle_iteration(seed, image, object, IterationDepth::Value(value)),
                    _=>unreachable!(),
                },
                _=>Err("Select the connecting segment, ray, or circle and an scalar parameter or calculation for depth".into()),
            }
        } else if self.selection_order.len() == 3 {
            let segment = self.selection_order.iter().copied().find(|id| {
                matches!(
                    self.doc.objects[*id - 1].kind,
                    Kind::Segment(_, _) | Kind::Circle(_, _)
                )
            });
            match segment {
                Some(segment) => match self.doc.objects[segment - 1].kind {
                    Kind::Circle(_, _) => self.doc.add_circle_iteration(seed, image, segment, IterationDepth::Fixed(depth)),
                    _ => self.doc.add_segment_iteration(seed, image, segment, IterationDepth::Fixed(depth)),
                },
                None => match self
                    .selection_order
                    .iter()
                    .copied()
                    .find(|id| self.doc.objects[*id - 1].kind.is_iteration_depth_value())
                {
                    Some(value) => self
                        .doc.add_point_iteration(seed, image, IterationDepth::Value(value)),
                    None => Err(
                        "Select their segment, circle, or an scalar parameter/calculation for depth".into(),
                    ),
                },
            }
        } else {
            self.doc
                .add_point_iteration(seed, image, IterationDepth::Fixed(depth))
        };
        match result {
            Ok(id) => {
                self.select_only(id);
                let actual_depth = self.doc.point_iteration_images(id).map_or_else(
                    || match self.doc.objects[id - 1].kind {
                        Kind::BranchingPolygonIteration {
                            depth: IterationDepth::Fixed(n),
                            ..
                        } => n as usize,
                        Kind::BranchingPolygonIteration {
                            depth: IterationDepth::Value(p),
                            ..
                        } => self
                            .doc
                            .unitless_numeric_value(p)
                            .unwrap_or(f64::from(depth)) as usize,
                        _ => depth as usize,
                    },
                    |points| points.len(),
                );
                self.status = format!("Created iteration {id} to depth {actual_depth}");
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn point_on_object(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let host = self.selection_order.iter().copied().find(|id| {
            self.doc
                .objects
                .get(id.saturating_sub(1))
                .is_some_and(|o| o.kind.is_polygon() || o.kind.is_arc() || is_circle_construction_host(&o.kind)
                    || matches!(o.kind, Kind::Locus(..) | Kind::ParameterLocus(..) | Kind::FunctionPlot(..) | Kind::SampledMapped(_) | Kind::SectorBoundary(_)))
                || matches!(
                    self.doc.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                    Some(
                        Kind::Polygon(_)
                            | Kind::Segment(_, _)
                            | Kind::Ray(_, _)
                            | Kind::Line(_, _)
                            | Kind::ThreePointArc(_, _, _)
                            | Kind::ArcOnCircle(_, _, _)
                            | Kind::CenterArc(_, _, _)
                    )
                )
        });
        let Some(host) = host else {
            self.status = "Select a segment, ray, line, circle, arc, polygon, sector, or point curve".into();
            return;
        };
        if self.doc.objects[host - 1].page != self.doc.active_page {
            self.status = "Select a host on the current page".into();
            return;
        }
        if self.selection_order.len() > 2 {
            self.status = "Select one host, optionally with a point to project".into();
            return;
        }
        let position = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| *id != host)
            .find_map(|id| self.doc.position(id))
            .or_else(|| {
                if let Some((a, b, _)) = self.doc.line_points(host) {
                    Some(Point {
                        x: (a.x + b.x) / 2.,
                        y: (a.y + b.y) / 2.,
                    })
                } else if let Some((center, radius)) = self.doc.circle_points(host) {
                    Some(Point {
                        x: center.x + radius,
                        y: center.y,
                    })
                } else if let Some(vertices) = self.doc.polygon_points(host) {
                    Some(Point {
                        x: (vertices[0].x + vertices[1].x) / 2.,
                        y: (vertices[0].y + vertices[1].y) / 2.,
                    })
                } else {
                    self.doc.point_on_arc(host, 0.5).or_else(|| self.doc.point_on_curve(host, 0.5)).or_else(|| self.doc.point_on_sector(host, 0.5))
                }
            });
        let Some(kind) = position.and_then(|p| self.doc.constrained_point(host, p)) else {
            self.status = "Selected host has no usable geometry".into();
            return;
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    pub(super) fn intersection(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids: Vec<_> = self.selection_order.clone();
        if ids.len() != 2 {
            self.status = "Select intersecting lines, circles, arcs, polygons or curves".into();
            return;
        }
        let (a, b) = (ids[0], ids[1]);
        if self.doc.objects[a - 1].page != self.doc.active_page
            || self.doc.objects[b - 1].page != self.doc.active_page
        {
            self.status = "Select objects on the current page".into();
            return;
        }
        let round = |id: usize| {
            self.doc.circle_points(id).is_some()
                || self.doc.objects[id - 1].kind.is_arc()
                    && self.doc.arc_points(id, 1).is_some()
        };
        let kinds = if self.doc.line_points(a).is_some() && self.doc.line_points(b).is_some() {
            vec![Kind::Intersection(a, b)]
        } else if self.doc.can_construct_boundary_intersection(a, b)
            || self.doc.can_construct_boundary_intersection(b, a) {
            let (straight, polygon) = if self.doc.objects[b - 1].kind.is_polygon() { (a, b) } else { (b, a) };
            vec![Kind::BoundaryIntersection(straight, polygon)]
        } else if (self.doc.line_points(a).is_some() && round(b))
            || (round(a) && self.doc.line_points(b).is_some())
        {
            let (line, circle) = if self.doc.line_points(a).is_some() {
                (a, b)
            } else {
                (b, a)
            };
            vec![
                Kind::LineCircleIntersection(line, circle, 0),
                Kind::LineCircleIntersection(line, circle, 1),
            ]
        } else if let Some(points) = self.doc.curve_intersections(a, b) {
            (0..points.len()).map(|branch| Kind::CurveIntersection(a, b, branch as u32)).collect()
        } else if let Some(points) = self.doc.curve_intersections(b, a) {
            (0..points.len()).map(|branch| Kind::CurveIntersection(b, a, branch as u32)).collect()
        } else if round(a) && round(b) {
            vec![
                Kind::CircleCircleIntersection(a, b, 0),
                Kind::CircleCircleIntersection(a, b, 1),
            ]
        } else {
            self.status = "Select intersecting lines, circles, arcs, polygons or curves".into();
            return;
        };
        self.doc.selected.clear();
        self.selection_order.clear();
        for kind in kinds {
            if self.doc.page_count > 1 {
                let mut probe = self.doc.clone();
                let probe_id = probe.add(kind.clone());
                if probe.position(probe_id).is_none() {
                    continue;
                }
            }
            let id = self.doc.add(kind);
            self.doc.selected.insert(id);
            self.selection_order.push(id);
        }
        if self.selection_order.is_empty() {
            self.status = "Selected paths have no intersection".into();
        }
    }
    pub(super) fn angle_bisector(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 || ids.iter().any(|id| self.doc.position(*id).is_none()) {
            self.status = "Select three points in order, with the vertex second".into();
            return;
        }
        if ids
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select points on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::Bisector(ids[0], ids[1], ids[2]));
        self.select_only(id);
    }
    pub(super) fn three_point_arc(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 || ids.iter().any(|id| self.doc.position(*id).is_none()) {
            self.status = "Select the arc start, a point on the arc, then its end".into();
            return;
        }
        if ids
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select points on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::ThreePointArc(ids[0], ids[1], ids[2]));
        self.select_only(id);
    }
    pub(super) fn center_arc(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 || ids.iter().any(|id| self.doc.position(*id).is_none()) {
            self.status = "Select the center, arc start, then its end direction".into();
            return;
        }
        if ids
            .iter()
            .any(|id| self.doc.objects[*id - 1].page != self.doc.active_page)
        {
            self.status = "Select points on the current page".into();
            return;
        }
        let id = self.doc.add(Kind::CenterArc(ids[0], ids[1], ids[2]));
        self.select_only(id);
    }
    pub(super) fn arc_on_circle(&mut self) {
        if self.doc.partial_view && self.doc.page_count == 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 {
            self.status = "Select a circle, then its start and end points".into();
            return;
        }
        let (circle, start, end) = (ids[0], ids[1], ids[2]);
        if [circle, start, end]
            .iter()
            .any(|id| self.doc.objects.get(id.saturating_sub(1)).is_none_or(|object| object.page != self.doc.active_page))
        {
            self.status = "Select objects on the current page".into();
            return;
        }
        let valid = [start, end].iter().all(|id| self.doc.objects.get(id.saturating_sub(1))
            .is_some_and(|object| !object.deleted && object.kind.is_point()))
            && self.doc.objects.get(circle.saturating_sub(1))
                .is_some_and(|object| !object.deleted && object.kind.is_circle());
        if !valid {
            self.status = "Select a circle, a start point for the radius, and an end direction point".into();
            return;
        }
        let id = self.doc.add(Kind::ArcOnCircle(circle, start, end));
        self.select_only(id);
    }
}
