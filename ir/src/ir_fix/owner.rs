use crate::ir_fix::{BackLink, Child, Link, Op, Parent};

/// The nodes that can own Op nodes
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Owner {
    Accessor(BackLink<Op>),
    Boundary(BackLink<Op>),
    Function(BackLink<Op>),
    Evaluator(BackLink<Op>),
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
