mod evaluator;
mod function;

pub use evaluator::Evaluator;
pub use function::Function;

use crate::ir3::{Leaf, Link, Op};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    position: usize,
}

impl Parameter {
    pub fn new(position: usize) -> Self {
        Self { position }
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
