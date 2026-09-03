//! Typed return value for one program CPU micro-step (rally replay debug).
//!
//! Runtime values are a tagged union: bool / i64 / f64. Display kinds map to
//! wire format `b`/`i`/`f` via `AnimationCpuStepResultKind` in robominer-sim
//! (`Double` ≡ [`CpuStepResultKind::Float`]).

use crate::ast::{AreaProperty, ExecutableAction, Operator, RobotProperty, ValueType};

/// How a CPU-step return value should be displayed in the replay UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuStepResultKind {
    Bool,
    Int,
    Float,
}

/// Typed result produced by a CPU micro-step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuStepResult {
    Bool(bool),
    Int(i64),
    Float(f64),
}

impl CpuStepResult {
    pub fn bool_value(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn int_value(value: i64) -> Self {
        Self::Int(value)
    }

    pub fn float_value(value: f64) -> Self {
        Self::Float(value)
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

    pub fn as_bool(self) -> bool {
        self.is_truthy()
    }

    pub fn as_i64(self) -> i64 {
        match self {
            Self::Bool(value) => i64::from(value),
            Self::Int(value) => value,
            Self::Float(value) => value.trunc() as i64,
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
            Self::Int(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    /// Wire/display numeric payload (bool as 0/1).
    pub fn wire_f64(self) -> f64 {
        self.as_f64()
    }

    pub fn coerce_to(self, value_type: ValueType) -> Self {
        match value_type {
            ValueType::Bool => Self::Bool(self.as_bool()),
            ValueType::Int => Self::Int(self.as_i64()),
            ValueType::Double => Self::Float(self.as_f64()),
        }
    }

    pub fn from_value_type(value_type: ValueType, value: f64) -> Self {
        Self::Float(value).coerce_to(value_type)
    }

    pub fn for_ore_distance(value: f64) -> Self {
        Self::float_value(value)
    }

    pub fn for_ore_type(value: f64) -> Self {
        Self::int_value(value.trunc() as i64)
    }

    pub fn for_action(action: ExecutableAction, value: f64) -> Self {
        match action {
            ExecutableAction::Move(_) => Self::float_value(value),
            ExecutableAction::Rotate(_)
            | ExecutableAction::Mine
            | ExecutableAction::Dump(_)
            | ExecutableAction::StartScan(_)
            | ExecutableAction::AwaitScanResult => Self::int_value(value.trunc() as i64),
        }
    }

    pub fn for_robot_property(property: RobotProperty, value: f64) -> Self {
        match property {
            RobotProperty::ForwardSpeed
            | RobotProperty::BackwardSpeed
            | RobotProperty::XPos
            | RobotProperty::YPos => Self::float_value(value),
            RobotProperty::RotateSpeed
            | RobotProperty::ScanTime
            | RobotProperty::ScanDistance
            | RobotProperty::OreCap
            | RobotProperty::OreStored
            | RobotProperty::OreStoredA
            | RobotProperty::OreStoredB
            | RobotProperty::OreStoredC
            | RobotProperty::DepotSizeA
            | RobotProperty::DepotSizeB
            | RobotProperty::DepotSizeC
            | RobotProperty::DepotStoredA
            | RobotProperty::DepotStoredB
            | RobotProperty::DepotStoredC
            | RobotProperty::MaxTurns
            | RobotProperty::MiningSpeed
            | RobotProperty::CpuSpeed
            | RobotProperty::Orientation => Self::int_value(value.trunc() as i64),
        }
    }

    pub fn for_area_property(property: AreaProperty, value: f64) -> Self {
        match property {
            AreaProperty::SizeX | AreaProperty::SizeY => Self::float_value(value),
            AreaProperty::ContainerTax
            | AreaProperty::DepotTax
            | AreaProperty::StartingOreA
            | AreaProperty::StartingOreB
            | AreaProperty::StartingOreC
            | AreaProperty::MiningTurns
            | AreaProperty::OreTarget => Self::int_value(value.trunc() as i64),
        }
    }

    /// Evaluate a binary operator with typed operands (spec promotion rules).
    pub fn evaluate_binary(operator: Operator, left: Self, right: Self) -> Self {
        match operator {
            Operator::Larger
            | Operator::Smaller
            | Operator::LargerEqual
            | Operator::SmallerEqual
            | Operator::Equal
            | Operator::NotEqual => {
                let cmp = if matches!(left, Self::Float(_)) || matches!(right, Self::Float(_)) {
                    let l = left.as_f64();
                    let r = right.as_f64();
                    match operator {
                        Operator::Larger => l > r,
                        Operator::Smaller => l < r,
                        Operator::LargerEqual => l >= r,
                        Operator::SmallerEqual => l <= r,
                        Operator::Equal => l == r,
                        Operator::NotEqual => l != r,
                        _ => unreachable!(),
                    }
                } else {
                    let l = left.as_i64();
                    let r = right.as_i64();
                    match operator {
                        Operator::Larger => l > r,
                        Operator::Smaller => l < r,
                        Operator::LargerEqual => l >= r,
                        Operator::SmallerEqual => l <= r,
                        Operator::Equal => l == r,
                        Operator::NotEqual => l != r,
                        _ => unreachable!(),
                    }
                };
                Self::Bool(cmp)
            }
            Operator::And => Self::Bool(left.is_truthy() && right.is_truthy()),
            Operator::Or => Self::Bool(left.is_truthy() || right.is_truthy()),
            Operator::Mod => {
                let l = left.as_i64();
                let r = right.as_i64();
                if r == 0 {
                    Self::Int(0)
                } else {
                    Self::Int(l % r)
                }
            }
            Operator::Division => {
                if matches!(left, Self::Float(_)) || matches!(right, Self::Float(_)) {
                    Self::Float(left.as_f64() / right.as_f64())
                } else {
                    let l = left.as_i64();
                    let r = right.as_i64();
                    if r == 0 {
                        Self::Int(0)
                    } else {
                        Self::Int(l / r)
                    }
                }
            }
            Operator::Addition | Operator::Subtraction | Operator::Multiply => {
                if matches!(left, Self::Float(_)) || matches!(right, Self::Float(_)) {
                    let l = left.as_f64();
                    let r = right.as_f64();
                    let value = match operator {
                        Operator::Addition => l + r,
                        Operator::Subtraction => l - r,
                        Operator::Multiply => l * r,
                        _ => unreachable!(),
                    };
                    Self::Float(value)
                } else {
                    let l = left.as_i64();
                    let r = right.as_i64();
                    let value = match operator {
                        Operator::Addition => l.wrapping_add(r),
                        Operator::Subtraction => l.wrapping_sub(r),
                        Operator::Multiply => l.wrapping_mul(r),
                        _ => unreachable!(),
                    };
                    Self::Int(value)
                }
            }
            Operator::Undefined => Self::Int(0),
        }
    }

    pub fn evaluate_min(left: Self, right: Self) -> Self {
        if matches!(left, Self::Float(_)) || matches!(right, Self::Float(_)) {
            Self::Float(left.as_f64().min(right.as_f64()))
        } else {
            Self::Int(left.as_i64().min(right.as_i64()))
        }
    }

    pub fn evaluate_max(left: Self, right: Self) -> Self {
        if matches!(left, Self::Float(_)) || matches!(right, Self::Float(_)) {
            Self::Float(left.as_f64().max(right.as_f64()))
        } else {
            Self::Int(left.as_i64().max(right.as_i64()))
        }
    }

    pub fn unary_minus(self) -> Self {
        match self {
            Self::Bool(value) => Self::Int(-i64::from(value)),
            Self::Int(value) => Self::Int(value.wrapping_neg()),
            Self::Float(value) => Self::Float(-value),
        }
    }

    pub fn abs(self) -> Self {
        match self {
            Self::Bool(value) => Self::Int(i64::from(value)),
            Self::Int(value) => Self::Int(value.wrapping_abs()),
            Self::Float(value) => Self::Float(value.abs()),
        }
    }
}
