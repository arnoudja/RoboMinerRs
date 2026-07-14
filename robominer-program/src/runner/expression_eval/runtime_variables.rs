use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeVariables {
    scopes: Vec<BTreeMap<String, f64>>,
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

    pub(crate) fn declare(&mut self, name: String, value: f64) {
        self.scopes
            .last_mut()
            .expect("runtime should always have a scope")
            .insert(name, value);
    }

    pub(crate) fn get(&self, name: &str) -> f64 {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .unwrap_or(0.0)
    }

    pub(crate) fn set(&mut self, name: &str, value: f64) {
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.contains_key(name))
        {
            scope.insert(name.to_owned(), value);
        } else {
            self.declare(name.to_owned(), value);
        }
    }

    pub(crate) fn update(&mut self, name: &str, delta: f64, return_updated: bool) -> f64 {
        let previous = self.get(name);
        let updated = previous + delta;
        self.set(name, updated);
        if return_updated { updated } else { previous }
    }
}
