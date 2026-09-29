//! Strict literal rich text with ordinary one-based parent-reference slots.
//!
//! The original cache, coordinate fields and alignment bytes are retained. A
//! caller resolves references using its document model; this parser does not
//! infer object names, values, or button behavior from cached text.
use crate::text_object::RichRunStyle;

type ReferenceResolver<'a> = dyn FnMut(usize, ReferenceRole, Option<&str>) -> Option<String> + 'a;

/// Native reference operand roles. The low sixteen bits remain a one-based
/// parent slot; role bits select live presentation rather than cached spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceRole {
    Value,
    Name,
    Definition,
    /// A numeric value formatted as an additive term, including its sign.
    SignedValue,
    Expression,
}

impl ReferenceRole {
    fn from_operand(operand: usize) -> Option<(usize, Self)> {
        let role = match operand >> 16 {
            0 => Self::Value,
            0xA => Self::Name,
            0xB => Self::Definition,
            0xD => Self::SignedValue,
            0xE => Self::Expression,
            _ => return None,
        };
        Some((operand & 0xffff, role))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferencedStyle {
    pub font: RichRunStyle,
    /// Native encoded color components (1..=256); interpretation belongs to
    /// the presentation layer rather than the markup parser.
    pub red: Option<u16>,
    pub green: Option<u16>,
    pub blue: Option<u16>,
    /// Native geometry-name decoration: overbar, segment, ray, line, or arc.
    pub geometry_decoration: Option<u8>,
}

impl ReferencedStyle {
    fn inherited(&self, child: &Self) -> Self {
        Self {
            font: RichRunStyle {
                bold: child.font.bold.or(self.font.bold),
                italic: child.font.italic.or(self.font.italic),
                underline: child.font.underline.or(self.font.underline),
                size_code: child.font.size_code.or(self.font.size_code),
                family_index: child.font.family_index.or(self.font.family_index),
                rgb: child.font.rgb.or(self.font.rgb),
                superscript_depth: self.font.superscript_depth + child.font.superscript_depth,
                script_size: child.font.script_size.or(self.font.script_size),
            },
            red: child.red.or(self.red),
            green: child.green.or(self.green),
            blue: child.blue.or(self.blue),
            geometry_decoration: child.geometry_decoration.or(self.geometry_decoration),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferencedNode {
    Literal(String),
    /// Native rich-text `<!70>` following a cached value reference.
    PiSuffix,
    Horizontal(Vec<Self>),
    Vertical(Vec<Self>),
    VerticalCentered(Vec<Self>),
    /// Native outer layout measurements. They affect the source frame, not
    /// the caption's textual or mathematical contents.
    FrameMetrics([Option<u32>; 5]),
    Styled {
        style: ReferencedStyle,
        children: Vec<Self>,
    },
    Radical(Box<Self>),
    Delimited {
        open: char,
        close: char,
        body: Box<Self>,
    },
    Fraction {
        numerator: Box<Self>,
        denominator: Box<Self>,
    },
    /// Native `<-base, index>` mathematical subscript.
    Subscript {
        base: Box<Self>,
        index: Box<Self>,
    },
    Superscript {
        base: Box<Self>,
        exponent: Box<Self>,
    },
    Reference {
        slot: usize,
        role: ReferenceRole,
        /// None denotes a bare reference. Cached literal text is retained
        /// separately from the value supplied by the live resolver.
        cached: Option<Vec<Self>>,
    },
}

impl ReferencedNode {
    pub fn has_math_layout(&self) -> bool {
        match self {
            Self::Fraction { .. }
            | Self::Subscript { .. }
            | Self::Superscript { .. }
            | Self::Radical(_)
            | Self::Delimited { .. }
            | Self::VerticalCentered(_) => true,
            Self::Horizontal(children) | Self::Vertical(children) => {
                children.iter().any(Self::has_math_layout)
            }
            Self::Styled { style, children } => {
                style.geometry_decoration.is_some() || children.iter().any(Self::has_math_layout)
            }
            Self::FrameMetrics(_) | Self::Literal(_) | Self::PiSuffix | Self::Reference { .. } => {
                false
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferencedRun {
    pub text: String,
    pub style: ReferencedStyle,
    pub reference: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferencedText {
    rich: Vec<u8>,
    nodes: Vec<ReferencedNode>,
}

impl ReferencedText {
    pub fn from_rich(rich: &[u8], reference_count: usize) -> Result<Self, String> {
        if rich.len() < 20 || rich[..4] != [0xfc, 8, 0, 0] || rich[8..12] != [4, 0, 0, 0] {
            return Err("Unsupported referenced rich-text header".into());
        }
        let markup_length = u16::from_le_bytes(rich[16..18].try_into().unwrap()) as usize;
        let inner = (18 + markup_length + 2) & !1;
        let outer = (inner + 3) & !3;
        if u32::from_le_bytes(rich[4..8].try_into().unwrap()) as usize != inner
            || rich.len() != outer
            || rich.get(18 + markup_length) != Some(&0)
        {
            return Err("Invalid referenced rich-text length or terminator".into());
        }
        let nodes = Parser::parse(&rich[18..18 + markup_length], reference_count)?;
        Ok(Self {
            rich: rich.to_vec(),
            nodes,
        })
    }

    pub fn from_markup(
        markup: &str,
        origin: (i16, i16),
        reference_count: usize,
    ) -> Result<Self, String> {
        let length = u16::try_from(markup.len()).map_err(|_| "Referenced text is too long")?;
        let inner = (18 + markup.len() + 2) & !1;
        let mut rich = vec![0; (inner + 3) & !3];
        rich[..4].copy_from_slice(&[0xfc, 8, 0, 0]);
        rich[4..8].copy_from_slice(&(inner as u32).to_le_bytes());
        rich[8..12].copy_from_slice(&[4, 0, 0, 0]);
        rich[12..14].copy_from_slice(&origin.0.to_le_bytes());
        rich[14..16].copy_from_slice(&origin.1.to_le_bytes());
        rich[16..18].copy_from_slice(&length.to_le_bytes());
        rich[18..18 + markup.len()].copy_from_slice(markup.as_bytes());
        Self::from_rich(&rich, reference_count)
    }

    pub fn rich_bytes(&self) -> &[u8] {
        &self.rich
    }
    pub fn nodes(&self) -> &[ReferencedNode] {
        &self.nodes
    }
    pub fn stored_origin(&self) -> (i16, i16) {
        (
            i16::from_le_bytes(self.rich[12..14].try_into().unwrap()),
            i16::from_le_bytes(self.rich[14..16].try_into().unwrap()),
        )
    }

    pub fn with_origin(&self, origin: (i16, i16)) -> Self {
        let mut text = self.clone();
        text.rich[12..14].copy_from_slice(&origin.0.to_le_bytes());
        text.rich[14..16].copy_from_slice(&origin.1.to_le_bytes());
        text
    }

    pub fn referenced_slots(&self) -> Vec<usize> {
        self.referenced_roles()
            .into_iter()
            .map(|(slot, _)| slot)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Active operands in markup order. Cached subtrees are presentation hints.
    pub fn reference_occurrences(&self) -> Vec<(usize, ReferenceRole)> {
        fn visit(nodes: &[ReferencedNode], out: &mut Vec<(usize, ReferenceRole)>) {
            for node in nodes {
                match node {
                    ReferencedNode::Reference { slot, role, .. } => out.push((*slot, *role)),
                    ReferencedNode::Horizontal(children)
                    | ReferencedNode::Vertical(children)
                    | ReferencedNode::VerticalCentered(children)
                    | ReferencedNode::Styled { children, .. } => visit(children, out),
                    ReferencedNode::Fraction {
                        numerator,
                        denominator,
                    }
                    | ReferencedNode::Subscript {
                        base: numerator,
                        index: denominator,
                    }
                    | ReferencedNode::Superscript {
                        base: numerator,
                        exponent: denominator,
                    } => {
                        visit(std::slice::from_ref(numerator), out);
                        visit(std::slice::from_ref(denominator), out);
                    }
                    ReferencedNode::Radical(body) | ReferencedNode::Delimited { body, .. } => {
                        visit(std::slice::from_ref(body), out)
                    }
                    ReferencedNode::Literal(_)
                    | ReferencedNode::PiSuffix
                    | ReferencedNode::FrameMetrics(_) => {}
                }
            }
        }
        let mut out = Vec::new();
        visit(&self.nodes, &mut out);
        out
    }

    pub fn referenced_roles(&self) -> Vec<(usize, ReferenceRole)> {
        fn visit(
            nodes: &[ReferencedNode],
            slots: &mut std::collections::BTreeSet<(usize, ReferenceRole)>,
        ) {
            for node in nodes {
                match node {
                    ReferencedNode::Reference { slot, role, .. } => {
                        slots.insert((*slot, *role));
                    }
                    ReferencedNode::Horizontal(children)
                    | ReferencedNode::Vertical(children)
                    | ReferencedNode::VerticalCentered(children)
                    | ReferencedNode::Styled { children, .. } => visit(children, slots),
                    ReferencedNode::Fraction {
                        numerator,
                        denominator,
                    }
                    | ReferencedNode::Subscript {
                        base: numerator,
                        index: denominator,
                    }
                    | ReferencedNode::Superscript {
                        base: numerator,
                        exponent: denominator,
                    } => {
                        visit(std::slice::from_ref(numerator), slots);
                        visit(std::slice::from_ref(denominator), slots);
                    }
                    ReferencedNode::Radical(body) | ReferencedNode::Delimited { body, .. } => {
                        visit(std::slice::from_ref(body), slots)
                    }
                    ReferencedNode::Literal(_)
                    | ReferencedNode::PiSuffix
                    | ReferencedNode::FrameMetrics(_) => {}
                }
            }
        }
        let mut slots = std::collections::BTreeSet::new();
        visit(&self.nodes, &mut slots);
        slots.into_iter().collect()
    }

    pub fn resolved_nodes(
        &self,
        mut resolver: impl FnMut(usize, ReferenceRole) -> Option<String>,
    ) -> Vec<ReferencedNode> {
        fn resolve(
            node: &mut ReferencedNode,
            resolver: &mut dyn FnMut(usize, ReferenceRole) -> Option<String>,
        ) {
            match node {
                ReferencedNode::Reference { slot, role, cached } => {
                    *cached = Some(vec![ReferencedNode::Literal(
                        resolver(*slot, *role).unwrap_or_else(|| "undefined".into()),
                    )]);
                }
                ReferencedNode::Horizontal(children)
                | ReferencedNode::Vertical(children)
                | ReferencedNode::VerticalCentered(children)
                | ReferencedNode::Styled { children, .. } => {
                    for child in children {
                        resolve(child, resolver);
                    }
                }
                ReferencedNode::Fraction {
                    numerator: a,
                    denominator: b,
                }
                | ReferencedNode::Subscript { base: a, index: b }
                | ReferencedNode::Superscript {
                    base: a,
                    exponent: b,
                } => {
                    resolve(a, resolver);
                    resolve(b, resolver);
                }
                ReferencedNode::Radical(body) | ReferencedNode::Delimited { body, .. } => {
                    resolve(body, resolver)
                }
                ReferencedNode::Literal(_)
                | ReferencedNode::PiSuffix
                | ReferencedNode::FrameMetrics(_) => {}
            }
        }
        let mut nodes = self.nodes.clone();
        for node in &mut nodes {
            resolve(node, &mut resolver);
        }
        nodes
    }

    /// Evaluate without modifying the original cache. The resolver receives a
    /// one-based slot and optional plain cached text, including row breaks.
    /// None returned by the resolver means the reference is currently undefined.
    pub fn evaluate(
        &self,
        mut resolver: impl FnMut(usize, Option<&str>) -> Option<String>,
    ) -> Vec<Vec<ReferencedRun>> {
        self.evaluate_roles(|slot, _role, cache| resolver(slot, cache))
    }

    /// Role-aware live evaluation; original rich bytes and caches stay intact.
    pub fn evaluate_roles(
        &self,
        mut resolver: impl FnMut(usize, ReferenceRole, Option<&str>) -> Option<String>,
    ) -> Vec<Vec<ReferencedRun>> {
        horizontal(&self.nodes, &ReferencedStyle::default(), &mut resolver)
    }
}

fn horizontal(
    nodes: &[ReferencedNode],
    inherited: &ReferencedStyle,
    resolver: &mut ReferenceResolver<'_>,
) -> Vec<Vec<ReferencedRun>> {
    let mut rows = vec![vec![]];
    for node in nodes {
        let mut next = evaluate_node(node, inherited, resolver).into_iter();
        if let Some(first) = next.next() {
            rows.last_mut().unwrap().extend(first);
        }
        rows.extend(next);
    }
    rows
}

fn evaluate_node(
    node: &ReferencedNode,
    inherited: &ReferencedStyle,
    resolver: &mut ReferenceResolver<'_>,
) -> Vec<Vec<ReferencedRun>> {
    match node {
        ReferencedNode::Literal(text) => vec![vec![ReferencedRun {
            text: text.clone(),
            style: inherited.clone(),
            reference: None,
        }]],
        ReferencedNode::PiSuffix => vec![vec![ReferencedRun {
            text: "π".into(),
            style: inherited.clone(),
            reference: None,
        }]],
        ReferencedNode::Horizontal(children) => horizontal(children, inherited, resolver),
        ReferencedNode::Vertical(children) | ReferencedNode::VerticalCentered(children) => children
            .iter()
            .flat_map(|child| evaluate_node(child, inherited, resolver))
            .collect(),
        ReferencedNode::FrameMetrics(_) => vec![],
        ReferencedNode::Styled { style, children } => {
            horizontal(children, &inherited.inherited(style), resolver)
        }
        ReferencedNode::Radical(body) => horizontal(
            &[
                ReferencedNode::Literal("√(".into()),
                (**body).clone(),
                ReferencedNode::Literal(")".into()),
            ],
            inherited,
            resolver,
        ),
        ReferencedNode::Delimited { open, close, body } => horizontal(
            &[
                ReferencedNode::Literal(open.to_string()),
                (**body).clone(),
                ReferencedNode::Literal(close.to_string()),
            ],
            inherited,
            resolver,
        ),
        ReferencedNode::Fraction {
            numerator,
            denominator,
        } => {
            // The typed fraction remains available to richer layout consumers.
            // Plain runs retain grouping when flattened for text/SVG consumers.
            horizontal(
                &[
                    ReferencedNode::Literal("(".into()),
                    (**numerator).clone(),
                    ReferencedNode::Literal(")/(".into()),
                    (**denominator).clone(),
                    ReferencedNode::Literal(")".into()),
                ],
                inherited,
                resolver,
            )
        }
        ReferencedNode::Subscript { base, index } => {
            let mut rows = evaluate_node(base, inherited, resolver);
            let index_rows = evaluate_node(index, inherited, resolver);
            if let Some(row) = rows.last_mut() {
                row.extend(index_rows.into_iter().flatten().map(|mut run| {
                    run.text = run.text.chars().map(subscript_character).collect();
                    run
                }));
            }
            rows
        }
        ReferencedNode::Superscript { base, exponent } => {
            let mut rows = evaluate_node(base, inherited, resolver);
            let mut style = inherited.clone();
            style.font.superscript_depth += 1;
            let exponent = evaluate_node(exponent, &style, resolver);
            if let Some(row) = rows.last_mut() {
                row.extend(exponent.into_iter().flatten());
            }
            rows
        }
        ReferencedNode::Reference { slot, role, cached } => {
            // Cache-only calculation wrappers contain no live document slots,
            // so no document resolver is required to inspect them.
            let cache = cached.as_ref().map(|children| {
                horizontal(children, inherited, &mut |_, _, _| None)
                    .into_iter()
                    .map(|row| row.into_iter().map(|run| run.text).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            });
            let text =
                resolver(*slot, *role, cache.as_deref()).unwrap_or_else(|| "undefined".into());
            vec![vec![ReferencedRun {
                text,
                style: inherited.clone(),
                reference: Some(*slot),
            }]]
        }
    }
}

fn subscript_character(character: char) -> char {
    match character {
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
        '+' => '₊',
        '-' | '−' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        other => other,
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    reference_count: usize,
    legacy_gbk: bool,
}
impl<'a> Parser<'a> {
    fn parse(bytes: &'a [u8], reference_count: usize) -> Result<Vec<ReferencedNode>, String> {
        let mut parser = Self {
            bytes,
            at: 0,
            reference_count,
            legacy_gbk: std::str::from_utf8(bytes).is_err(),
        };
        let nodes = parser.sequence(false, false, 0, false)?;
        if nodes.is_empty() || parser.at != bytes.len() {
            return Err("Empty or trailing referenced markup".into());
        }
        Ok(nodes)
    }
    fn sequence(
        &mut self,
        closed: bool,
        in_cache: bool,
        depth: usize,
        vertical_parent: bool,
    ) -> Result<Vec<ReferencedNode>, String> {
        if depth > 128 {
            return Err("Referenced markup nesting is too deep".into());
        }
        let mut nodes = vec![];
        while self.at < self.bytes.len() {
            if self.bytes[self.at] == b'>' {
                if !closed {
                    return Err("Unexpected rich-text closing token".into());
                }
                self.at += 1;
                return Ok(nodes);
            }
            if self.bytes[self.at..].starts_with(b"<T''>") {
                self.at += 5;
                nodes.push(ReferencedNode::Literal(String::new()));
            } else if self.bytes[self.at..].starts_with(b"<0>") {
                self.at += 3;
                nodes.push(ReferencedNode::Literal(String::new()));
            } else if !in_cache
                && self.bytes[self.at..].starts_with(b"<!70>")
                && matches!(
                    nodes.last(),
                    Some(ReferencedNode::Reference {
                        role: ReferenceRole::Value,
                        cached: Some(_),
                        ..
                    })
                )
            {
                self.at += 5;
                nodes.push(ReferencedNode::PiSuffix);
            } else if self.bytes[self.at..].starts_with(b"<!") {
                self.at += 2;
                let code = self.hex()?;
                self.expect(b'>')?;
                let symbol = crate::text_object::markup_symbol(code)
                    .ok_or("Unsupported rich-text symbol")?;
                nodes.push(ReferencedNode::Literal(symbol.into()));
            } else if self.bytes[self.at..].starts_with(b"<+") {
                self.at += 2;
                let children = self.sequence(true, in_cache, depth + 1, false)?;
                let [base, exponent]: [ReferencedNode; 2] = children
                    .try_into()
                    .map_err(|_| "A superscript requires two operands")?;
                nodes.push(ReferencedNode::Superscript {
                    base: Box::new(base),
                    exponent: Box::new(exponent),
                });
            } else if self.bytes[self.at..].starts_with(b"<-") {
                self.at += 2;
                let children = self.sequence(true, in_cache, depth + 1, false)?;
                let [base, index]: [ReferencedNode; 2] = children
                    .try_into()
                    .map_err(|_| "A subscript requires two operands")?;
                nodes.push(ReferencedNode::Subscript {
                    base: Box::new(base),
                    index: Box::new(index),
                });
            } else if [b"<R".as_slice(), b"<(", b"<|"]
                .iter()
                .any(|prefix| self.bytes[self.at..].starts_with(prefix))
            {
                let marker = self.bytes[self.at + 1];
                self.at += 2;
                let children = self.sequence(true, in_cache, depth + 1, false)?;
                let [body]: [ReferencedNode; 1] = children
                    .try_into()
                    .map_err(|_| "A radical or delimiter requires one operand")?;
                nodes.push(match marker {
                    b'R' => ReferencedNode::Radical(Box::new(body)),
                    b'(' => ReferencedNode::Delimited {
                        open: '(',
                        close: ')',
                        body: Box::new(body),
                    },
                    _ => ReferencedNode::Delimited {
                        open: '|',
                        close: '|',
                        body: Box::new(body),
                    },
                });
            } else if self.bytes[self.at..].starts_with(b"<T") {
                self.at += 2;
                let length = self.hex()?;
                self.expect(b'x')?;
                let end = self
                    .at
                    .checked_add(length)
                    .ok_or("Invalid literal length")?;
                let text = self
                    .bytes
                    .get(self.at..end)
                    .ok_or("Truncated rich-text literal")?;
                if text.contains(&0) {
                    return Err("NUL inside rich-text literal".into());
                }
                let text = crate::text_object::decode_markup_literal(text, self.legacy_gbk)
                    .ok_or("Invalid rich-text literal encoding")?;
                self.at = end;
                self.expect(b'>')?;
                nodes.push(ReferencedNode::Literal(text));
            } else if self.bytes[self.at..].starts_with(b"<VL")
                || self.bytes[self.at..].starts_with(b"<VC")
                || self.bytes[self.at..].starts_with(b"<H")
            {
                let vertical = self.bytes[self.at..].starts_with(b"<VL");
                let centered = self.bytes[self.at..].starts_with(b"<VC");
                self.at += if vertical || centered { 3 } else { 2 };
                let children = self.sequence(true, in_cache, depth + 1, vertical || centered)?;
                if children.is_empty() {
                    return Err("Empty rich-text layout".into());
                }
                nodes.push(if centered {
                    ReferencedNode::VerticalCentered(children)
                } else if vertical {
                    ReferencedNode::Vertical(children)
                } else {
                    ReferencedNode::Horizontal(children)
                });
            } else if !in_cache
                && (vertical_parent || depth == 0)
                && self.bytes[self.at..].starts_with(b"<F")
            {
                self.at += 2;
                let mut fields = [None; 5];
                for field in &mut fields {
                    *field = self.optional_hex_field()?;
                }
                self.expect(b'>')?;
                if self.at < self.bytes.len() && self.bytes[self.at] != b'>' {
                    return Err("Frame metrics must end an outer layout".into());
                }
                nodes.push(ReferencedNode::FrameMetrics(fields));
            } else if in_cache
                && (self.bytes[self.at..].starts_with(b"<?3")
                    || self.bytes[self.at..].starts_with(b"<?4"))
            {
                // Native calculation caches wrap their already-rendered math
                // tree in observed ?3/?4 presentation containers. They are
                // not live reference operands; retain the parsed cache only as
                // a fallback display tree for the outer reference.
                self.at += 3;
                let children = self.sequence(true, true, depth + 1, false)?;
                if children.is_empty() {
                    return Err("Empty calculated reference cache".into());
                }
                nodes.push(ReferencedNode::Horizontal(children));
            } else if self.bytes[self.at..].starts_with(b"<?1<") {
                // Older native files use ?1 as a cache-only presentation
                // wrapper, including nested wrappers around styled names.
                // It carries no live slot; retain and render its explicit tree.
                self.at += 3;
                let children = self.sequence(true, true, depth + 1, false)?;
                if children.is_empty() {
                    return Err("Empty legacy presentation cache".into());
                }
                nodes.push(ReferencedNode::Horizontal(children));
            } else if self.bytes[self.at..].starts_with(b"<?") {
                if in_cache {
                    return Err("Unsupported nested reference cache".into());
                }
                self.at += 2;
                let cached = match self.bytes.get(self.at) {
                    Some(b'1') => {
                        self.at += 1;
                        true
                    }
                    Some(b'x') => false,
                    _ => return Err("Unsupported reference mode".into()),
                };
                self.expect(b'x')?;
                let (slot, role) = ReferenceRole::from_operand(self.hex()?)
                    .ok_or("Unsupported reference role bits")?;
                if slot == 0 || slot > self.reference_count {
                    return Err("Unsupported reference slot or flags".into());
                }
                let cache = if cached {
                    let children = self.sequence(true, true, depth + 1, false)?;
                    if children.is_empty() {
                        return Err("Empty reference cache".into());
                    }
                    Some(children)
                } else {
                    self.expect(b'>')?;
                    None
                };
                nodes.push(ReferencedNode::Reference {
                    slot,
                    role,
                    cached: cache,
                });
            } else if self.bytes[self.at..].starts_with(b"</") {
                self.at += 2;
                let children = self.sequence(true, in_cache, depth + 1, false)?;
                let [numerator, denominator]: [ReferencedNode; 2] = children
                    .try_into()
                    .map_err(|_| "A rich fraction requires two operands")?;
                nodes.push(ReferencedNode::Fraction {
                    numerator: Box::new(numerator),
                    denominator: Box::new(denominator),
                });
            } else if self.bytes[self.at..].starts_with(b"<S") {
                self.at += 2;
                let style = self.style(in_cache)?;
                let children = self.sequence(true, in_cache, depth + 1, false)?;
                if children.is_empty() {
                    return Err("Empty rich-text style".into());
                }
                nodes.push(ReferencedNode::Styled { style, children });
            } else {
                return Err("Unsupported referenced rich-text token".into());
            }
        }
        if closed {
            Err("Unclosed rich-text token".into())
        } else {
            Ok(nodes)
        }
    }
    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.bytes.get(self.at) != Some(&byte) {
            return Err("Invalid rich-text token boundary".into());
        }
        self.at += 1;
        Ok(())
    }
    fn hex(&mut self) -> Result<usize, String> {
        let start = self.at;
        while self.bytes.get(self.at).is_some_and(u8::is_ascii_hexdigit) {
            self.at += 1;
        }
        if self.at == start {
            return Err("Missing rich-text hexadecimal value".into());
        }
        usize::from_str_radix(
            std::str::from_utf8(&self.bytes[start..self.at]).unwrap(),
            16,
        )
        .map_err(|_| "Rich-text hexadecimal overflow".into())
    }
    fn optional_hex_field(&mut self) -> Result<Option<u32>, String> {
        let start = self.at;
        while self.bytes.get(self.at).is_some_and(u8::is_ascii_hexdigit) {
            self.at += 1;
        }
        let value = if self.at == start {
            None
        } else {
            Some(
                u32::from_str_radix(
                    std::str::from_utf8(&self.bytes[start..self.at]).unwrap(),
                    16,
                )
                .map_err(|_| "Frame metric overflow")?,
            )
        };
        self.expect(b'x')?;
        Ok(value)
    }
    fn style(&mut self, in_cache: bool) -> Result<ReferencedStyle, String> {
        let mut out = ReferencedStyle::default();
        // Native dirty-save embeds a source-font context as SP#... in caches.
        if in_cache && self.bytes.get(self.at..self.at + 2) == Some(b"P#") {
            self.at += 1;
        }
        let mut cache_spacing = false;
        while self.bytes.get(self.at) != Some(&b'<') {
            let attr = *self.bytes.get(self.at).ok_or("Truncated rich-text style")?;
            self.at += 1;
            let value =
                if attr == b'P' && !self.bytes.get(self.at).is_some_and(u8::is_ascii_hexdigit) {
                    0
                } else {
                    u16::try_from(self.hex()?).map_err(|_| "Rich-text style value overflow")?
                };
            match attr {
                b'#' if (8..=512).contains(&value) && out.font.size_code.is_none() => {
                    out.font.size_code = Some(value)
                }
                b'P' if out.font.family_index.is_none() => out.font.family_index = Some(value),
                b'+' if out.font.script_size.is_none() => {
                    out.font.script_size =
                        Some(u8::try_from(value).map_err(|_| "Invalid script-size level")?)
                }
                b'O' if (1..=5).contains(&value) && out.geometry_decoration.is_none() => {
                    out.geometry_decoration = Some(value as u8)
                }
                b'T' if in_cache && value == 1 && !cache_spacing => cache_spacing = true,
                b'W' if (1..=2).contains(&value) && out.font.bold.is_none() => {
                    out.font.bold = Some(value == 2)
                }
                b'S' if (1..=2).contains(&value) && out.font.italic.is_none() => {
                    out.font.italic = Some(value == 2)
                }
                b'U' if (1..=2).contains(&value) && out.font.underline.is_none() => {
                    out.font.underline = Some(value == 2)
                }
                b'R' if (1..=256).contains(&value) && out.red.is_none() => out.red = Some(value),
                b'G' if (1..=256).contains(&value) && out.green.is_none() => {
                    out.green = Some(value)
                }
                b'L' if (1..=256).contains(&value) && out.blue.is_none() => out.blue = Some(value),
                _ => return Err("Unsupported or duplicate rich-text style attribute".into()),
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_references_keep_literal_rows_styles_and_slot_identity() {
        let markup = "<VL<H<T6x值：><?1x1<H<T6x999.99>>>><H<SW2R1G1L100<T1x[><?x2><T1x]>>>>";
        let text = ReferencedText::from_markup(markup, (-12, 45), 2).unwrap();
        let mut requests = vec![];
        let rows = text.evaluate(|slot, cache| {
            requests.push((slot, cache.map(str::to_owned)));
            Some(if slot == 1 { "3.00" } else { "隐藏对象" }.into())
        });
        assert_eq!(requests, [(1, Some("999.99".into())), (2, None)]);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "值：3.00"
        );
        assert_eq!(
            rows[1]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "[隐藏对象]"
        );
        assert_eq!(rows[0][1].reference, Some(1));
        assert_eq!(rows[1][1].reference, Some(2));
        assert_eq!(rows[1][1].style.font.bold, Some(true));
        assert_eq!(rows[1][1].style.blue, Some(256));
        assert_eq!(text.referenced_slots(), [1, 2]);
        assert_eq!(text.stored_origin(), (-12, 45));
        assert_eq!(text.rich_bytes()[18..18 + markup.len()], *markup.as_bytes());
    }

    #[test]
    fn position_edit_preserves_nonzero_alignment_and_cached_markup() {
        let original = ReferencedText::from_markup("<?1x1<T1x0>>", (200, 80), 1).unwrap();
        let mut bytes = original.rich_bytes().to_vec();
        let nul = 18 + "<?1x1<T1x0>>".len();
        for byte in &mut bytes[nul + 1..] {
            *byte = 0xe6;
        }
        let text = ReferencedText::from_rich(&bytes, 1).unwrap();
        assert_eq!(text.rich_bytes(), bytes);
        let moved = text.with_origin((-100, 180));
        let mut expected = bytes.clone();
        expected[12..14].copy_from_slice(&(-100i16).to_le_bytes());
        expected[14..16].copy_from_slice(&180i16.to_le_bytes());
        assert_eq!(moved.rich_bytes(), expected);
        assert_eq!(text.stored_origin(), (200, 80));
        assert_eq!(
            moved.evaluate(|_, _| Some("5.00".into()))[0][0].text,
            "5.00"
        );
    }

    #[test]
    fn unproved_flags_math_and_malformed_markup_are_rejected_completely() {
        for markup in [
            "<?xC0001>",
            "<?x1A0001>",
            "<?xA0000>",
            "<?xA0002>",
            "</<T1x1>>",
            "</<T1x1><T1x2><T1x3>>",
            "<?2x1<T1x0>>",
            "<?x0>",
            "<?x2>",
            "<?1x1>",
            "<?1x1<?x1>>",
            "<H<?x1>",
            "<?x1>>",
            "<?x1>trailing",
            "<T2x值>",
            "<T4xabc>",
            "<T1x\0>",
            "<SR0<T1xx>>",
            "<SW3<T1xx>>",
            "<SW2W1<T1xx>>",
        ] {
            assert!(
                ReferencedText::from_markup(markup, (0, 0), 1).is_err(),
                "{markup:?}"
            );
        }
        let valid = ReferencedText::from_markup("<?x1>", (0, 0), 1).unwrap();
        for at in [0, 4, 8, 16, 23] {
            let mut malformed = valid.rich_bytes().to_vec();
            malformed[at] ^= 1;
            assert!(
                ReferencedText::from_rich(&malformed, 1).is_err(),
                "corruption at {at}"
            );
        }
    }

    #[test]
    fn native_roles_keep_low_slots_and_live_fraction_operands_separate_from_cache() {
        let markup = "<VL<H<?xA0001><T1x=></<?1xB0001<T5xSTALE>><T1x2>><T1x+><?xE0002>>>";
        let text = ReferencedText::from_markup(markup, (12, 34), 2).unwrap();
        assert_eq!(text.referenced_slots(), [1, 2]);
        assert_eq!(
            text.referenced_roles(),
            [
                (1, ReferenceRole::Name),
                (1, ReferenceRole::Definition),
                (2, ReferenceRole::Expression)
            ]
        );
        let mut requests = Vec::new();
        let rows = text.evaluate_roles(|slot, role, cache| {
            requests.push((slot, role, cache.map(str::to_owned)));
            Some(
                match role {
                    ReferenceRole::Name => "n",
                    ReferenceRole::Definition => "n+1",
                    ReferenceRole::SignedValue => panic!(),
                    ReferenceRole::Expression => "x²",
                    ReferenceRole::Value => panic!(),
                }
                .into(),
            )
        });
        assert_eq!(
            rows[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "n=(n+1)/(2)+x²"
        );
        assert_eq!(
            requests,
            [
                (1, ReferenceRole::Name, None),
                (1, ReferenceRole::Definition, Some("STALE".into())),
                (2, ReferenceRole::Expression, None)
            ]
        );
        assert_eq!(&text.rich_bytes()[18..18 + markup.len()], markup.as_bytes());

        let calculated = "<?1x2<H<H<H<T''><?3<+<H<!D7><T2x10>><T''>><!A5><H<T1x2><SS2<!70>>><S+1</<H<T''><SS2<!70>>><T''>>>>>><T''>>>";
        let text = ReferencedText::from_markup(calculated, (0, 0), 2).unwrap();
        assert_eq!(text.referenced_slots(), [2]);
        assert_eq!(
            text.evaluate_roles(|slot, role, _| {
                assert_eq!((slot, role), (2, ReferenceRole::Value));
                Some("LIVE".into())
            })[0][0]
                .text,
            "LIVE"
        );
    }

    #[test]
    fn nested_style_inherits_only_overridden_fields_and_unicode_literal_uses_byte_length() {
        let text =
            ReferencedText::from_markup("<SW2#18P3R100<T3x甲><SW1U2G1<?x1>>>", (0, 0), 1).unwrap();
        let rows = text.evaluate(|_, _| Some("乙".into()));
        assert_eq!(rows[0][0].style.font.bold, Some(true));
        assert_eq!(rows[0][1].style.font.bold, Some(false));
        assert_eq!(rows[0][1].style.font.underline, Some(true));
        assert_eq!(rows[0][1].style.font.size_code, Some(24));
        assert_eq!(rows[0][1].style.font.family_index, Some(3));
        assert_eq!(rows[0][1].style.red, Some(256));
        assert_eq!(rows[0][1].style.green, Some(1));
        assert_eq!(rows[0][1].reference, Some(1));

        let decorated = ReferencedText::from_markup("<SO3<T2xAB>>", (0, 0), 1).unwrap();
        assert!(decorated.nodes()[0].has_math_layout());
        assert_eq!(
            decorated.evaluate(|_, _| None)[0][0]
                .style
                .geometry_decoration,
            Some(3)
        );

        let centered =
            ReferencedText::from_markup("<VC<H<T1xa>><H<T3xbbb>><F1Cxx205x20x004x>>", (0, 0), 1)
                .unwrap();
        let ReferencedNode::VerticalCentered(children) = &centered.nodes()[0] else {
            panic!("expected centered vertical layout")
        };
        assert!(centered.nodes()[0].has_math_layout());
        assert_eq!(
            children.last(),
            Some(&ReferencedNode::FrameMetrics([
                Some(0x1c),
                None,
                Some(0x205),
                Some(0x20),
                Some(0x4)
            ]))
        );
        assert_eq!(
            centered
                .evaluate(|_, _| None)
                .into_iter()
                .map(|row| row.into_iter().map(|run| run.text).collect::<String>())
                .collect::<Vec<_>>(),
            ["a", "bbb"]
        );

        let legacy = ReferencedText::from_markup("<H<T2xaa><T2xbb>>", (0, 0), 1).unwrap();
        let mut bytes = legacy.rich_bytes().to_vec();
        bytes[24..26].copy_from_slice(&[0xca, 0xb1]);
        bytes[31..33].copy_from_slice(&[0xb7, 0xd6]);
        let legacy = ReferencedText::from_rich(&bytes, 1).unwrap();
        assert_eq!(
            legacy.evaluate(|_, _| None)[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "时分"
        );
    }
    #[test]
    fn native_dirty_cache_math_stays_inert_and_rejects_nested_dependencies() {
        for cache in [
            "<-<SS2<T1xm>><T1x3>>",
            "</<H<SS2<T1xA>><0><SS2<T1xC>>><H<SS2<T1xA>><0><SS2<T1xB>>>>",
            "<H<SS2<T''>><SO2<SS2<T2xAB>>>>",
            "<SP#18W2S1U1O1T1R1G1L1<VL<T3xOLD>>>",
            "<H<SS2<T1xf>><(<SS2<T1xx>>><T3x = ><+<SS2<T1xx>><T1x2>>>",
            "<H<T1xn><!103><T1x1>>",
        ] {
            let markup = format!("<?1xA0001{cache}>");
            let text = ReferencedText::from_markup(&markup, (0, 0), 1).unwrap();
            assert_eq!(text.reference_occurrences(), [(1, ReferenceRole::Name)]);
            let rows = text.evaluate_roles(|_, _, _| Some("LIVE".into()));
            assert_eq!(rows[0][0].text, "LIVE");
            assert_eq!(&text.rich_bytes()[18..18 + markup.len()], markup.as_bytes());
        }
        for markup in [
            "<?1x1<-<T1xm>>>",
            "<?1x1<+<T1xx><T1x2><T1x3>>>",
            "<?1x1<-<T1xm><?x1>>>",
            "<?1x1<SO2O1<T1xx>>>",
            "<?1x1<Q<T1xx>>>",
            "<!FFFF>",
        ] {
            assert!(
                ReferencedText::from_markup(markup, (0, 0), 1).is_err(),
                "{markup}"
            );
        }
    }

    #[test]
    fn native_pi_suffix_uses_live_cached_reference_and_retains_its_rich_bytes() {
        let markup = "<H<?1x1<H<T4x–1>>><!70>>";
        let text = ReferencedText::from_markup(markup, (12, 34), 1).unwrap();
        assert_eq!(text.reference_occurrences(), [(1, ReferenceRole::Value)]);
        for (value, expected) in [("−1", "−1π"), ("1", "1π")] {
            let rows = text.evaluate(|slot, cache| {
                assert_eq!(slot, 1);
                assert_eq!(cache, Some("–1"));
                Some(value.into())
            });
            assert_eq!(
                rows[0]
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect::<String>(),
                expected
            );
        }
        assert_eq!(&text.rich_bytes()[18..18 + markup.len()], markup.as_bytes());

        let symbols = ReferencedText::from_markup(
            "<H<!5C><!5E><!71><!A3><!A5><!AE><!B1><!B7><!B8><!BB><!F2><!100><!103>>",
            (0, 0),
            1,
        )
        .unwrap();
        assert_eq!(
            symbols.evaluate(|_, _| None)[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "∴⊥θ≤∞→±•÷≈∫⊙−"
        );

        let signed =
            ReferencedText::from_markup("<H<T2xy=><?1xD0001<T4x + 2>>>", (0, 0), 1).unwrap();
        assert_eq!(
            signed.reference_occurrences(),
            [(1, ReferenceRole::SignedValue)]
        );
        assert_eq!(
            signed.evaluate_roles(|_, role, cache| {
                assert_eq!(role, ReferenceRole::SignedValue);
                assert_eq!(cache, Some(" + 2"));
                Some(" − 3".into())
            })[0]
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "y= − 3"
        );
        for invalid in ["<!>", "<H<?x1><!FFFF>>", "<H<?1x1<H<T1x1>>><!72>>"] {
            assert!(ReferencedText::from_markup(invalid, (0, 0), 1).is_err());
        }
    }
}
