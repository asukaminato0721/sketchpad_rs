//! Strict literal rich text with ordinary one-based parent-reference slots.
//!
//! The original cache, coordinate fields and alignment bytes are retained. A
//! caller resolves references using its document model; this parser does not
//! infer object names, values, or button behavior from cached text.
use crate::text_object::RichRunStyle;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferencedStyle {
    pub font: RichRunStyle,
    /// Native encoded color components (1..=256); interpretation belongs to
    /// the presentation layer rather than the markup parser.
    pub red: Option<u16>,
    pub green: Option<u16>,
    pub blue: Option<u16>,
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
            },
            red: child.red.or(self.red),
            green: child.green.or(self.green),
            blue: child.blue.or(self.blue),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferencedNode {
    Literal(String),
    Horizontal(Vec<Self>),
    Vertical(Vec<Self>),
    Styled {
        style: ReferencedStyle,
        children: Vec<Self>,
    },
    Reference {
        slot: usize,
        /// None denotes a bare reference. Cached literal text is retained
        /// separately from the value supplied by the live resolver.
        cached: Option<Vec<Self>>,
    },
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
        fn visit(nodes: &[ReferencedNode], slots: &mut std::collections::BTreeSet<usize>) {
            for node in nodes {
                match node {
                    ReferencedNode::Reference { slot, .. } => {
                        slots.insert(*slot);
                    }
                    ReferencedNode::Horizontal(children)
                    | ReferencedNode::Vertical(children)
                    | ReferencedNode::Styled { children, .. } => visit(children, slots),
                    ReferencedNode::Literal(_) => {}
                }
            }
        }
        let mut slots = std::collections::BTreeSet::new();
        visit(&self.nodes, &mut slots);
        slots.into_iter().collect()
    }

    /// Evaluate without modifying the original cache. The resolver receives a
    /// one-based slot and optional plain cached text, including row breaks.
    /// None returned by the resolver means the reference is currently undefined.
    pub fn evaluate(
        &self,
        mut resolver: impl FnMut(usize, Option<&str>) -> Option<String>,
    ) -> Vec<Vec<ReferencedRun>> {
        horizontal(&self.nodes, &ReferencedStyle::default(), &mut resolver)
    }
}

fn horizontal(
    nodes: &[ReferencedNode],
    inherited: &ReferencedStyle,
    resolver: &mut dyn FnMut(usize, Option<&str>) -> Option<String>,
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
    resolver: &mut dyn FnMut(usize, Option<&str>) -> Option<String>,
) -> Vec<Vec<ReferencedRun>> {
    match node {
        ReferencedNode::Literal(text) => vec![vec![ReferencedRun {
            text: text.clone(),
            style: inherited.clone(),
            reference: None,
        }]],
        ReferencedNode::Horizontal(children) => horizontal(children, inherited, resolver),
        ReferencedNode::Vertical(children) => children
            .iter()
            .flat_map(|child| evaluate_node(child, inherited, resolver))
            .collect(),
        ReferencedNode::Styled { style, children } => {
            horizontal(children, &inherited.inherited(style), resolver)
        }
        ReferencedNode::Reference { slot, cached } => {
            // Nested references are rejected by the parser, so no document
            // resolver is required to inspect a literal cache.
            let cache = cached.as_ref().map(|children| {
                horizontal(children, inherited, &mut |_, _| None)
                    .into_iter()
                    .map(|row| row.into_iter().map(|run| run.text).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            });
            let text = resolver(*slot, cache.as_deref()).unwrap_or_else(|| "undefined".into());
            vec![vec![ReferencedRun {
                text,
                style: inherited.clone(),
                reference: Some(*slot),
            }]]
        }
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    reference_count: usize,
}
impl<'a> Parser<'a> {
    fn parse(bytes: &'a [u8], reference_count: usize) -> Result<Vec<ReferencedNode>, String> {
        let mut parser = Self {
            bytes,
            at: 0,
            reference_count,
        };
        let nodes = parser.sequence(false, false, 0)?;
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
            if self.bytes[self.at..].starts_with(b"<T") {
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
                let text = std::str::from_utf8(text)
                    .map_err(|_| "Invalid UTF-8 rich-text literal")?
                    .to_owned();
                self.at = end;
                self.expect(b'>')?;
                nodes.push(ReferencedNode::Literal(text));
            } else if self.bytes[self.at..].starts_with(b"<VL")
                || self.bytes[self.at..].starts_with(b"<H")
            {
                let vertical = self.bytes[self.at..].starts_with(b"<VL");
                self.at += if vertical { 3 } else { 2 };
                let children = self.sequence(true, in_cache, depth + 1)?;
                if children.is_empty() {
                    return Err("Empty rich-text layout".into());
                }
                nodes.push(if vertical {
                    ReferencedNode::Vertical(children)
                } else {
                    ReferencedNode::Horizontal(children)
                });
            } else if self.bytes[self.at..].starts_with(b"<?") {
                if in_cache {
                    return Err("Nested reference cache is unsupported".into());
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
                let slot = self.hex()?;
                if slot == 0 || slot > self.reference_count {
                    return Err("Unsupported reference slot or flags".into());
                }
                let cache = if cached {
                    let children = self.sequence(true, true, depth + 1)?;
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
                    cached: cache,
                });
            } else if self.bytes[self.at..].starts_with(b"<S") {
                self.at += 2;
                let style = self.style()?;
                let children = self.sequence(true, in_cache, depth + 1)?;
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
    fn style(&mut self) -> Result<ReferencedStyle, String> {
        let mut out = ReferencedStyle::default();
        let mut count = 0;
        while self.bytes.get(self.at) != Some(&b'<') {
            let attr = *self.bytes.get(self.at).ok_or("Truncated rich-text style")?;
            self.at += 1;
            let value = u16::try_from(self.hex()?).map_err(|_| "Rich-text style value overflow")?;
            match attr {
                b'#' if (8..=512).contains(&value) && out.font.size_code.is_none() => {
                    out.font.size_code = Some(value)
                }
                b'P' if out.font.family_index.is_none() => out.font.family_index = Some(value),
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
            count += 1;
        }
        if count == 0 {
            return Err("Empty rich-text style attributes".into());
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
            "<?xA0001>",
            "<?1xE0001<T1x0>>",
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
            "<SO2<T1xx>>",
            "<VL<-<T1x0><?1x1<H<T1x1>>>>>",
            "<H<?x1><!70>>",
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
    }
}
