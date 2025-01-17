use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

use crate::ir::{get_inner, get_inner_mut, Evaluator, Function, Link, Node, Op, Parent};

/// The root nodes of the MIR Graph
/// These represent the top level functions and evaluators
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Root {
    Function(Function),
    Evaluator(Evaluator),
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
    pub fn as_node(&self) -> Link<Node> {
        match self.borrow().deref() {
            Root::Function(_) => Node::Function(self.clone().into()).into(),
            Root::Evaluator(_) => Node::Evaluator(self.clone().into()).into(),
            Root::None => Node::None.into(),
        }
    }
    pub fn as_function(&self) -> Option<Ref<Function>> {
        get_inner(self.borrow(), |root| match root {
            Root::Function(f) => Some(f),
            _ => None,
        })
    }
    pub fn as_function_mut(&self) -> Option<RefMut<Function>> {
        get_inner_mut(self.borrow_mut(), |root| match root {
            Root::Function(f) => Some(f),
            _ => None,
        })
    }
    pub fn as_evaluator(&self) -> Option<Ref<Evaluator>> {
        get_inner(self.borrow(), |root| match root {
            Root::Evaluator(e) => Some(e),
            _ => None,
        })
    }
    pub fn as_evaluator_mut(&self) -> Option<RefMut<Evaluator>> {
        get_inner_mut(self.borrow_mut(), |root| match root {
            Root::Evaluator(e) => Some(e),
            _ => None,
        })
    }
}
