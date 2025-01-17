use crate::ir::{
    get_inner, get_inner_mut, Accessor, Add, BackLink, Boundary, Call, Child, Enf, Fold, For, If,
    Matrix, Mul, Op, Parameter, Sub, Value, Vector,
};

use super::{Link, Owner, Parent, Root};
use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

/// All the nodes that can be in the MIR Graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
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
    pub fn as_op(&self) -> Option<Link<Op>> {
        match self.borrow().deref() {
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
            _ => None,
        }
    }
    pub fn as_enf(&self) -> Option<Ref<Enf>> {
        get_inner(self.borrow(), |node| match node {
            Node::Enf(op) => match op.to_link().unwrap().borrow().deref() {
                Op::Enf(inner) => Some(inner),
                _ => None,
            },
            _ => None,
        })
    }
    pub fn as_enf_mut(&self) -> Option<RefMut<Enf>> {
        match self.borrow_mut().deref_mut() {
            Node::Enf(inner) => inner.to_link().unwrap().as_enf_mut(),
            _ => None,
        }
    }
    pub fn as_boundary(&self) -> Option<Ref<Boundary>> {
        match self.borrow().deref() {
            Node::Boundary(inner) => inner.to_link().unwrap().as_boundary(),
            _ => None,
        }
    }
    pub fn as_boundary_mut(&self) -> Option<RefMut<Boundary>> {
        match self.borrow_mut().deref_mut() {
            Node::Boundary(inner) => inner.to_link().unwrap().as_boundary_mut(),
            _ => None,
        }
    }
    pub fn as_add(&self) -> Option<Ref<Add>> {
        match self.borrow().deref() {
            Node::Add(inner) => inner.to_link().unwrap().as_add(),
            _ => None,
        }
    }
    pub fn as_add_mut(&self) -> Option<RefMut<Add>> {
        match self.borrow_mut().deref_mut() {
            Node::Add(inner) => inner.to_link().unwrap().as_add_mut(),
            _ => None,
        }
    }
    pub fn as_sub(&self) -> Option<Ref<Sub>> {
        match self.borrow().deref() {
            Node::Sub(inner) => inner.to_link().unwrap().as_sub(),
            _ => None,
        }
    }
    pub fn as_sub_mut(&self) -> Option<RefMut<Sub>> {
        match self.borrow_mut().deref_mut() {
            Node::Sub(inner) => inner.to_link().unwrap().as_sub_mut(),
            _ => None,
        }
    }
    pub fn as_mul(&self) -> Option<Ref<Mul>> {
        match self.borrow().deref() {
            Node::Mul(inner) => inner.to_link().unwrap().as_mul(),
            _ => None,
        }
    }
    pub fn as_mul_mut(&self) -> Option<RefMut<Mul>> {
        match self.borrow_mut().deref_mut() {
            Node::Mul(inner) => inner.to_link().unwrap().as_mul_mut(),
            _ => None,
        }
    }
    pub fn as_if(&self) -> Option<Ref<If>> {
        match self.borrow().deref() {
            Node::If(inner) => inner.to_link().unwrap().as_if(),
            _ => None,
        }
    }
    pub fn as_if_mut(&self) -> Option<RefMut<If>> {
        match self.borrow_mut().deref_mut() {
            Node::If(inner) => inner.to_link().unwrap().as_if_mut(),
            _ => None,
        }
    }
    pub fn as_for(&self) -> Option<Ref<For>> {
        match self.borrow().deref() {
            Node::For(inner) => inner.to_link().unwrap().as_for(),
            _ => None,
        }
    }
    pub fn as_for_mut(&self) -> Option<RefMut<For>> {
        match self.borrow_mut().deref_mut() {
            Node::For(inner) => inner.to_link().unwrap().as_for_mut(),
            _ => None,
        }
    }
    pub fn as_call(&self) -> Option<Ref<Call>> {
        match self.borrow().deref() {
            Node::Call(inner) => inner.to_link().unwrap().as_call(),
            _ => None,
        }
    }
    pub fn as_call_mut(&self) -> Option<RefMut<Call>> {
        match self.borrow_mut().deref_mut() {
            Node::Call(inner) => inner.to_link().unwrap().as_call_mut(),
            _ => None,
        }
    }
    pub fn as_fold(&self) -> Option<Ref<Fold>> {
        match self.borrow().deref() {
            Node::Fold(inner) => inner.to_link().unwrap().as_fold(),
            _ => None,
        }
    }
    pub fn as_fold_mut(&self) -> Option<RefMut<Fold>> {
        match self.borrow_mut().deref_mut() {
            Node::Fold(inner) => inner.to_link().unwrap().as_fold_mut(),
            _ => None,
        }
    }
    pub fn as_vector(&self) -> Option<Ref<Vector>> {
        match self.borrow().deref() {
            Node::Vector(inner) => inner.to_link().unwrap().as_vector(),
            _ => None,
        }
    }
    pub fn as_vector_mut(&self) -> Option<RefMut<Vector>> {
        match self.borrow_mut().deref_mut() {
            Node::Vector(inner) => inner.to_link().unwrap().as_vector_mut(),
            _ => None,
        }
    }
    pub fn as_matrix(&self) -> Option<Ref<Matrix>> {
        match self.borrow().deref() {
            Node::Matrix(inner) => inner.to_link().unwrap().as_matrix(),
            _ => None,
        }
    }
    pub fn as_matrix_mut(&self) -> Option<RefMut<Matrix>> {
        match self.borrow_mut().deref_mut() {
            Node::Matrix(inner) => inner.to_link().unwrap().as_matrix_mut(),
            _ => None,
        }
    }
    pub fn as_accessor(&self) -> Option<Ref<Accessor>> {
        match self.borrow().deref() {
            Node::Accessor(inner) => inner.to_link().unwrap().as_accessor(),
            _ => None,
        }
    }
    pub fn as_accessor_mut(&self) -> Option<RefMut<Accessor>> {
        match self.borrow_mut().deref_mut() {
            Node::Accessor(inner) => inner.to_link().unwrap().as_accessor_mut(),
            _ => None,
        }
    }
    pub fn as_parameter(&self) -> Option<Ref<Parameter>> {
        match self.borrow().deref() {
            Node::Parameter(inner) => inner.to_link().unwrap().as_parameter(),
            _ => None,
        }
    }
    pub fn as_parameter_mut(&self) -> Option<RefMut<Parameter>> {
        match self.borrow_mut().deref_mut() {
            Node::Parameter(inner) => inner.to_link().unwrap().as_parameter_mut(),
            _ => None,
        }
    }
    pub fn as_value(&self) -> Option<Ref<Value>> {
        match self.borrow().deref() {
            Node::Value(inner) => inner.to_link().unwrap().as_value(),
            _ => None,
        }
    }
    pub fn as_value_mut(&self) -> Option<RefMut<Value>> {
        match self.borrow_mut().deref_mut() {
            Node::Value(inner) => inner.to_link().unwrap().as_value_mut(),
            _ => None,
        }
    }
}
