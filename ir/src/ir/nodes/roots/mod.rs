mod evaluator;
mod function;

pub use evaluator::Evaluator;
pub use function::Function;

use super::value::MirType;
use crate::ir::{Leaf, Link, Op};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    pub position: usize,
    pub ty: MirType,
}

impl Parameter {
    pub fn new(position: usize, ty: MirType) -> Self {
        Self { position, ty }
    }
    pub fn as_leaf(self) -> Leaf {
        Leaf::Parameter(self)
    }
    pub fn as_op(self) -> Op {
        Op::Parameter(self)
    }
}

impl Link<Parameter> {
    pub fn as_leaf(self) -> Link<Leaf> {
        Link::new(Leaf::Parameter(self.borrow().clone()))
    }
    pub fn as_op(self) -> Link<Op> {
        Link::new(Op::Parameter(self.borrow().clone()))
    }
}
