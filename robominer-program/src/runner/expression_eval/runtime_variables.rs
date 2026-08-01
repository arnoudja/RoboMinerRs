use std::collections::BTreeMap;

use crate::ast::ValueType;
use crate::cpu_step_result::CpuStepResult;

#[derive(Debug, Clone, PartialEq)]
struct RuntimeBinding {
    value: f64,
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

    pub(crate) fn declare(&mut self, name: String, value: f64, value_type: ValueType) {
        self.scopes
            .last_mut()
            .expect("runtime should always have a scope")
            .insert(name, RuntimeBinding { value, value_type });
    }

    pub(crate) fn get(&self, name: &str) -> f64 {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|binding| binding.value))
            .unwrap_or(0.0)
    }

    pub(crate) fn get_typed(&self, name: &str) -> CpuStepResult {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| {
                scope.get(name).map(|binding| {
                    CpuStepResult::from_value_type(binding.value_type, binding.value)
                })
            })
            .unwrap_or_else(|| CpuStepResult::int_value(0.0))
    }

    pub(crate) fn set(&mut self, name: &str, value: f64) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            if let Some(binding) = scope.get_mut(name) {
                binding.value = value;
            }
        } else {
            self.declare(name.to_owned(), value, ValueType::Int);
        }
    }

    pub(crate) fn update(&mut self, name: &str, delta: f64, return_updated: bool) -> CpuStepResult {
        let previous = self.get_typed(name);
        let updated = previous.value + delta;
        self.set(name, updated);
        if return_updated {
            CpuStepResult {
                kind: previous.kind,
                value: updated,
            }
        } else {
            previous
        }
    }

    /// Flattened visible bindings (outer→inner so inner shadows outer).
    pub(crate) fn snapshot(&self) -> BTreeMap<String, CpuStepResult> {
        let mut out = BTreeMap::new();
        for scope in &self.scopes {
            for (name, binding) in scope {
                out.insert(
                    name.clone(),
                    CpuStepResult::from_value_type(binding.value_type, binding.value),
                );
            }
        }
        out
    }
}
