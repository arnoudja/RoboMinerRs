use crate::cpu_step_result::{CpuStepResult, CpuStepResultKind};
use crate::runner::expression_eval::schedule::{ExpressionWork, Truthy, evaluate_operator};
use crate::runner::ExecutableRunner;
use crate::types::*;

impl ExecutableRunner {
    pub(super) fn apply_expression_work(
        &mut self,
        context: &ExecutionContext,
        action_result: &mut Option<f64>,
        work: ExpressionWork,
    ) -> Result<(), ()> {
        let eval = self.expression_eval.as_mut().ok_or(())?;
        match work {
            ExpressionWork::PushNumber(value) => {
                eval.values.push(CpuStepResult::for_number_literal(value));
                eval.index += 1;
            }
            ExpressionWork::PushBool(value) => {
                eval.values
                    .push(CpuStepResult::bool_value(if value { 1.0 } else { 0.0 }));
                eval.index += 1;
            }
            ExpressionWork::PushVariable(name) => {
                eval.values.push(self.variables.get_typed(&name));
                eval.index += 1;
            }
            ExpressionWork::PushVariableUpdate { name, operator } => {
                let result = match operator {
                    VariableOperator::PreIncrement => self.variables.update(&name, 1.0, true),
                    VariableOperator::PreDecrement => self.variables.update(&name, -1.0, true),
                    VariableOperator::PostIncrement => self.variables.update(&name, 1.0, false),
                    VariableOperator::PostDecrement => self.variables.update(&name, -1.0, false),
                    VariableOperator::None => self.variables.get_typed(&name),
                };
                eval.values.push(result);
                eval.index += 1;
            }
            ExpressionWork::PushTime => {
                eval.values
                    .push(CpuStepResult::int_value(context.time_left as f64));
                eval.index += 1;
            }
            ExpressionWork::PushRobotProperty(property) => {
                let value = property
                    .stored_ore_value(&context.ore)
                    .or_else(|| property.depot_value(&context.depot, &context.depot_capacity))
                    .or_else(|| property.value(&context.robot))
                    .ok_or(())?;
                eval.values
                    .push(CpuStepResult::for_robot_property(property, value));
                eval.index += 1;
            }
            ExpressionWork::PushAreaProperty(property) => {
                let value = property.value(&context.area);
                eval.values
                    .push(CpuStepResult::for_area_property(property, value));
                eval.index += 1;
            }
            // Deprecated: prefer robot.oreStored / robot.oreStoredA|B|C.
            ExpressionWork::PushOre => {
                let ore_type = eval.values.pop().ok_or(())?.value as i32;
                let amount = if ore_type == 0 {
                    context.ore.iter().sum::<i32>() as f64
                } else if ore_type > 0 {
                    context
                        .ore
                        .get((ore_type - 1) as usize)
                        .copied()
                        .unwrap_or(0) as f64
                } else {
                    0.0
                };
                eval.values.push(CpuStepResult::int_value(amount));
                eval.index += 1;
            }
            ExpressionWork::PushAction(action) => {
                let value = action_result.take().ok_or(())?;
                eval.values.push(CpuStepResult::for_action(action, value));
                eval.index += 1;
            }
            ExpressionWork::ApplyUnaryNot => {
                let value = eval.values.pop().ok_or(())?.value;
                eval.values
                    .push(CpuStepResult::bool_value(if value.is_truthy() {
                        0.0
                    } else {
                        1.0
                    }));
                eval.index += 1;
            }
            ExpressionWork::ApplyUnaryMinus => {
                let operand = eval.values.pop().ok_or(())?;
                let value = -operand.value;
                eval.values
                    .push(if operand.kind == CpuStepResultKind::Float {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    });
                eval.index += 1;
            }
            ExpressionWork::ApplyAbs => {
                let operand = eval.values.pop().ok_or(())?;
                let value = operand.value.abs();
                eval.values
                    .push(if operand.kind == CpuStepResultKind::Float {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    });
                eval.index += 1;
            }
            ExpressionWork::ApplySqrt => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.sqrt()));
                eval.index += 1;
            }
            ExpressionWork::ApplySin => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.to_radians().sin()));
                eval.index += 1;
            }
            ExpressionWork::ApplyCos => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.to_radians().cos()));
                eval.index += 1;
            }
            ExpressionWork::ApplyTan => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.to_radians().tan()));
                eval.index += 1;
            }
            ExpressionWork::ApplyMin => {
                let right = eval.values.pop().ok_or(())?;
                let left = eval.values.pop().ok_or(())?;
                let value = left.value.min(right.value);
                eval.values.push(
                    if matches!(left.kind, CpuStepResultKind::Float)
                        || matches!(right.kind, CpuStepResultKind::Float)
                    {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    },
                );
                eval.index += 1;
            }
            ExpressionWork::ApplyMax => {
                let right = eval.values.pop().ok_or(())?;
                let left = eval.values.pop().ok_or(())?;
                let value = left.value.max(right.value);
                eval.values.push(
                    if matches!(left.kind, CpuStepResultKind::Float)
                        || matches!(right.kind, CpuStepResultKind::Float)
                    {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    },
                );
                eval.index += 1;
            }
            ExpressionWork::ApplyBinary(operator) => {
                let right = eval.values.pop().ok_or(())?;
                let left = eval.values.pop().ok_or(())?;
                let value = evaluate_operator(operator, left.value, right.value);
                eval.values.push(CpuStepResult::for_binary_operator(
                    operator, left.kind, right.kind, value,
                ));
                eval.index += 1;
            }
            ExpressionWork::PushStartScan
            | ExpressionWork::PushDynamicMove
            | ExpressionWork::PushDynamicRotate
            | ExpressionWork::PushDynamicDump
            | ExpressionWork::PushOreDistance
            | ExpressionWork::PushOreType => return Err(()),
        }
        Ok(())
    }
}
