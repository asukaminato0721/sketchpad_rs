//! Encoder for an original Sketchpad 5 free-text object's literal UTF-8 run.
//!
//! The Text tool stores the byte length twice: as hexadecimal text after
//! `<T`, and as the 0x08fc markup byte count. The 0x08fc inner size is rounded
//! to two bytes; its record payload is rounded to four bytes.

const OBJECT_HEADER: [u8; 28] = [
    0, 0, 0, 0, 0, 0, 0, 0, 19, 0, 33, 1, 0, 0, 0, 0, 255, 255, 57, 32, 193, 189, 240, 255, 0, 0,
    0, 0,
];

/// Native symbol-font tokens, shared by literal and live captions.
pub(crate) fn markup_symbol(code: usize) -> Option<&'static str> {
    Some(match code {
        0x5c => "∴",
        0x5e => "⊥",
        0x70 => "π",
        0x71 => "θ",
        0x7e => "∼",
        0xa3 => "≤",
        0xa5 => "∞",
        0xae => "→",
        0xb0 => "°",
        0xb1 => "±",
        0xb7 => "•",
        0xb8 => "÷",
        0xbb => "≈",
        0xd0 => "∠",
        0xd7 => "*",
        0xde => "⇒",
        0xf2 => "∫",
        0x100 => "⊙",
        0x102 => "△",
        0x103 => "−",
        _ => return None,
    })
}

/// Native Sketchpad files normally store UTF-8 markup, while a small legacy
/// Chinese cohort stores complete literal runs in Windows code page 936.
pub(crate) fn decode_markup_literal(bytes: &[u8], legacy_gbk: bool) -> Option<String> {
    if legacy_gbk {
        encoding_rs::GBK
            .decode_without_bom_handling_and_without_replacement(bytes)
            .map(std::borrow::Cow::into_owned)
    } else {
        std::str::from_utf8(bytes).map(str::to_owned).ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RichRunStyle {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub size_code: Option<u16>,
    pub family_index: Option<u16>,
    /// Native R/G/L channels, each encoded as one plus an 8-bit component.
    pub rgb: Option<[u16; 3]>,
    pub superscript_depth: u8,
    /// Explicit native `+` script-size level, independent of baseline placement.
    pub script_size: Option<u8>,
}

impl RichRunStyle {
    pub fn script_metrics(self, base_size: f32) -> (f32, f32) {
        let level = i32::from(self.script_size.unwrap_or(self.superscript_depth));
        (
            base_size * 0.75_f32.powi(level),
            base_size * 0.35 * f32::from(self.superscript_depth),
        )
    }
    pub fn rgb_color(self) -> Option<[u8; 3]> {
        self.rgb
            .filter(|channels| channels.iter().all(|value| (1..=256).contains(value)))
            .map(|channels| channels.map(|value| (value - 1) as u8))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichTextRun {
    pub text: String,
    pub style: RichRunStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichTextTemplate {
    rich: Vec<u8>,
    bbox: [u8; 8],
}

impl RichTextTemplate {
    pub fn from_parts(rich: &[u8], bbox: &[u8]) -> Result<Self, String> {
        decode_rich_text(rich, bbox)?;
        Ok(Self {
            rich: rich.to_vec(),
            bbox: bbox.try_into().unwrap(),
        })
    }

    pub fn math_nodes(&self) -> Option<Vec<crate::document::referenced_text::ReferencedNode>> {
        let text =
            crate::document::referenced_text::ReferencedText::from_rich(&self.rich, 0).ok()?;
        text.nodes()
            .iter()
            .any(crate::document::referenced_text::ReferencedNode::has_math_layout)
            .then(|| text.nodes().to_vec())
    }

    pub fn decode(&self) -> Result<(String, TextRect, Vec<Vec<RichTextRun>>), String> {
        decode_rich_text(&self.rich, &self.bbox)
    }

    pub fn encode_parts(&self, text: &str, rect: TextRect) -> Result<(Vec<u8>, [u8; 8]), String> {
        encode_from_template(&self.rich, &self.bbox, text, rect)
    }

    pub fn encode_group(&self, text: &str, rect: TextRect) -> Result<Vec<u8>, String> {
        let (rich, bbox) = self.encode_parts(text, rect)?;
        let mut group = Vec::new();
        append_record(&mut group, 0x07d0, &OBJECT_HEADER)?;
        append_record(&mut group, 0x08fc, &rich)?;
        append_record(&mut group, 0x07d8, &[0, 0])?;
        append_record(&mut group, 0x0898, &bbox)?;
        append_record(&mut group, 0x07d7, &[])?;
        Ok(group)
    }
}

pub fn encode_rich_runs(
    rows: &[Vec<RichTextRun>],
    rect: TextRect,
) -> Result<RichTextTemplate, String> {
    if rows.is_empty() || rows.iter().any(Vec::is_empty) {
        return Err("Text must have at least one run per line".into());
    }
    rect.validate()?;
    let mut markup = b"<VL".to_vec();
    for row in rows {
        let horizontal = row.len() > 1 || row[0].style != RichRunStyle::default();
        if horizontal {
            markup.extend_from_slice(b"<H");
        }
        for run in row {
            let font_style = RichRunStyle {
                superscript_depth: 0,
                ..run.style
            };
            for _ in 0..run.style.superscript_depth {
                markup.extend_from_slice(b"<+<0>");
            }
            if font_style != RichRunStyle::default() {
                markup.extend_from_slice(b"<S");
                if let Some(size) = run.style.size_code {
                    if !(8..=512).contains(&size) {
                        return Err("Unsupported rich-text size".into());
                    }
                    markup.extend_from_slice(format!("#{size:X}").as_bytes());
                }
                if let Some(index) = run.style.family_index {
                    markup.extend_from_slice(format!("P{index:X}").as_bytes());
                }
                if let Some(level) = run.style.script_size {
                    markup.extend_from_slice(format!("+{level:X}").as_bytes());
                }
                if let Some([red, green, blue]) = run.style.rgb {
                    if ![red, green, blue]
                        .iter()
                        .all(|channel| (1..=256).contains(channel))
                    {
                        return Err("Unsupported rich-text color channel".into());
                    }
                    markup.extend_from_slice(format!("R{red:X}G{green:X}L{blue:X}").as_bytes());
                }
                for (attribute, value) in [
                    (b'W', run.style.bold),
                    (b'S', run.style.italic),
                    (b'U', run.style.underline),
                ] {
                    if let Some(value) = value {
                        markup.push(attribute);
                        markup.push(if value { b'2' } else { b'1' });
                    }
                }
            }
            if run.text.as_bytes().contains(&0) {
                return Err("Text contains a NUL byte".into());
            }
            markup.extend_from_slice(format!("<T{:X}x", run.text.len()).as_bytes());
            markup.extend_from_slice(run.text.as_bytes());
            markup.push(b'>');
            if font_style != RichRunStyle::default() {
                markup.push(b'>');
            }
            markup.extend(std::iter::repeat_n(
                b'>',
                usize::from(run.style.superscript_depth),
            ));
        }
        if horizontal {
            markup.push(b'>');
        }
    }
    markup.push(b'>');
    let rich = encode_rich_markup(&markup, rect)?;
    let mut bbox = [0; 8];
    for (index, coordinate) in [rect.left, rect.top, rect.right, rect.bottom]
        .into_iter()
        .enumerate()
    {
        bbox[index * 2..index * 2 + 2].copy_from_slice(&coordinate.to_le_bytes());
    }
    RichTextTemplate::from_parts(&rich, &bbox)
}

/// Carry character styles through a plain-text edit. Equal characters retain
/// their styles; inserted text follows the style of the replaced range, or the
/// character on its left for a pure insertion.
pub fn remap_rich_text(
    rows: &[Vec<RichTextRun>],
    new_text: &str,
    rect: TextRect,
) -> Result<RichTextTemplate, String> {
    let normalized = normalize_line_breaks(new_text);
    if normalized.is_empty() {
        return Err("Text object cannot be empty".into());
    }
    let mut old_chars = Vec::new();
    let mut old_styles = Vec::new();
    for (line_index, row) in rows.iter().enumerate() {
        if line_index > 0 {
            old_chars.push('\n');
            old_styles.push(RichRunStyle::default());
        }
        for run in row {
            for ch in run.text.chars() {
                old_chars.push(ch);
                old_styles.push(run.style);
            }
        }
    }
    let new_chars = normalized.chars().collect::<Vec<_>>();
    let old_len = old_chars.len();
    let new_len = new_chars.len();
    let mut matched = vec![None; new_len];
    let mut prefix = 0;
    while prefix < old_len && prefix < new_len && old_chars[prefix] == new_chars[prefix] {
        matched[prefix] = Some(prefix);
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < old_len - prefix
        && suffix < new_len - prefix
        && old_chars[old_len - suffix - 1] == new_chars[new_len - suffix - 1]
    {
        matched[new_len - suffix - 1] = Some(old_len - suffix - 1);
        suffix += 1;
    }
    let old_mid = old_len - prefix - suffix;
    let new_mid = new_len - prefix - suffix;
    if old_mid > 0 && new_mid > 0 && old_mid.saturating_mul(new_mid) <= 4_000_000 {
        let width = new_mid + 1;
        let mut lcs = vec![0u32; (old_mid + 1) * width];
        for i in (0..old_mid).rev() {
            for j in (0..new_mid).rev() {
                lcs[i * width + j] = if old_chars[prefix + i] == new_chars[prefix + j] {
                    1 + lcs[(i + 1) * width + j + 1]
                } else {
                    lcs[(i + 1) * width + j].max(lcs[i * width + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < old_mid && j < new_mid {
            if old_chars[prefix + i] == new_chars[prefix + j] {
                matched[prefix + j] = Some(prefix + i);
                i += 1;
                j += 1;
            } else if lcs[(i + 1) * width + j] >= lcs[i * width + j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    let style_near = |index: usize| {
        if old_chars.get(index) == Some(&'\n') {
            old_styles
                .get(..index)
                .and_then(|styles| styles.last().copied())
                .or_else(|| old_styles.get(index + 1).copied())
                .unwrap_or_default()
        } else {
            old_styles.get(index).copied().unwrap_or_default()
        }
    };
    let mut new_styles = vec![RichRunStyle::default(); new_len];
    for (index, source) in matched.iter().enumerate() {
        if let Some(source) = source {
            new_styles[index] = style_near(*source);
        }
    }
    let mut at = 0;
    while at < new_len {
        if matched[at].is_some() {
            at += 1;
            continue;
        }
        let start = at;
        while at < new_len && matched[at].is_none() {
            at += 1;
        }
        let previous = (start > 0).then(|| matched[start - 1]).flatten();
        let next = matched.get(at).copied().flatten();
        let deleted_start = previous.map_or(0, |index| index + 1);
        let deleted_end = next.unwrap_or(old_len);
        let inherited = if deleted_start < deleted_end {
            style_near(deleted_start)
        } else if let Some(previous) = previous {
            style_near(previous)
        } else if let Some(next) = next {
            style_near(next)
        } else {
            RichRunStyle::default()
        };
        new_styles[start..at].fill(inherited);
    }
    let mut result = Vec::new();
    let mut row = Vec::<RichTextRun>::new();
    for (ch, style) in new_chars.into_iter().zip(new_styles) {
        if ch == '\n' {
            if row.is_empty() {
                row.push(RichTextRun {
                    text: String::new(),
                    style: RichRunStyle::default(),
                });
            }
            result.push(std::mem::take(&mut row));
        } else if let Some(last) = row.last_mut().filter(|run| run.style == style) {
            last.text.push(ch);
        } else {
            row.push(RichTextRun {
                text: ch.to_string(),
                style,
            });
        }
    }
    if row.is_empty() {
        row.push(RichTextRun {
            text: String::new(),
            style: RichRunStyle::default(),
        });
    }
    result.push(row);
    encode_rich_runs(&result, rect)
}

struct MarkupParser<'a> {
    markup: &'a [u8],
    pos: usize,
    depth: usize,
    legacy_gbk: bool,
}

impl MarkupParser<'_> {
    fn sequence(
        &mut self,
        vertical: bool,
        style: RichRunStyle,
    ) -> Result<Vec<Vec<RichTextRun>>, String> {
        if self.depth > 64 {
            return Err("Text markup is nested too deeply".into());
        }
        let mut lines = Vec::new();
        let mut row = Vec::new();
        while self.pos < self.markup.len() && self.markup[self.pos] != b'>' {
            let child = self.node(style)?;
            if vertical {
                lines.extend(child);
            } else {
                if child.len() != 1 {
                    return Err("Multiline text inside a horizontal run".into());
                }
                row.extend(child.into_iter().next().unwrap());
            }
        }
        if vertical {
            if lines.is_empty() {
                return Err("Empty text markup".into());
            }
            Ok(lines)
        } else if row.is_empty() {
            Err("Empty horizontal text run".into())
        } else {
            Ok(vec![row])
        }
    }

    fn group(
        &mut self,
        vertical: bool,
        style: RichRunStyle,
    ) -> Result<Vec<Vec<RichTextRun>>, String> {
        self.depth += 1;
        let lines = self.sequence(vertical, style)?;
        self.depth -= 1;
        if self.markup.get(self.pos) != Some(&b'>') {
            return Err("Unterminated rich-text group".into());
        }
        self.pos += 1;
        Ok(lines)
    }

    fn node(&mut self, mut style: RichRunStyle) -> Result<Vec<Vec<RichTextRun>>, String> {
        if self.depth > 64 {
            return Err("Text markup is nested too deeply".into());
        }
        let rest = &self.markup[self.pos..];
        if rest.starts_with(b"<0>") {
            self.pos += 3;
            return Ok(vec![vec![RichTextRun {
                text: String::new(),
                style,
            }]]);
        }
        if rest.starts_with(b"<+") {
            self.pos += 2;
            self.depth += 1;
            let mut base = self.node(style)?;
            style.superscript_depth += 1;
            let exponent = self.node(style)?;
            if self.markup.get(self.pos) != Some(&b'>') {
                return Err("Unterminated superscript".into());
            }
            self.pos += 1;
            self.depth -= 1;
            base.last_mut()
                .unwrap()
                .extend(exponent.into_iter().flatten());
            return Ok(base);
        }
        if rest.starts_with(b"<!") {
            let end = rest
                .iter()
                .position(|byte| *byte == b'>')
                .ok_or("Unterminated rich-text symbol")?;
            let code = usize::from_str_radix(
                std::str::from_utf8(&rest[2..end]).map_err(|_| "Invalid rich-text symbol")?,
                16,
            )
            .map_err(|_| "Invalid rich-text symbol")?;
            let text = markup_symbol(code).ok_or("Unsupported rich-text symbol")?;
            self.pos += end + 1;
            return Ok(vec![vec![RichTextRun {
                text: text.into(),
                style,
            }]]);
        }
        if rest.starts_with(b"<T") {
            self.pos += 2;
            let start = self.pos;
            while self.markup.get(self.pos).is_some_and(u8::is_ascii_hexdigit) {
                self.pos += 1;
            }
            if self.pos == start || self.markup.get(self.pos) != Some(&b'x') {
                return Err("Invalid rich-text run length".into());
            }
            let hex = std::str::from_utf8(&self.markup[start..self.pos])
                .map_err(|error| error.to_string())?;
            let len = usize::from_str_radix(hex, 16).map_err(|error| error.to_string())?;
            self.pos += 1;
            let end = self
                .pos
                .checked_add(len)
                .ok_or("Text run length overflow")?;
            if self.markup.get(end) != Some(&b'>') {
                return Err("Invalid rich-text run boundary".into());
            }
            let bytes = &self.markup[self.pos..end];
            if bytes.contains(&0) {
                return Err("Text contains a NUL byte".into());
            }
            let text = decode_markup_literal(bytes, self.legacy_gbk)
                .ok_or("Invalid rich-text literal encoding")?;
            self.pos = end + 1;
            return Ok(vec![vec![RichTextRun { text, style }]]);
        }
        if rest.starts_with(b"<VL") {
            self.pos += 3;
            return self.group(true, style);
        }
        if rest.starts_with(b"<H") {
            self.pos += 2;
            return self.group(false, style);
        }
        if rest.starts_with(b"<S") {
            self.pos += 2;
            let mut attributes = 0;
            let mut channels = [None; 3];
            let mut next_channel = 0;
            while self.markup.get(self.pos) != Some(&b'<') {
                let attribute = *self
                    .markup
                    .get(self.pos)
                    .ok_or("Unterminated rich-text style")?;
                if matches!(attribute, b'#' | b'P' | b'+' | b'R' | b'G' | b'L') {
                    if next_channel > 0 && !matches!(attribute, b'G' | b'L') {
                        return Err("Unsupported style after rich-text color channels".into());
                    }
                    self.pos += 1;
                    let start = self.pos;
                    while self.markup.get(self.pos).is_some_and(u8::is_ascii_hexdigit) {
                        self.pos += 1;
                    }
                    if start == self.pos {
                        return Err("Missing rich-text numeric style value".into());
                    }
                    let hex = std::str::from_utf8(&self.markup[start..self.pos])
                        .map_err(|error| error.to_string())?;
                    let value = u16::from_str_radix(hex, 16)
                        .map_err(|_| "Invalid rich-text numeric style value")?;
                    match attribute {
                        b'#' if (8..=512).contains(&value) => style.size_code = Some(value),
                        b'P' => style.family_index = Some(value),
                        b'+' => {
                            style.script_size =
                                Some(u8::try_from(value).map_err(|_| "Invalid script-size level")?)
                        }
                        b'R' | b'G' | b'L' if (1..=256).contains(&value) => {
                            let index = match attribute {
                                b'R' => 0,
                                b'G' => 1,
                                _ => 2,
                            };
                            if index != next_channel {
                                return Err("Unsupported rich-text color channel order".into());
                            }
                            channels[index] = Some(value);
                            next_channel += 1;
                        }
                        _ => return Err("Unsupported rich-text numeric style value".into()),
                    }
                    attributes += 1;
                    continue;
                }
                let enabled = match self.markup.get(self.pos + 1) {
                    Some(b'1') => false,
                    Some(b'2') => true,
                    _ => return Err("Unsupported rich-text style value".into()),
                };
                if next_channel > 0 {
                    return Err("Unsupported style after rich-text color channels".into());
                }
                match attribute {
                    b'W' => style.bold = Some(enabled),
                    b'S' => style.italic = Some(enabled),
                    b'U' => style.underline = Some(enabled),
                    _ => return Err("Unsupported rich-text style attribute".into()),
                }
                attributes += 1;
                self.pos += 2;
            }
            if attributes == 0 {
                return Err("Empty rich-text style".into());
            }
            if channels.iter().any(Option::is_some) {
                let [Some(red), Some(green), Some(blue)] = channels else {
                    return Err("Incomplete rich-text color".into());
                };
                style.rgb = Some([red, green, blue]);
            }
            return self.group(false, style);
        }
        Err("Unsupported rich-text markup".into())
    }
}

/// Decode literal and mixed-format text, including symbols and superscripts.
pub fn decode_rich_text(
    rich: &[u8],
    bbox: &[u8],
) -> Result<(String, TextRect, Vec<Vec<RichTextRun>>), String> {
    if rich.len() < 20
        || u32::from_le_bytes(rich[..4].try_into().unwrap()) != 0x08fc
        || u32::from_le_bytes(rich[8..12].try_into().unwrap()) != 4
        || bbox.len() != 8
    {
        return Err("Unsupported free-text record".into());
    }
    let inner = u32::from_le_bytes(rich[4..8].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(rich[16..18].try_into().unwrap()) as usize;
    if inner < 18 + count + 1
        || inner > rich.len()
        || rich.len() - inner > 3
        || rich[18 + count] != 0
    {
        return Err("Unsupported free-text padding".into());
    }
    let markup = &rich[18..18 + count];
    let mut parser = MarkupParser {
        markup,
        pos: 0,
        depth: 0,
        legacy_gbk: std::str::from_utf8(markup).is_err(),
    };
    let lines = match parser.sequence(true, RichRunStyle::default()) {
        Ok(lines) if parser.pos == markup.len() => lines,
        _ => crate::document::referenced_text::ReferencedText::from_rich(rich, 0)?
            .evaluate(|_, _| None)
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|run| {
                        let mut style = run.style.font;
                        if let (Some(r), Some(g), Some(b)) =
                            (run.style.red, run.style.green, run.style.blue)
                        {
                            style.rgb = Some([r, g, b]);
                        }
                        RichTextRun {
                            text: run.text,
                            style,
                        }
                    })
                    .collect()
            })
            .collect(),
    };
    let word = |at| i16::from_le_bytes(bbox[at..at + 2].try_into().unwrap());
    let rect = TextRect {
        left: word(0),
        top: word(2),
        right: word(4),
        bottom: word(6),
    };
    rect.validate()?;
    if i16::from_le_bytes(rich[12..14].try_into().unwrap()) != rect.left
        || i16::from_le_bytes(rich[14..16].try_into().unwrap()) != rect.top
    {
        return Err("Text anchor and box disagree".into());
    }
    let text = lines
        .iter()
        .map(|line| line.iter().map(|run| run.text.as_str()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    Ok((text, rect, lines))
}

pub fn decode_plain_text(rich: &[u8], bbox: &[u8]) -> Result<(String, TextRect), String> {
    let (text, rect, _) = decode_rich_text(rich, bbox)?;
    Ok((text, rect))
}

/// Preserve a saved free-text object's run style while changing its text or
/// rectangle. Sketchpad can leave nonzero padding after a shorter styled run.
pub fn encode_from_template(
    rich: &[u8],
    bbox: &[u8],
    text: &str,
    rect: TextRect,
) -> Result<(Vec<u8>, [u8; 8]), String> {
    let (old_text, _, runs) = decode_rich_text(rich, bbox)?;
    rect.validate()?;
    let mut result = if old_text == normalize_line_breaks(text) {
        rich.to_vec()
    } else {
        if runs
            .iter()
            .flatten()
            .any(|run| run.style != RichRunStyle::default())
        {
            return Err("Editing mixed-format text is not yet supported".into());
        }
        encode_rich_payload(text, rect, rich.get(18..21) == Some(b"<VL"))?
    };
    result[12..14].copy_from_slice(&rect.left.to_le_bytes());
    result[14..16].copy_from_slice(&rect.top.to_le_bytes());
    let mut box_bytes = [0; 8];
    for (index, coordinate) in [rect.left, rect.top, rect.right, rect.bottom]
        .into_iter()
        .enumerate()
    {
        box_bytes[index * 2..index * 2 + 2].copy_from_slice(&coordinate.to_le_bytes());
    }
    Ok((result, box_bytes))
}

impl TextRect {
    pub fn roomy_at(left: i16, top: i16, text: &str) -> Result<Self, String> {
        let normalized = normalize_line_breaks(text);
        let mut width: i32 = 8;
        let mut lines: i32 = 0;
        for line in normalized.split('\n') {
            lines += 1;
            let mut line_width = 8_i32;
            for ch in line.chars() {
                line_width = line_width
                    .checked_add(if ch.is_ascii() { 10 } else { 22 })
                    .ok_or("Text box width overflow")?;
            }
            width = width.max(line_width);
        }
        let right =
            i16::try_from(i32::from(left) + width).map_err(|_| "Text box x coordinate overflow")?;
        let height = 23_i32
            .checked_add(
                (lines - 1)
                    .checked_mul(19)
                    .ok_or("Text box height overflow")?,
            )
            .ok_or("Text box height overflow")?;
        let bottom =
            i16::try_from(i32::from(top) + height).map_err(|_| "Text box y coordinate overflow")?;
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    fn validate(self) -> Result<(), String> {
        if self.right <= self.left || self.bottom <= self.top {
            Err("Text rectangle must have positive width and height".into())
        } else {
            Ok(())
        }
    }
}

pub fn encode_plain_text_group(text: &str, rect: TextRect) -> Result<Vec<u8>, String> {
    let rich = encode_rich_payload(text, rect, true)?;
    let mut bbox = Vec::with_capacity(8);
    for coordinate in [rect.left, rect.top, rect.right, rect.bottom] {
        bbox.extend_from_slice(&coordinate.to_le_bytes());
    }

    let mut group = Vec::new();
    append_record(&mut group, 0x07d0, &OBJECT_HEADER)?;
    append_record(&mut group, 0x08fc, &rich)?;
    append_record(&mut group, 0x07d8, &[0, 0])?;
    append_record(&mut group, 0x0898, &bbox)?;
    append_record(&mut group, 0x07d7, &[])?;
    Ok(group)
}

fn encode_rich_payload(text: &str, rect: TextRect, wrapped: bool) -> Result<Vec<u8>, String> {
    if text.is_empty() {
        return Err("Text object cannot be empty".into());
    }
    if text.as_bytes().contains(&0) {
        return Err("Text contains a NUL byte".into());
    }
    rect.validate()?;

    let normalized = normalize_line_breaks(text);
    let mut markup = if wrapped { b"<VL".to_vec() } else { Vec::new() };
    for line in normalized.split('\n') {
        markup.extend_from_slice(format!("<T{:X}x", line.len()).as_bytes());
        markup.extend_from_slice(line.as_bytes());
        markup.push(b'>');
    }
    if wrapped {
        markup.push(b'>');
    }
    encode_rich_markup(&markup, rect)
}

fn encode_rich_markup(markup: &[u8], rect: TextRect) -> Result<Vec<u8>, String> {
    let markup_len = u16::try_from(markup.len()).map_err(|_| "Text markup exceeds 65535 bytes")?;

    let mut rich = Vec::new();
    rich.extend_from_slice(&0x08fc_u32.to_le_bytes());
    rich.extend_from_slice(&0_u32.to_le_bytes()); // 2-byte aligned inner size
    rich.extend_from_slice(&4_u32.to_le_bytes());
    rich.extend_from_slice(&rect.left.to_le_bytes());
    rich.extend_from_slice(&rect.top.to_le_bytes());
    rich.extend_from_slice(&markup_len.to_le_bytes());
    rich.extend_from_slice(markup);
    rich.push(0);
    if !rich.len().is_multiple_of(2) {
        rich.push(0);
    }
    let inner_len = u32::try_from(rich.len()).map_err(|_| "Text payload is too large")?;
    rich[4..8].copy_from_slice(&inner_len.to_le_bytes());
    while !rich.len().is_multiple_of(4) {
        rich.push(0);
    }

    Ok(rich)
}

fn normalize_line_breaks(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Add one free-text group to a verified blank, one-page GSP4 document.
/// Other groups are refused because their ordinal references need model-level
/// validation before a structural rewrite.
pub fn add_to_blank_document(blank: &[u8], text: &str, rect: TextRect) -> Result<Vec<u8>, String> {
    if blank.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut at = 4;
    let mut page_count_offset = None;
    let mut trailer_offset = None;
    let mut page_count = 0;
    while at < blank.len() {
        if at + 8 > blank.len() {
            return Err("Truncated record header".into());
        }
        let len = u32::from_le_bytes(blank[at..at + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(blank[at + 4..at + 8].try_into().unwrap());
        let end = at
            .checked_add(8 + len + len % 2)
            .ok_or("Record length overflow")?;
        if end > blank.len() {
            return Err("Truncated record payload".into());
        }
        match kind {
            0x03e8 => {
                if len < 28 {
                    return Err("Page header is too short".into());
                }
                page_count += 1;
                page_count_offset = Some(at + 8 + 24);
            }
            0x07d0 => return Err("Document is not blank".into()),
            0x03e9 => trailer_offset = Some(at),
            _ => {}
        }
        at = end;
    }
    if page_count != 1 {
        return Err("Expected exactly one page".into());
    }
    let count_at = page_count_offset.unwrap();
    if u32::from_le_bytes(blank[count_at..count_at + 4].try_into().unwrap()) != 0 {
        return Err("Blank page has a nonzero object count".into());
    }
    let trailer = trailer_offset.ok_or("Missing page trailer")?;
    let group = encode_plain_text_group(text, rect)?;
    let mut output = Vec::with_capacity(blank.len() + group.len());
    output.extend_from_slice(&blank[..trailer]);
    output.extend_from_slice(&group);
    output.extend_from_slice(&blank[trailer..]);
    output[count_at..count_at + 4].copy_from_slice(&1_u32.to_le_bytes());
    Ok(output)
}

fn append_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) -> Result<(), String> {
    let len = u32::try_from(payload.len()).map_err(|_| "Record payload is too large")?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(payload);
    if !payload.len().is_multiple_of(2) {
        out.push(0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_payloads(bytes: &[u8]) -> (&[u8], &[u8]) {
        let mut at = 4;
        let mut rich = None;
        let mut bbox = None;
        while at < bytes.len() {
            let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let kind = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
            let payload = &bytes[at + 8..at + 8 + len];
            match kind {
                0x08fc => rich = Some(payload),
                0x0898 => bbox = Some(payload),
                _ => {}
            }
            at += 8 + len + len % 2;
        }
        (rich.unwrap(), bbox.unwrap())
    }

    fn markup(rich: &[u8]) -> &[u8] {
        let len = u16::from_le_bytes(rich[16..18].try_into().unwrap()) as usize;
        &rich[18..18 + len]
    }

    #[test]
    fn colored_run_round_trips_through_safe_move_encoding() {
        let rect = TextRect {
            left: 10,
            top: 20,
            right: 100,
            bottom: 40,
        };
        let style = RichRunStyle {
            rgb: Some([1, 2, 256]),
            ..Default::default()
        };
        let rows = vec![vec![RichTextRun {
            text: "Blue".into(),
            style,
        }]];
        let template = encode_rich_runs(&rows, rect).unwrap();
        assert!(
            markup(&template.rich)
                .windows(b"R1G2L100".len())
                .any(|window| window == b"R1G2L100")
        );
        assert_eq!(template.decode().unwrap().2, rows);
        let moved = TextRect {
            left: 50,
            right: 140,
            ..rect
        };
        let (edited_rich, edited_bbox) = template.encode_parts("Blue", moved).unwrap();
        let edited = RichTextTemplate::from_parts(&edited_rich, &edited_bbox).unwrap();
        assert_eq!(
            edited.decode().unwrap().2[0][0].style.rgb_color(),
            Some([0, 1, 255])
        );
        assert!(template.encode_parts("Bluer", rect).is_err());
    }

    #[test]
    fn native_mixed_style_runs_decode_without_losing_boundaries() {
        for (source, expected) in [
            (
                include_bytes!("../../../tests/fixtures/text_mixed_bold_off_he_original.gsp")
                    .as_slice(),
                RichRunStyle {
                    bold: Some(false),
                    ..Default::default()
                },
            ),
            (
                include_bytes!("../../../tests/fixtures/text_mixed_italic_he_original.gsp")
                    .as_slice(),
                RichRunStyle {
                    italic: Some(true),
                    ..Default::default()
                },
            ),
            (
                include_bytes!("../../../tests/fixtures/text_mixed_underline_he_original.gsp")
                    .as_slice(),
                RichRunStyle {
                    underline: Some(true),
                    ..Default::default()
                },
            ),
            (
                include_bytes!("../../../tests/fixtures/text_mixed_bold_italic_he_original.gsp")
                    .as_slice(),
                RichRunStyle {
                    bold: Some(false),
                    italic: Some(true),
                    ..Default::default()
                },
            ),
            (
                include_bytes!("../../../tests/fixtures/text_mixed_size36_he_original.gsp")
                    .as_slice(),
                RichRunStyle {
                    size_code: Some(36),
                    ..Default::default()
                },
            ),
            (
                include_bytes!("../../../tests/fixtures/text_mixed_adventor_he_original.gsp")
                    .as_slice(),
                RichRunStyle {
                    family_index: Some(1),
                    ..Default::default()
                },
            ),
        ] {
            let (rich, bbox) = text_payloads(source);
            let (text, rect, rows) = decode_rich_text(rich, bbox).unwrap();
            assert_eq!(text, "Hello");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[0][0].text, "He");
            assert_eq!(rows[0][0].style, expected);
            assert_eq!(rows[0][1].text, "llo");
            assert_eq!(rows[0][1].style, RichRunStyle::default());
            let moved = TextRect {
                left: rect.left + 10,
                top: rect.top + 10,
                right: rect.right + 10,
                bottom: rect.bottom + 10,
            };
            let (new_rich, new_bbox) = encode_from_template(rich, bbox, "Hello", moved).unwrap();
            let (_, _, moved_rows) = decode_rich_text(&new_rich, &new_bbox).unwrap();
            assert_eq!(moved_rows, rows);
            assert!(encode_from_template(rich, bbox, "Goodbye", rect).is_err());
            let rebuilt = encode_rich_runs(&rows, rect).unwrap();
            assert_eq!(rebuilt.decode().unwrap().2, rows);
        }
    }

    #[test]
    fn mixed_style_edit_inherits_native_boundary_styles() {
        let source =
            include_bytes!("../../../tests/fixtures/text_mixed_bold_italic_he_original.gsp");
        let (rich, bbox) = text_payloads(source);
        let (_, rect, rows) = decode_rich_text(rich, bbox).unwrap();
        let inserted = remap_rich_text(&rows, "Hexllo", rect).unwrap();
        let (_, _, inserted_rows) = inserted.decode().unwrap();
        assert_eq!(inserted_rows[0][0].text, "Hex");
        assert_eq!(inserted_rows[0][0].style, rows[0][0].style);
        assert_eq!(inserted_rows[0][1].text, "llo");
        assert_eq!(inserted_rows[0][1].style, rows[0][1].style);
        let (native_insert, _) = text_payloads(include_bytes!(
            "../../../tests/fixtures/text_mixed_insert_boundary_original.gsp"
        ));
        assert_eq!(markup(&inserted.rich), markup(native_insert));

        let replaced = remap_rich_text(&rows, "Hexlo", rect).unwrap();
        let (_, _, replaced_rows) = replaced.decode().unwrap();
        assert_eq!(replaced_rows[0][0].text, "He");
        assert_eq!(replaced_rows[0][1].text, "xlo");
        assert_eq!(replaced_rows[0][1].style, rows[0][1].style);
        let (native_replace, _) = text_payloads(include_bytes!(
            "../../../tests/fixtures/text_mixed_replace_boundary_original.gsp"
        ));
        assert_eq!(markup(&replaced.rich), markup(native_replace));

        let deleted = remap_rich_text(&rows, "llo", rect).unwrap();
        let (_, _, deleted_rows) = deleted.decode().unwrap();
        assert_eq!(deleted_rows[0][0].text, "llo");
        assert_eq!(deleted_rows[0][0].style, RichRunStyle::default());

        let unicode = remap_rich_text(&rows, "H猫ello", rect).unwrap();
        let (_, _, unicode_rows) = unicode.decode().unwrap();
        assert_eq!(unicode_rows[0][0].text, "H猫e");
        assert_eq!(unicode_rows[0][0].style, rows[0][0].style);

        let separated = remap_rich_text(&rows, "XelloY", rect).unwrap();
        let (_, _, separated_rows) = separated.decode().unwrap();
        assert_eq!(separated_rows[0][0].text, "Xe");
        assert_eq!(separated_rows[0][0].style, rows[0][0].style);
        assert_eq!(separated_rows[0][1].text, "lloY");
        assert_eq!(separated_rows[0][1].style, rows[0][1].style);

        let multiline = remap_rich_text(&rows, "He\nllo", rect).unwrap();
        let (_, _, multiline_rows) = multiline.decode().unwrap();
        assert_eq!(multiline_rows.len(), 2);
        assert_eq!(multiline_rows[0][0].text, "He");
        assert_eq!(multiline_rows[0][0].style, rows[0][0].style);
        assert_eq!(multiline_rows[1][0].text, "llo");
        assert_eq!(multiline_rows[1][0].style, rows[0][1].style);
    }

    fn object_group(bytes: &[u8]) -> &[u8] {
        let start = bytes
            .windows(8)
            .position(|w| w == [28, 0, 0, 0, 0xd0, 7, 0, 0])
            .unwrap();
        let end_rel = bytes[start..]
            .windows(8)
            .position(|w| w == [0, 0, 0, 0, 0xd7, 7, 0, 0])
            .unwrap();
        &bytes[start..start + end_rel + 8]
    }

    #[test]
    fn generated_a_group_matches_original_except_opaque_padding() {
        let original = include_bytes!("../../../tests/fixtures/text_object_original_A.gsp");
        let generated = encode_plain_text_group(
            "A",
            TextRect {
                left: 91,
                top: 81,
                right: 109,
                bottom: 104,
            },
        )
        .unwrap();
        assert_eq!(&generated, object_group(original));
    }

    #[test]
    fn generated_hello_group_matches_original_except_opaque_padding() {
        let original = include_bytes!("../../../tests/fixtures/text_object_original_Hello.gsp");
        let generated = encode_plain_text_group(
            "Hello",
            TextRect {
                left: 91,
                top: 81,
                right: 137,
                bottom: 104,
            },
        )
        .unwrap();
        assert_eq!(&generated, object_group(original));
    }

    #[test]
    fn generated_multiline_group_matches_original() {
        let original = include_bytes!("../../../tests/fixtures/text_object_original_multiline.gsp");
        let generated = encode_plain_text_group(
            "A\nB",
            TextRect {
                left: 91,
                top: 81,
                right: 109,
                bottom: 123,
            },
        )
        .unwrap();
        assert_eq!(&generated, object_group(original));
    }

    #[test]
    fn length_prefix_counts_utf8_bytes_and_uses_hex() {
        let group =
            encode_plain_text_group("猫", TextRect::roomy_at(91, 81, "猫").unwrap()).unwrap();
        assert!(
            group
                .windows(b"<VL<T3x\xe7\x8c\xab>>".len())
                .any(|w| w == b"<VL<T3x\xe7\x8c\xab>>")
        );
        let group = encode_plain_text_group(
            "1234567890ABCDEF",
            TextRect::roomy_at(91, 81, "1234567890ABCDEF").unwrap(),
        )
        .unwrap();
        assert!(
            group
                .windows(b"<VL<T10x1234567890ABCDEF>>".len())
                .any(|w| w == b"<VL<T10x1234567890ABCDEF>>")
        );
    }

    #[test]
    fn common_line_endings_encode_as_the_same_multiline_object() {
        let rect = TextRect::roomy_at(91, 81, "A\nB").unwrap();
        let expected = encode_plain_text_group("A\nB", rect).unwrap();
        assert_eq!(encode_plain_text_group("A\r\nB", rect).unwrap(), expected);
        assert_eq!(encode_plain_text_group("A\rB", rect).unwrap(), expected);
        assert_eq!(TextRect::roomy_at(91, 81, "A\r\nB").unwrap(), rect);
    }

    #[test]
    fn blank_document_gain_is_one_group_and_one_object_count() {
        let blank = include_bytes!("../../../tests/fixtures/text_object_blank.gsp");
        let out =
            add_to_blank_document(blank, "Hello", TextRect::roomy_at(91, 81, "Hello").unwrap())
                .unwrap();
        assert_eq!(
            out.len(),
            blank.len()
                + encode_plain_text_group("Hello", TextRect::roomy_at(91, 81, "Hello").unwrap())
                    .unwrap()
                    .len()
        );
        assert!(
            add_to_blank_document(&out, "Again", TextRect::roomy_at(91, 81, "Again").unwrap())
                .is_err()
        );
    }
}
