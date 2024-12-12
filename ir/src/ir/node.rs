use crate::ir::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, If, Matrix, Mul, Op,
    Parameter, Sub, Value, Vector,
};

use super::{Leaf, Link, Owner, Root};

/// All the nodes that can be in the MIR Graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Node {
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
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}

impl Node {
    pub fn as_function(self) -> Option<Function> {
        match self {
            Node::Function(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_evaluator(self) -> Option<Evaluator> {
        match self {
            Node::Evaluator(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_enf(self) -> Option<Enf> {
        match self {
            Node::Enf(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_boundary(self) -> Option<Boundary> {
        match self {
            Node::Boundary(b) => Some(b),
            _ => None,
        }
    }
    pub fn as_add(self) -> Option<Add> {
        match self {
            Node::Add(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_sub(self) -> Option<Sub> {
        match self {
            Node::Sub(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_mul(self) -> Option<Mul> {
        match self {
            Node::Mul(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_if(self) -> Option<If> {
        match self {
            Node::If(i) => Some(i),
            _ => None,
        }
    }
    pub fn as_for(self) -> Option<For> {
        match self {
            Node::For(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_call(self) -> Option<Call> {
        match self {
            Node::Call(c) => Some(c),
            _ => None,
        }
    }
    pub fn as_fold(self) -> Option<Fold> {
        match self {
            Node::Fold(f) => Some(f),
            _ => None,
        }
    }
    pub fn as_vector(self) -> Option<Vector> {
        match self {
            Node::Vector(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_matrix(self) -> Option<Matrix> {
        match self {
            Node::Matrix(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_index_access(self) -> Option<Accessor> {
        match self {
            Node::Accessor(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_parameter(self) -> Option<Parameter> {
        match self {
            Node::Parameter(p) => Some(p),
            _ => None,
        }
    }
    pub fn as_value(self) -> Option<Value> {
        match self {
            Node::Value(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_op(self) -> Option<Op> {
        match self {
            Node::Function(f) => None,
            Node::Evaluator(e) => None,
            Node::Enf(e) => Some(Op::Enf(e)),
            Node::Boundary(b) => Some(Op::Boundary(b)),
            Node::Add(a) => Some(Op::Add(a)),
            Node::Sub(s) => Some(Op::Sub(s)),
            Node::Mul(m) => Some(Op::Mul(m)),
            Node::If(i) => Some(Op::If(i)),
            Node::For(f) => Some(Op::For(f)),
            Node::Call(c) => Some(Op::Call(c)),
            Node::Fold(f) => Some(Op::Fold(f)),
            Node::Vector(v) => Some(Op::Vector(v)),
            Node::Matrix(m) => Some(Op::Matrix(m)),
            Node::Accessor(a) => Some(Op::Accessor(a)),
            Node::Parameter(p) => Some(Op::Parameter(p)),
            Node::Value(v) => Some(Op::Value(v)),
            Node::None => None,
        }
    }
    pub fn as_owner(self) -> Option<Owner> {
        match self {
            Node::Function(f) => Some(Owner::Function(f)),
            Node::Evaluator(e) => Some(Owner::Evaluator(e)),
            Node::Enf(e) => Some(Owner::Enf(e)),
            Node::Boundary(b) => Some(Owner::Boundary(b)),
            Node::Add(a) => Some(Owner::Add(a)),
            Node::Sub(s) => Some(Owner::Sub(s)),
            Node::Mul(m) => Some(Owner::Mul(m)),
            Node::If(i) => Some(Owner::If(i)),
            Node::For(f) => Some(Owner::For(f)),
            Node::Call(c) => Some(Owner::Call(c)),
            Node::Fold(f) => Some(Owner::Fold(f)),
            Node::Vector(v) => Some(Owner::Vector(v)),
            Node::Matrix(m) => Some(Owner::Matrix(m)),
            Node::Accessor(a) => Some(Owner::Accessor(a)),
            Node::Parameter(p) => None,
            Node::Value(v) => None,
            Node::None => None,
        }
    }
    fn as_leaf(self) -> Option<Leaf> {
        match self {
            Node::Value(v) => Some(Leaf::Value(v)),
            Node::Parameter(p) => Some(Leaf::Parameter(p)),
            _ => None,
        }
    }
    fn as_root(self) -> Option<Root> {
        match self {
            Node::Function(f) => Some(Root::Function(f)),
            Node::Evaluator(e) => Some(Root::Evaluator(e)),
            _ => None,
        }
    }
}

impl Link<Node> {
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
    pub fn as_parameter(self) -> Option<Link<Parameter>> {
        self.borrow().clone().as_parameter().map(|p| p.into())
    }
    pub fn as_value(self) -> Option<Link<Value>> {
        self.borrow().clone().as_value().map(|v| v.into())
    }
    pub fn as_op(self) -> Option<Link<Op>> {
        self.borrow().clone().as_op().map(|o| o.into())
    }
    pub fn as_owner(self) -> Option<Link<Owner>> {
        self.borrow().clone().as_owner().map(|o| o.into())
    }
    pub fn as_root(self) -> Option<Link<Root>> {
        self.borrow().clone().as_root().map(|r| r.into())
    }
}
