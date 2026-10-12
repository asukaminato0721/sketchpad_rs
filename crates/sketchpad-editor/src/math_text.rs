//! Shared two-dimensional caption layout for canvas, SVG, and picking.
use crate::document::{
    Document,
    referenced_text::{ReferencedNode, ReferencedStyle},
};
use crate::text_style_records::FontState;
use egui::{Color32, Pos2};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Run {
    pub text: String,
    pub font: FontState,
    pub size: f32,
    pub color: Color32,
    pub position: Pos2,
    pub baseline: f32,
    // Canvas-only fields are intentionally unused when render.rs is compiled
    // inside sketchpad-engine's SVG regression tests.
    #[allow(dead_code)]
    pub galley: Arc<egui::Galley>,
    #[allow(dead_code)]
    pub reference: Option<usize>,
}
#[derive(Clone)]
pub(crate) struct Rule {
    pub points: Vec<Pos2>,
    pub color: Color32,
}
#[derive(Default)]
pub(crate) struct Layout {
    pub width: f32,
    pub above: f32,
    pub below: f32,
    pub runs: Vec<Run>,
    pub rules: Vec<Rule>,
}
impl Layout {
    pub fn height(&self) -> f32 {
        self.above + self.below
    }
    #[allow(dead_code)]
    pub fn paint(&self, painter: &egui::Painter, at: Pos2) {
        for run in &self.runs {
            let position = at + run.position.to_vec2();
            painter.galley(position, run.galley.clone(), run.color);
            if run.font.bold {
                painter.galley(
                    position + egui::vec2(0.7, 0.),
                    run.galley.clone(),
                    run.color,
                );
            }
        }
        for rule in &self.rules {
            painter.add(egui::Shape::line(
                rule.points.iter().map(|p| at + p.to_vec2()).collect(),
                egui::Stroke::new(1., rule.color),
            ));
        }
    }

    pub fn write_svg(&self, svg: &mut String, at: Pos2) {
        use std::fmt::Write;
        for run in &self.runs {
            let color = format!(
                "#{:02x}{:02x}{:02x}",
                run.color.r(),
                run.color.g(),
                run.color.b()
            );
            writeln!(svg,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\" fill=\"{}\" xml:space=\"preserve\">{}</text>",
                at.x + run.position.x, at.y + run.position.y + run.baseline,
                super::xml_text(&run.font.family).replace('"', "&quot;"), run.size,
                if run.font.bold { "bold" } else { "normal" },
                if run.font.italic { "italic" } else { "normal" },
                if run.font.underline { "underline" } else { "none" }, color, super::xml_text(&run.text)).unwrap();
        }
        for rule in &self.rules {
            let points = rule
                .points
                .iter()
                .map(|p| format!("{},{}", at.x + p.x, at.y + p.y))
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(svg, "<polyline points=\"{points}\" fill=\"none\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"1\"/>", rule.color.r(), rule.color.g(), rule.color.b()).unwrap();
        }
    }

    fn append(&mut self, mut child: Self, x: f32, y: f32) {
        for run in &mut child.runs {
            run.position += egui::vec2(x, y);
        }
        for rule in &mut child.rules {
            for point in &mut rule.points {
                *point += egui::vec2(x, y);
            }
        }
        self.runs.extend(child.runs);
        self.rules.extend(child.rules);
    }
}
#[derive(Clone)]
struct Style {
    font: FontState,
    color: Color32,
    script: u8,
}
impl Style {
    fn size(&self) -> f32 {
        let size = f32::from(self.font.size_code);
        if self.script == 0 {
            size
        } else {
            (size * 0.625 * 0.8_f32.powi(i32::from(self.script) - 1))
                .round()
                .max(1.)
        }
    }
    fn child(&self, next: &ReferencedStyle, families: &[String]) -> Self {
        let mut result = self.clone();
        if let Some(v) = next.font.size_code {
            result.font.size_code = v;
        }
        if let Some(v) = next.font.bold {
            result.font.bold = v;
        }
        if let Some(v) = next.font.italic {
            result.font.italic = v;
        }
        if let Some(v) = next.font.underline {
            result.font.underline = v;
        }
        if let Some(v) = next
            .font
            .family_index
            .and_then(|i| families.get(usize::from(i)))
        {
            result.font.family = v.clone();
        }
        if let Some(v) = next.font.script_size {
            result.script += v;
        }
        let rgb = next.font.rgb_color().or_else(|| {
            Some([
                u8::try_from(next.red? - 1).ok()?,
                u8::try_from(next.green? - 1).ok()?,
                u8::try_from(next.blue? - 1).ok()?,
            ])
        });
        if let Some([r, g, b]) = rgb {
            result.color = Color32::from_rgb(r, g, b);
        }
        result
    }
}

pub(crate) fn text_format(
    font: &FontState,
    size: f32,
    color: Color32,
    family: egui::FontFamily,
) -> egui::TextFormat {
    egui::TextFormat {
        font_id: egui::FontId::new(size, family),
        color,
        italics: font.italic,
        underline: if font.underline {
            egui::Stroke::new(1., color)
        } else {
            egui::Stroke::NONE
        },
        ..Default::default()
    }
}

pub(crate) fn layout(
    doc: &Document,
    id: usize,
    mut measure: impl FnMut(&str, &FontState, f32, Color32) -> Arc<egui::Galley>,
) -> Option<Layout> {
    let nodes = doc.text_math_nodes(id)?;
    Some(layout_nodes(doc, id, &nodes, &mut measure))
}

fn layout_nodes(
    doc: &Document,
    id: usize,
    nodes: &[ReferencedNode],
    measure: &mut Measure<'_>,
) -> Layout {
    let families = doc.text_font_families().unwrap_or_default();
    let style = Style {
        font: doc
            .numeric_label_font_state(id)
            .or_else(|| doc.text_font_state(id))
            .unwrap_or(FontState {
                family: "sans-serif".into(),
                size_code: 16,
                bold: false,
                italic: false,
                underline: false,
            }),
        color: Color32::from_gray(35),
        script: 0,
    };
    let mut result = row(nodes, &style, &families, None, measure);
    // The layout axis is internal; consumers place the top of the box at the
    // document's saved origin.
    for run in &mut result.runs {
        run.position.y += result.above;
    }
    for rule in &mut result.rules {
        for p in &mut rule.points {
            p.y += result.above;
        }
    }
    result
}

type Measure<'a> = dyn FnMut(&str, &FontState, f32, Color32) -> Arc<egui::Galley> + 'a;
fn row(
    nodes: &[ReferencedNode],
    style: &Style,
    families: &[String],
    reference: Option<usize>,
    measure: &mut Measure<'_>,
) -> Layout {
    let mut out = Layout::default();
    for node in nodes {
        let child = node_layout(node, style, families, reference, measure);
        out.above = out.above.max(child.above);
        out.below = out.below.max(child.below);
        let x = out.width;
        out.width += child.width;
        out.append(child, x, 0.);
    }
    out
}
fn node_layout(
    node: &ReferencedNode,
    style: &Style,
    families: &[String],
    reference: Option<usize>,
    measure: &mut Measure<'_>,
) -> Layout {
    match node {
        ReferencedNode::Literal(text) => {
            let galley = measure(text, &style.font, style.size(), style.color);
            let height = galley.size().y;
            let baseline = galley
                .rows
                .first()
                .and_then(|r| r.glyphs.first())
                .map_or(height, |g| g.pos.y);
            Layout {
                width: galley.size().x,
                above: height / 2.,
                below: height / 2.,
                rules: vec![],
                runs: vec![Run {
                    text: text.clone(),
                    font: style.font.clone(),
                    size: style.size(),
                    color: style.color,
                    position: egui::pos2(0., -height / 2.),
                    baseline,
                    galley,
                    reference,
                }],
            }
        }
        ReferencedNode::PiSuffix => node_layout(
            &ReferencedNode::Literal("π".into()),
            style,
            families,
            reference,
            measure,
        ),
        ReferencedNode::FrameMetrics(_) => Layout::default(),
        ReferencedNode::Horizontal(children) => row(children, style, families, reference, measure),
        ReferencedNode::Styled {
            style: next,
            children,
        } => {
            let child_style = style.child(next, families);
            let mut out = row(children, &child_style, families, reference, measure);
            if let Some(decoration) = next.geometry_decoration {
                let y = -out.above - 1.;
                let width = out.width.max(2.);
                let points = match decoration {
                    5 => vec![
                        egui::pos2(0., y + 1.5),
                        egui::pos2(width / 2., y - 1.5),
                        egui::pos2(width, y + 1.5),
                    ],
                    _ => vec![egui::pos2(0., y), egui::pos2(width, y)],
                };
                out.rules.push(Rule {
                    points,
                    color: child_style.color,
                });
                if matches!(decoration, 3 | 4) {
                    out.rules.push(Rule {
                        points: vec![
                            egui::pos2(width - 3., y - 2.),
                            egui::pos2(width, y),
                            egui::pos2(width - 3., y + 2.),
                        ],
                        color: child_style.color,
                    });
                }
                if decoration == 4 {
                    out.rules.push(Rule {
                        points: vec![
                            egui::pos2(3., y - 2.),
                            egui::pos2(0., y),
                            egui::pos2(3., y + 2.),
                        ],
                        color: child_style.color,
                    });
                }
                out.above += 4.;
            }
            out
        }
        ReferencedNode::Reference { slot, cached, .. } => row(
            cached.as_deref().unwrap_or_default(),
            style,
            families,
            reference.or(Some(*slot)),
            measure,
        ),
        ReferencedNode::Vertical(children) | ReferencedNode::VerticalCentered(children) => {
            let centered = matches!(node, ReferencedNode::VerticalCentered(_));
            let children = children
                .iter()
                .map(|child| node_layout(child, style, families, reference, measure))
                .collect::<Vec<_>>();
            let mut out = Layout::default();
            let mut top = 0.;
            out.width = children
                .iter()
                .map(|child| child.width)
                .fold(0.0_f32, f32::max);
            for child in children {
                let x = if centered {
                    (out.width - child.width) / 2.
                } else {
                    0.
                };
                out.width = out.width.max(child.width);
                if top == 0. {
                    out.above = child.above;
                }
                let y = top + child.above - out.above;
                top += child.height();
                out.append(child, x, y);
            }
            out.below = top - out.above;
            out
        }
        ReferencedNode::Fraction {
            numerator,
            denominator,
        } => {
            let n = node_layout(numerator, style, families, reference, measure);
            let d = node_layout(denominator, style, families, reference, measure);
            let width = n.width.max(d.width) + 4.;
            let mut out = Layout {
                width,
                above: n.height() + 1.,
                below: d.height() + 2.,
                ..Default::default()
            };
            let (nx, ny, dx, dy) = (
                (width - n.width) / 2.,
                -1. - n.below,
                (width - d.width) / 2.,
                2. + d.above,
            );
            out.append(n, nx, ny);
            out.append(d, dx, dy);
            out.rules.push(Rule {
                points: vec![egui::pos2(0., 0.), egui::pos2(width, 0.)],
                color: style.color,
            });
            out
        }
        ReferencedNode::Subscript { base, index }
        | ReferencedNode::Superscript {
            base,
            exponent: index,
        } => {
            let base = node_layout(base, style, families, reference, measure);
            let mut script_style = style.clone();
            script_style.script += 1;
            let script = node_layout(index, &script_style, families, reference, measure);
            let dy = if matches!(node, ReferencedNode::Subscript { .. }) {
                style.size() * 0.2
            } else {
                -style.size() * 0.35
            };
            let mut out = Layout {
                width: base.width + script.width,
                above: base.above.max(script.above - dy),
                below: base.below.max(script.below + dy),
                ..Default::default()
            };
            let x = base.width;
            out.append(base, 0., 0.);
            out.append(script, x, dy);
            out
        }
        ReferencedNode::Radical(body) => {
            let body = node_layout(body, style, families, reference, measure);
            let lead = style.size() * 0.6;
            let mut out = Layout {
                width: lead + body.width + 2.,
                above: body.above + 2.,
                below: body.below,
                ..Default::default()
            };
            out.rules.push(Rule {
                points: vec![
                    egui::pos2(0., 0.),
                    egui::pos2(lead * 0.2, -2.),
                    egui::pos2(lead * 0.45, out.below),
                    egui::pos2(lead, -out.above),
                    egui::pos2(out.width, -out.above),
                ],
                color: style.color,
            });
            out.append(body, lead, 0.);
            out
        }
        ReferencedNode::Delimited { open, close, body } => {
            let body = node_layout(body, style, families, reference, measure);
            let mut delimiter_style = style.clone();
            delimiter_style.font.size_code = body.height().ceil().max(style.size()) as u16;
            delimiter_style.script = 0;
            let left = node_layout(
                &ReferencedNode::Literal(open.to_string()),
                &delimiter_style,
                families,
                reference,
                measure,
            );
            let right = node_layout(
                &ReferencedNode::Literal(close.to_string()),
                &delimiter_style,
                families,
                reference,
                measure,
            );
            let mut out = Layout {
                width: left.width + body.width + right.width,
                above: body.above.max(left.above),
                below: body.below.max(left.below),
                ..Default::default()
            };
            let (bx, rx) = (left.width, left.width + body.width);
            out.append(left, 0., 0.);
            out.append(body, bx, 0.);
            out.append(right, rx, 0.);
            out
        }
    }
}

pub(crate) fn font_context(doc: &Document) -> egui::Context {
    let ctx = egui::Context::default();
    let mut definitions = egui::FontDefinitions::default();
    super::system_fonts::add_cjk_fallbacks(&mut definitions);
    let mut families = doc.text_font_families().unwrap_or_default();
    families.extend(
        doc.objects()
            .iter()
            .filter_map(|o| doc.text_font_state(o.id).map(|f| f.family)),
    );
    families.sort();
    families.dedup();
    for family in families {
        if let Some(data) =
            super::system_fonts::system_font(family.strip_prefix('@').unwrap_or(&family))
                .and_then(|font| font.data())
        {
            let key = format!("system:{family}");
            definitions.font_data.insert(key.clone(), Arc::new(data));
            let mut chain = vec![key];
            chain.extend(
                definitions.families[&egui::FontFamily::Proportional]
                    .iter()
                    .cloned(),
            );
            definitions
                .families
                .insert(egui::FontFamily::Name(family.into()), chain);
        }
    }
    ctx.set_fonts(definitions);
    ctx.begin_pass(Default::default());
    ctx
}
pub(crate) fn measured_layout(doc: &Document, id: usize, ctx: &egui::Context) -> Option<Layout> {
    layout(doc, id, |text, font, size, color| {
        measure(ctx, text, font, size, color)
    })
}

pub(crate) fn measured_nodes(
    doc: &Document,
    id: usize,
    nodes: &[ReferencedNode],
    ctx: &egui::Context,
) -> Layout {
    layout_nodes(doc, id, nodes, &mut |text, font, size, color| {
        measure(ctx, text, font, size, color)
    })
}

pub(super) fn measure(
    ctx: &egui::Context,
    text: &str,
    font: &FontState,
    size: f32,
    color: Color32,
) -> Arc<egui::Galley> {
    let family = egui::FontFamily::Name(font.family.clone().into());
    let family = if ctx.fonts(|f| f.families().contains(&family)) {
        family
    } else {
        egui::FontFamily::Proportional
    };
    ctx.fonts_mut(|f| {
        f.layout_job(egui::text::LayoutJob::single_section(
            text.into(),
            text_format(font, size, color, family),
        ))
    })
}
