use crate::ir::{BackLink, Child, Op};

use super::{Link, Owner, Parent, Root};
use std::ops::Deref;

/// All the nodes that can be in the MIR Graph
#[derive(Default, Clone, Eq, Debug)]
pub enum Node {
    Function(BackLink<Root>),
    Evaluator(BackLink<Root>),
    Enf(BackLink<Op>),
    Boundary(BackLink<Op>),
    Add(BackLink<Op>),
    Sub(BackLink<Op>),
    Mul(BackLink<Op>),
    If(BackLink<Op>),
    For(BackLink<Op>),
    Call(BackLink<Op>),
    Fold(BackLink<Op>),
    Vector(BackLink<Op>),
    Matrix(BackLink<Op>),
    Accessor(BackLink<Op>),
    Parameter(BackLink<Op>),
    Value(BackLink<Op>),
    #[default]
    None,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Function(lhs), Node::Function(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Evaluator(lhs), Node::Evaluator(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Enf(lhs), Node::Enf(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Boundary(lhs), Node::Boundary(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Add(lhs), Node::Add(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Sub(lhs), Node::Sub(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Mul(lhs), Node::Mul(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::If(lhs), Node::If(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::For(lhs), Node::For(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Call(lhs), Node::Call(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Fold(lhs), Node::Fold(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Vector(lhs), Node::Vector(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Matrix(lhs), Node::Matrix(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Accessor(lhs), Node::Accessor(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Parameter(lhs), Node::Parameter(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::Value(lhs), Node::Value(rhs)) => lhs.to_link() == rhs.to_link(),
            (Node::None, Node::None) => true,
            _ => false,
        }
    }
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Node::Function(f) => f.to_link().hash(state),
            Node::Evaluator(e) => e.to_link().hash(state),
            Node::Enf(e) => e.to_link().hash(state),
            Node::Boundary(b) => b.to_link().hash(state),
            Node::Add(a) => a.to_link().hash(state),
            Node::Sub(s) => s.to_link().hash(state),
            Node::Mul(m) => m.to_link().hash(state),
            Node::If(i) => i.to_link().hash(state),
            Node::For(f) => f.to_link().hash(state),
            Node::Call(c) => c.to_link().hash(state),
            Node::Fold(f) => f.to_link().hash(state),
            Node::Vector(v) => v.to_link().hash(state),
            Node::Matrix(m) => m.to_link().hash(state),
            Node::Accessor(a) => a.to_link().hash(state),
            Node::Parameter(p) => p.to_link().hash(state),
            Node::Value(v) => v.to_link().hash(state),
            Node::None => Node::None.hash(state),
        }
    }
}

impl Parent for Node {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        match self {
            Node::Function(f) => f.children(),
            Node::Evaluator(e) => e.children(),
            Node::Enf(e) => e.children(),
            Node::Boundary(b) => b.children(),
            Node::Add(a) => a.children(),
            Node::Sub(s) => s.children(),
            Node::Mul(m) => m.children(),
            Node::If(i) => i.children(),
            Node::For(f) => f.children(),
            Node::Call(c) => c.children(),
            Node::Fold(f) => f.children(),
            Node::Vector(v) => v.children(),
            Node::Matrix(m) => m.children(),
            Node::Accessor(a) => a.children(),
            Node::Parameter(_p) => Link::default(),
            Node::Value(_v) => Link::default(),
            Node::None => Link::default(),
        }
    }
}

impl Child for Node {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        match self {
            Node::Function(_f) => Vec::default(),
            Node::Evaluator(_e) => Vec::default(),
            Node::Enf(e) => e.get_parents(),
            Node::Boundary(b) => b.get_parents(),
            Node::Add(a) => a.get_parents(),
            Node::Sub(s) => s.get_parents(),
            Node::Mul(m) => m.get_parents(),
            Node::If(i) => i.get_parents(),
            Node::For(f) => f.get_parents(),
            Node::Call(c) => c.get_parents(),
            Node::Fold(f) => f.get_parents(),
            Node::Vector(v) => v.get_parents(),
            Node::Matrix(m) => m.get_parents(),
            Node::Accessor(a) => a.get_parents(),
            Node::Parameter(p) => p.get_parents(),
            Node::Value(v) => v.get_parents(),
            Node::None => Vec::default(),
        }
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Node::Function(_f) => (),
            Node::Evaluator(_e) => (),
            Node::Enf(e) => e.add_parent(parent),
            Node::Boundary(b) => b.add_parent(parent),
            Node::Add(a) => a.add_parent(parent),
            Node::Sub(s) => s.add_parent(parent),
            Node::Mul(m) => m.add_parent(parent),
            Node::If(i) => i.add_parent(parent),
            Node::For(f) => f.add_parent(parent),
            Node::Call(c) => c.add_parent(parent),
            Node::Fold(f) => f.add_parent(parent),
            Node::Vector(v) => v.add_parent(parent),
            Node::Matrix(m) => m.add_parent(parent),
            Node::Accessor(a) => a.add_parent(parent),
            Node::Parameter(p) => p.add_parent(parent),
            Node::Value(v) => v.add_parent(parent),
            Node::None => {}
        }
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Node::Function(_f) => (),
            Node::Evaluator(_e) => (),
            Node::Enf(e) => e.remove_parent(parent),
            Node::Boundary(b) => b.remove_parent(parent),
            Node::Add(a) => a.remove_parent(parent),
            Node::Sub(s) => s.remove_parent(parent),
            Node::Mul(m) => m.remove_parent(parent),
            Node::If(i) => i.remove_parent(parent),
            Node::For(f) => f.remove_parent(parent),
            Node::Call(c) => c.remove_parent(parent),
            Node::Fold(f) => f.remove_parent(parent),
            Node::Vector(v) => v.remove_parent(parent),
            Node::Matrix(m) => m.remove_parent(parent),
            Node::Accessor(a) => a.remove_parent(parent),
            Node::Parameter(p) => p.remove_parent(parent),
            Node::Value(v) => v.remove_parent(parent),
            Node::None => {}
        }
    }
}

impl Link<Node> {
    pub fn debug(&self) -> String {
        match self.as_root() {
            Some(root) => format!("Node::Root({})", root.debug()),
            None => match self.as_op() {
                Some(op) => format!("Node::Op({})", op.debug()),
                None => "Node::None".to_string(),
            },
        }
    }
    pub fn as_root(&self) -> Option<Link<Root>> {
        match self.borrow().deref() {
            Node::Function(f) => f.to_link(),
            Node::Evaluator(e) => e.to_link(),
            Node::Enf(_) => None,
            Node::Boundary(_) => None,
            Node::Add(_) => None,
            Node::Sub(_) => None,
            Node::Mul(_) => None,
            Node::If(_) => None,
            Node::For(_) => None,
            Node::Call(_) => None,
            Node::Fold(_) => None,
            Node::Vector(_) => None,
            Node::Matrix(_) => None,
            Node::Accessor(_) => None,
            Node::Parameter(_) => None,
            Node::Value(_) => None,
            Node::None => None,
        }
    }
    pub fn as_op(&self) -> Option<Link<Op>> {
        match self.borrow().deref() {
            Node::Function(_) => None,
            Node::Evaluator(_) => None,
            Node::Enf(inner) => inner.to_link(),
            Node::Boundary(inner) => inner.to_link(),
            Node::Add(inner) => inner.to_link(),
            Node::Sub(inner) => inner.to_link(),
            Node::Mul(inner) => inner.to_link(),
            Node::If(inner) => inner.to_link(),
            Node::For(inner) => inner.to_link(),
            Node::Call(inner) => inner.to_link(),
            Node::Fold(inner) => inner.to_link(),
            Node::Vector(inner) => inner.to_link(),
            Node::Matrix(inner) => inner.to_link(),
            Node::Accessor(inner) => inner.to_link(),
            Node::Parameter(inner) => inner.to_link(),
            Node::Value(inner) => inner.to_link(),
            Node::None => None,
        }
    }
    pub fn as_owner(&self) -> Option<Link<Owner>> {
        match self.as_root() {
            Some(root) => Some(root.as_owner()),
            None => self.as_op().and_then(|op| op.as_owner()),
        }
    }
}
