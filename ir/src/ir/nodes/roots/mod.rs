mod evaluator;
mod function;

pub use evaluator::Evaluator;
pub use function::Function;

use super::value::MirType;
use crate::ir::{BackLink, Child, Leaf, Link, Node, Op, Owner};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    parent: BackLink<Owner>,
    pub position: usize,
    pub ty: MirType,
}

impl Parameter {
    pub fn create(position: usize, ty: MirType) -> Link<Self> {
        Self {
            parent: BackLink::default(),
            position,
            ty,
        }
        .into()
    }
}

impl Child for Parameter {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
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
