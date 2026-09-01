use std::collections::BTreeMap;

use crate::ast::ValueType;
use crate::cpu_step_result::CpuStepResult;
use crate::program_value::{ProgramValue, coerce_to_value_type};

#[derive(Debug, Clone, PartialEq)]
struct RuntimeBinding {
    value: ProgramValue,
    /// Display/type kind from declaration; sticky across later assigns/updates.
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

    pub(crate) fn declare(&mut self, name: String, value: ProgramValue, value_type: ValueType) {
        if self.scopes.is_empty() {
            self.scopes.push(BTreeMap::new());
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name,
                RuntimeBinding {
                    value: coerce_to_value_type(value, value_type),
                    value_type,
                },
            );
        }
    }

    pub(crate) fn declare_default(&mut self, name: String, value_type: ValueType) {
        self.declare(name, ProgramValue::default_for_type(value_type), value_type);
    }

    pub(crate) fn get(&self, name: &str) -> f64 {
        self.get_typed(name).wire_value()
    }

    pub(crate) fn get_typed(&self, name: &str) -> CpuStepResult {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| {
                scope.get(name).map(|binding| CpuStepResult {
                    value: binding.value,
                })
            })
            .unwrap_or_else(|| CpuStepResult::int_value(0))
    }

    pub(crate) fn set(&mut self, name: &str, value: ProgramValue) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            if let Some(binding) = scope.get_mut(name) {
                binding.value = coerce_to_value_type(value, binding.value_type);
            }
        } else {
            self.declare(name.to_owned(), value, ValueType::Int);
        }
    }

    pub(crate) fn update(&mut self, name: &str, delta: i32, return_updated: bool) -> CpuStepResult {
        let previous = self.get_typed(name);
        let updated = match previous.value {
            ProgramValue::Int(value) => ProgramValue::Int(value.wrapping_add(delta)),
            ProgramValue::Float(value) => ProgramValue::Float(value + f64::from(delta)),
            ProgramValue::Bool(value) => ProgramValue::Int(i32::from(value).wrapping_add(delta)),
        };
        self.set(name, updated);
        if return_updated {
            CpuStepResult::from_program_value(updated)
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
                    CpuStepResult {
                        value: binding.value,
                    },
                );
            }
        }
        out
    }
}
