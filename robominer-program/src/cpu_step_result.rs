//! Typed return value for one program CPU micro-step (rally replay debug).
//!
//! Display kinds are UI-oriented and map from AST [`ValueType`] (`Double` ≡ [`CpuStepResultKind::Float`])
//! and expression heuristics (`for_number_literal`, `for_action`, …). Wire format uses
//! `b`/`i`/`f` via `AnimationCpuStepResultKind` in robominer-sim.

use crate::ast::{ExecutableAction, Operator, RobotProperty, ValueType};

/// How a CPU-step return value should be displayed in the replay UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuStepResultKind {
    Bool,
    Int,
    Float,
}

/// Numeric result produced by a CPU micro-step, with display kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuStepResult {
    pub kind: CpuStepResultKind,
    pub value: f64,
}

impl CpuStepResult {
    pub fn bool_value(value: f64) -> Self {
        Self {
            kind: CpuStepResultKind::Bool,
            value,
        }
    }

    pub fn int_value(value: f64) -> Self {
        Self {
            kind: CpuStepResultKind::Int,
            value,
        }
    }

    pub fn float_value(value: f64) -> Self {
        Self {
            kind: CpuStepResultKind::Float,
            value,
        }
    }

    pub fn from_value_type(value_type: ValueType, value: f64) -> Self {
        match value_type {
            ValueType::Bool => Self::bool_value(value),
            ValueType::Int => Self::int_value(value),
            ValueType::Double => Self::float_value(value),
        }
    }

    /// Display heuristic for bare numeric literals: whole numbers as int, otherwise float.
    /// Not an assign/typechecking truth source — declaration and AST types own semantics.
    pub fn for_number_literal(value: f64) -> Self {
        if (value - value.round()).abs() < 1e-9 {
            Self::int_value(value)
        } else {
            Self::float_value(value)
        }
    }

    pub fn for_ore_distance(value: f64) -> Self {
        Self::float_value(value)
    }

    pub fn for_ore_type(value: f64) -> Self {
        Self::int_value(value)
    }

    pub fn for_action(action: ExecutableAction, value: f64) -> Self {
        match action {
            ExecutableAction::Move(_) => Self::float_value(value),
            ExecutableAction::Rotate(_)
            | ExecutableAction::Mine
            | ExecutableAction::Dump(_)
            | ExecutableAction::StartScan(_) => Self::int_value(value),
            ExecutableAction::AwaitScanResult => Self::int_value(value),
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
            | RobotProperty::MaxCycles
            | RobotProperty::MiningSpeed
            | RobotProperty::CpuSpeed
            | RobotProperty::Orientation => Self::int_value(value),
        }
    }

    pub fn for_binary_operator(
        operator: Operator,
        left: CpuStepResultKind,
        right: CpuStepResultKind,
        value: f64,
    ) -> Self {
        match operator {
            Operator::Larger
            | Operator::Smaller
            | Operator::LargerEqual
            | Operator::SmallerEqual
            | Operator::Equal
            | Operator::NotEqual
            | Operator::And
            | Operator::Or => Self::bool_value(value),
            Operator::Division => Self::float_value(value),
            Operator::Mod => Self::int_value(value),
            Operator::Addition | Operator::Subtraction | Operator::Multiply => {
                if matches!(left, CpuStepResultKind::Float)
                    || matches!(right, CpuStepResultKind::Float)
                {
                    Self::float_value(value)
                } else {
                    Self::int_value(value)
                }
            }
            Operator::Undefined => Self::int_value(value),
        }
    }
}
