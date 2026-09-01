//! Typed runtime values for robot program expressions and variables.

use crate::ast::{Operator, ValueType};
use crate::cpu_step_result::CpuStepResultKind;

/// A typed value on the expression stack or in a runtime variable binding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgramValue {
    Bool(bool),
    Int(i32),
    Float(f64),
}

impl ProgramValue {
    pub fn default_for_type(value_type: ValueType) -> Self {
        match value_type {
            ValueType::Bool => Self::Bool(false),
            ValueType::Int => Self::Int(0),
            ValueType::Double => Self::Float(0.0),
        }
    }

    pub fn kind(self) -> CpuStepResultKind {
        match self {
            Self::Bool(_) => CpuStepResultKind::Bool,
            Self::Int(_) => CpuStepResultKind::Int,
            Self::Float(_) => CpuStepResultKind::Float,
        }
    }

    pub fn is_truthy(self) -> bool {
        match self {
            Self::Bool(value) => value,
            Self::Int(value) => value != 0,
            Self::Float(value) => value != 0.0,
        }
    }

    pub fn as_f64(self) -> f64 {
        match self {
            Self::Bool(value) => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Int(value) => f64::from(value),
            Self::Float(value) => value,
        }
    }

    pub fn as_i32(self) -> Option<i32> {
        match self {
            Self::Int(value) => Some(value),
            Self::Bool(value) => Some(i32::from(value)),
            Self::Float(_) => None,
        }
    }
}

/// Round to nearest integer, half away from zero (same as JS `Math.round`).
pub fn round_to_i32(value: f64) -> i32 {
    value.round() as i32
}

/// Convert a runtime value to a declared variable type.
pub fn coerce_to_value_type(value: ProgramValue, target: ValueType) -> ProgramValue {
    match target {
        ValueType::Int => ProgramValue::Int(match value {
            ProgramValue::Int(value) => value,
            ProgramValue::Float(value) => round_to_i32(value),
            ProgramValue::Bool(value) => i32::from(value),
        }),
        ValueType::Double => ProgramValue::Float(match value {
            ProgramValue::Float(value) => value,
            ProgramValue::Int(value) => f64::from(value),
            ProgramValue::Bool(value) => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
        }),
        ValueType::Bool => ProgramValue::Bool(value.is_truthy()),
    }
}

/// Exact promotion for motion/scan action arguments (`4 → 4.0`).
pub fn as_f64_for_action_arg(value: ProgramValue) -> f64 {
    value.as_f64()
}

fn numeric_as_f64(value: ProgramValue) -> f64 {
    value.as_f64()
}

fn both_int(left: ProgramValue, right: ProgramValue) -> Option<(i32, i32)> {
    match (left, right) {
        (ProgramValue::Int(left), ProgramValue::Int(right)) => Some((left, right)),
        _ => None,
    }
}

fn either_float(left: ProgramValue, right: ProgramValue) -> bool {
    matches!(left, ProgramValue::Float(_)) || matches!(right, ProgramValue::Float(_))
}

/// Evaluate a binary operator with typed operands and promotion rules.
pub fn evaluate_binary_operator(
    operator: Operator,
    left: ProgramValue,
    right: ProgramValue,
) -> ProgramValue {
    match operator {
        Operator::Addition | Operator::Subtraction | Operator::Multiply => {
            if either_float(left, right) {
                let left = numeric_as_f64(left);
                let right = numeric_as_f64(right);
                ProgramValue::Float(match operator {
                    Operator::Addition => left + right,
                    Operator::Subtraction => left - right,
                    Operator::Multiply => left * right,
                    _ => unreachable!(),
                })
            } else {
                let left = left.as_i32().unwrap_or(0);
                let right = right.as_i32().unwrap_or(0);
                ProgramValue::Int(match operator {
                    Operator::Addition => left.wrapping_add(right),
                    Operator::Subtraction => left.wrapping_sub(right),
                    Operator::Multiply => left.wrapping_mul(right),
                    _ => unreachable!(),
                })
            }
        }
        Operator::Division => {
            if let Some((left, right)) = both_int(left, right) {
                ProgramValue::Int(left / right)
            } else {
                ProgramValue::Float(numeric_as_f64(left) / numeric_as_f64(right))
            }
        }
        Operator::Mod => {
            if let Some((left, right)) = both_int(left, right) {
                ProgramValue::Int(left % right)
            } else {
                ProgramValue::Float(
                    (numeric_as_f64(left) as i32 % numeric_as_f64(right) as i32) as f64,
                )
            }
        }
        Operator::Larger
        | Operator::Smaller
        | Operator::LargerEqual
        | Operator::SmallerEqual
        | Operator::Equal
        | Operator::NotEqual => {
            let result = if either_float(left, right) {
                let left = numeric_as_f64(left);
                let right = numeric_as_f64(right);
                match operator {
                    Operator::Larger => left > right,
                    Operator::Smaller => left < right,
                    Operator::LargerEqual => left >= right,
                    Operator::SmallerEqual => left <= right,
                    Operator::Equal => left == right,
                    Operator::NotEqual => left != right,
                    _ => unreachable!(),
                }
            } else if let Some((left, right)) = both_int(left, right) {
                match operator {
                    Operator::Larger => left > right,
                    Operator::Smaller => left < right,
                    Operator::LargerEqual => left >= right,
                    Operator::SmallerEqual => left <= right,
                    Operator::Equal => left == right,
                    Operator::NotEqual => left != right,
                    _ => unreachable!(),
                }
            } else {
                match operator {
                    Operator::Larger => left.is_truthy() && !right.is_truthy(),
                    Operator::Smaller => !left.is_truthy() && right.is_truthy(),
                    Operator::LargerEqual => left.is_truthy() || !right.is_truthy(),
                    Operator::SmallerEqual => !left.is_truthy() || right.is_truthy(),
                    Operator::Equal => left.is_truthy() == right.is_truthy(),
                    Operator::NotEqual => left.is_truthy() != right.is_truthy(),
                    _ => unreachable!(),
                }
            };
            ProgramValue::Bool(result)
        }
        Operator::And => ProgramValue::Bool(left.is_truthy() && right.is_truthy()),
        Operator::Or => ProgramValue::Bool(left.is_truthy() || right.is_truthy()),
        Operator::Undefined => ProgramValue::Int(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_half_away_from_zero() {
        assert_eq!(round_to_i32(3.5), 4);
        assert_eq!(round_to_i32(-3.5), -4);
        assert_eq!(round_to_i32(3.75), 4);
        assert_eq!(round_to_i32(3.4), 3);
    }

    #[test]
    fn int_division_truncates_toward_zero() {
        let result = evaluate_binary_operator(
            Operator::Division,
            ProgramValue::Int(7),
            ProgramValue::Int(2),
        );
        assert_eq!(result, ProgramValue::Int(3));
    }

    #[test]
    fn mixed_compare_promotes_int_to_float() {
        let result = evaluate_binary_operator(
            Operator::Larger,
            ProgramValue::Float(3.75),
            ProgramValue::Int(4),
        );
        assert_eq!(result, ProgramValue::Bool(false));
    }

    #[test]
    fn float_to_int_coercion_rounds() {
        assert_eq!(
            coerce_to_value_type(ProgramValue::Float(3.75), ValueType::Int),
            ProgramValue::Int(4)
        );
    }
}
