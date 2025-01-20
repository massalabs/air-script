use std::{
    cell::{Ref, RefMut},
    ops::DerefMut,
};

use crate::ir::{
    get_inner, get_inner_mut, BackLink, Evaluator, Function, Link, Node, Op, Owner, Parent,
};

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
        let back: BackLink<Root> = self.clone().into();
        match self.borrow_mut().deref_mut() {
            Root::Function(Function {
                _node: Some(link), ..
            }) => link.clone(),
            Root::Function(ref mut f) => {
                let node: Link<Node> = Node::Function(back).into();
                f._node = Some(node.clone());
                node
            }
            Root::Evaluator(Evaluator {
                _node: Some(link), ..
            }) => link.clone(),
            Root::Evaluator(ref mut e) => {
                let node: Link<Node> = Node::Evaluator(back).into();
                e._node = Some(node.clone());
                node
            }
            Root::None => Node::None.into(),
        }
    }
    pub fn as_owner(&self) -> Link<Owner> {
        let back: BackLink<Root> = self.clone().into();
        match self.borrow_mut().deref_mut() {
            Root::Function(Function {
                _owner: Some(link), ..
            }) => link.clone(),
            Root::Function(ref mut f) => {
                let owner: Link<Owner> = Owner::Function(back).into();
                f._owner = Some(owner.clone());
                owner
            }
            Root::Evaluator(Evaluator {
                _owner: Some(link), ..
            }) => link.clone(),
            Root::Evaluator(ref mut e) => {
                let owner: Link<Owner> = Owner::Evaluator(back).into();
                e._owner = Some(owner.clone());
                owner
            }
            Root::None => Owner::None.into(),
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
