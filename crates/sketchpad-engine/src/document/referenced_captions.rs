//! Shared live presentation for native referenced captions.
use super::referenced_text::{ReferenceRole, ReferencedRun, ReferencedText};
use super::{Document, Kind, Object};
use crate::text_object::{RichTextRun, TextRect};

const LEGACY_GBK_CAPTION_HEADER: [u8; 16] = [73, 0, 0, 0, 0, 0, 0, 0, 59, 0, 1, 0, 255, 0, 0, 110];

fn modern_header(visible: bool, presentation_suppressed: bool) -> [u8; 28] {
    let mut header = [
        73, 0, 0, 0, 0, 0, 0, 0, 59, 0, 33, 1, 0, 0, 1, 0, 255, 255, 81, 92, 193, 189, 240, 255, 0,
        0, 0, 0,
    ];
    header[2] = u8::from(!visible) | (u8::from(presentation_suppressed) << 1);
    header
}

fn supported_source(objects: &[Object], object: &Object, role: ReferenceRole) -> bool {
    if role == ReferenceRole::SignedValue {
        return object.kind.numeric_unit().is_some();
    }
    if role != ReferenceRole::Value {
        if role == ReferenceRole::Name
            && object
                .label
                .as_ref()
                .is_some_and(|label| label.name_utf8().is_ok_and(|name| !name.is_empty()))
        {
            return true;
        }
        return match &object.kind {
            Kind::Parameter(group) => role == ReferenceRole::Expression || !group.name().is_empty(),
            Kind::Calculation(_) => role == ReferenceRole::Expression,
            Kind::SegmentLength(segment) if role != ReferenceRole::Name => objects
                .get(segment.saturating_sub(1))
                .is_some_and(|source| {
                    !source.deleted
                        && source.page == object.page
                        && matches!(source.kind, Kind::Segment(..))
                }),
            Kind::ParameterCalculation(..)
            | Kind::SegmentRatio(..)
            | Kind::PointRatio(..)
            | Kind::SegmentLength(..) => true,
            Kind::AngleMarker(..)
            | Kind::SignedAngleValue(..)
            | Kind::AngleMeasurement(..)
            | Kind::ArcAngle(..)
            | Kind::CoordinateMeasurement {
                kind:
                    crate::coordinate_measure_records::CoordinateMeasureKind::Abscissa
                    | crate::coordinate_measure_records::CoordinateMeasureKind::Ordinate
                    | crate::coordinate_measure_records::CoordinateMeasureKind::GraphCoordinateX
                    | crate::coordinate_measure_records::CoordinateMeasureKind::GraphCoordinateY
                    | crate::coordinate_measure_records::CoordinateMeasureKind::CoordinateDistance,
                ..
            }
            | Kind::NumericAlias { .. }
            | Kind::PointDistance(..)
            | Kind::PolygonArea(..)
            | Kind::ValueEquation {
                kind: crate::value_equation_records::ValueEquationKind::PointValue,
                ..
            } => role == ReferenceRole::Definition,
            Kind::FunctionDefinition(..)
            | Kind::ReferencedFunction(..)
            | Kind::DerivativeFunction(..)
            | Kind::PlainText { .. }
            | Kind::ReferencedText { .. } => true,
            _ => false,
        };
    }
    object.kind.numeric_unit().is_some()
        || object.kind.is_action_button()
        || super::kind::is_point_kind(&object.kind)
        || object.kind.is_circular_segment()
        || object.label.is_some()
        || matches!(
            object.kind,
            Kind::PlainText { .. }
                | Kind::ReferencedText { .. }
                | Kind::FunctionDefinition(..)
                | Kind::ReferencedFunction(..)
                | Kind::DerivativeFunction(..)
                | Kind::CenterArc(..)
                | Kind::Circle(..)
                | Kind::Colorized(..)
                | Kind::FunctionPlot(..)
                | Kind::GraphAxis(..)
                | Kind::Image { .. }
                | Kind::Line(..)
                | Kind::Locus(..)
                | Kind::PointNumericLabel(..)
                | Kind::PointReferencedText { .. }
                | Kind::Polygon(..)
                | Kind::Ray(..)
                | Kind::SampledImage(..)
                | Kind::Segment(..)
                | Kind::CoordinateMeasurement {
                    kind: crate::coordinate_measure_records::CoordinateMeasureKind::Coordinates,
                    ..
                }
        )
}

pub(super) fn supported(
    parents: &[usize],
    text: &ReferencedText,
    objects: &[Object],
    page: usize,
    id: usize,
) -> bool {
    let get = |parent: usize| {
        parent
            .checked_sub(1)
            .and_then(|index| objects.get(index))
            .filter(|object| {
                parent < id
                    && object.page == page
                    && !object.deleted
                    && !matches!(object.kind, Kind::Unsupported(_))
            })
    };
    parents.iter().all(|parent| get(*parent).is_some())
        && text.referenced_roles().iter().all(|(slot, role)| {
            parents
                .get(slot - 1)
                .and_then(|parent| get(*parent))
                .is_some_and(|source| supported_source(objects, source, *role))
        })
}

pub(super) fn decode(
    objects: &[Object],
    page: usize,
    id: usize,
    header: &[u8],
    parents: &[usize],
    group: &[u8],
    font_count: usize,
) -> Option<Kind> {
    let text = decode_group(header, parents, group, font_count, 73)?;
    supported(parents, &text, objects, page, id).then(|| Kind::ReferencedText {
        parents: parents.to_vec(),
        text,
    })
}

pub(super) fn decode_group(
    header: &[u8],
    parents: &[usize],
    group: &[u8],
    font_count: usize,
    code: u16,
) -> Option<ReferencedText> {
    let modern_header =
        header.len() == 28 && header[..2] == code.to_le_bytes() && header[6..10] == [0, 0, 59, 0];
    let legacy_header = code == 73 && header == LEGACY_GBK_CAPTION_HEADER;
    if !modern_header && !legacy_header {
        return None;
    }
    let mut records = Vec::new();
    let mut at = 0usize;
    while at < group.len() {
        let prefix = group.get(at..at.checked_add(8)?)?;
        let len = u32::from_le_bytes(prefix[..4].try_into().ok()?) as usize;
        let kind = u32::from_le_bytes(prefix[4..].try_into().ok()?);
        let end = at.checked_add(8)?.checked_add(len)?;
        records.push((kind, group.get(at + 8..end)?));
        at = end.checked_add(len % 2)?;
    }
    if at != group.len() {
        return None;
    }
    let [
        (0x7d0, _),
        (0x7d2, refs),
        (0x8fc, rich),
        (0x7d8, font),
        tail @ ..,
    ] = records.as_slice()
    else {
        return None;
    };
    if refs.len() != 4 + 4 * parents.len()
        || refs[..4] != (parents.len() as u32).to_le_bytes()
        || font.len() != 2
    {
        return None;
    }
    if usize::from(u16::from_le_bytes((*font).try_into().ok()?)) >= font_count {
        return None;
    }
    if !matches!(tail,[(0x7d7,p)] if p.is_empty())
        && !matches!(tail,[(0x232e,p),(0x7d7,q)] if p.is_empty()&&q.is_empty())
    {
        return None;
    }
    ReferencedText::from_rich(rich, parents.len()).ok()
}

impl Document {
    pub fn text_math_nodes(
        &self,
        id: usize,
    ) -> Option<Vec<super::referenced_text::ReferencedNode>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        match &object.kind {
            Kind::PlainText { .. } => self.text_rich_template(id)?.math_nodes(),
            Kind::ReferencedText { parents, text }
                if text
                    .nodes()
                    .iter()
                    .any(super::referenced_text::ReferencedNode::has_math_layout) =>
            {
                Some(text.resolved_nodes(|slot, role| {
                    self.referenced_source_text(*parents.get(slot - 1)?, role, None)
                }))
            }
            Kind::PointReferencedText {
                point,
                source,
                text,
            } if text
                .nodes()
                .iter()
                .any(super::referenced_text::ReferencedNode::has_math_layout) =>
            {
                let parents = [*point, *source];
                Some(text.resolved_nodes(|slot, role| {
                    self.referenced_source_text(*parents.get(slot - 1)?, role, None)
                }))
            }
            _ => None,
        }
    }

    pub fn referenced_text_runs(&self, id: usize) -> Option<Vec<Vec<ReferencedRun>>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::ReferencedText { parents, text } = &object.kind else {
            return None;
        };
        Some(self.evaluate_referenced_text(parents, text))
    }

    pub(super) fn evaluate_referenced_text(
        &self,
        parents: &[usize],
        text: &ReferencedText,
    ) -> Vec<Vec<ReferencedRun>> {
        text.evaluate_roles(|slot, role, cached| {
            self.referenced_source_text(*parents.get(slot.checked_sub(1)?)?, role, cached)
        })
    }

    fn referenced_source_text(
        &self,
        source: usize,
        role: ReferenceRole,
        cached: Option<&str>,
    ) -> Option<String> {
        if let Some(original) = self.frozen_evaluation_document(source) {
            return original.referenced_source_text(source, role, cached);
        }
        let object = self.objects.get(source.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        if role == ReferenceRole::SignedValue {
            let value = self.numeric_value(source)?;
            if value.is_nan() {
                return Some("undefined".into());
            }
            let text = self.numeric_value_text(source)?;
            if value.is_sign_negative() {
                return Some(format!(" − {}", text.trim_start_matches(['-', '−', '–'])));
            }
            return Some(format!(" + {text}"));
        }
        if matches!(
            object.kind,
            Kind::PlainText { .. } | Kind::ReferencedText { .. }
        ) {
            return self.text_content(source).map(|content| content.0);
        }
        if object.kind.is_function() {
            return match role {
                ReferenceRole::Name => self.function_symbol(source),
                ReferenceRole::Expression => self
                    .function_display_text(source)?
                    .split_once('=')
                    .map(|(_, rhs)| rhs.trim().to_owned()),
                _ => self.function_display_text(source),
            };
        }
        if role != ReferenceRole::Value {
            return match &object.kind {
                Kind::Parameter(_) if role == ReferenceRole::Expression => {
                    self.numeric_value_text(source)
                }
                Kind::Calculation(group) if role == ReferenceRole::Expression => {
                    Some(group.expression())
                }
                Kind::Parameter(group) => Some(group.name().to_owned()),
                _ if role == ReferenceRole::Name => self.measurement_reference_name(source),
                _ if role == ReferenceRole::Definition => self.referenced_definition_text(source),
                Kind::ParameterCalculation(..) => self.parameter_calculation_expression(source),
                Kind::PointRatio(a, b, c) => Some(format!(
                    "{}/{}",
                    self.reference_segment_name(*a, *c)?,
                    self.reference_segment_name(*a, *b)?
                )),
                Kind::SegmentRatio(first, second) => Some(format!(
                    "{}/{}",
                    self.reference_geometry_name(*first)?,
                    self.reference_geometry_name(*second)?
                )),
                Kind::SegmentLength(segment) => {
                    let Kind::Segment(a, b) = self.objects.get(segment.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    Some(
                        self.reference_segment_name(a, b)?
                            .chars()
                            .flat_map(|ch| [ch, '\u{305}'])
                            .collect(),
                    )
                }
                _ => None,
            };
        }
        if object.kind.numeric_unit().is_some() {
            return self.numeric_value_text(source);
        }
        if matches!(object.kind, Kind::CoordinateMeasurement { .. }) {
            return self.referenced_coordinate_value(source, cached);
        }
        if matches!(
            object.kind,
            Kind::PointNumericLabel(..) | Kind::PointReferencedText { .. }
        ) {
            return self.point_numeric_label(source).map(|(_, text)| text);
        }
        if matches!(object.kind, Kind::Image { .. } | Kind::SampledImage(..)) {
            // Pictures expose only their generated display name. Native stores
            // that localized name in the reference cache (for example,
            // "Picture [1]" or "图片 [1]").
            return Some(cached.map(str::to_owned).unwrap_or_else(|| {
                let ordinal = self
                    .objects
                    .iter()
                    .take(source)
                    .filter(|candidate| {
                        !candidate.deleted
                            && candidate.page == object.page
                            && matches!(candidate.kind, Kind::Image { .. } | Kind::SampledImage(..))
                    })
                    .count();
                format!("Picture [{ordinal}]")
            }));
        }
        if let Some(label) = &object.label {
            return label.name_utf8().ok().map(str::to_owned);
        }
        if super::kind::is_point_kind(&object.kind) {
            return self.coordinate_point_name(source);
        }
        if let Some(name) = self.reference_geometry_name(source) {
            return Some(name);
        }
        match &object.kind {
            Kind::HideShowButton { label, .. }
            | Kind::AnimationButton { label, .. }
            | Kind::SoundButton { label, .. }
            | Kind::MovementButton { label, .. }
            | Kind::ScrollButton { label, .. }
            | Kind::LinkUrlButton { label, .. }
            | Kind::LinkPageButton { label, .. }
            | Kind::PresentationButton { label, .. } => Some(label.clone()),
            _ => None,
        }
    }

    fn reference_segment_name(&self, a: usize, b: usize) -> Option<String> {
        Some(format!(
            "{}{}",
            self.coordinate_point_name(a)?,
            self.coordinate_point_name(b)?
        ))
    }

    fn referenced_definition_text(&self, source: usize) -> Option<String> {
        let object = self.objects.get(source.checked_sub(1)?)?;
        if let Some(name) = object
            .label
            .as_ref()
            .and_then(|label| label.name_utf8().ok())
            .filter(|name| !name.is_empty())
        {
            return Some(name.to_owned());
        }
        match object.kind {
            Kind::AngleMarker(first, vertex, last, _)
            | Kind::SignedAngleValue(first, vertex, last)
            | Kind::AngleMeasurement(first, vertex, last) => Some(format!(
                "∠{}{}{}",
                self.coordinate_point_name(first)?,
                self.coordinate_point_name(vertex)?,
                self.coordinate_point_name(last)?
            )),
            Kind::ArcAngle(arc) => self.reference_geometry_name(arc),
            Kind::CoordinateMeasurement {
                kind: crate::coordinate_measure_records::CoordinateMeasureKind::Abscissa,
                first,
                ..
            } => Some(format!("x{}", self.coordinate_point_name(first)?)),
            Kind::CoordinateMeasurement {
                kind: crate::coordinate_measure_records::CoordinateMeasureKind::Ordinate,
                first,
                ..
            } => Some(format!("y{}", self.coordinate_point_name(first)?)),
            Kind::CoordinateMeasurement {
                kind: crate::coordinate_measure_records::CoordinateMeasureKind::GraphCoordinateX,
                first,
                ..
            } => Some(format!("r{}", self.coordinate_point_name(first)?)),
            Kind::CoordinateMeasurement {
                kind: crate::coordinate_measure_records::CoordinateMeasureKind::GraphCoordinateY,
                first,
                ..
            } => Some(format!("θ{}", self.coordinate_point_name(first)?)),
            Kind::CoordinateMeasurement {
                kind: crate::coordinate_measure_records::CoordinateMeasureKind::CoordinateDistance,
                first,
                second: Some(second),
                ..
            }
            | Kind::PointDistance(first, second) => self.reference_segment_name(first, second),
            Kind::NumericAlias { source, .. } => self.referenced_definition_text(source),
            Kind::PolygonArea(polygon) => Some(format!(
                "Area of {}",
                self.reference_geometry_name(polygon)?
            )),
            Kind::ValueEquation {
                kind: crate::value_equation_records::ValueEquationKind::PointValue,
                first,
                second,
            } => Some(format!(
                "Value of {} on {}",
                self.coordinate_point_name(first)?,
                self.reference_geometry_name(second)?
            )),
            _ => self.calculation_source_name(source),
        }
    }

    fn reference_geometry_name(&self, source: usize) -> Option<String> {
        let object = self.objects.get(source.checked_sub(1)?)?;
        if let Some(name) = object
            .label
            .as_ref()
            .and_then(|label| label.name_utf8().ok())
            .filter(|name| !name.is_empty())
        {
            return Some(name.to_owned());
        }
        if object.kind.is_point() {
            return self.coordinate_point_name(source);
        }
        match &object.kind {
            Kind::Segment(first, second) | Kind::Ray(first, second) | Kind::Line(first, second) => {
                self.reference_segment_name(*first, *second)
            }
            Kind::Polygon(vertices) => {
                let name = vertices
                    .iter()
                    .map(|point| self.coordinate_point_name(*point))
                    .collect::<Option<String>>()?;
                Some(if vertices.len() == 3 {
                    format!("△{name}")
                } else {
                    name
                })
            }
            Kind::Circle(center, edge) => Some(format!(
                "⊙{}{}",
                self.coordinate_point_name(*center)?,
                self.coordinate_point_name(*edge)?
            )),
            Kind::CircleBySegment(center, segment) => Some(format!(
                "⊙{}{}",
                self.coordinate_point_name(*center)?,
                self.reference_geometry_name(*segment)?
            )),
            Kind::CircularSegmentBoundary(_) => {
                let ordinal = self
                    .objects
                    .iter()
                    .take(source)
                    .filter(|candidate| {
                        !candidate.deleted
                            && candidate.page == object.page
                            && matches!(
                                candidate.kind,
                                Kind::CircleInterior(_)
                                    | Kind::SectorBoundary(_)
                                    | Kind::CircularSegmentBoundary(_)
                            )
                    })
                    .count();
                Some(format!("A{}", subscript_number(ordinal)))
            }
            Kind::Transformed(definition)
                if definition.element() == super::transform_model::ElementType::CircularSegment =>
            {
                Some(format!(
                    "{}'",
                    self.reference_geometry_name(definition.source())?
                ))
            }
            Kind::Colorized(definition) => self.reference_geometry_name(definition.source()),
            Kind::FunctionPlot(function, ..) => {
                Some(format!("y = {}(x)", self.function_symbol(*function)?))
            }
            Kind::GraphAxis(_, _, form) => match form {
                0 => Some("y".into()),
                1 | 0x1_0001 => Some("x".into()),
                _ => None,
            },
            Kind::Locus(..) => {
                let page = object.page;
                let ordinal = self
                    .objects
                    .iter()
                    .take(source)
                    .filter(|object| {
                        !object.deleted
                            && object.page == page
                            && matches!(object.kind, Kind::Locus(..))
                    })
                    .count();
                Some(format!("L{}", subscript_number(ordinal)))
            }
            Kind::ThreePointArc(first, middle, last) => Some(format!(
                "{}{}{}",
                self.coordinate_point_name(*first)?,
                self.coordinate_point_name(*middle)?,
                self.coordinate_point_name(*last)?
            )),
            Kind::ArcOnCircle(_, first, last) | Kind::CenterArc(_, first, last) => {
                self.reference_segment_name(*first, *last)
            }
            _ => None,
        }
    }

    fn referenced_coordinate_value(&self, source: usize, cached: Option<&str>) -> Option<String> {
        let object = self.objects.get(source.checked_sub(1)?)?;
        let crate::coordinate_measure_records::CoordinateMeasureValue::Pair(point) =
            self.coordinate_measure_value(source)?
        else {
            return None;
        };
        // Coordinate-pair readouts store their formatting in rich text rather
        // than the scalar measurement precision field. Recover the decimal
        // count from the first cached component when Native supplied one.
        let cached_precision = cached.and_then(|text| {
            let first = text
                .trim()
                .trim_start_matches('(')
                .split_once(',')?
                .0
                .trim();
            Some(first.split_once('.').map_or(0, |(_, decimals)| {
                decimals.chars().take_while(char::is_ascii_digit).count()
            }))
        });
        let precision = cached_precision
            .unwrap_or_else(|| usize::from(object.measurement_precision.unwrap_or(2)));
        let x = if point.x == 0. { 0. } else { point.x };
        let y = if point.y == 0. { 0. } else { point.y };
        Some(format!(
            "({}, {})",
            crate::table_records::format_numeric_value(x, precision),
            crate::table_records::format_numeric_value(y, precision)
        ))
    }
    pub(super) fn referenced_caption_content(&self, id: usize) -> Option<(String, TextRect)> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::ReferencedText { text, .. } = &object.kind else {
            return None;
        };
        let rows = self.referenced_text_runs(id)?;
        let content = rows
            .iter()
            .map(|row| row.iter().map(|run| run.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        let (x, y) = text.stored_origin();
        let rect = TextRect {
            left: x,
            top: y,
            right: x.saturating_add(1),
            bottom: y.saturating_add(1),
        };
        Some((content, rect))
    }
    pub(super) fn referenced_caption_rich_runs(&self, id: usize) -> Option<Vec<Vec<RichTextRun>>> {
        Some(
            self.referenced_text_runs(id)?
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|run| RichTextRun {
                            text: run.text,
                            style: run.style.font,
                        })
                        .collect()
                })
                .collect(),
        )
    }
    pub fn add_referenced_text(
        &mut self,
        parents: Vec<usize>,
        markup: &str,
        origin: (i16, i16),
    ) -> Result<usize, String> {
        let text = ReferencedText::from_markup(markup, origin, parents.len())?;
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || !supported(
                &parents,
                &text,
                &self.objects,
                self.active_page,
                self.objects.len() + 1,
            )
        {
            return Err("Select supported live caption sources on this page".into());
        }
        Ok(self.add(Kind::ReferencedText { parents, text }))
    }
}

fn subscript_number(value: usize) -> String {
    value
        .to_string()
        .chars()
        .map(|digit| match digit {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            other => other,
        })
        .collect()
}

pub(super) fn encode_group(object: &Object, parents: &[u32], text: &ReferencedText) -> Vec<u8> {
    use super::records::put_record;
    // Native class zero/subtype59 referenced-caption header.
    let header = modern_header(object.visible, object.presentation_suppressed);
    let mut group = Vec::new();
    put_record(&mut group, 0x7d0, &header);
    let mut refs = (parents.len() as u32).to_le_bytes().to_vec();
    for parent in parents {
        refs.extend_from_slice(&parent.to_le_bytes());
    }
    put_record(&mut group, 0x7d2, &refs);
    put_record(&mut group, 0x8fc, text.rich_bytes());
    put_record(&mut group, 0x7d8, &0u16.to_le_bytes());
    put_record(&mut group, 0x7d7, &[]);
    group
}
