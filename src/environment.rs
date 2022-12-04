use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::RuntimeError, token::Literal};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    /// Creates a new environment with no enclosing environment.
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    /// Creates a new environment with the given enclosing environment.
    pub fn new_enclosed(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    /// Define a variable in the environment.
    pub fn define(&mut self, name: String, value: Literal) {
        // Allow redefinition of variables.
        //
        // "When in doubt, do what Scheme does."
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.define(name.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }

    /// Lookup a variable in the environment.
    pub fn get(&self, name: &str) -> Result<Literal, RuntimeError> {
        if let Some(value) = self.values.get(name).cloned() {
            Ok(value)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }
}
