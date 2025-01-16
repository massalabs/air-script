use crate::ir_fix::{
    Accessor, Add, BackLink, Boundary, Call, Child, Enf, Fold, For, If, Link, Matrix, Mul, Owner,
    Parameter, Parent, Sub, Value, Vector,
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
            Op::Parameter(p) => Link::default(),
            Op::Value(v) => Link::default(),
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
