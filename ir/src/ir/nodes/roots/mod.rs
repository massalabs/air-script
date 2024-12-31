mod evaluator;
mod function;

pub use evaluator::Evaluator;
pub use function::Function;

use super::value::MirType;
use crate::ir::{Leaf, Link, Node, Op};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    pub position: usize,
    pub ty: MirType,
}

impl Parameter {
    pub fn create(position: usize, ty: MirType) -> Link<Self> {
        Self { position, ty }.into()
    }
}

impl Link<Parameter> {
    pub fn as_leaf(self) -> Link<Leaf> {
        Leaf::Parameter(self.clone()).into()
    }
    pub fn as_op(self) -> Link<Op> {
        Op::Parameter(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Parameter(self).into()
    }
}
