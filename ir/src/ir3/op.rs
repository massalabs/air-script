use std::ops::Deref;

use crate::ir3::{
    Accessor, Add, Boundary, Call, Enf, Fold, For, If, Link, Matrix, Mul, Owner, Parameter, Sub,
    Value, Vector,
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

impl Op {
    pub fn as_enf(self) -> Option<Enf> {
        match self {
            Op::Enf(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_boundary(self) -> Option<Boundary> {
        match self {
            Op::Boundary(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_add(self) -> Option<Add> {
        match self {
            Op::Add(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_sub(self) -> Option<Sub> {
        match self {
            Op::Sub(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_mul(self) -> Option<Mul> {
        match self {
            Op::Mul(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_if(self) -> Option<If> {
        match self {
            Op::If(i) => Some(i),
            _ => None,
        }
    }
    pub fn as_for(self) -> Option<For> {
        match self {
            Op::For(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_call(self) -> Option<Call> {
        match self {
            Op::Call(c) => Some(c),
            _ => None,
        }
    }
    pub fn as_fold(self) -> Option<Fold> {
        match self {
            Op::Fold(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_vector(self) -> Option<Vector> {
        match self {
            Op::Vector(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_matrix(self) -> Option<Matrix> {
        match self {
            Op::Matrix(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_index_access(self) -> Option<Accessor> {
        match self {
            Op::Accessor(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_parameter(self) -> Option<Parameter> {
        match self {
            Op::Parameter(p) => Some(p),
            _ => None,
        }
    }
    pub fn as_value(self) -> Option<Value> {
        match self {
            Op::Value(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_owner(self) -> Option<Owner> {
        match self {
            Op::Enf(e) => Some(Owner::Enf(e)),
            Op::Boundary(b) => Some(Owner::Boundary(b)),
            Op::Add(a) => Some(Owner::Add(a)),
            Op::Sub(s) => Some(Owner::Sub(s)),
            Op::Mul(m) => Some(Owner::Mul(m)),
            Op::If(i) => Some(Owner::If(i)),
            Op::For(f) => Some(Owner::For(f)),
            Op::Call(c) => Some(Owner::Call(c)),
            Op::Fold(f) => Some(Owner::Fold(f)),
            Op::Vector(v) => Some(Owner::Vector(v)),
            Op::Matrix(m) => Some(Owner::Matrix(m)),
            Op::Accessor(a) => Some(Owner::Accessor(a)),
            Op::Parameter(p) => None,
            Op::Value(v) => None,
            Op::None => None,
        }
    }
}

impl Link<Op> {
    pub fn as_enf(self) -> Option<Link<Enf>> {
        self.borrow().clone().as_enf().map(|e| e.into())
    }
    pub fn as_boundary(self) -> Option<Link<Boundary>> {
        self.borrow().clone().as_boundary().map(|b| b.into())
    }
    pub fn as_add(self) -> Option<Link<Add>> {
        self.borrow().clone().as_add().map(|a| a.into())
    }
    pub fn as_sub(self) -> Option<Link<Sub>> {
        self.borrow().clone().as_sub().map(|s| s.into())
    }
    pub fn as_mul(self) -> Option<Link<Mul>> {
        self.borrow().clone().as_mul().map(|m| m.into())
    }
    pub fn as_if(self) -> Option<Link<If>> {
        self.borrow().clone().as_if().map(|i| i.into())
    }
    pub fn as_for(self) -> Option<Link<For>> {
        self.borrow().clone().as_for().map(|f| f.into())
    }
    pub fn as_call(self) -> Option<Link<Call>> {
        self.borrow().clone().as_call().map(|c| c.into())
    }
    pub fn as_fold(self) -> Option<Link<Fold>> {
        self.borrow().clone().as_fold().map(|f| f.into())
    }
    pub fn as_vector(self) -> Option<Link<Vector>> {
        self.borrow().clone().as_vector().map(|v| v.into())
    }
    pub fn as_matrix(self) -> Option<Link<Matrix>> {
        self.borrow().clone().as_matrix().map(|m| m.into())
    }
    pub fn as_index_access(self) -> Option<Link<Accessor>> {
        self.borrow().clone().as_index_access().map(|a| a.into())
    }
    pub fn as_parameter(self) -> Option<Link<Parameter>> {
        self.borrow().clone().as_parameter().map(|p| p.into())
    }
    pub fn as_value(self) -> Option<Link<Value>> {
        self.borrow().clone().as_value().map(|v| v.into())
    }
    pub fn as_owner(self) -> Option<Link<Owner>> {
        self.borrow().clone().as_owner().map(|o| o.into())
    }
}
