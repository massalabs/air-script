use crate::ir3::{
    Accessor, Add, Boundary, Call, Enf, Fold, For, If, Matrix, Mul, Parameter, Sub, Value, Vector,
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
    IndexAccess(Accessor),
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}
