//! Flat calculator expressions. Intermediate infinities and NaNs are retained;
//! only numeric object boundaries reject non-finite values.
use super::{CALCULATOR_PI, ParameterExpr};

#[derive(Debug)]
enum Instruction {
    Source(usize),
    Literal(f64),
    Neg(usize),
    Binary(u16, usize, usize),
    Call(u16, usize),
}
#[derive(Debug)]
pub(crate) struct CalculationTape {
    instructions: Vec<Instruction>,
}
impl CalculationTape {
    pub(super) fn compile(tree: &ParameterExpr) -> Option<Self> {
        fn emit(tree: &ParameterExpr, code: &mut Vec<Instruction>, depth: usize) -> Option<usize> {
            if depth > 256 || code.len() >= 4096 {
                return None;
            }
            let op = match tree {
                ParameterExpr::Source(index) => Instruction::Source(*index),
                ParameterExpr::Literal(value, _) => Instruction::Literal(*value),
                ParameterExpr::Neg(inner) => Instruction::Neg(emit(inner, code, depth + 1)?),
                ParameterExpr::Group(inner) => return emit(inner, code, depth + 1),
                ParameterExpr::Binary(op, a, b) => {
                    Instruction::Binary(*op, emit(a, code, depth + 1)?, emit(b, code, depth + 1)?)
                }
                ParameterExpr::Call(op, a) => Instruction::Call(*op, emit(a, code, depth + 1)?),
                ParameterExpr::FunctionSource(..) => return None,
            };
            let slot = code.len();
            code.push(op);
            Some(slot)
        }
        let mut instructions = Vec::new();
        emit(tree, &mut instructions, 0)?;
        Some(Self { instructions })
    }
    pub(crate) fn evaluate(
        &self,
        source: impl Fn(usize) -> Option<f64>,
        radians: bool,
        scratch: &mut Vec<f64>,
    ) -> Option<f64> {
        scratch.resize(self.instructions.len(), 0.);
        self.evaluate_scratch(source, radians, scratch)
    }
    pub(super) fn evaluate_values(&self, values: &[f64], radians: bool) -> Option<f64> {
        let source = |i| values.get(i).copied();
        if self.instructions.len() <= 64 {
            self.evaluate_scratch(source, radians, &mut [0.; 64])
        } else {
            self.evaluate_scratch(source, radians, &mut vec![0.; self.instructions.len()])
        }
    }
    fn evaluate_scratch(
        &self,
        source: impl Fn(usize) -> Option<f64>,
        radians: bool,
        scratch: &mut [f64],
    ) -> Option<f64> {
        for (slot, op) in self.instructions.iter().enumerate() {
            let value = match *op {
                Instruction::Source(i) => source(i)?,
                Instruction::Literal(v) => v,
                Instruction::Neg(i) => -scratch[i],
                Instruction::Binary(op, a, b) => binary(op, scratch[a], scratch[b])?,
                Instruction::Call(op, a) => call(op, scratch[a], radians)?,
            };
            scratch[slot] = value;
        }
        scratch
            .get(self.instructions.len().checked_sub(1)?)
            .copied()
    }
}

pub(super) fn call(function: u16, value: f64, radians: bool) -> Option<f64> {
    // Native calculator PI constant, present at executable offset 0x272690.
    let angle = if radians {
        value
    } else {
        value / 180. * CALCULATOR_PI
    };
    Some(match function {
        0x2000 => angle.sin(),
        0x2001 => angle.cos(),
        0x2002 => angle.tan(),
        0x2003 => {
            if radians {
                value.asin()
            } else {
                value.asin() / CALCULATOR_PI * 180.
            }
        }
        0x2004 => {
            if radians {
                value.acos()
            } else {
                value.acos() / CALCULATOR_PI * 180.
            }
        }
        0x2005 => {
            if radians {
                value.atan()
            } else {
                value.atan() / CALCULATOR_PI * 180.
            }
        }
        0x2006 => value.abs(),
        0x2007 => value.sqrt(),
        0x2008 => value.ln(),
        0x2009 => value.log10(),
        0x200a => {
            if value == 0. {
                0.
            } else {
                value.signum()
            }
        }
        0x200b => (value + 0.5).floor(),
        0x200c => value.trunc(),
        _ => return None,
    })
}

pub(super) fn binary(operator: u16, left: f64, right: f64) -> Option<f64> {
    Some(match operator {
        0x1000 => left + right,
        0x1001 => left - right,
        0x1002 => left * right,
        0x1003 => left / right,
        0x1004 => left.powf(right),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crate::table_records::{NumericUnit, ParameterCalculationGroup};

    #[test]
    fn flat_expressions_preserve_raw_results_and_angle_modes() {
        let mut scratch = Vec::new();
        for expression in [
            "-(n+2)*(n-3)/(n+1)",
            "n^2",
            "(1/n)^0",
            "sin(n)+cos(n)+tan(n)",
            "asin(n)+acos(n)+atan(n)",
            "abs(n)+sqrt(n)+ln(n)+log(n)",
            "round(n)+trunc(n)+sgn(n)",
        ] {
            let group =
                ParameterCalculationGroup::from_expression(1, "n", expression, (0, 0)).unwrap();
            let code = group.batch_program(&[NumericUnit::Scalar]).unwrap();
            for radians in [false, true] {
                for value in [-2., -1., -0., 0., 0.5, 1., 90., f64::MAX] {
                    let expected = group.expression_tree().eval_raw_context(
                        &[value],
                        &[NumericUnit::Scalar],
                        radians,
                        &|_, _, _| None,
                    );
                    let actual = code.evaluate(|_| Some(value), radians, &mut scratch);
                    assert_eq!(
                        actual.map(f64::to_bits),
                        expected.map(f64::to_bits),
                        "{expression} at {value}, radians={radians}"
                    );
                }
            }
            assert!(group.batch_program(&[]).is_none());
        }
        let units = ParameterCalculationGroup::from_expression(1, "n", "n*2", (0, 0)).unwrap();
        assert_eq!(units.value_for_sources(&[2.], &[NumericUnit::Length]), None);
        assert_eq!(
            units.value_for_sources(&[2.], &[NumericUnit::Scalar]),
            Some(4.)
        );
        assert_eq!(units.value_for_sources(&[2.], &[NumericUnit::Length]), None);
        let group = ParameterCalculationGroup::from_expression(1, "n", "(1/n)^0", (0, 0)).unwrap();
        assert_eq!(
            group
                .batch_program(&[NumericUnit::Scalar])
                .unwrap()
                .evaluate(|_| Some(0.), false, &mut scratch),
            Some(1.)
        );
    }
}
