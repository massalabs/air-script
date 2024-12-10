use crate::ir3::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, If, Matrix, Mul, Sub,
    Vector,
};

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
    IndexAccess(Accessor),
    #[default]
    None,
}
