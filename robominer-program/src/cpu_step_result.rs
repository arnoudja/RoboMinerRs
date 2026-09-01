//! Typed return value for one program CPU micro-step (rally replay debug).
//!
//! Display kinds map from [`ProgramValue`] and AST [`ValueType`] (`Double` ≡
//! [`CpuStepResultKind::Float`]). Wire format uses `b`/`i`/`f` via
//! `AnimationCpuStepResultKind` in robominer-sim.

use crate::ast::{AreaProperty, ExecutableAction, Operator, RobotProperty, ValueType};
use crate::program_value::ProgramValue;

/// How a CPU-step return value should be displayed in the replay UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuStepResultKind {
    Bool,
    Int,
    Float,
}

/// Typed result produced by a CPU micro-step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuStepResult {
    pub value: ProgramValue,
}

impl CpuStepResult {
    pub fn kind(self) -> CpuStepResultKind {
        self.value.kind()
    }

    /// Numeric value for animation wire format (`cpu[].r.v`, `cpu[].vs.*.v`).
    pub fn wire_value(self) -> f64 {
        self.value.as_f64()
    }

    pub fn bool_value(value: bool) -> Self {
        Self {
            value: ProgramValue::Bool(value),
        }
    }

    pub fn int_value(value: i32) -> Self {
        Self {
            value: ProgramValue::Int(value),
        }
    }

    pub fn float_value(value: f64) -> Self {
        Self {
            value: ProgramValue::Float(value),
        }
    }

    pub fn from_program_value(value: ProgramValue) -> Self {
        Self { value }
    }

    pub fn from_value_type(value_type: ValueType, value: ProgramValue) -> Self {
        Self {
            value: crate::program_value::coerce_to_value_type(value, value_type),
        }
    }

    /// Display heuristic for bare numeric literals: whole numbers as int, otherwise float.
    pub fn for_number_literal(value: f64) -> Self {
        if (value - value.round()).abs() < 1e-9 {
            Self::int_value(value.round() as i32)
        } else {
            Self::float_value(value)
        }
    }

    pub fn for_ore_distance(value: f64) -> Self {
        Self::float_value(value)
    }

    pub fn for_ore_type(value: f64) -> Self {
        Self::int_value(value.round() as i32)
    }

    pub fn for_action(action: ExecutableAction, value: f64) -> Self {
        match action {
            ExecutableAction::Move(_) => Self::float_value(value),
            ExecutableAction::Rotate(_)
            | ExecutableAction::Mine
            | ExecutableAction::Dump(_)
            | ExecutableAction::StartScan(_) => Self::int_value(value.round() as i32),
            ExecutableAction::AwaitScanResult => Self::int_value(value.round() as i32),
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
            | RobotProperty::Orientation => Self::int_value(value.round() as i32),
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
            | AreaProperty::OreTarget => Self::int_value(value.round() as i32),
        }
    }

    pub fn for_binary_operator(operator: Operator, result: ProgramValue) -> Self {
        match operator {
            Operator::Larger
            | Operator::Smaller
            | Operator::LargerEqual
            | Operator::SmallerEqual
            | Operator::Equal
            | Operator::NotEqual
            | Operator::And
            | Operator::Or => Self::from_program_value(result),
            Operator::Division => Self::from_program_value(result),
            Operator::Mod => Self::from_program_value(result),
            Operator::Addition | Operator::Subtraction | Operator::Multiply => {
                Self::from_program_value(result)
            }
            Operator::Undefined => Self::int_value(0),
        }
    }
}
