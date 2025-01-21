use std::ops::Deref;

use crate::ir::{BackLink, Child, Link, Op, Parent, Root};

/// The nodes that can own Op nodes
#[derive(Default, Clone, Eq, Debug)]
pub enum Owner {
    Function(BackLink<Root>),
    Evaluator(BackLink<Root>),
    Accessor(BackLink<Op>),
    Boundary(BackLink<Op>),
    Vector(BackLink<Op>),
    Matrix(BackLink<Op>),
    Call(BackLink<Op>),
    Fold(BackLink<Op>),
    Add(BackLink<Op>),
    Sub(BackLink<Op>),
    Mul(BackLink<Op>),
    Enf(BackLink<Op>),
    For(BackLink<Op>),
    If(BackLink<Op>),
    #[default]
    None,
}

impl Parent for Owner {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        match self {
            Owner::Function(f) => f.children(),
            Owner::Evaluator(e) => e.children(),
            Owner::Enf(e) => e.children(),
            Owner::Boundary(b) => b.children(),
            Owner::Add(a) => a.children(),
            Owner::Sub(s) => s.children(),
            Owner::Mul(m) => m.children(),
            Owner::If(i) => i.children(),
            Owner::For(f) => f.children(),
            Owner::Call(c) => c.children(),
            Owner::Fold(f) => f.children(),
            Owner::Vector(v) => v.children(),
            Owner::Matrix(m) => m.children(),
            Owner::Accessor(a) => a.children(),
            Owner::None => Link::default(),
        }
    }
}

impl Child for Owner {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        match self {
            Owner::Function(_f) => Vec::default(),
            Owner::Evaluator(_e) => Vec::default(),
            Owner::Enf(e) => e.get_parents(),
            Owner::Boundary(b) => b.get_parents(),
            Owner::Add(a) => a.get_parents(),
            Owner::Sub(s) => s.get_parents(),
            Owner::Mul(m) => m.get_parents(),
            Owner::If(i) => i.get_parents(),
            Owner::For(f) => f.get_parents(),
            Owner::Call(c) => c.get_parents(),
            Owner::Fold(f) => f.get_parents(),
            Owner::Vector(v) => v.get_parents(),
            Owner::Matrix(m) => m.get_parents(),
            Owner::Accessor(a) => a.get_parents(),
            Owner::None => Vec::default(),
        }
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Owner::Function(_f) => (),
            Owner::Evaluator(_e) => (),
            Owner::Enf(e) => e.add_parent(parent),
            Owner::Boundary(b) => b.add_parent(parent),
            Owner::Add(a) => a.add_parent(parent),
            Owner::Sub(s) => s.add_parent(parent),
            Owner::Mul(m) => m.add_parent(parent),
            Owner::If(i) => i.add_parent(parent),
            Owner::For(f) => f.add_parent(parent),
            Owner::Call(c) => c.add_parent(parent),
            Owner::Fold(f) => f.add_parent(parent),
            Owner::Vector(v) => v.add_parent(parent),
            Owner::Matrix(m) => m.add_parent(parent),
            Owner::Accessor(a) => a.add_parent(parent),
            Owner::None => (),
        }
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Owner::Function(_f) => (),
            Owner::Evaluator(_e) => (),
            Owner::Enf(e) => e.remove_parent(parent),
            Owner::Boundary(b) => b.remove_parent(parent),
            Owner::Add(a) => a.remove_parent(parent),
            Owner::Sub(s) => s.remove_parent(parent),
            Owner::Mul(m) => m.remove_parent(parent),
            Owner::If(i) => i.remove_parent(parent),
            Owner::For(f) => f.remove_parent(parent),
            Owner::Call(c) => c.remove_parent(parent),
            Owner::Fold(f) => f.remove_parent(parent),
            Owner::Vector(v) => v.remove_parent(parent),
            Owner::Matrix(m) => m.remove_parent(parent),
            Owner::Accessor(a) => a.remove_parent(parent),
            Owner::None => (),
        }
    }
}

impl PartialEq for Owner {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Owner::Function(lhs), Owner::Function(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Evaluator(lhs), Owner::Evaluator(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Enf(lhs), Owner::Enf(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Boundary(lhs), Owner::Boundary(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Add(lhs), Owner::Add(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Sub(lhs), Owner::Sub(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Mul(lhs), Owner::Mul(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::If(lhs), Owner::If(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::For(lhs), Owner::For(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Call(lhs), Owner::Call(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Fold(lhs), Owner::Fold(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Vector(lhs), Owner::Vector(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Matrix(lhs), Owner::Matrix(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::Accessor(lhs), Owner::Accessor(rhs)) => lhs.to_link() == rhs.to_link(),
            (Owner::None, Owner::None) => true,
            _ => false,
        }
    }
}

impl std::hash::Hash for Owner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Owner::Function(f) => f.to_link().hash(state),
            Owner::Evaluator(e) => e.to_link().hash(state),
            Owner::Enf(e) => e.to_link().hash(state),
            Owner::Boundary(b) => b.to_link().hash(state),
            Owner::Add(a) => a.to_link().hash(state),
            Owner::Sub(s) => s.to_link().hash(state),
            Owner::Mul(m) => m.to_link().hash(state),
            Owner::If(i) => i.to_link().hash(state),
            Owner::For(f) => f.to_link().hash(state),
            Owner::Call(c) => c.to_link().hash(state),
            Owner::Fold(f) => f.to_link().hash(state),
            Owner::Vector(v) => v.to_link().hash(state),
            Owner::Matrix(m) => m.to_link().hash(state),
            Owner::Accessor(a) => a.to_link().hash(state),
            Owner::None => Owner::None.hash(state),
        }
    }
}

impl Link<Owner> {
    pub fn as_root(&self) -> Option<Link<Root>> {
        match self.borrow().deref() {
            Owner::Function(f) => f.to_link(),
            Owner::Evaluator(e) => e.to_link(),
            Owner::Accessor(_) => None,
            Owner::Boundary(_) => None,
            Owner::Vector(_) => None,
            Owner::Matrix(_) => None,
            Owner::Call(_) => None,
            Owner::Fold(_) => None,
            Owner::Add(_) => None,
            Owner::Sub(_) => None,
            Owner::Mul(_) => None,
            Owner::Enf(_) => None,
            Owner::For(_) => None,
            Owner::If(_) => None,
            Owner::None => None,
        }
    }
    pub fn as_op(&self) -> Option<Link<Op>> {
        match self.borrow().deref() {
            Owner::Function(_) => None,
            Owner::Evaluator(_) => None,
            Owner::Accessor(back) => back.to_link(),
            Owner::Boundary(back) => back.to_link(),
            Owner::Vector(back) => back.to_link(),
            Owner::Matrix(back) => back.to_link(),
            Owner::Call(back) => back.to_link(),
            Owner::Fold(back) => back.to_link(),
            Owner::Add(back) => back.to_link(),
            Owner::Sub(back) => back.to_link(),
            Owner::Mul(back) => back.to_link(),
            Owner::Enf(back) => back.to_link(),
            Owner::For(back) => back.to_link(),
            Owner::If(back) => back.to_link(),
            Owner::None => None,
        }
    }
}
