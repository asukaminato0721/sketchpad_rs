//! Arithmetic and trigonometric function expressions observed in Sketchpad 5 files.

const MAX_EXPRESSION_WORDS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimpleFunction {
    Identity,
    Square,
    Arithmetic {
        words: [u16; MAX_EXPRESSION_WORDS],
        len: u8,
    },
}

impl SimpleFunction {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut words = Vec::new();
        let normalized = input.trim().replace('²', "^2").to_lowercase();
        let chars = normalized.chars().collect::<Vec<_>>();
        let mut index = 0;
        let mut function_open = false;
        let mut parentheses = Vec::new();
        while index < chars.len() {
            if chars[index..].starts_with(&['s', 'i', 'n']) {
                words.push(0x2000);
                function_open = true;
                index += 3;
                continue;
            }
            if chars[index..].starts_with(&['c', 'o', 's']) {
                words.push(0x2001);
                function_open = true;
                index += 3;
                continue;
            }
            if chars[index..].starts_with(&['t', 'a', 'n']) {
                words.push(0x2002);
                function_open = true;
                index += 3;
                continue;
            }
            let c = chars[index];
            let word = match c {
                '0'..='9' => c as u16 - '0' as u16,
                '.' => 0x000a,
                '(' => {
                    parentheses.push(function_open);
                    function_open = false;
                    0x000b
                }
                ')' => 0x000c,
                'x' => 0x000f,
                '+' => 0x1000,
                '-' | '−' => 0x1001,
                '*' | '×' => 0x1002,
                '/' | '÷' => 0x1003,
                '^' => 0x1004,
                c if c.is_whitespace() => {
                    index += 1;
                    continue;
                }
                _ => return Err("Unsupported function character".into()),
            };
            words.push(word);
            if c == ')' && parentheses.pop().unwrap_or(false) {
                words.push(0x000c);
            }
            index += 1;
        }
        Self::from_words(&words)
    }
    fn from_words(words: &[u16]) -> Result<Self, String> {
        if words.is_empty() || words.len() > MAX_EXPRESSION_WORDS {
            return Err("Invalid function expression length".into());
        }
        let mut parser = ArithmeticParser::new(words, 0.0, true);
        parser.expression(0)?;
        if parser.index != words.len() {
            return Err("Invalid function expression".into());
        }
        if words == [0x000f] {
            return Ok(Self::Identity);
        }
        if words == [0x000f, 0x1004, 2] {
            return Ok(Self::Square);
        }
        let mut stored = [0u16; MAX_EXPRESSION_WORDS];
        stored[..words.len()].copy_from_slice(words);
        Ok(Self::Arithmetic {
            words: stored,
            len: words.len() as u8,
        })
    }
    fn words(&self) -> &[u16] {
        match self {
            Self::Identity => &[0x000f],
            Self::Square => &[0x000f, 0x1004, 2],
            Self::Arithmetic { words, len } => &words[..*len as usize],
        }
    }
    pub fn display(self) -> String {
        if self == Self::Square {
            return "f(x) = x²".into();
        }
        let mut text = String::from("f(x) = ");
        let mut parentheses = Vec::new();
        let mut function_open = false;
        let mut skip_close = false;
        for word in self.words() {
            if skip_close {
                skip_close = false;
                if *word == 0x000c {
                    continue;
                }
            }
            if matches!(*word, 0x2000..=0x2002) {
                function_open = true;
            } else if *word == 0x000b {
                parentheses.push(function_open);
                function_open = false;
            } else if *word == 0x000c && parentheses.pop().unwrap_or(false) {
                skip_close = true;
            }
            text.push_str(match word {
                0..=9 => match word {
                    0 => "0",
                    1 => "1",
                    2 => "2",
                    3 => "3",
                    4 => "4",
                    5 => "5",
                    6 => "6",
                    7 => "7",
                    8 => "8",
                    _ => "9",
                },
                0x000a => ".",
                0x000b => "(",
                0x000c => ")",
                0x000f => "x",
                0x1000 => " + ",
                0x1001 => " − ",
                0x1002 => " × ",
                0x1003 => " ÷ ",
                0x1004 => "^",
                0x2000 => "sin",
                0x2001 => "cos",
                0x2002 => "tan",
                _ => "?",
            });
        }
        text
    }
    pub fn evaluate(self, x: f64) -> f64 {
        self.evaluate_with_angle_unit(x, true)
    }
    pub fn evaluate_with_angle_unit(self, x: f64, radians: bool) -> f64 {
        match self {
            Self::Identity => x,
            Self::Square => x * x,
            Self::Arithmetic { .. } => ArithmeticParser::new(self.words(), x, radians)
                .expression(0)
                .unwrap_or(f64::NAN),
        }
    }
    pub fn uses_trigonometry(self) -> bool {
        self.words()
            .iter()
            .any(|word| matches!(*word, 0x2000..=0x2002))
    }
    pub fn may_have_poles(self) -> bool {
        self.words().iter().enumerate().any(|(index, word)| {
            *word == 0x1003
                || *word == 0x2002
                || (*word == 0x1004 && self.words().get(index + 1) == Some(&0x1001))
        })
    }
    pub fn symbolic_derivative(self) -> Result<Self, String> {
        let mut parser = SymbolicParser {
            words: self.words(),
            index: 0,
        };
        let expression = parser.expression(0)?;
        if parser.index != parser.words.len() {
            return Err("Invalid symbolic function expression".into());
        }
        Self::parse(&expression.derivative()?.render(0))
    }
}

struct ArithmeticParser<'a> {
    words: &'a [u16],
    index: usize,
    x: f64,
    radians: bool,
}

impl<'a> ArithmeticParser<'a> {
    fn new(words: &'a [u16], x: f64, radians: bool) -> Self {
        Self {
            words,
            index: 0,
            x,
            radians,
        }
    }
    fn expression(&mut self, min_binding: u8) -> Result<f64, String> {
        let word = *self
            .words
            .get(self.index)
            .ok_or("Incomplete function expression")?;
        self.index += 1;
        let mut left = match word {
            0x1000 => self.expression(6)?,
            0x1001 => -self.expression(6)?,
            0x000b => {
                let value = self.expression(0)?;
                if self.words.get(self.index) != Some(&0x000c) {
                    return Err("Unclosed function parentheses".into());
                }
                self.index += 1;
                value
            }
            0x000f => self.x,
            0x2000..=0x2002 => {
                if self.words.get(self.index) != Some(&0x000b) {
                    return Err("Missing function argument".into());
                }
                self.index += 1;
                let argument = self.expression(0)?;
                if self.words.get(self.index..self.index + 2) != Some(&[0x000c, 0x000c][..]) {
                    return Err("Unclosed function argument".into());
                }
                self.index += 2;
                let angle = if self.radians {
                    argument
                } else {
                    argument.to_radians()
                };
                match word {
                    0x2000 => angle.sin(),
                    0x2001 => angle.cos(),
                    _ => angle.tan(),
                }
            }
            0..=9 | 0x000a => {
                self.index -= 1;
                self.number()?
            }
            _ => return Err("Invalid function operand".into()),
        };
        while let Some(&op) = self.words.get(self.index) {
            let (left_binding, right_binding) = match op {
                0x1000 | 0x1001 => (1, 2),
                0x1002 | 0x1003 => (3, 4),
                0x1004 => (7, 7),
                _ => break,
            };
            if left_binding < min_binding {
                break;
            }
            self.index += 1;
            let right = self.expression(right_binding)?;
            left = match op {
                0x1000 => left + right,
                0x1001 => left - right,
                0x1002 => left * right,
                0x1003 => left / right,
                0x1004 => left.powf(right),
                _ => unreachable!(),
            };
        }
        Ok(left)
    }
    fn number(&mut self) -> Result<f64, String> {
        let mut text = String::new();
        let mut digits = 0;
        let mut decimal = false;
        while let Some(&word) = self.words.get(self.index) {
            match word {
                0..=9 => {
                    text.push(char::from(b'0' + word as u8));
                    digits += 1;
                }
                0x000a if !decimal => {
                    text.push('.');
                    decimal = true;
                }
                _ => break,
            }
            self.index += 1;
        }
        if digits == 0 || text.ends_with('.') {
            return Err("Invalid function number".into());
        }
        text.parse::<f64>()
            .map_err(|_| "Invalid function number".into())
    }
}

#[derive(Clone, PartialEq)]
enum SymbolicExpr {
    Number(f64),
    Variable,
    Neg(Box<Self>),
    Binary(u16, Box<Self>, Box<Self>),
    Call(u16, Box<Self>),
}

impl SymbolicExpr {
    fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
    fn neg(value: Self) -> Self {
        match value {
            Self::Number(number) => Self::Number(-number),
            Self::Neg(inner) => *inner,
            other => Self::Neg(Box::new(other)),
        }
    }
    fn binary(op: u16, left: Self, right: Self) -> Self {
        let a = left.number();
        let b = right.number();
        match op {
            0x1000 => {
                if a == Some(0.) {
                    return right;
                }
                if b == Some(0.) {
                    return left;
                }
                if left == right {
                    return Self::binary(0x1002, Self::Number(2.), left);
                }
            }
            0x1001 => {
                if b == Some(0.) {
                    return left;
                }
                if a == Some(0.) {
                    return Self::neg(right);
                }
                if left == right {
                    return Self::Number(0.);
                }
            }
            0x1002 => {
                if a == Some(0.) || b == Some(0.) {
                    return Self::Number(0.);
                }
                if a == Some(1.) {
                    return right;
                }
                if b == Some(1.) {
                    return left;
                }
                if a == Some(-1.) {
                    return Self::neg(right);
                }
                if b == Some(-1.) {
                    return Self::neg(left);
                }
            }
            0x1003 => {
                if a == Some(0.) {
                    return Self::Number(0.);
                }
                if b == Some(1.) {
                    return left;
                }
            }
            0x1004 => {
                if b == Some(0.) {
                    return Self::Number(1.);
                }
                if b == Some(1.) {
                    return left;
                }
                if a == Some(1.) {
                    return Self::Number(1.);
                }
            }
            _ => {}
        }
        if let (Some(a), Some(b)) = (a, b) {
            let value = match op {
                0x1000 => a + b,
                0x1001 => a - b,
                0x1002 => a * b,
                0x1003 => a / b,
                0x1004 => a.powf(b),
                _ => f64::NAN,
            };
            if value.is_finite() {
                return Self::Number(value);
            }
        }
        Self::Binary(op, Box::new(left), Box::new(right))
    }
    fn derivative(&self) -> Result<Self, String> {
        Ok(match self {
            Self::Number(_) => Self::Number(0.),
            Self::Variable => Self::Number(1.),
            Self::Neg(value) => Self::neg(value.derivative()?),
            Self::Binary(0x1000, a, b) => Self::binary(0x1000, a.derivative()?, b.derivative()?),
            Self::Binary(0x1001, a, b) => Self::binary(0x1001, a.derivative()?, b.derivative()?),
            Self::Binary(0x1002, a, b) => Self::binary(
                0x1000,
                Self::binary(0x1002, a.derivative()?, (**b).clone()),
                Self::binary(0x1002, (**a).clone(), b.derivative()?),
            ),
            Self::Binary(0x1003, a, b) => Self::binary(
                0x1003,
                Self::binary(
                    0x1001,
                    Self::binary(0x1002, a.derivative()?, (**b).clone()),
                    Self::binary(0x1002, (**a).clone(), b.derivative()?),
                ),
                Self::binary(0x1004, (**b).clone(), Self::Number(2.)),
            ),
            Self::Binary(0x1004, base, exponent) => {
                let Some(power) = exponent.number() else {
                    return Err("Symbolic derivative requires a constant exponent".into());
                };
                Self::binary(
                    0x1002,
                    Self::binary(
                        0x1002,
                        Self::Number(power),
                        Self::binary(0x1004, (**base).clone(), Self::Number(power - 1.)),
                    ),
                    base.derivative()?,
                )
            }
            Self::Call(0x2000, argument) => Self::binary(
                0x1002,
                Self::Call(0x2001, argument.clone()),
                argument.derivative()?,
            ),
            Self::Call(0x2001, argument) => Self::binary(
                0x1002,
                Self::neg(Self::Call(0x2000, argument.clone())),
                argument.derivative()?,
            ),
            Self::Call(0x2002, argument) => Self::binary(
                0x1003,
                argument.derivative()?,
                Self::binary(
                    0x1004,
                    Self::Call(0x2001, argument.clone()),
                    Self::Number(2.),
                ),
            ),
            _ => return Err("Unsupported symbolic derivative".into()),
        })
    }
    fn precedence(&self) -> u8 {
        match self {
            Self::Binary(0x1000 | 0x1001, _, _) => 1,
            Self::Binary(0x1002 | 0x1003, _, _) => 3,
            Self::Binary(0x1004, _, _) => 7,
            Self::Neg(_) => 6,
            _ => 10,
        }
    }
    fn render(&self, parent: u8) -> String {
        let text = match self {
            Self::Number(value) => value.to_string(),
            Self::Variable => "x".into(),
            Self::Neg(value) => format!("-{}", value.render(6)),
            Self::Call(code, value) => {
                let name = match code {
                    0x2000 => "sin",
                    0x2001 => "cos",
                    _ => "tan",
                };
                format!("{name}({})", value.render(0))
            }
            Self::Binary(code, left, right) => {
                let op = match code {
                    0x1000 => "+",
                    0x1001 => "-",
                    0x1002 => "*",
                    0x1003 => "/",
                    _ => "^",
                };
                let precedence = self.precedence();
                let left_parent = if *code == 0x1004 {
                    precedence + 1
                } else {
                    precedence
                };
                let right_parent = if matches!(code, 0x1001 | 0x1003) {
                    precedence + 1
                } else {
                    precedence
                };
                format!(
                    "{}{}{}",
                    left.render(left_parent),
                    op,
                    right.render(right_parent)
                )
            }
        };
        if self.precedence() < parent {
            format!("({text})")
        } else {
            text
        }
    }
}

struct SymbolicParser<'a> {
    words: &'a [u16],
    index: usize,
}

impl<'a> SymbolicParser<'a> {
    fn expression(&mut self, min_binding: u8) -> Result<SymbolicExpr, String> {
        let word = *self
            .words
            .get(self.index)
            .ok_or("Incomplete symbolic expression")?;
        self.index += 1;
        let mut left = match word {
            0x1000 => self.expression(6)?,
            0x1001 => SymbolicExpr::neg(self.expression(6)?),
            0x000b => {
                let value = self.expression(0)?;
                if self.words.get(self.index) != Some(&0x000c) {
                    return Err("Unclosed symbolic parentheses".into());
                }
                self.index += 1;
                value
            }
            0x000f => SymbolicExpr::Variable,
            0x2000..=0x2002 => {
                if self.words.get(self.index) != Some(&0x000b) {
                    return Err("Missing function argument".into());
                }
                self.index += 1;
                let argument = self.expression(0)?;
                if self.words.get(self.index..self.index + 2) != Some(&[0x000c, 0x000c][..]) {
                    return Err("Unclosed function argument".into());
                }
                self.index += 2;
                SymbolicExpr::Call(word, Box::new(argument))
            }
            0..=9 | 0x000a => {
                self.index -= 1;
                let mut text = String::new();
                while let Some(word @ (0..=9 | 0x000a)) = self.words.get(self.index) {
                    text.push(if *word == 0x000a {
                        '.'
                    } else {
                        char::from(b'0' + *word as u8)
                    });
                    self.index += 1;
                }
                SymbolicExpr::Number(text.parse().map_err(|_| "Invalid symbolic number")?)
            }
            _ => return Err("Invalid symbolic operand".into()),
        };
        while let Some(&op) = self.words.get(self.index) {
            let (left_binding, right_binding) = match op {
                0x1000 | 0x1001 => (1, 2),
                0x1002 | 0x1003 => (3, 4),
                0x1004 => (7, 7),
                _ => break,
            };
            if left_binding < min_binding {
                break;
            }
            self.index += 1;
            let right = self.expression(right_binding)?;
            left = SymbolicExpr::binary(op, left, right);
        }
        Ok(left)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FunctionPlot {
    pub expression: SimpleFunction,
    pub definition_id: usize,
    pub coordinate_system_id: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub samples: u32,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    kind: u32,
    start: usize,
    payload_start: usize,
    end: usize,
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn records(bytes: &[u8]) -> Result<Vec<Record>, String> {
    if !bytes.starts_with(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut records = Vec::new();
    let mut start = 4usize;
    while start < bytes.len() {
        if bytes.len() - start < 8 {
            return Err("Truncated record header".into());
        }
        let length = u32_at(bytes, start) as usize;
        let payload_start = start + 8;
        let end = payload_start
            .checked_add(length)
            .and_then(|end| end.checked_add(length & 1))
            .ok_or("Record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated record payload".into());
        }
        records.push(Record {
            kind: u32_at(bytes, start + 4),
            start,
            payload_start,
            end,
        });
        start = end;
    }
    Ok(records)
}

fn payload<'a>(bytes: &'a [u8], record: Record) -> &'a [u8] {
    let length = u32_at(bytes, record.start) as usize;
    &bytes[record.payload_start..record.payload_start + length]
}

pub fn decode_simple_expression(data: &[u8]) -> Result<SimpleFunction, String> {
    // The 0x0907 payload has a 92-byte wrapper followed by little-endian
    // infix tokens. Bytes 48..92 contain session-dependent presentation data.
    if data.len() < 94
        || (data.len() - 92) % 2 != 0
        || (data.len() - 92) / 2 > MAX_EXPRESSION_WORDS
        || &data[..14]
            != &include_bytes!("../tests/fixtures/function_plot_identity.gsp")[0x96c..0x97a]
        || u16::from_le_bytes([data[14], data[15]]) > 4096
        || &data[16..18]
            != &include_bytes!("../tests/fixtures/function_plot_identity.gsp")[0x97c..0x97e]
        || &data[18..22] != b"<0>\0"
        || u32_at(data, 32) != 0x0907
        || u32_at(data, 36) as usize != data.len() - 32
        || u32_at(data, 44) as usize != (data.len() - 92) / 2
    {
        return Err("Unsupported function expression wrapper".into());
    }
    let words = data[92..]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    match SimpleFunction::from_words(&words) {
        Ok(expression) => Ok(expression),
        Err(_) if words.len() == 3 && matches!(words[0], 0x2000..=0x2002) && words[2] == 0x000c => {
            SimpleFunction::from_words(&[words[0], 0x000b, words[1], 0x000c, 0x000c])
        }
        Err(error) => Err(error),
    }
}

pub fn encode_expression_record(
    data: &[u8],
    expression: SimpleFunction,
) -> Result<Vec<u8>, String> {
    decode_simple_expression(data)?;
    let mut payload = data[..92].to_vec();
    payload[36..40].copy_from_slice(&((60 + expression.words().len() * 2) as u32).to_le_bytes());
    payload[44..48].copy_from_slice(&(expression.words().len() as u32).to_le_bytes());
    for word in expression.words() {
        payload.extend_from_slice(&word.to_le_bytes());
    }
    let mut record = Vec::with_capacity(payload.len() + 9);
    record.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    record.extend_from_slice(&0x0907u32.to_le_bytes());
    record.extend_from_slice(&payload);
    if payload.len() % 2 != 0 {
        record.push(0);
    }
    Ok(record)
}

fn group_ranges(bytes: &[u8], records: &[Record]) -> Result<Vec<(usize, usize, u16)>, String> {
    let mut groups = Vec::new();
    let mut open = None;
    for (index, record) in records.iter().enumerate() {
        if record.kind == 0x07d0 {
            if open.is_some() || payload(bytes, *record).len() < 2 {
                return Err("Malformed object group".into());
            }
            open = Some((
                index,
                u16::from_le_bytes(payload(bytes, *record)[..2].try_into().unwrap()),
            ));
        } else if record.kind == 0x07d7 {
            if let Some((start, code)) = open.take() {
                groups.push((start, index, code));
            }
        }
    }
    if open.is_some() {
        return Err("Unterminated object group".into());
    }
    Ok(groups)
}

pub fn decode_minimal_plot(bytes: &[u8]) -> Result<FunctionPlot, String> {
    let records = records(bytes)?;
    let groups = group_ranges(bytes, &records)?;
    let mut found = None;
    for (ordinal, &(start, end, code)) in groups.iter().enumerate() {
        if code != 72 {
            continue;
        }
        if found.is_some() {
            return Err("Multiple function plots are unsupported".into());
        }
        let refs = records[start + 1..end]
            .iter()
            .find(|record| record.kind == 0x07d2)
            .ok_or("Function plot has no dependencies")?;
        let refs = payload(bytes, *refs);
        if refs.len() != 12 || u32_at(refs, 0) != 2 {
            return Err("Unsupported function plot dependencies".into());
        }
        let definition_id = u32_at(refs, 4) as usize;
        let coordinate_system_id = u32_at(refs, 8) as usize;
        if definition_id == 0 || definition_id >= ordinal + 1 || coordinate_system_id == 0 {
            return Err("Invalid function plot dependencies".into());
        }
        let &(definition_start, definition_end, definition_code) = groups
            .get(definition_id - 1)
            .ok_or("Missing function definition")?;
        if definition_code != 0 {
            return Err("Unsupported function definition object".into());
        }
        let expression_record = records[definition_start + 1..definition_end]
            .iter()
            .find(|record| record.kind == 0x0907)
            .ok_or("Function definition has no expression")?;
        let expression = decode_simple_expression(payload(bytes, *expression_record))?;
        let &(coordinates_start, _, coordinates_code) = groups
            .get(coordinate_system_id - 1)
            .ok_or("Missing coordinate system")?;
        if coordinates_code != 61 || coordinates_start >= start {
            return Err("Function plot references no coordinate system".into());
        }
        let descriptor = records[start + 1..end]
            .iter()
            .find(|record| record.kind == 0x0902)
            .ok_or("Function plot has no range")?;
        let descriptor = payload(bytes, *descriptor);
        if descriptor.len() != 40 || u32_at(descriptor, 20) & 0xffff != 0 {
            return Err("Unsupported function plot mode".into());
        }
        let x_min = f64::from_le_bytes(descriptor[0..8].try_into().unwrap());
        let x_max = f64::from_le_bytes(descriptor[8..16].try_into().unwrap());
        let samples = u32_at(descriptor, 16);
        if !x_min.is_finite()
            || !x_max.is_finite()
            || x_min >= x_max
            || !(2..=4096).contains(&samples)
        {
            return Err("Invalid function plot range".into());
        }
        found = Some(FunctionPlot {
            expression,
            definition_id,
            coordinate_system_id,
            x_min,
            x_max,
            samples,
        });
    }
    found.ok_or_else(|| "No Cartesian function plot".into())
}

fn original_template(expression: SimpleFunction) -> &'static [u8] {
    match expression {
        SimpleFunction::Identity => include_bytes!("../tests/fixtures/function_plot_identity.gsp"),
        SimpleFunction::Square => include_bytes!("../tests/fixtures/function_plot_square.gsp"),
        SimpleFunction::Arithmetic { .. } => {
            include_bytes!("../tests/fixtures/function_plot_identity.gsp")
        }
    }
}

pub fn decode_plot_range(data: &[u8]) -> Result<(f64, f64, u32), String> {
    if data.len() != 40 || u32_at(data, 20) & 0xffff != 0 {
        return Err("Unsupported function plot mode".into());
    }
    let x_min = f64::from_le_bytes(data[0..8].try_into().unwrap());
    let x_max = f64::from_le_bytes(data[8..16].try_into().unwrap());
    let samples = u32_at(data, 16);
    if !x_min.is_finite() || !x_max.is_finite() || x_min >= x_max || !(2..=4096).contains(&samples)
    {
        return Err("Invalid function plot range".into());
    }
    Ok((x_min, x_max, samples))
}

pub fn decode_parametric_range(data: &[u8], polar: bool) -> Result<(f64, f64, u32), String> {
    if data.len() != 40 || u32_at(data, 20) != if polar { 0x30002 } else { 0x30000 } {
        return Err("Unsupported parametric plot mode".into());
    }
    let min = f64::from_le_bytes(data[0..8].try_into().unwrap());
    let max = f64::from_le_bytes(data[8..16].try_into().unwrap());
    let samples = u32_at(data, 16);
    if !min.is_finite() || !max.is_finite() || min >= max || !(2..=4096).contains(&samples) {
        return Err("Invalid parametric plot range".into());
    }
    Ok((min, max, samples))
}

pub fn encode_parametric_plot_group(
    x_definition: usize,
    y_definition: usize,
    frame: usize,
    min: f64,
    max: f64,
    samples: u32,
    polar: bool,
) -> Result<Vec<u8>, String> {
    if x_definition == 0
        || y_definition == 0
        || frame == 0
        || x_definition == y_definition
        || !min.is_finite()
        || !max.is_finite()
        || min >= max
        || !(2..=4096).contains(&samples)
    {
        return Err("Invalid parametric plot".into());
    }
    let source = if polar {
        include_bytes!("../tests/fixtures/parametric_polar_original.gsp").as_slice()
    } else {
        include_bytes!("../tests/fixtures/parametric_rectangular_original.gsp").as_slice()
    };
    let rs = records(source)?;
    let groups = group_ranges(source, &rs)?;
    let (first, last, code) = *groups.last().ok_or("Missing parametric template group")?;
    if code != if polar { 127 } else { 126 } {
        return Err("Invalid parametric template".into());
    }
    let start = rs[first].start;
    let mut group = source[start..rs[last].end].to_vec();
    let refs = rs[first + 1..last]
        .iter()
        .find(|record| record.kind == 0x07d2)
        .ok_or("Parametric template has no dependencies")?;
    if payload(source, *refs).len() != 16 {
        return Err("Invalid parametric template dependencies".into());
    }
    for (index, id) in [x_definition, y_definition, frame].into_iter().enumerate() {
        let id = u32::try_from(id).map_err(|_| "Too many objects")?;
        let offset = refs.payload_start - start + 4 + index * 4;
        group[offset..offset + 4].copy_from_slice(&id.to_le_bytes());
    }
    let range = rs[first + 1..last]
        .iter()
        .find(|record| record.kind == 0x0902)
        .ok_or("Parametric template has no range")?;
    if payload(source, *range).len() != 40 {
        return Err("Invalid parametric template range".into());
    }
    let offset = range.payload_start - start;
    group[offset..offset + 8].copy_from_slice(&min.to_le_bytes());
    group[offset + 8..offset + 16].copy_from_slice(&max.to_le_bytes());
    group[offset + 16..offset + 20].copy_from_slice(&samples.to_le_bytes());
    Ok(group)
}

pub fn encode_definition_group(expression: SimpleFunction) -> Result<Vec<u8>, String> {
    let source = original_template(expression);
    let rs = records(source)?;
    let groups = group_ranges(source, &rs)?;
    let (first, last, code) = groups[6];
    if code != 0 {
        return Err("Invalid function definition template".into());
    }
    let mut group = source[rs[first].start..rs[last].end].to_vec();
    if let SimpleFunction::Arithmetic { .. } = expression {
        let record = rs[first + 1..last]
            .iter()
            .find(|record| record.kind == 0x0907)
            .ok_or("Function template has no expression")?;
        let offset = record.start - rs[first].start;
        let encoded_record = encode_expression_record(payload(source, *record), expression)?;
        group.splice(offset..offset + (record.end - record.start), encoded_record);
    }
    Ok(group)
}

pub fn encode_derivative_group(
    source_id: usize,
    expression: SimpleFunction,
) -> Result<Vec<u8>, String> {
    let source_id = u32::try_from(source_id).map_err(|_| "Too many objects")?;
    if source_id == 0 {
        return Err("Invalid derivative source".into());
    }
    let source = include_bytes!("../tests/fixtures/function_derivative_square_original.gsp");
    let rs = records(source)?;
    let groups = group_ranges(source, &rs)?;
    let (first, last, code) = *groups.last().ok_or("Missing derivative template")?;
    if code != 78 {
        return Err("Invalid derivative template".into());
    }
    let start = rs[first].start;
    let mut group = source[start..rs[last].end].to_vec();
    let refs = rs[first + 1..last]
        .iter()
        .find(|record| record.kind == 0x07d2)
        .ok_or("Derivative template has no source")?;
    if payload(source, *refs).len() != 8 {
        return Err("Invalid derivative template source".into());
    }
    let ref_offset = refs.payload_start - start + 4;
    group[ref_offset..ref_offset + 4].copy_from_slice(&source_id.to_le_bytes());
    let record = rs[first + 1..last]
        .iter()
        .find(|record| record.kind == 0x0907)
        .ok_or("Derivative template has no expression")?;
    let replacement = encode_expression_record(payload(source, *record), expression)?;
    group.splice(record.start - start..record.end - start, replacement);
    Ok(group)
}

pub fn encode_plot_group(
    expression: SimpleFunction,
    definition_id: usize,
    frame_id: usize,
) -> Result<Vec<u8>, String> {
    let both = encode_plot_groups(expression, definition_id, frame_id)?;
    let definition_length = encode_definition_group(expression)?.len();
    Ok(both[definition_length..].to_vec())
}

pub fn encode_plot_group_with_range(
    expression: SimpleFunction,
    definition_id: usize,
    frame_id: usize,
    min: f64,
    max: f64,
    samples: u32,
) -> Result<Vec<u8>, String> {
    if !min.is_finite() || !max.is_finite() || min >= max || !(2..=4096).contains(&samples) {
        return Err("Invalid function plot range".into());
    }
    let group = encode_plot_group(expression, definition_id, frame_id)?;
    let mut bytes = b"GSP4".to_vec();
    bytes.extend_from_slice(&group);
    let range = records(&bytes)?
        .into_iter()
        .find(|record| record.kind == 0x0902)
        .ok_or("Function plot template has no range")?;
    decode_plot_range(payload(&bytes, range))?;
    bytes[range.payload_start..range.payload_start + 8].copy_from_slice(&min.to_le_bytes());
    bytes[range.payload_start + 8..range.payload_start + 16].copy_from_slice(&max.to_le_bytes());
    bytes[range.payload_start + 16..range.payload_start + 20]
        .copy_from_slice(&samples.to_le_bytes());
    Ok(bytes[4..].to_vec())
}

/// Encode the two observed object groups, changing only the plot's references.
pub fn encode_plot_groups(
    expression: SimpleFunction,
    definition_id: usize,
    coordinate_system_id: usize,
) -> Result<Vec<u8>, String> {
    if definition_id == 0 || coordinate_system_id == 0 || coordinate_system_id == definition_id {
        return Err("Invalid plot object IDs".into());
    }
    let definition_id = u32::try_from(definition_id).map_err(|_| "Too many objects")?;
    let coordinate_system_id =
        u32::try_from(coordinate_system_id).map_err(|_| "Too many objects")?;
    let source = original_template(expression);
    let records = records(source)?;
    let groups = group_ranges(source, &records)?;
    if groups.len() != 8 || groups[6].2 != 0 || groups[7].2 != 72 {
        return Err("Function template has unexpected groups".into());
    }
    let start = records[groups[7].0].start;
    let end = records[groups[7].1].end;
    let mut plot = source[start..end].to_vec();
    let ref_record = records[groups[7].0 + 1..groups[7].1]
        .iter()
        .find(|record| record.kind == 0x07d2)
        .ok_or("Function plot template has no dependencies")?;
    let ref_offset = ref_record.payload_start - start;
    plot[ref_offset + 4..ref_offset + 8].copy_from_slice(&definition_id.to_le_bytes());
    plot[ref_offset + 8..ref_offset + 12].copy_from_slice(&coordinate_system_id.to_le_bytes());
    let mut encoded = encode_definition_group(expression)?;
    encoded.extend_from_slice(&plot);
    Ok(encoded)
}

/// Standalone writer proof: assemble original function groups on the port's
/// six-object coordinate-system document. The shared document writer can use
/// `encode_plot_groups` when function objects are integrated later.
pub fn encode_minimal_plot_document(expression: SimpleFunction) -> Result<Vec<u8>, String> {
    let mut bytes = include_bytes!("../tests/fixtures/function_base_frame.gsp").to_vec();
    let records = records(&bytes)?;
    let page = records
        .iter()
        .find(|record| record.kind == 0x03e8)
        .ok_or("Missing page record")?;
    if payload(&bytes, *page).len() < 28 || u32_at(&bytes, page.payload_start + 24) != 6 {
        return Err("Function base has unexpected object count".into());
    }
    let link = records
        .iter()
        .find(|record| record.kind == 0x232d)
        .ok_or("Missing coordinate-system page link")?;
    if payload(&bytes, *link) != 6u32.to_le_bytes() {
        return Err("Function base has unexpected coordinate system".into());
    }
    bytes[page.payload_start + 24..page.payload_start + 28].copy_from_slice(&8u32.to_le_bytes());
    let mut appended = encode_plot_groups(expression, 7, 6)?;
    appended.extend_from_slice(&4u32.to_le_bytes());
    appended.extend_from_slice(&0x2328u32.to_le_bytes());
    appended.extend_from_slice(&8u32.to_le_bytes());
    bytes.splice(link.start..link.start, appended);
    let decoded = decode_minimal_plot(&bytes)?;
    if decoded.expression != expression
        || decoded.definition_id != 7
        || decoded.coordinate_system_id != 6
    {
        return Err("Encoded function plot failed to reparse".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_identity_and_square_decode() {
        for (expression, source) in [
            (
                SimpleFunction::Identity,
                include_bytes!("../tests/fixtures/function_plot_identity.gsp").as_slice(),
            ),
            (
                SimpleFunction::Square,
                include_bytes!("../tests/fixtures/function_plot_square.gsp").as_slice(),
            ),
        ] {
            let plot = decode_minimal_plot(source).unwrap();
            assert_eq!(plot.expression, expression);
            assert_eq!(
                (plot.definition_id, plot.coordinate_system_id, plot.samples),
                (7, 6, 750)
            );
            assert!((plot.x_min + plot.x_max).abs() < 1e-12);
        }
    }

    #[test]
    fn symbolic_derivatives_match_native_examples_and_numeric_slope() {
        for (source, expected) in [
            ("x", "1"),
            ("x^2", "2*x"),
            ("x^3", "3*x^2"),
            ("sin(x)", "cos(x)"),
        ] {
            assert_eq!(
                SimpleFunction::parse(source)
                    .unwrap()
                    .symbolic_derivative()
                    .unwrap(),
                SimpleFunction::parse(expected).unwrap(),
            );
        }
        for source in ["2*(x+1)", "1/x", "sin(x)+cos(x)", "tan(x)"] {
            let function = SimpleFunction::parse(source).unwrap();
            let derivative = function.symbolic_derivative().unwrap();
            let x = 0.3;
            let slope = (function.evaluate(x + 1e-6) - function.evaluate(x - 1e-6)) / 2e-6;
            assert!((derivative.evaluate(x) - slope).abs() < 1e-5, "{source}");
        }
    }

    #[test]
    fn expression_rejects_unknown_token_without_guessing() {
        let source = include_bytes!("../tests/fixtures/function_plot_identity.gsp");
        let records = records(source).unwrap();
        let expression = records.iter().find(|record| record.kind == 0x0907).unwrap();
        let mut data = payload(source, *expression).to_vec();
        assert_eq!(
            decode_simple_expression(&data),
            Ok(SimpleFunction::Identity)
        );
        data[92] = 14;
        assert!(decode_simple_expression(&data).is_err());
    }

    #[test]
    fn native_arithmetic_expressions_decode_and_evaluate() {
        for (text, source, expected) in [
            (
                "x+1",
                include_bytes!("../tests/fixtures/function_plot_x_plus_1.gsp").as_slice(),
                3.0,
            ),
            (
                "x^3",
                include_bytes!("../tests/fixtures/function_plot_x_cubed.gsp").as_slice(),
                8.0,
            ),
            (
                "2*(x+1)",
                include_bytes!("../tests/fixtures/function_plot_two_times_x_plus_1.gsp").as_slice(),
                6.0,
            ),
            (
                "x+0.5",
                include_bytes!("../tests/fixtures/function_plot_x_plus_half.gsp").as_slice(),
                2.5,
            ),
            (
                "-x",
                include_bytes!("../tests/fixtures/function_plot_negative_x.gsp").as_slice(),
                -2.0,
            ),
            (
                "1/x",
                include_bytes!("../tests/fixtures/function_plot_reciprocal_original.gsp")
                    .as_slice(),
                0.5,
            ),
        ] {
            let expression = SimpleFunction::parse(text).unwrap();
            let native = decode_minimal_plot(source).unwrap();
            assert_eq!(native.expression, expression);
            assert_eq!(native.expression.evaluate(2.0), expected);
            let generated = encode_minimal_plot_document(expression).unwrap();
            assert_eq!(
                decode_minimal_plot(&generated).unwrap().expression,
                expression
            );
        }
        assert_eq!(SimpleFunction::parse("x²").unwrap(), SimpleFunction::Square);
        assert_eq!(SimpleFunction::parse("2^3^2").unwrap().evaluate(0.0), 512.0);
        assert!(SimpleFunction::parse("x+").is_err());
        assert!(SimpleFunction::parse("(x+1").is_err());
        assert!(SimpleFunction::parse("x.1").is_err());
    }

    #[test]
    fn native_trigonometric_expressions_decode_and_evaluate() {
        for (text, source, at, expected) in [
            (
                "sin(x)",
                include_bytes!("../tests/fixtures/function_plot_sin_original.gsp").as_slice(),
                std::f64::consts::FRAC_PI_2,
                1.0,
            ),
            (
                "cos(x)",
                include_bytes!("../tests/fixtures/function_plot_cos_original.gsp").as_slice(),
                std::f64::consts::FRAC_PI_2,
                0.0,
            ),
            (
                "tan(x)",
                include_bytes!("../tests/fixtures/function_plot_tan_original.gsp").as_slice(),
                std::f64::consts::FRAC_PI_4,
                1.0,
            ),
        ] {
            let expression = SimpleFunction::parse(text).unwrap();
            let native = decode_minimal_plot(source).unwrap();
            assert_eq!(native.expression, expression);
            assert!(expression.uses_trigonometry());
            assert!((native.expression.evaluate(at) - expected).abs() < 1e-12);
            assert_eq!(expression.display(), format!("f(x) = {text}"));
        }
        let sine = SimpleFunction::parse("sin(x)").unwrap();
        assert!((sine.evaluate_with_angle_unit(90.0, false) - 1.0).abs() < 1e-12);
        assert_eq!(
            SimpleFunction::parse("sin(x)+cos(x)")
                .unwrap()
                .evaluate(0.0),
            1.0
        );
    }

    #[test]
    fn number_new_function_uses_the_same_identity_expression() {
        let source = include_bytes!("../tests/fixtures/function_definition_identity.gsp");
        let records = records(source).unwrap();
        let expression = records.iter().find(|record| record.kind == 0x0907).unwrap();
        assert_eq!(
            decode_simple_expression(payload(source, *expression)),
            Ok(SimpleFunction::Identity)
        );
    }

    #[test]
    fn template_groups_and_minimal_document_reparse() {
        for expression in [SimpleFunction::Identity, SimpleFunction::Square] {
            let bytes = encode_minimal_plot_document(expression).unwrap();
            let plot = decode_minimal_plot(&bytes).unwrap();
            assert_eq!(plot.expression, expression);
            assert_eq!(
                expression.evaluate(2.0),
                match expression {
                    SimpleFunction::Identity => 2.0,
                    SimpleFunction::Square => 4.0,
                    SimpleFunction::Arithmetic { .. } => unreachable!(),
                }
            );
        }
    }

    #[test]
    fn moved_original_plot_keeps_expression_and_dependencies() {
        let moved = include_bytes!("../tests/fixtures/function_port_square_moved_wine.gsp");
        let plot = decode_minimal_plot(moved).unwrap();
        assert_eq!(plot.expression, SimpleFunction::Square);
        assert_eq!((plot.definition_id, plot.coordinate_system_id), (7, 6));
    }
}
