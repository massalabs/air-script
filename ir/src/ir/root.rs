use std::ops::Deref;

use crate::ir::{Evaluator, Function, Link, Node, Op, Owner, Parent};

/// The root nodes of the MIR Graph
/// These represent the top level functions and evaluators
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Root {
    Function(Link<Function>),
    Evaluator(Link<Evaluator>),
    #[default]
    None,
}

impl Parent for Root {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        match self {
            Root::Function(f) => f.children(),
            Root::Evaluator(e) => e.children(),
            Root::None => Link::default(),
        }
    }
}

impl Link<Root> {
    pub fn as_function(self) -> Option<Link<Function>> {
        match self.borrow().deref() {
            Root::Function(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_evaluator(self) -> Option<Link<Evaluator>> {
        match self.borrow().deref() {
            Root::Evaluator(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn as_owner(self) -> Link<Owner> {
        match self.borrow().deref() {
            Root::Function(f) => Owner::Function(f.clone()).into(),
            Root::Evaluator(e) => Owner::Evaluator(e.clone()).into(),
            Root::None => Owner::None.into(),
        }
    }
    pub fn as_node(self) -> Link<Node> {
        match self.borrow().deref() {
            Root::Function(f) => Node::Function(f.clone()).into(),
            Root::Evaluator(e) => Node::Evaluator(e.clone()).into(),
            Root::None => Node::None.into(),
        }
    }
}
