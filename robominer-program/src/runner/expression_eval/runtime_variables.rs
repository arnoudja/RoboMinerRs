use std::collections::BTreeMap;

use crate::ast::ValueType;
use crate::cpu_step_result::CpuStepResult;

#[derive(Debug, Clone, PartialEq)]
struct RuntimeBinding {
    value: CpuStepResult,
    /// Declaration type; sticky across later assigns (values are coerced on write).
    value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeVariables {
    scopes: Vec<BTreeMap<String, RuntimeBinding>>,
}

impl Default for RuntimeVariables {
    fn default() -> Self {
        Self {
            scopes: vec![BTreeMap::new()],
        }
    }
}

impl RuntimeVariables {
    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub(crate) fn declare(&mut self, name: String, value: CpuStepResult, value_type: ValueType) {
        if self.scopes.is_empty() {
            self.scopes.push(BTreeMap::new());
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name,
                RuntimeBinding {
                    value: value.coerce_to(value_type),
                    value_type,
                },
            );
        }
    }

    pub(crate) fn get_typed(&self, name: &str) -> CpuStepResult {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|binding| binding.value))
            .unwrap_or(CpuStepResult::Int(0))
    }

    pub(crate) fn set(&mut self, name: &str, value: CpuStepResult) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            if let Some(binding) = scope.get_mut(name) {
                binding.value = value.coerce_to(binding.value_type);
            }
        } else {
            self.declare(name.to_owned(), value, ValueType::Int);
        }
    }

    pub(crate) fn update(&mut self, name: &str, delta: i64, return_updated: bool) -> CpuStepResult {
        let previous = self.get_typed(name);
        let value_type = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|binding| binding.value_type))
            .unwrap_or(ValueType::Int);
        let updated = match value_type {
            ValueType::Double => CpuStepResult::Float(previous.as_f64() + delta as f64),
            ValueType::Int | ValueType::Bool => {
                CpuStepResult::Int(previous.as_i64().wrapping_add(delta))
            }
        };
        self.set(name, updated);
        if return_updated {
            self.get_typed(name)
        } else {
            previous
        }
    }

    /// Flattened visible bindings (outer→inner so inner shadows outer).
    pub(crate) fn snapshot(&self) -> BTreeMap<String, CpuStepResult> {
        let mut out = BTreeMap::new();
        for scope in &self.scopes {
            for (name, binding) in scope {
                out.insert(name.clone(), binding.value);
            }
        }
        out
    }
}
