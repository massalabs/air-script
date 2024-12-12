use std::ops::Deref;

use crate::ir::{Evaluator, Function, Link, Owner};

use super::Node;

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
    pub fn as_node(self) -> Node {
        match self {
            Root::Function(f) => Node::Function(f),
            Root::Evaluator(e) => Node::Evaluator(e),
            Root::None => Node::None,
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
    pub fn as_node(self) -> Link<Node> {
        self.borrow().deref().clone().as_node().into()
    }
}
