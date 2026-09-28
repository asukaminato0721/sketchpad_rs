//! Numeric quantity dimensions independent of record encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericUnit {
    Scalar,
    Degrees,
    Length,
    Area,
    Compound { length: i32, degrees: i32 },
}

impl NumericUnit {
    pub fn dimensions(self) -> (i32, i32) {
        match self {
            Self::Scalar => (0, 0),
            Self::Length => (1, 0),
            Self::Area => (2, 0),
            Self::Degrees => (0, 1),
            Self::Compound { length, degrees } => (length, degrees),
        }
    }
    pub fn from_dimensions(length: i32, degrees: i32) -> Self {
        match (length, degrees) {
            (0, 0) => Self::Scalar,
            (1, 0) => Self::Length,
            (2, 0) => Self::Area,
            (0, 1) => Self::Degrees,
            _ => Self::Compound { length, degrees },
        }
    }
    pub fn suffix(self) -> String {
        fn exponent(value: i32) -> String {
            if value == 1 {
                return String::new();
            }
            value
                .to_string()
                .chars()
                .map(|c| match c {
                    '-' => '⁻',
                    '0' => '⁰',
                    '1' => '¹',
                    '2' => '²',
                    '3' => '³',
                    '4' => '⁴',
                    '5' => '⁵',
                    '6' => '⁶',
                    '7' => '⁷',
                    '8' => '⁸',
                    '9' => '⁹',
                    _ => unreachable!(),
                })
                .collect()
        }
        let (length, degrees) = self.dimensions();
        let mut text = String::new();
        if length != 0 {
            text.push_str(&format!(" cm{}", exponent(length)));
        }
        if degrees != 0 {
            text.push_str(&format!("°{}", exponent(degrees)));
        }
        text
    }
}
