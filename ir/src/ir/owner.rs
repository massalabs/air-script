use crate::ir::{
    Accessor, Add, BackLink, Boundary, Call, Child, Enf, Evaluator, Fold, For, Function, If, Link,
    Matrix, Mul, Node, Op, Parent, Sub, Vector,
};
use std::ops::Deref;

/// The nodes that can own Op nodes
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Owner {
    Function(Link<Function>),
    Evaluator(Link<Evaluator>),
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
    fn get_parent(&self) -> BackLink<Self::Parent> {
        match self {
            Owner::Function(f) => BackLink::default(),
            Owner::Evaluator(e) => BackLink::default(),
            Owner::Enf(e) => e.get_parent(),
            Owner::Boundary(b) => b.get_parent(),
            Owner::Add(a) => a.get_parent(),
            Owner::Sub(s) => s.get_parent(),
            Owner::Mul(m) => m.get_parent(),
            Owner::If(i) => i.get_parent(),
            Owner::For(f) => f.get_parent(),
            Owner::Call(c) => c.get_parent(),
            Owner::Fold(f) => f.get_parent(),
            Owner::Vector(v) => v.get_parent(),
            Owner::Matrix(m) => m.get_parent(),
            Owner::Accessor(a) => a.get_parent(),
            Owner::None => BackLink::default(),
        }
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Owner::Function(f) => (),
            Owner::Evaluator(e) => (),
            Owner::Enf(e) => e.set_parent(parent),
            Owner::Boundary(b) => b.set_parent(parent),
            Owner::Add(a) => a.set_parent(parent),
            Owner::Sub(s) => s.set_parent(parent),
            Owner::Mul(m) => m.set_parent(parent),
            Owner::If(i) => i.set_parent(parent),
            Owner::For(f) => f.set_parent(parent),
            Owner::Call(c) => c.set_parent(parent),
            Owner::Fold(f) => f.set_parent(parent),
            Owner::Vector(v) => v.set_parent(parent),
            Owner::Matrix(m) => m.set_parent(parent),
            Owner::Accessor(a) => a.set_parent(parent),
            Owner::None => (),
        }
    }
}

impl Link<Owner> {
    pub fn as_function(self) -> Option<Link<Function>> {
        match self.borrow().deref() {
            Owner::Function(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_evaluator(self) -> Option<Link<Evaluator>> {
        match self.borrow().deref() {
            Owner::Evaluator(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn as_enf(self) -> Option<Link<Enf>> {
        match self.borrow().deref() {
            Owner::Enf(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn as_boundary(self) -> Option<Link<Boundary>> {
        match self.borrow().deref() {
            Owner::Boundary(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn as_add(self) -> Option<Link<Add>> {
        match self.borrow().deref() {
            Owner::Add(a) => Some(a.clone()),
            _ => None,
        }
    }
    pub fn as_sub(self) -> Option<Link<Sub>> {
        match self.borrow().deref() {
            Owner::Sub(s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn as_mul(self) -> Option<Link<Mul>> {
        match self.borrow().deref() {
            Owner::Mul(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_if(self) -> Option<Link<If>> {
        match self.borrow().deref() {
            Owner::If(i) => Some(i.clone()),
            _ => None,
        }
    }
    pub fn as_for(self) -> Option<Link<For>> {
        match self.borrow().deref() {
            Owner::For(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_call(self) -> Option<Link<Call>> {
        match self.borrow().deref() {
            Owner::Call(c) => Some(c.clone()),
            _ => None,
        }
    }
    pub fn as_fold(self) -> Option<Link<Fold>> {
        match self.borrow().deref() {
            Owner::Fold(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_vector(self) -> Option<Link<Vector>> {
        match self.borrow().deref() {
            Owner::Vector(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub fn as_matrix(self) -> Option<Link<Matrix>> {
        match self.borrow().deref() {
            Owner::Matrix(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_accessor(self) -> Option<Link<Accessor>> {
        match self.borrow().deref() {
            Owner::Accessor(a) => Some(a.clone()),
            _ => None,
        }
    }
    pub fn as_op(self) -> Option<Link<Op>> {
        match self.borrow().deref() {
            Owner::Function(f) => None,
            Owner::Evaluator(e) => None,
            Owner::Enf(e) => Some(Op::Enf(e.clone()).into()),
            Owner::Boundary(b) => Some(Op::Boundary(b.clone()).into()),
            Owner::Add(a) => Some(Op::Add(a.clone()).into()),
            Owner::Sub(s) => Some(Op::Sub(s.clone()).into()),
            Owner::Mul(m) => Some(Op::Mul(m.clone()).into()),
            Owner::If(i) => Some(Op::If(i.clone()).into()),
            Owner::For(f) => Some(Op::For(f.clone()).into()),
            Owner::Call(c) => Some(Op::Call(c.clone()).into()),
            Owner::Fold(f) => Some(Op::Fold(f.clone()).into()),
            Owner::Vector(v) => Some(Op::Vector(v.clone()).into()),
            Owner::Matrix(m) => Some(Op::Matrix(m.clone()).into()),
            Owner::Accessor(a) => Some(Op::Accessor(a.clone()).into()),
            Owner::None => None,
        }
    }
    pub fn as_node(self) -> Link<Node> {
        match self.borrow().deref() {
            Owner::Function(f) => Node::Function(f.clone()).into(),
            Owner::Evaluator(e) => Node::Evaluator(e.clone()).into(),
            Owner::Enf(e) => Node::Enf(e.clone()).into(),
            Owner::Boundary(b) => Node::Boundary(b.clone()).into(),
            Owner::Add(a) => Node::Add(a.clone()).into(),
            Owner::Sub(s) => Node::Sub(s.clone()).into(),
            Owner::Mul(m) => Node::Mul(m.clone()).into(),
            Owner::If(i) => Node::If(i.clone()).into(),
            Owner::For(f) => Node::For(f.clone()).into(),
            Owner::Call(c) => Node::Call(c.clone()).into(),
            Owner::Fold(f) => Node::Fold(f.clone()).into(),
            Owner::Vector(v) => Node::Vector(v.clone()).into(),
            Owner::Matrix(m) => Node::Matrix(m.clone()).into(),
            Owner::Accessor(a) => Node::Accessor(a.clone()).into(),
            Owner::None => Node::None.into(),
        }
    }
}
