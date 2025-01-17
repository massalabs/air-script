use crate::ir::{
    get_inner, get_inner_mut, Accessor, Add, BackLink, Boundary, Call, Child, Enf, Fold, For, If,
    Link, Matrix, Mul, Node, Owner, Parameter, Parent, Sub, Value, Vector,
};

use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

/// The combined Operators and Leaves of the MIR Graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Op {
    Enf(Enf),
    Boundary(Boundary),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    If(If),
    For(For),
    Call(Call),
    Fold(Fold),
    Vector(Vector),
    Matrix(Matrix),
    Accessor(Accessor),
    Parameter(Parameter),
    Value(Value),
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
            Op::Parameter(_) => Link::default(),
            Op::Value(_) => Link::default(),
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
            Op::None => Default::default(),
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
    pub fn as_node(&self) -> Link<Node> {
        let back: BackLink<Op> = self.clone().into();
        match self.borrow().deref() {
            Op::Enf(_) => Node::Enf(back),
            Op::Boundary(_) => Node::Boundary(back),
            Op::Add(_) => Node::Add(back),
            Op::Sub(_) => Node::Sub(back),
            Op::Mul(_) => Node::Mul(back),
            Op::If(_) => Node::If(back),
            Op::For(_) => Node::For(back),
            Op::Call(_) => Node::Call(back),
            Op::Fold(_) => Node::Fold(back),
            Op::Vector(_) => Node::Vector(back),
            Op::Matrix(_) => Node::Matrix(back),
            Op::Accessor(_) => Node::Accessor(back),
            Op::Parameter(_) => Node::Parameter(back),
            Op::Value(_) => Node::Value(back),
            Op::None => Node::None,
        }
        .into()
    }
    pub fn as_enf(&self) -> Option<Ref<Enf>> {
        get_inner(self.borrow(), |op| match op {
            Op::Enf(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_enf_mut(&self) -> Option<RefMut<Enf>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Enf(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_boundary(&self) -> Option<Ref<Boundary>> {
        get_inner(self.borrow(), |op| match op {
            Op::Boundary(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_boundary_mut(&self) -> Option<RefMut<Boundary>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Boundary(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_add(&self) -> Option<Ref<Add>> {
        get_inner(self.borrow(), |op| match op {
            Op::Add(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_add_mut(&self) -> Option<RefMut<Add>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Add(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_sub(&self) -> Option<Ref<Sub>> {
        get_inner(self.borrow(), |op| match op {
            Op::Sub(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_sub_mut(&self) -> Option<RefMut<Sub>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Sub(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_mul(&self) -> Option<Ref<Mul>> {
        get_inner(self.borrow(), |op| match op {
            Op::Mul(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_mul_mut(&self) -> Option<RefMut<Mul>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Mul(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_if(&self) -> Option<Ref<If>> {
        get_inner(self.borrow(), |op| match op {
            Op::If(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_if_mut(&self) -> Option<RefMut<If>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::If(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_for(&self) -> Option<Ref<For>> {
        get_inner(self.borrow(), |op| match op {
            Op::For(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_for_mut(&self) -> Option<RefMut<For>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::For(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_call(&self) -> Option<Ref<Call>> {
        get_inner(self.borrow(), |op| match op {
            Op::Call(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_call_mut(&self) -> Option<RefMut<Call>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Call(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_fold(&self) -> Option<Ref<Fold>> {
        get_inner(self.borrow(), |op| match op {
            Op::Fold(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_fold_mut(&self) -> Option<RefMut<Fold>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Fold(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_vector(&self) -> Option<Ref<Vector>> {
        get_inner(self.borrow(), |op| match op {
            Op::Vector(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_vector_mut(&self) -> Option<RefMut<Vector>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Vector(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_matrix(&self) -> Option<Ref<Matrix>> {
        get_inner(self.borrow(), |op| match op {
            Op::Matrix(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_matrix_mut(&self) -> Option<RefMut<Matrix>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Matrix(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_accessor(&self) -> Option<Ref<Accessor>> {
        get_inner(self.borrow(), |op| match op {
            Op::Accessor(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_accessor_mut(&self) -> Option<RefMut<Accessor>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Accessor(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_parameter(&self) -> Option<Ref<Parameter>> {
        get_inner(self.borrow(), |op| match op {
            Op::Parameter(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_parameter_mut(&self) -> Option<RefMut<Parameter>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Parameter(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_value(&self) -> Option<Ref<Value>> {
        get_inner(self.borrow(), |op| match op {
            Op::Value(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_value_mut(&self) -> Option<RefMut<Value>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Value(inner) => Some(inner),
            _ => None,
        })
    }
}
