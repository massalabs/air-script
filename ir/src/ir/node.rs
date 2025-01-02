use crate::ir::{
    Accessor, Add, BackLink, Boundary, Call, Child, Enf, Evaluator, Fold, For, Function, If,
    Matrix, Mul, Op, Parameter, Sub, Value, Vector,
};
use std::ops::Deref;

use super::{Leaf, Link, Owner, Parent, Root};

/// All the nodes that can be in the MIR Graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Node {
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
    Parameter(Link<Parameter>),
    Value(Link<Value>),
    #[default]
    None,
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
            Node::Parameter(p) => Link::default(),
            Node::Value(v) => Link::default(),
            Node::None => Link::default(),
        }
    }
}

impl Child for Node {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        match self {
            Node::Function(f) => Vec::default(),
            Node::Evaluator(e) => Vec::default(),
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
            Node::Function(f) => (),
            Node::Evaluator(e) => (),
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
            Node::Function(f) => (),
            Node::Evaluator(e) => (),
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
    pub fn as_function(self) -> Option<Link<Function>> {
        match self.borrow().deref() {
            Node::Function(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_evaluator(self) -> Option<Link<Evaluator>> {
        match self.borrow().deref() {
            Node::Evaluator(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn as_enf(self) -> Option<Link<Enf>> {
        match self.borrow().deref() {
            Node::Enf(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn as_boundary(self) -> Option<Link<Boundary>> {
        match self.borrow().deref() {
            Node::Boundary(b) => Some(b.clone()),
            _ => None,
        }
    }
    pub fn as_add(self) -> Option<Link<Add>> {
        match self.borrow().deref() {
            Node::Add(a) => Some(a.clone()),
            _ => None,
        }
    }
    pub fn as_sub(self) -> Option<Link<Sub>> {
        match self.borrow().deref() {
            Node::Sub(s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn as_mul(self) -> Option<Link<Mul>> {
        match self.borrow().deref() {
            Node::Mul(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_if(self) -> Option<Link<If>> {
        match self.borrow().deref() {
            Node::If(i) => Some(i.clone()),
            _ => None,
        }
    }
    pub fn as_for(self) -> Option<Link<For>> {
        match self.borrow().deref() {
            Node::For(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_call(self) -> Option<Link<Call>> {
        match self.borrow().deref() {
            Node::Call(c) => Some(c.clone()),
            _ => None,
        }
    }
    pub fn as_fold(self) -> Option<Link<Fold>> {
        match self.borrow().deref() {
            Node::Fold(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_vector(self) -> Option<Link<Vector>> {
        match self.borrow().deref() {
            Node::Vector(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub fn as_matrix(self) -> Option<Link<Matrix>> {
        match self.borrow().deref() {
            Node::Matrix(m) => Some(m.clone()),
            _ => None,
        }
    }
    pub fn as_accessor(self) -> Option<Link<Accessor>> {
        match self.borrow().deref() {
            Node::Accessor(a) => Some(a.clone()),
            _ => None,
        }
    }
    pub fn as_parameter(self) -> Option<Link<Parameter>> {
        match self.borrow().deref() {
            Node::Parameter(p) => Some(p.clone()),
            _ => None,
        }
    }
    pub fn as_value(self) -> Option<Link<Value>> {
        match self.borrow().deref() {
            Node::Value(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub fn as_op(self) -> Option<Link<Op>> {
        match self.borrow().deref() {
            Node::Function(f) => None,
            Node::Evaluator(e) => None,
            Node::Enf(e) => Some(Op::Enf(e.clone()).into()),
            Node::Boundary(b) => Some(Op::Boundary(b.clone()).into()),
            Node::Add(a) => Some(Op::Add(a.clone()).into()),
            Node::Sub(s) => Some(Op::Sub(s.clone()).into()),
            Node::Mul(m) => Some(Op::Mul(m.clone()).into()),
            Node::If(i) => Some(Op::If(i.clone()).into()),
            Node::For(f) => Some(Op::For(f.clone()).into()),
            Node::Call(c) => Some(Op::Call(c.clone()).into()),
            Node::Fold(f) => Some(Op::Fold(f.clone()).into()),
            Node::Vector(v) => Some(Op::Vector(v.clone()).into()),
            Node::Matrix(m) => Some(Op::Matrix(m.clone()).into()),
            Node::Accessor(a) => Some(Op::Accessor(a.clone()).into()),
            Node::Parameter(p) => Some(Op::Parameter(p.clone()).into()),
            Node::Value(v) => Some(Op::Value(v.clone()).into()),
            Node::None => None,
        }
    }
    pub fn as_owner(self) -> Option<Link<Owner>> {
        match self.borrow().deref() {
            Node::Function(f) => Some(Owner::Function(f.clone()).into()),
            Node::Evaluator(e) => Some(Owner::Evaluator(e.clone()).into()),
            Node::Enf(e) => Some(Owner::Enf(e.clone()).into()),
            Node::Boundary(b) => Some(Owner::Boundary(b.clone()).into()),
            Node::Add(a) => Some(Owner::Add(a.clone()).into()),
            Node::Sub(s) => Some(Owner::Sub(s.clone()).into()),
            Node::Mul(m) => Some(Owner::Mul(m.clone()).into()),
            Node::If(i) => Some(Owner::If(i.clone()).into()),
            Node::For(f) => Some(Owner::For(f.clone()).into()),
            Node::Call(c) => Some(Owner::Call(c.clone()).into()),
            Node::Fold(f) => Some(Owner::Fold(f.clone()).into()),
            Node::Vector(v) => Some(Owner::Vector(v.clone()).into()),
            Node::Matrix(m) => Some(Owner::Matrix(m.clone()).into()),
            Node::Accessor(a) => Some(Owner::Accessor(a.clone()).into()),
            Node::Parameter(p) => None,
            Node::Value(v) => None,
            Node::None => None,
        }
    }
    pub fn as_leaf(self) -> Option<Link<Leaf>> {
        match self.borrow().deref() {
            Node::Value(v) => Some(Leaf::Value(v.clone()).into()),
            Node::Parameter(p) => Some(Leaf::Parameter(p.clone()).into()),
            _ => None,
        }
    }
    pub fn as_root(self) -> Option<Link<Root>> {
        match self.borrow().deref() {
            Node::Function(f) => Some(Root::Function(f.clone()).into()),
            Node::Evaluator(e) => Some(Root::Evaluator(e.clone()).into()),
            _ => None,
        }
    }
}
