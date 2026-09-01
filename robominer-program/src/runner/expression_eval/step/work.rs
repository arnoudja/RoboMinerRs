use crate::cpu_step_result::CpuStepResult;
use crate::program_value::{ProgramValue, evaluate_binary_operator};
use crate::runner::ExecutableRunner;
use crate::runner::expression_eval::schedule::ExpressionWork;
use crate::types::*;

use super::OngoingExpressionEval;

impl ExecutableRunner {
    pub(super) fn apply_expression_work(
        &mut self,
        context: &ExecutionContext,
        action_result: &mut Option<f64>,
        work: ExpressionWork,
    ) -> Result<(), ()> {
        match work {
            ExpressionWork::PushNumber(value) => self.push_number_work(value),
            ExpressionWork::PushBool(value) => self.push_bool_work(value),
            ExpressionWork::PushVariable(name) => self.push_variable_work(&name),
            ExpressionWork::PushVariableUpdate { name, operator } => {
                self.push_variable_update_work(&name, operator)
            }
            ExpressionWork::PushTime => self.push_time_work(context),
            ExpressionWork::PushRobotProperty(property) => {
                self.push_robot_property_work(context, property)
            }
            ExpressionWork::PushAreaProperty(property) => {
                self.push_area_property_work(context, property)
            }
            ExpressionWork::PushOre => self.push_ore_work(context),
            ExpressionWork::PushAction(action) => self.push_action_work(action_result, action),
            ExpressionWork::ApplyUnaryNot => self.apply_unary_not_work(),
            ExpressionWork::ApplyUnaryMinus => self.apply_unary_minus_work(),
            ExpressionWork::ApplyAbs => self.apply_abs_work(),
            ExpressionWork::ApplySqrt => self.apply_sqrt_work(),
            ExpressionWork::ApplySin => self.apply_sin_work(),
            ExpressionWork::ApplyCos => self.apply_cos_work(),
            ExpressionWork::ApplyTan => self.apply_tan_work(),
            ExpressionWork::ApplyMin => self.apply_min_work(),
            ExpressionWork::ApplyMax => self.apply_max_work(),
            ExpressionWork::ApplyBinary(operator) => self.apply_binary_work(operator),
            ExpressionWork::PushStartScan
            | ExpressionWork::PushDynamicMove
            | ExpressionWork::PushDynamicRotate
            | ExpressionWork::PushDynamicDump
            | ExpressionWork::PushOreDistance
            | ExpressionWork::PushOreType => Err(()),
        }
    }

    fn eval_mut(&mut self) -> Result<&mut OngoingExpressionEval, ()> {
        self.expression_eval.as_mut().ok_or(())
    }

    fn push_number_work(&mut self, value: f64) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        eval.values.push(CpuStepResult::for_number_literal(value));
        eval.index += 1;
        Ok(())
    }

    fn push_bool_work(&mut self, value: bool) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        eval.values.push(CpuStepResult::bool_value(value));
        eval.index += 1;
        Ok(())
    }

    fn push_variable_work(&mut self, name: &str) -> Result<(), ()> {
        let typed = self.variables.get_typed(name);
        let eval = self.eval_mut()?;
        eval.values.push(typed);
        eval.index += 1;
        Ok(())
    }

    fn push_variable_update_work(
        &mut self,
        name: &str,
        operator: VariableOperator,
    ) -> Result<(), ()> {
        let result = match operator {
            VariableOperator::PreIncrement => self.variables.update(name, 1, true),
            VariableOperator::PreDecrement => self.variables.update(name, -1, true),
            VariableOperator::PostIncrement => self.variables.update(name, 1, false),
            VariableOperator::PostDecrement => self.variables.update(name, -1, false),
            VariableOperator::None => self.variables.get_typed(name),
        };
        let eval = self.eval_mut()?;
        eval.values.push(result);
        eval.index += 1;
        Ok(())
    }

    fn push_time_work(&mut self, context: &ExecutionContext) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        eval.values
            .push(CpuStepResult::int_value(context.time_left));
        eval.index += 1;
        Ok(())
    }

    fn push_robot_property_work(
        &mut self,
        context: &ExecutionContext,
        property: RobotProperty,
    ) -> Result<(), ()> {
        let value = property
            .stored_ore_value(&context.ore)
            .or_else(|| property.depot_value(&context.depot, &context.depot_capacity))
            .or_else(|| property.value(&context.robot))
            .ok_or(())?;
        let eval = self.eval_mut()?;
        eval.values
            .push(CpuStepResult::for_robot_property(property, value));
        eval.index += 1;
        Ok(())
    }

    fn push_area_property_work(
        &mut self,
        context: &ExecutionContext,
        property: AreaProperty,
    ) -> Result<(), ()> {
        let value = property.value(&context.area);
        let eval = self.eval_mut()?;
        eval.values
            .push(CpuStepResult::for_area_property(property, value));
        eval.index += 1;
        Ok(())
    }

    fn push_ore_work(&mut self, context: &ExecutionContext) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let ore_type = eval.values.pop().ok_or(())?.value.as_i32().unwrap_or(0);
        let amount = if ore_type == 0 {
            context.ore.iter().sum::<i32>()
        } else if ore_type > 0 {
            context
                .ore
                .get((ore_type - 1) as usize)
                .copied()
                .unwrap_or(0)
        } else {
            0
        };
        eval.values.push(CpuStepResult::int_value(amount));
        eval.index += 1;
        Ok(())
    }

    fn push_action_work(
        &mut self,
        action_result: &mut Option<f64>,
        action: ExecutableAction,
    ) -> Result<(), ()> {
        let value = action_result.take().ok_or(())?;
        let eval = self.eval_mut()?;
        eval.values.push(CpuStepResult::for_action(action, value));
        eval.index += 1;
        Ok(())
    }

    fn apply_unary_not_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let value = eval.values.pop().ok_or(())?.value;
        eval.values
            .push(CpuStepResult::bool_value(!value.is_truthy()));
        eval.index += 1;
        Ok(())
    }

    fn apply_unary_minus_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let operand = eval.values.pop().ok_or(())?;
        let value = match operand.value {
            ProgramValue::Float(value) => CpuStepResult::float_value(-value),
            ProgramValue::Int(value) => CpuStepResult::int_value(value.wrapping_neg()),
            ProgramValue::Bool(value) => CpuStepResult::int_value(-i32::from(value)),
        };
        eval.values.push(value);
        eval.index += 1;
        Ok(())
    }

    fn apply_abs_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let operand = eval.values.pop().ok_or(())?;
        let value = match operand.value {
            ProgramValue::Float(value) => CpuStepResult::float_value(value.abs()),
            ProgramValue::Int(value) => CpuStepResult::int_value(value.abs()),
            ProgramValue::Bool(value) => CpuStepResult::int_value(i32::from(value)),
        };
        eval.values.push(value);
        eval.index += 1;
        Ok(())
    }

    fn apply_sqrt_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let operand = eval.values.pop().ok_or(())?;
        eval.values
            .push(CpuStepResult::float_value(operand.value.as_f64().sqrt()));
        eval.index += 1;
        Ok(())
    }

    fn apply_sin_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let operand = eval.values.pop().ok_or(())?;
        eval.values.push(CpuStepResult::float_value(
            operand.value.as_f64().to_radians().sin(),
        ));
        eval.index += 1;
        Ok(())
    }

    fn apply_cos_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let operand = eval.values.pop().ok_or(())?;
        eval.values.push(CpuStepResult::float_value(
            operand.value.as_f64().to_radians().cos(),
        ));
        eval.index += 1;
        Ok(())
    }

    fn apply_tan_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let operand = eval.values.pop().ok_or(())?;
        eval.values.push(CpuStepResult::float_value(
            operand.value.as_f64().to_radians().tan(),
        ));
        eval.index += 1;
        Ok(())
    }

    fn apply_min_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let right = eval.values.pop().ok_or(())?;
        let left = eval.values.pop().ok_or(())?;
        eval.values
            .push(promote_min_max(left.value, right.value, true));
        eval.index += 1;
        Ok(())
    }

    fn apply_max_work(&mut self) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let right = eval.values.pop().ok_or(())?;
        let left = eval.values.pop().ok_or(())?;
        eval.values
            .push(promote_min_max(left.value, right.value, false));
        eval.index += 1;
        Ok(())
    }

    fn apply_binary_work(&mut self, operator: Operator) -> Result<(), ()> {
        let eval = self.eval_mut()?;
        let right = eval.values.pop().ok_or(())?.value;
        let left = eval.values.pop().ok_or(())?.value;
        let value = evaluate_binary_operator(operator, left, right);
        eval.values
            .push(CpuStepResult::for_binary_operator(operator, value));
        eval.index += 1;
        Ok(())
    }
}

fn promote_min_max(left: ProgramValue, right: ProgramValue, min: bool) -> CpuStepResult {
    if matches!(left, ProgramValue::Float(_)) || matches!(right, ProgramValue::Float(_)) {
        let left = left.as_f64();
        let right = right.as_f64();
        CpuStepResult::float_value(if min {
            left.min(right)
        } else {
            left.max(right)
        })
    } else {
        let left = left.as_i32().unwrap_or(0);
        let right = right.as_i32().unwrap_or(0);
        CpuStepResult::int_value(if min {
            left.min(right)
        } else {
            left.max(right)
        })
    }
}
