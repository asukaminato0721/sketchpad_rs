//! Live color properties over ordinary source geometry.
use super::kind::{is_arc_kind, is_point_kind, polygon_vertex_count, straight_kind_mode};
use super::records::{f64_at, u32_at};
use super::transform_model::ElementType;
use super::{Document, Kind, LineMode, Object};
use crate::style_records::{RgbColor, VisualStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorRangeBehavior {
    Clamp,
    Cycle,
    Reflect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorBinding {
    Spectrum {
        value: usize,
    },
    Grayscale {
        value: usize,
    },
    Rgb {
        red: usize,
        green: usize,
        blue: usize,
    },
    Hsv {
        hue: usize,
        saturation: usize,
        value: usize,
    },
}
impl ColorBinding {
    pub fn native_operands(&self) -> Vec<usize> {
        match *self {
            Self::Spectrum { value } | Self::Grayscale { value } => vec![value],
            Self::Rgb { red, green, blue } => vec![blue, green, red],
            Self::Hsv {
                hue,
                saturation,
                value,
            } => vec![value, saturation, hue],
        }
    }
    fn native_base(&self) -> u32 {
        match self {
            Self::Spectrum { .. } => 0x10,
            Self::Grayscale { .. } => 0x20,
            Self::Rgb { .. } => 0x30,
            Self::Hsv { .. } => 0x40,
        }
    }
    fn remap(&self, ids: &[usize]) -> Self {
        match *self {
            Self::Spectrum { value } => Self::Spectrum { value: ids[value] },
            Self::Grayscale { value } => Self::Grayscale { value: ids[value] },
            Self::Rgb { red, green, blue } => Self::Rgb {
                red: ids[red],
                green: ids[green],
                blue: ids[blue],
            },
            Self::Hsv {
                hue,
                saturation,
                value,
            } => Self::Hsv {
                hue: ids[hue],
                saturation: ids[saturation],
                value: ids[value],
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorizedDefinition {
    source: usize,
    element: ElementType,
    binding: ColorBinding,
    range_start: f64,
    range_end: f64,
    behavior: ColorRangeBehavior,
}
impl ColorizedDefinition {
    pub fn new(
        source: usize,
        element: ElementType,
        binding: ColorBinding,
        range_start: f64,
        range_end: f64,
        behavior: ColorRangeBehavior,
    ) -> Result<Self, String> {
        if source == 0
            || binding.native_operands().contains(&0)
            || !range_start.is_finite()
            || !range_end.is_finite()
            || !(range_end - range_start).is_finite()
        {
            return Err(
                "Select live geometry and numeric color operands with a finite range".into(),
            );
        }
        Ok(Self {
            source,
            element,
            binding,
            range_start,
            range_end,
            behavior,
        })
    }
    pub fn source(&self) -> usize {
        self.source
    }
    pub fn element(&self) -> ElementType {
        self.element
    }
    pub(super) fn specialize_element(mut self, element: ElementType) -> Result<Self, String> {
        if !matches!(
            element,
            ElementType::Segment
                | ElementType::Ray
                | ElementType::Line
                | ElementType::Circle
                | ElementType::Arc
                | ElementType::Polygon
        ) {
            return Err("Tool path has an unsupported colorized element type".into());
        }
        self.element = element;
        Ok(self)
    }
    pub fn binding(&self) -> &ColorBinding {
        &self.binding
    }
    pub fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }
    pub fn behavior(&self) -> ColorRangeBehavior {
        self.behavior
    }
    pub fn native_operation(&self) -> u32 {
        self.binding.native_base()
            + match self.behavior {
                ColorRangeBehavior::Clamp => 0,
                ColorRangeBehavior::Cycle => 1,
                ColorRangeBehavior::Reflect => 2,
            }
    }
    pub fn native_references(&self) -> Vec<usize> {
        let mut r = vec![self.source];
        r.extend(self.binding.native_operands());
        r
    }
    pub fn native_payload(&self) -> [u8; 24] {
        let mut p = [0; 24];
        p[..4].copy_from_slice(&75u32.to_le_bytes());
        p[4..8].copy_from_slice(&self.native_operation().to_le_bytes());
        p[8..16].copy_from_slice(&self.range_start.to_le_bytes());
        p[16..].copy_from_slice(&self.range_end.to_le_bytes());
        p
    }
    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        Self {
            source: ids[self.source],
            element: self.element,
            binding: self.binding.remap(ids),
            range_start: self.range_start,
            range_end: self.range_end,
            behavior: self.behavior,
        }
    }
    pub(super) fn supported(
        &self,
        objects: &(impl super::object_store::ObjectRead + ?Sized),
        page: usize,
        id: usize,
    ) -> bool {
        self.supported_internal(objects, page, id, false)
    }

    fn supported_internal(
        &self,
        objects: &(impl super::object_store::ObjectRead + ?Sized),
        page: usize,
        id: usize,
        tool_prototype: bool,
    ) -> bool {
        let get = |parent: usize| {
            objects
                .get(parent.checked_sub(1)?)
                .filter(|o| parent < id && o.id == parent && o.page == page && !o.deleted)
        };
        get(self.source).is_some_and(|o| {
            geometry_element(&o.kind) == Some(self.element)
                || tool_prototype
                    && self.element == ElementType::Point
                    && matches!(
                        o.kind,
                        Kind::ToolPointOnPath(_)
                            | Kind::ToolInput(super::EmbeddedToolInputKind::Path)
                    )
        })
            // Native normalizes any live numeric quantity against the stored
            // range, including compound units such as inverse area.
            && self
                .binding
                .native_operands()
                .iter()
                .all(|parent| {
                    get(*parent).is_some_and(|o| {
                        o.kind.numeric_unit().is_some()
                            || tool_prototype
                                && matches!(
                                    o.kind,
                                    Kind::ToolInput(super::EmbeddedToolInputKind::Numeric)
                                )
                    })
                })
    }
}

pub(super) fn geometry_element(kind: &Kind) -> Option<ElementType> {
    if is_point_kind(kind) && !matches!(kind, Kind::ToolInput(_) | Kind::ToolPointOnPath(_)) {
        return Some(ElementType::Point);
    }
    if let Some(mode) = straight_kind_mode(kind) {
        return Some(match mode {
            LineMode::Segment => ElementType::Segment,
            LineMode::Ray => ElementType::Ray,
            LineMode::Line => ElementType::Line,
        });
    }
    if kind.is_circle() {
        return Some(ElementType::Circle);
    }
    if kind.is_circle_interior() {
        return Some(ElementType::Disk);
    }
    if kind.is_polygon() {
        return Some(ElementType::Polygon);
    }
    if kind.is_sector() {
        return Some(ElementType::Sector);
    }
    if is_arc_kind(kind) {
        return Some(ElementType::Arc);
    }
    None
}

pub(super) fn decode_colorized(
    objects: &[Object],
    page: usize,
    id: usize,
    header: &[u8],
    refs: &[usize],
    payload: Option<&[u8]>,
    tool_prototype: bool,
) -> Option<ColorizedDefinition> {
    if header.len() != 28 || header[..2] != [75, 0] {
        return None;
    }
    let p = payload.filter(|p| p.len() == 24 && p[..4] == 75u32.to_le_bytes())?;
    let operation = u32_at(p, 4);
    let behavior = match operation & 15 {
        0 => ColorRangeBehavior::Clamp,
        1 => ColorRangeBehavior::Cycle,
        2 => ColorRangeBehavior::Reflect,
        _ => return None,
    };
    let (source, binding) = match (operation & !15, refs) {
        (0x10, [source, value]) => (*source, ColorBinding::Spectrum { value: *value }),
        (0x20, [source, value]) => (*source, ColorBinding::Grayscale { value: *value }),
        (0x30, [source, blue, green, red]) => (
            *source,
            ColorBinding::Rgb {
                red: *red,
                green: *green,
                blue: *blue,
            },
        ),
        (0x40, [source, value, saturation, hue]) => (
            *source,
            ColorBinding::Hsv {
                hue: *hue,
                saturation: *saturation,
                value: *value,
            },
        ),
        _ => return None,
    };
    let class = u16::from_le_bytes(header[6..8].try_into().ok()?);
    let subtype = u16::from_le_bytes(header[8..10].try_into().ok()?);
    let source_kind = &objects.get(source.checked_sub(1)?)?.kind;
    let polymorphic_tool_path = tool_prototype
        && matches!(
            source_kind,
            Kind::ToolInput(super::EmbeddedToolInputKind::Path)
        )
        && (class, subtype) == (1, 6);
    // The stored generic Path header has no concrete element category. Point is
    // an isolated-prototype sentinel; invocation replaces it before validation.
    let element = geometry_element(source_kind).or_else(|| {
        (tool_prototype
            && ((matches!(source_kind, Kind::ToolPointOnPath(_)) && (class, subtype) == (1, 4))
                || polymorphic_tool_path))
            .then_some(ElementType::Point)
    })?;
    let matches_header = match element {
        ElementType::Point => (class, subtype) == (1, 4) || polymorphic_tool_path,
        ElementType::Segment => (class, subtype) == (2, 14),
        ElementType::Ray => (class, subtype) == (2, 13),
        ElementType::Line => (class, subtype) == (2, 12),
        ElementType::Circle => (class, subtype) == (3, 28),
        ElementType::Arc => (class, subtype) == (12, 67),
        ElementType::Sector => (class, subtype) == (13, 68),
        ElementType::Disk => (class, subtype) == (5, 16),
        ElementType::Polygon => {
            class == 4
                && (super::records::polygon_subtype(polygon_vertex_count(objects, source)?)?
                    == subtype
                    || (subtype == 15
                        && polygon_vertex_count(objects, source).is_some()
                        && matches!(operation, 0x12 | 0x22 | 0x32 | 0x42)))
        }
        ElementType::CircularSegment | ElementType::Image => false,
    };
    if !matches_header {
        return None;
    }
    let definition = ColorizedDefinition::new(
        source,
        element,
        binding,
        f64_at(p, 8),
        f64_at(p, 16),
        behavior,
    )
    .ok()?;
    definition
        .supported_internal(objects, page, id, tool_prototype)
        .then_some(definition)
}

fn channel(value: f64) -> u8 {
    (value.clamp(0., 1.) * 255.).floor() as u8
}
fn spectrum(value: f64) -> RgbColor {
    // Native optical Spectrum samples. Interpolation between these measured
    // knots is a deliberate display approximation, not its internal formula.
    const COLORS: [[u8; 3]; 9] = [
        [233, 0, 255],
        [79, 0, 255],
        [0, 111, 239],
        [0, 241, 104],
        [69, 249, 0],
        [168, 210, 0],
        [229, 135, 0],
        [254, 23, 0],
        [255, 0, 0],
    ];
    let v = value.clamp(0., 1.) * 8.;
    let i = (v.floor() as usize).min(7);
    let t = v - i as f64;
    let c = |j: usize| {
        (COLORS[i][j] as f64 + (COLORS[i + 1][j] as f64 - COLORS[i][j] as f64) * t).floor() as u8
    };
    RgbColor {
        red: c(0),
        green: c(1),
        blue: c(2),
    }
}
fn hsv(h: f64, s: f64, v: f64) -> RgbColor {
    let h = h.rem_euclid(1.) * 6.;
    let f = h - h.floor();
    let p = v * (1. - s);
    let q = v * (1. - s * f);
    let t = v * (1. - s * (1. - f));
    let (r, g, b) = match h.floor() as usize % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    RgbColor {
        red: channel(r),
        green: channel(g),
        blue: channel(b),
    }
}

impl Document {
    pub fn colorized_color(&self, id: usize) -> Option<RgbColor> {
        if let Some(original) = self.frozen_evaluation_document(id) {
            return original.colorized_color(id);
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::Colorized(d) = &object.kind else {
            return None;
        };
        if object.deleted || !d.supported(&self.objects, object.page, id) {
            return None;
        }
        let normalized = |operand: usize| {
            let value = self.numeric_value(operand)?;
            let q = if d.range_start == d.range_end {
                1.
            } else {
                (value - d.range_start) / (d.range_end - d.range_start)
            };
            if !q.is_finite() {
                return None;
            }
            Some(match d.behavior {
                ColorRangeBehavior::Clamp => q.clamp(0., 1.),
                ColorRangeBehavior::Cycle => {
                    if (0. ..=1.).contains(&q) {
                        q
                    } else if q < 0. && q.fract() == 0. {
                        1.
                    } else {
                        q.rem_euclid(1.)
                    }
                }
                ColorRangeBehavior::Reflect => {
                    let q = q.rem_euclid(2.);
                    if q <= 1. { q } else { 2. - q }
                }
            })
        };
        Some(match d.binding {
            ColorBinding::Spectrum { value } => spectrum(normalized(value)?),
            ColorBinding::Grayscale { value } => {
                let c = channel(normalized(value)?);
                RgbColor {
                    red: c,
                    green: c,
                    blue: c,
                }
            }
            ColorBinding::Rgb { red, green, blue } => RgbColor {
                red: channel(normalized(red)?),
                green: channel(normalized(green)?),
                blue: channel(normalized(blue)?),
            },
            ColorBinding::Hsv {
                hue,
                saturation,
                value,
            } => hsv(
                normalized(hue)?,
                normalized(saturation)?,
                normalized(value)?,
            ),
        })
    }
    pub fn effective_visual_style(&self, id: usize) -> Option<VisualStyle> {
        if let Some(original) = self.frozen_evaluation_document(id) {
            return original.effective_visual_style(id);
        }
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        // A transformed image without a local style retains its source style.
        let style = object.visual_style.or_else(|| {
            object
                .kind
                .transform_definition()
                .and_then(|definition| self.effective_visual_style(definition.source()))
        })?;
        if !matches!(object.kind, Kind::Colorized(_)) {
            return Some(style);
        }
        let color = self.colorized_color(id)?;
        Some(match style {
            VisualStyle::Point { shape, .. } => VisualStyle::Point { shape, color },
            VisualStyle::Stroke { line, .. } => VisualStyle::Stroke { line, color },
            VisualStyle::Polygon { opacity, .. } => VisualStyle::Polygon { opacity, color },
        })
    }
    pub fn add_colorized(
        &mut self,
        source: usize,
        binding: ColorBinding,
        range_start: f64,
        range_end: f64,
        behavior: ColorRangeBehavior,
    ) -> Result<usize, String> {
        let element = self
            .objects
            .get(source.checked_sub(1).ok_or("Invalid source geometry")?)
            .and_then(|o| geometry_element(&o.kind))
            .ok_or("Select supported source geometry")?;
        let definition =
            ColorizedDefinition::new(source, element, binding, range_start, range_end, behavior)?;
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || !definition.supported(&self.objects, self.active_page, self.objects.len() + 1)
        {
            return Err("Select supported source geometry and numeric color operands".into());
        }
        let style = self.objects[source - 1].visual_style;
        self.checkpoint();
        let id = self.append_object(Kind::Colorized(definition));
        self.objects.edit(id - 1).visual_style = style;
        Ok(id)
    }
}
