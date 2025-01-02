use std::ops::Deref;

use crate::ir::{
    Accessor, Add, BackLink, Boundary, Call, Child, Enf, Fold, For, If, Link, Matrix, Mul, Owner,
    Parameter, Parent, Sub, Value, Vector,
};

use super::Node;

/// The combined Operators and Leaves of the MIR Graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Op {
    Enf(Link<Enf>),
    Boundary(Link<Boundary>),
    Add(Link<Add>),
    Sub(Link<Sub>),
    Mul(Link<Mul>),
    If(Link<If>),
    For(Link<For>),
    Call(Link<Call>),
    Fold(Link<Fold>),
    Vector(Link<Vector>),
    Matrix(Link<Matrix>),
    Accessor(Link<Accessor>),
    Parameter(Link<Parameter>),
    Value(Link<Value>),
    #[default]
    None,
}

impl Parent for Op {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        match self {
            Op::Enf(e) => e.children(),
            Op::Boundary(b) => b.children(),
            Op::Add(a) => a.children(),
            Op::Sub(s) => s.children(),
            Op::Mul(m) => m.children(),
            Op::If(i) => i.children(),
            Op::For(f) => f.children(),
            Op::Call(c) => c.children(),
            Op::Fold(f) => f.children(),
            Op::Vector(v) => v.children(),
            Op::Matrix(m) => m.children(),
            Op::Accessor(a) => a.children(),
            Op::Parameter(_p) => Link::default(),
            Op::Value(_v) => Link::default(),
            Op::None => Link::default(),
        }
    }
}

impl Child for Op {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        match self {
            Op::Enf(e) => e.get_parents(),
            Op::Boundary(b) => b.get_parents(),
            Op::Add(a) => a.get_parents(),
            Op::Sub(s) => s.get_parents(),
            Op::Mul(m) => m.get_parents(),
            Op::If(i) => i.get_parents(),
            Op::For(f) => f.get_parents(),
            Op::Call(c) => c.get_parents(),
            Op::Fold(f) => f.get_parents(),
            Op::Vector(v) => v.get_parents(),
            Op::Matrix(m) => m.get_parents(),
            Op::Accessor(a) => a.get_parents(),
            Op::Parameter(p) => p.get_parents(),
            Op::Value(v) => v.get_parents(),
            Op::None => Vec::default(),
        }
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Op::Enf(e) => e.add_parent(parent),
            Op::Boundary(b) => b.add_parent(parent),
            Op::Add(a) => a.add_parent(parent),
            Op::Sub(s) => s.add_parent(parent),
            Op::Mul(m) => m.add_parent(parent),
            Op::If(i) => i.add_parent(parent),
            Op::For(f) => f.add_parent(parent),
            Op::Call(c) => c.add_parent(parent),
            Op::Fold(f) => f.add_parent(parent),
            Op::Vector(v) => v.add_parent(parent),
            Op::Matrix(m) => m.add_parent(parent),
            Op::Accessor(a) => a.add_parent(parent),
            Op::Parameter(p) => p.add_parent(parent),
            Op::Value(v) => v.add_parent(parent),
            Op::None => {}
        }
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Op::Enf(e) => e.remove_parent(parent),
            Op::Boundary(b) => b.remove_parent(parent),
            Op::Add(a) => a.remove_parent(parent),
            Op::Sub(s) => s.remove_parent(parent),
            Op::Mul(m) => m.remove_parent(parent),
            Op::If(i) => i.remove_parent(parent),
            Op::For(f) => f.remove_parent(parent),
            Op::Call(c) => c.remove_parent(parent),
            Op::Fold(f) => f.remove_parent(parent),
            Op::Vector(v) => v.remove_parent(parent),
            Op::Matrix(m) => m.remove_parent(parent),
            Op::Accessor(a) => a.remove_parent(parent),
            Op::Parameter(p) => p.remove_parent(parent),
            Op::Value(v) => v.remove_parent(parent),
            Op::None => {}
        }
    }
}

impl Link<Op> {
    pub fn as_enf(self) -> Option<Link<Enf>> {
        match self.borrow().deref() {
            Op::Enf(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn as_boundary(self) -> Option<Link<Boundary>> {
        match self.borrow().deref() {
            Op::Boundary(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn as_add(self) -> Option<Link<Add>> {
        match self.borrow().deref() {
            Op::Add(a) => Some(a.clone()),
            _ => None,
        }
    }
    pub fn as_sub(self) -> Option<Link<Sub>> {
        match self.borrow().deref() {
            Op::Sub(s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn as_mul(self) -> Option<Link<Mul>> {
        match self.borrow().deref() {
            Op::Mul(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_if(self) -> Option<Link<If>> {
        match self.borrow().deref() {
            Op::If(i) => Some(i.clone()),
            _ => None,
        }
    }
    pub fn as_for(self) -> Option<Link<For>> {
        match self.borrow().deref() {
            Op::For(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_call(self) -> Option<Link<Call>> {
        match self.borrow().deref() {
            Op::Call(c) => Some(c.clone()),
            _ => None,
        }
    }
    pub fn as_fold(self) -> Option<Link<Fold>> {
        match self.borrow().deref() {
            Op::Fold(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_vector(self) -> Option<Link<Vector>> {
        match self.borrow().deref() {
            Op::Vector(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub fn as_matrix(self) -> Option<Link<Matrix>> {
        match self.borrow().deref() {
            Op::Matrix(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_index_access(self) -> Option<Link<Accessor>> {
        match self.borrow().deref() {
            Op::Accessor(a) => Some(a.clone()),
            _ => None,
        }
    }
    pub fn as_parameter(self) -> Option<Link<Parameter>> {
        match self.borrow().deref() {
            Op::Parameter(p) => Some(p.clone()),
            _ => None,
        }
    }
    pub fn as_value(self) -> Option<Link<Value>> {
        match self.borrow().deref() {
            Op::Value(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub fn as_owner(self) -> Option<Link<Owner>> {
        match self.borrow().deref() {
            Op::Enf(e) => Some(Owner::Enf(e.clone()).into()),
            Op::Boundary(b) => Some(Owner::Boundary(b.clone()).into()),
            Op::Add(a) => Some(Owner::Add(a.clone()).into()),
            Op::Sub(s) => Some(Owner::Sub(s.clone()).into()),
            Op::Mul(m) => Some(Owner::Mul(m.clone()).into()),
            Op::If(i) => Some(Owner::If(i.clone()).into()),
            Op::For(f) => Some(Owner::For(f.clone()).into()),
            Op::Call(c) => Some(Owner::Call(c.clone()).into()),
            Op::Fold(f) => Some(Owner::Fold(f.clone()).into()),
            Op::Vector(v) => Some(Owner::Vector(v.clone()).into()),
            Op::Matrix(m) => Some(Owner::Matrix(m.clone()).into()),
            Op::Accessor(a) => Some(Owner::Accessor(a.clone()).into()),
            Op::Parameter(_p) => None,
            Op::Value(_v) => None,
            Op::None => None,
        }
    }
    pub fn as_node(self) -> Link<Node> {
        match self.borrow().deref() {
            Op::Enf(e) => Node::Enf(e.clone()).into(),
            Op::Boundary(b) => Node::Boundary(b.clone()).into(),
            Op::Add(a) => Node::Add(a.clone()).into(),
            Op::Sub(s) => Node::Sub(s.clone()).into(),
            Op::Mul(m) => Node::Mul(m.clone()).into(),
            Op::If(i) => Node::If(i.clone()).into(),
            Op::For(f) => Node::For(f.clone()).into(),
            Op::Call(c) => Node::Call(c.clone()).into(),
            Op::Fold(f) => Node::Fold(f.clone()).into(),
            Op::Vector(v) => Node::Vector(v.clone()).into(),
            Op::Matrix(m) => Node::Matrix(m.clone()).into(),
            Op::Accessor(a) => Node::Accessor(a.clone()).into(),
            Op::Parameter(p) => Node::Parameter(p.clone()).into(),
            Op::Value(v) => Node::Value(v.clone()).into(),
            Op::None => Node::None.into(),
        }
    }
}
