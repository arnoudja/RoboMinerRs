use crate::ast::{Operator, ValueType};
use crate::cpu_step_result::{CpuStepResult, CpuStepResultKind};

#[test]
fn coerce_table_matches_spec() {
    assert_eq!(
        CpuStepResult::Float(3.9).coerce_to(ValueType::Int),
        CpuStepResult::Int(3)
    );
    assert_eq!(
        CpuStepResult::Float(-3.9).coerce_to(ValueType::Int),
        CpuStepResult::Int(-3)
    );
    assert_eq!(
        CpuStepResult::Bool(true).coerce_to(ValueType::Int),
        CpuStepResult::Int(1)
    );
    assert_eq!(
        CpuStepResult::Int(0).coerce_to(ValueType::Bool),
        CpuStepResult::Bool(false)
    );
    assert_eq!(
        CpuStepResult::Int(7).coerce_to(ValueType::Bool),
        CpuStepResult::Bool(true)
    );
    assert_eq!(
        CpuStepResult::Bool(false).coerce_to(ValueType::Double),
        CpuStepResult::Float(0.0)
    );
}

#[test]
fn int_division_truncates_float_division_promotes() {
    assert_eq!(
        CpuStepResult::evaluate_binary(
            Operator::Division,
            CpuStepResult::Int(5),
            CpuStepResult::Int(2)
        ),
        CpuStepResult::Int(2)
    );
    assert_eq!(
        CpuStepResult::evaluate_binary(
            Operator::Division,
            CpuStepResult::Int(5),
            CpuStepResult::Float(2.0)
        ),
        CpuStepResult::Float(2.5)
    );
}

#[test]
fn bool_participates_in_arithmetic_as_int() {
    assert_eq!(
        CpuStepResult::evaluate_binary(
            Operator::Addition,
            CpuStepResult::Bool(true),
            CpuStepResult::Int(1)
        ),
        CpuStepResult::Int(2)
    );
}

#[test]
fn int_div_and_mod_by_zero_are_zero() {
    assert_eq!(
        CpuStepResult::evaluate_binary(
            Operator::Division,
            CpuStepResult::Int(5),
            CpuStepResult::Int(0)
        ),
        CpuStepResult::Int(0)
    );
    assert_eq!(
        CpuStepResult::evaluate_binary(Operator::Mod, CpuStepResult::Int(5), CpuStepResult::Int(0)),
        CpuStepResult::Int(0)
    );
}

#[test]
fn comparisons_and_logic_yield_bool() {
    let cmp = CpuStepResult::evaluate_binary(
        Operator::Larger,
        CpuStepResult::Int(3),
        CpuStepResult::Int(2),
    );
    assert_eq!(cmp.kind(), CpuStepResultKind::Bool);
    assert_eq!(cmp, CpuStepResult::Bool(true));

    let and = CpuStepResult::evaluate_binary(
        Operator::And,
        CpuStepResult::Int(1),
        CpuStepResult::Bool(false),
    );
    assert_eq!(and, CpuStepResult::Bool(false));
}
