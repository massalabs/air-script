use std::ops::Deref;

use crate::ir3::{Evaluator, Function, Link, Owner};

/// The root nodes of the MIR Graph
/// These represent the top level functions and evaluators
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Root {
    Function(Function),
    Evaluator(Evaluator),
    #[default]
    None,
}

impl Root {
    pub fn as_function(self) -> Option<Function> {
        match self {
            Root::Function(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_evaluator(self) -> Option<Evaluator> {
        match self {
            Root::Evaluator(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_owner(self) -> Owner {
        match self {
            Root::Function(f) => Owner::Function(f.clone()),
            Root::Evaluator(e) => Owner::Evaluator(e.clone()),
            Root::None => Owner::None,
        }
    }
}

impl Link<Root> {
    pub fn as_function(self) -> Option<Link<Function>> {
        self.borrow()
            .deref()
            .clone()
            .as_function()
            .map(|f| f.into())
    }
    pub fn as_evaluator(self) -> Option<Link<Evaluator>> {
        self.borrow()
            .deref()
            .clone()
            .as_evaluator()
            .map(|e| e.into())
    }
    pub fn as_owner(self) -> Link<Owner> {
        self.borrow().deref().clone().as_owner().into()
    }
}
