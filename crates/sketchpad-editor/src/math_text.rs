//! Shared two-dimensional caption layout for canvas, SVG, and picking.
use sketchpad_engine::document::{Document, referenced_text::{ReferencedNode, ReferencedStyle}};
use sketchpad_engine::text_style_records::FontState;
use egui::{Color32, Pos2, Vec2};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Run {
    pub text: String,
    pub font: FontState,
    pub size: f32,
    pub color: Color32,
    pub position: Pos2,
    pub baseline: f32,
    pub galley: Arc<egui::Galley>,
    pub reference: Option<usize>,
}
#[derive(Clone)]
pub(crate) struct Rule { pub points: Vec<Pos2>, pub color: Color32 }
#[derive(Default)]
pub(crate) struct Layout {
    pub width: f32,
    pub above: f32,
    pub below: f32,
    pub runs: Vec<Run>,
    pub rules: Vec<Rule>,
}
impl Layout {
    pub fn height(&self) -> f32 { self.above + self.below }
    fn append(&mut self, mut child: Self, x: f32, y: f32) {
        for run in &mut child.runs { run.position += egui::vec2(x, y); }
        for rule in &mut child.rules { for point in &mut rule.points { *point += egui::vec2(x, y); } }
        self.runs.extend(child.runs);
        self.rules.extend(child.rules);
    }
}
#[derive(Clone)]
struct Style { font: FontState, color: Color32, script: u8 }
impl Style {
    fn size(&self) -> f32 { f32::from(self.font.size_code) * 0.75_f32.powi(i32::from(self.script)) }
    fn child(&self, next: &ReferencedStyle, families: &[String]) -> Self {
        let mut result = self.clone();
        if let Some(v) = next.font.size_code { result.font.size_code = v; }
        if let Some(v) = next.font.bold { result.font.bold = v; }
        if let Some(v) = next.font.italic { result.font.italic = v; }
        if let Some(v) = next.font.underline { result.font.underline = v; }
        if let Some(v) = next.font.family_index.and_then(|i| families.get(usize::from(i))) { result.font.family = v.clone(); }
        if let Some(v) = next.font.script_size { result.script = v; }
        let rgb = next.font.rgb_color().or_else(|| Some([
            u8::try_from(next.red? - 1).ok()?, u8::try_from(next.green? - 1).ok()?, u8::try_from(next.blue? - 1).ok()?,
        ]));
        if let Some([r,g,b]) = rgb { result.color = Color32::from_rgb(r,g,b); }
        result
    }
}

pub(crate) fn text_format(font: &FontState, size: f32, color: Color32, family: egui::FontFamily) -> egui::TextFormat {
    egui::TextFormat {
        font_id: egui::FontId::new(size, family), color, italics: font.italic,
        underline: if font.underline { egui::Stroke::new(1., color) } else { egui::Stroke::NONE },
        ..Default::default()
    }
}

pub(crate) fn layout(
    doc: &Document, id: usize,
    mut measure: impl FnMut(&str, &FontState, f32, Color32) -> Arc<egui::Galley>,
) -> Option<Layout> {
    let nodes = doc.text_math_nodes(id)?;
    let families = doc.text_font_families().unwrap_or_default();
    let style = Style {
        font: doc.text_font_state(id).unwrap_or(FontState { family: "sans-serif".into(), size_code: 16, bold: false, italic: false, underline: false }),
        color: Color32::from_gray(35), script: 0,
    };
    let mut result = row(&nodes, &style, &families, None, &mut measure);
    // The layout axis is internal; consumers place the top of the box at the
    // document's saved origin.
    for run in &mut result.runs { run.position.y += result.above; }
    for rule in &mut result.rules { for p in &mut rule.points { p.y += result.above; } }
    Some(result)
}

type Measure<'a> = dyn FnMut(&str, &FontState, f32, Color32) -> Arc<egui::Galley> + 'a;
fn row(nodes: &[ReferencedNode], style: &Style, families: &[String], reference: Option<usize>, measure: &mut Measure<'_>) -> Layout {
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
fn node_layout(node: &ReferencedNode, style: &Style, families: &[String], reference: Option<usize>, measure: &mut Measure<'_>) -> Layout {
    match node {
        ReferencedNode::Literal(text) => {
            let galley = measure(text, &style.font, style.size(), style.color);
            let height = galley.size().y;
            let baseline = galley.rows.first().and_then(|r| r.glyphs.first()).map_or(height, |g| g.pos.y);
            Layout { width: galley.size().x, above: height / 2., below: height / 2., rules: vec![], runs: vec![Run {
                text: text.clone(), font: style.font.clone(), size: style.size(), color: style.color,
                position: egui::pos2(0., -height / 2.), baseline, galley, reference,
            }] }
        }
        ReferencedNode::PiSuffix => node_layout(&ReferencedNode::Literal("π".into()), style, families, reference, measure),
        ReferencedNode::Horizontal(children) => row(children, style, families, reference, measure),
        ReferencedNode::Styled { style: next, children } => row(children, &style.child(next, families), families, reference, measure),
        ReferencedNode::Reference { slot, cached, .. } => row(cached.as_deref().unwrap_or_default(), style, families, Some(*slot), measure),
        ReferencedNode::Vertical(children) => {
            let mut out = Layout::default();
            let mut top = 0.;
            for child in children {
                let child = node_layout(child, style, families, reference, measure);
                out.width = out.width.max(child.width);
                let y = top + child.above;
                top += child.height();
                out.append(child, 0., y);
            }
            out.below = top;
            out
        }
        ReferencedNode::Fraction { numerator, denominator } => {
            let n = node_layout(numerator, style, families, reference, measure);
            let d = node_layout(denominator, style, families, reference, measure);
            let width = n.width.max(d.width) + 4.;
            let mut out = Layout { width, above: n.height() + 1., below: d.height() + 2., ..Default::default() };
            let (nx, ny, dx, dy) = ((width-n.width)/2., -1.-n.below, (width-d.width)/2., 2.+d.above);
            out.append(n, nx, ny); out.append(d, dx, dy);
            out.rules.push(Rule { points: vec![egui::pos2(0.,0.),egui::pos2(width,0.)], color: style.color });
            out
        }
        ReferencedNode::Subscript { base, index } | ReferencedNode::Superscript { base, exponent: index } => {
            let base = node_layout(base, style, families, reference, measure);
            let mut script_style = style.clone(); script_style.script += 1;
            let script = node_layout(index, &script_style, families, reference, measure);
            let dy = if matches!(node, ReferencedNode::Subscript { .. }) { style.size() * 0.2 } else { -style.size() * 0.35 };
            let mut out = Layout { width: base.width + script.width, above: base.above.max(script.above-dy), below: base.below.max(script.below+dy), ..Default::default() };
            let x = base.width;
            out.append(base,0.,0.); out.append(script,x,dy); out
        }
        ReferencedNode::Radical(body) => {
            let body = node_layout(body, style, families, reference, measure);
            let lead = style.size() * 0.6;
            let mut out = Layout { width: lead + body.width + 2., above: body.above+2., below: body.below, ..Default::default() };
            out.rules.push(Rule { points: vec![egui::pos2(0.,0.),egui::pos2(lead*0.2,-2.),egui::pos2(lead*0.45,out.below),egui::pos2(lead,-out.above),egui::pos2(out.width,-out.above)], color: style.color });
            out.append(body,lead,0.); out
        }
        ReferencedNode::Delimited { open, close, body } => {
            let body = node_layout(body, style, families, reference, measure);
            let mut delimiter_style = style.clone();
            delimiter_style.font.size_code = body.height().ceil().max(style.size()) as u16;
            delimiter_style.script = 0;
            let left = node_layout(&ReferencedNode::Literal(open.to_string()), &delimiter_style, families, reference, measure);
            let right = node_layout(&ReferencedNode::Literal(close.to_string()), &delimiter_style, families, reference, measure);
            let mut out = Layout { width: left.width+body.width+right.width, above: body.above.max(left.above), below: body.below.max(left.below), ..Default::default() };
            let (bx,rx)=(left.width,left.width+body.width);
            out.append(left,0.,0.);out.append(body,bx,0.);out.append(right,rx,0.);out
        }
    }
}

pub(crate) fn font_context(doc: &Document) -> egui::Context {
    let ctx = egui::Context::default();
    let mut definitions = egui::FontDefinitions::default();
    let mut families = doc.text_font_families().unwrap_or_default();
    families.extend(doc.objects.iter().filter_map(|o| doc.text_font_state(o.id).map(|f| f.family)));
    families.sort(); families.dedup();
    for family in families {
        if let Some(data) = crate::editor::fonts::system_font_data(&family) {
            let key = format!("system:{family}");
            definitions.font_data.insert(key.clone(), Arc::new(data));
            let mut chain = vec![key];
            chain.extend(definitions.families[&egui::FontFamily::Proportional].iter().cloned());
            definitions.families.insert(egui::FontFamily::Name(family.into()), chain);
        }
    }
    ctx.set_fonts(definitions);
    ctx.begin_pass(Default::default());
    ctx
}
pub(crate) fn measured_layout(doc: &Document, id: usize, ctx: &egui::Context) -> Option<Layout> {
    layout(doc,id,|text,font,size,color| {
        let family = egui::FontFamily::Name(font.family.clone().into());
        let family = if ctx.fonts(|f| f.families().contains(&family)) { family } else { egui::FontFamily::Proportional };
        ctx.fonts_mut(|f| f.layout_job(egui::text::LayoutJob::single_section(text.into(),text_format(font,size,color,family))))
    })
}
