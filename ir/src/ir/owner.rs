use crate::ir::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, If, Matrix, Mul, Op, Sub,
    Vector,
};

use super::{Link, Node};

/// The nodes that can own Op nodes
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Owner {
    Function(Function),
    Evaluator(Evaluator),
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
    #[default]
    None,
}

impl Owner {
    pub fn as_function(self) -> Option<Function> {
        match self {
            Owner::Function(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_evaluator(self) -> Option<Evaluator> {
        match self {
            Owner::Evaluator(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_enf(self) -> Option<Enf> {
        match self {
            Owner::Enf(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_boundary(self) -> Option<Boundary> {
        match self {
            Owner::Boundary(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_add(self) -> Option<Add> {
        match self {
            Owner::Add(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_sub(self) -> Option<Sub> {
        match self {
            Owner::Sub(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_mul(self) -> Option<Mul> {
        match self {
            Owner::Mul(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_if(self) -> Option<If> {
        match self {
            Owner::If(i) => Some(i),
            _ => None,
        }
    }
    pub fn as_for(self) -> Option<For> {
        match self {
            Owner::For(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_call(self) -> Option<Call> {
        match self {
            Owner::Call(c) => Some(c),
            _ => None,
        }
    }
    pub fn as_fold(self) -> Option<Fold> {
        match self {
            Owner::Fold(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_vector(self) -> Option<Vector> {
        match self {
            Owner::Vector(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_matrix(self) -> Option<Matrix> {
        match self {
            Owner::Matrix(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_index_access(self) -> Option<Accessor> {
        match self {
            Owner::Accessor(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_op(self) -> Option<Op> {
        match self {
            Owner::Function(f) => None,
            Owner::Evaluator(e) => None,
            Owner::Enf(e) => Some(Op::Enf(e)),
            Owner::Boundary(b) => Some(Op::Boundary(b)),
            Owner::Add(a) => Some(Op::Add(a)),
            Owner::Sub(s) => Some(Op::Sub(s)),
            Owner::Mul(m) => Some(Op::Mul(m)),
            Owner::If(i) => Some(Op::If(i)),
            Owner::For(f) => Some(Op::For(f)),
            Owner::Call(c) => Some(Op::Call(c)),
            Owner::Fold(f) => Some(Op::Fold(f)),
            Owner::Vector(v) => Some(Op::Vector(v)),
            Owner::Matrix(m) => Some(Op::Matrix(m)),
            Owner::Accessor(a) => Some(Op::Accessor(a)),
            Owner::None => None,
        }
    }
    pub fn as_node(self) -> Node {
        match self {
            Owner::Function(f) => Node::Function(f),
            Owner::Evaluator(e) => Node::Evaluator(e),
            Owner::Enf(e) => Node::Enf(e),
            Owner::Boundary(b) => Node::Boundary(b),
            Owner::Add(a) => Node::Add(a),
            Owner::Sub(s) => Node::Sub(s),
            Owner::Mul(m) => Node::Mul(m),
            Owner::If(i) => Node::If(i),
            Owner::For(f) => Node::For(f),
            Owner::Call(c) => Node::Call(c),
            Owner::Fold(f) => Node::Fold(f),
            Owner::Vector(v) => Node::Vector(v),
            Owner::Matrix(m) => Node::Matrix(m),
            Owner::Accessor(a) => Node::Accessor(a),
            Owner::None => Node::None,
        }
    }
}

impl Link<Owner> {
    pub fn as_function(self) -> Option<Link<Function>> {
        self.borrow().clone().as_function().map(|f| f.into())
    }
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
    pub fn as_op(self) -> Option<Link<Op>> {
        self.borrow().clone().as_op().map(|o| o.into())
    }
    pub fn as_node(self) -> Link<Node> {
        self.borrow().clone().as_node().into()
    }
}
