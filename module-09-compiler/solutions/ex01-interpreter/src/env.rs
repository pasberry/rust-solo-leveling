use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    pub(crate) store: HashMap<String, Value>,
    pub(crate) outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn with_outer(outer: Environment) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.store.get(name).cloned().or_else(|| {
            self.outer.as_ref().and_then(|env| env.get(name))
        })
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.store.insert(name, value);
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
