//! Construction descriptions use stored definitions, without evaluating geometry
//! or recursively expanding source formulas.
use super::i18n::tr;
use sketchpad_engine::document::transform_model::{DisplacementExpr, NumericExpr, Transform};
use sketchpad_engine::document::{Document, Kind, Object};
use sketchpad_engine::table_records::NumericUnit;

pub(super) struct Definition {
    template: &'static str,
    operands: Vec<String>,
}

impl Definition {
    fn new(template: &'static str, operands: impl IntoIterator<Item = String>) -> Self {
        Self {
            template,
            operands: operands.into_iter().collect(),
        }
    }

    pub(super) fn text(&self) -> String {
        // Translate only the template. Labels and formulas are document content.
        // Substitute in one pass so braces in labels cannot become placeholders.
        let template = tr(self.template);
        let mut parts = template.split("{}");
        let mut result = parts.next().unwrap_or_default().to_owned();
        for (operand, part) in self.operands.iter().zip(parts) {
            result.push_str(operand);
            result.push_str(part);
        }
        result
    }
}

pub(super) fn single_line(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

fn source_name(doc: &Document, id: usize) -> Option<String> {
    let object = id.checked_sub(1).and_then(|index| doc.objects.get(index))?;
    if object.deleted {
        return None;
    }
    object
        .label
        .as_ref()
        .and_then(|label| label.name_utf8().ok())
        .filter(|name| !name.is_empty())
        .map(single_line)
        .or_else(|| match &object.kind {
            Kind::Parameter(_) => doc
                .parameter_display_name(id)
                .map(|name| single_line(&name)),
            Kind::Calculation(group) => group.name().map(single_line),
            _ => None,
        })
}

pub(super) fn definition(doc: &Document, object: &Object) -> Definition {
    let reference = |id| match source_name(doc, id) {
        Some(name) => format!("{name} [#{id}]"),
        None => format!("#{id}"),
    };
    let formula_source = |id| source_name(doc, id).unwrap_or_else(|| format!("#{id}"));
    let formula = |text| Definition::new("{}", [text]);
    let refs =
        |template, ids: &[usize]| Definition::new(template, ids.iter().map(|&id| reference(id)));
    match &object.kind {
        Kind::Calculation(group) => formula(group.expression()),
        Kind::ParameterCalculation(_, group) => {
            // Kind dependencies resolve native page-local source IDs and retain
            // binding order. Raw group source IDs can belong to another page.
            let names: Vec<_> = object
                .kind
                .dependencies()
                .into_iter()
                .map(formula_source)
                .collect();
            formula(
                group.expression_for_sources(&names.iter().map(String::as_str).collect::<Vec<_>>()),
            )
        }
        Kind::MeasuredCalculation(source, group) => formula(format!(
            "{} {} {}",
            formula_source(*source),
            if group.divides() { "÷" } else { "·" },
            group.factor(),
        )),
        Kind::LengthPairCalculation(first, second, group) => formula(format!(
            "{} {} {}",
            formula_source(*first),
            group.operator_text(),
            formula_source(*second),
        )),
        Kind::FunctionDefinition(function) => formula(function.display_for_sources(&[])),
        Kind::ReferencedFunction(sources, function) => {
            let names: Vec<_> = sources.iter().map(|&id| formula_source(id)).collect();
            formula(
                function.display_for_sources(&names.iter().map(String::as_str).collect::<Vec<_>>()),
            )
        }
        Kind::Perpendicular(point, line) => {
            refs("Through {}, perpendicular to {}", &[*point, *line])
        }
        Kind::Parallel(point, line) => refs("Through {}, parallel to {}", &[*point, *line]),
        Kind::Segment(a, b) => refs("Endpoints: {}, {}", &[*a, *b]),
        Kind::Line(a, b) => refs("Through {} and {}", &[*a, *b]),
        Kind::Ray(a, b) => refs("From {} through {}", &[*a, *b]),
        Kind::Midpoint(a, b) => refs("Midpoint of {} and {}", &[*a, *b]),
        Kind::SegmentMidpoint(host) => refs("Midpoint of {}", &[*host]),
        Kind::Circle(center, point) => refs("Center: {}, through {}", &[*center, *point]),
        Kind::CircleBySegment(center, segment) => {
            refs("Center: {}, radius from {}", &[*center, *segment])
        }
        Kind::Bisector(first, vertex, last) => refs(
            "Angle bisector: {}, vertex {}, {}",
            &[*first, *vertex, *last],
        ),
        Kind::Intersection(a, b) => refs("Intersection of {} and {}", &[*a, *b]),
        Kind::LineCircleIntersection(a, b, branch)
        | Kind::CircleCircleIntersection(a, b, branch) => Definition::new(
            "Intersection of {} and {}, branch {}",
            [reference(*a), reference(*b), branch.to_string()],
        ),
        Kind::BoundaryIntersection(path, boundary) => refs(
            "First boundary intersection of {} with {}",
            &[*path, *boundary],
        ),
        Kind::CurveIntersection(path, curve, rank) => Definition::new(
            "Intersection of {} and {}, rank {}",
            [reference(*path), reference(*curve), rank.to_string()],
        ),
        Kind::PointOnSegment(host, t)
        | Kind::PointOnRay(host, t)
        | Kind::PointOnLine(host, t)
        | Kind::PointOnArc(host, t)
        | Kind::PointOnCurve(host, t) => {
            Definition::new("On {}, parameter {}", [reference(*host), t.to_string()])
        }
        Kind::PointOnCircle(host, ..) => refs("On circle {}", &[*host]),
        Kind::CircleInterior(host) => refs("Interior of {}", &[*host]),
        Kind::Polygon(vertices) => Definition::new(
            "Vertices: {}",
            [vertices
                .iter()
                .map(|&id| reference(id))
                .collect::<Vec<_>>()
                .join(", ")],
        ),
        Kind::ThreePointArc(a, b, c) => refs("Arc through {}, {}, {}", &[*a, *b, *c]),
        Kind::CenterArc(center, start, end) => {
            refs("Center: {}, start: {}, end: {}", &[*center, *start, *end])
        }
        Kind::ArcOnCircle(circle, start, end) => {
            refs("On circle {}, start: {}, end: {}", &[*circle, *start, *end])
        }
        Kind::SegmentLength(host) => refs("Length of {}", &[*host]),
        Kind::PointDistance(a, b) | Kind::PointLineDistance(a, b) => {
            refs("Distance between {} and {}", &[*a, *b])
        }
        Kind::CircleRadius(host) => refs("Radius of {}", &[*host]),
        Kind::CircleCircumference(host) | Kind::PolygonPerimeter(host) => {
            refs("Perimeter of {}", &[*host])
        }
        Kind::CircleArea(host) | Kind::PolygonArea(host) => refs("Area of {}", &[*host]),
        Kind::ArcLength(host) => refs("Arc length of {}", &[*host]),
        Kind::ArcAngle(host) => refs("Arc angle of {}", &[*host]),
        Kind::AngleMeasurement(a, vertex, b) | Kind::AngleMarker(a, vertex, b, _) => {
            refs("Angle: {}, vertex {}, {}", &[*a, *vertex, *b])
        }
        Kind::SegmentRatio(a, b) => refs("Length ratio: {} / {}", &[*a, *b]),
        Kind::NumericAlias { source, .. } => refs("Value of {}", &[*source]),
        Kind::Transformed(transform) | Kind::TransformedCroppedImage(transform) => {
            let source = reference(transform.source());
            let numeric = |value| numeric_expression(value, &reference);
            match transform.transform() {
                Transform::Rotate { center, angle } => Definition::new(
                    "Rotate {} about {} by {}",
                    [source, reference(*center), numeric(angle)],
                ),
                Transform::Dilate { center, ratio } => Definition::new(
                    "Dilate {} about {} by factor {}",
                    [source, reference(*center), numeric(ratio)],
                ),
                Transform::Reflect { mirror } => {
                    Definition::new("Reflect {} across {}", [source, reference(*mirror)])
                }
                Transform::Translate(DisplacementExpr::Vector { start, end }) => Definition::new(
                    "Translate {} by vector {} → {}",
                    [source, reference(*start), reference(*end)],
                ),
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal,
                    vertical,
                }) => Definition::new(
                    "Translate {} by horizontal {}, vertical {}",
                    [source, numeric(horizontal), numeric(vertical)],
                ),
                Transform::Translate(DisplacementExpr::Polar { angle, distance }) => {
                    Definition::new(
                        "Translate {} by distance {} at angle {}",
                        [source, numeric(distance), numeric(angle)],
                    )
                }
            }
        }
        _ => {
            let parents = doc.dependencies_of(object.id).unwrap_or_default();
            if parents.is_empty() {
                Definition::new("", [])
            } else {
                Definition::new(
                    "Sources: {}",
                    [parents
                        .into_iter()
                        .map(reference)
                        .collect::<Vec<_>>()
                        .join(", ")],
                )
            }
        }
    }
}

fn numeric_expression(value: &NumericExpr, reference: &impl Fn(usize) -> String) -> String {
    match value {
        NumericExpr::Literal {
            magnitude,
            unit,
            scale,
        } => match unit {
            NumericUnit::Degrees => format!("{}°", magnitude * scale),
            NumericUnit::Length if *scale == 37.79527559055118 => format!("{magnitude} cm"),
            NumericUnit::Length => format!("{} px", magnitude * scale),
            _ => (magnitude * scale).to_string(),
        },
        NumericExpr::ObjectValue { id, .. } => reference(*id),
        NumericExpr::DirectedAngle {
            first,
            vertex,
            last,
        } => format!(
            "∠({}, {}, {})",
            reference(*first),
            reference(*vertex),
            reference(*last)
        ),
        NumericExpr::DistanceRatio {
            origin,
            base,
            target,
        } => format!(
            "|{}, {}| / |{}, {}|",
            reference(*origin),
            reference(*target),
            reference(*origin),
            reference(*base)
        ),
        NumericExpr::SegmentRatio {
            numerator,
            denominator,
        } => format!(
            "|{}| / |{}|",
            reference(*numerator),
            reference(*denominator)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::super::i18n::Language;
    use super::*;
    use sketchpad_engine::document::Point;
    use sketchpad_engine::table_records::{
        CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup,
        ParameterCalculationGroup, ParameterGroup,
    };

    fn text(doc: &Document, id: usize) -> String {
        definition(doc, &doc.objects[id - 1]).text()
    }

    #[test]
    fn all_calculation_families_show_the_expression_and_source_bindings() {
        Language::English.activate();
        let mut doc = Document::new();
        doc.add(Kind::Parameter(ParameterGroup::named("n", 2.).unwrap()));
        let literal = doc.add(Kind::Calculation(
            CalculationGroup::from_expression("2+3").unwrap(),
        ));
        let dependent = doc.add(Kind::ParameterCalculation(
            1,
            ParameterCalculationGroup::from_expression(1, "n", "sin(n)+1", (0, 0)).unwrap(),
        ));
        let measured = doc.add(Kind::MeasuredCalculation(
            1,
            MeasuredCalculationGroup::divide(1, 3, (0, 0)).unwrap(),
        ));
        let pair = doc.add(Kind::LengthPairCalculation(
            1,
            2,
            LengthPairCalculationGroup::with_operator(1, 2, 1, (0, 0)).unwrap(),
        ));
        assert_eq!(text(&doc, literal), "2 + 3");
        assert_eq!(text(&doc, dependent), "(sin(n) + 1)");
        assert_eq!(text(&doc, measured), "n ÷ 3");
        assert_eq!(text(&doc, pair), "n − #2");
        Language::Chinese.activate();
        assert_eq!(text(&doc, dependent), "(sin(n) + 1)");
        Language::English.activate();
    }

    #[test]
    fn imported_page_local_formula_sources_use_global_object_names() {
        let mut doc = Document::new();
        for name in ["wrong_a", "wrong_b", "a", "b"] {
            doc.add(Kind::Parameter(ParameterGroup::named(name, 2.).unwrap()));
        }
        let group = ParameterCalculationGroup::from_sources(
            &[1, 2],
            "v1-v2",
            (0, 0),
            &[NumericUnit::Scalar; 2],
        )
        .unwrap();
        let calculation = doc.add(Kind::ParameterCalculation(3, group));
        for id in 3..=calculation {
            doc.objects.edit(id - 1).page = 1;
        }
        assert_eq!(text(&doc, calculation), "(a − b)");
    }

    #[test]
    fn construction_roles_translate_without_translating_labels_or_evaluating_geometry() {
        Language::English.activate();
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 0., y: 0. }));
        doc.set_point_label(1, "Point{}", true).unwrap();
        let perpendicular = doc.add(Kind::Perpendicular(1, 900));
        let definition = definition(&doc, &doc.objects[perpendicular - 1]);
        assert_eq!(
            definition.text(),
            "Through Point{} [#1], perpendicular to #900"
        );
        Language::Chinese.activate();
        assert_eq!(definition.text(), "过 Point{} [#1]，垂直于 #900");
        Language::English.activate();
        let rotation = doc.add(Kind::RotatedPoint(1, 900, 60.));
        assert_eq!(
            text(&doc, rotation),
            "Rotate Point{} [#1] about #900 by 60°"
        );
        let translation = doc.add(Kind::FixedRectangularTranslatedPoint(
            1,
            37.79527559055118,
            2.,
            1.,
            5.,
        ));
        assert_eq!(
            text(&doc, translation),
            "Translate Point{} [#1] by horizontal 2 cm, vertical 5 px"
        );
    }

    #[test]
    fn shared_formula_chains_keep_direct_references_without_recursive_expansion() {
        let mut doc = Document::new();
        doc.add(Kind::Parameter(ParameterGroup::named("n", 1.).unwrap()));
        let mut previous = 1;
        let mut last = 1;
        for _ in 0..100 {
            let group = ParameterCalculationGroup::from_sources(
                &[previous as u32, last as u32],
                "v1+v2",
                (0, 0),
                &[NumericUnit::Scalar; 2],
            )
            .unwrap();
            let next = doc.add(Kind::ParameterCalculation(previous, group));
            previous = last;
            last = next;
        }
        assert_eq!(text(&doc, last), "(#99 + #100)");
        doc.objects.edit(98).deleted = true;
        assert_eq!(text(&doc, last), "(#99 + #100)");
    }
}
