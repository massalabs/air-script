mod evaluator;
mod function;
pub use evaluator::Evaluator;
pub use function::Function;

use super::MirType;
use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner};

#[derive(Builder, Default, Clone, PartialEq, Eq, Debug, Hash)]
#[enum_wrapper(Op)]
pub struct Parameter {
    parents: Vec<BackLink<Owner>>,
    pub ref_node: Link<Node>,
    pub position: usize,
    pub ty: MirType,
}

impl Parameter {
    pub fn create(position: usize, ty: MirType) -> Link<Op> {
        Op::Parameter(Self {
            parents: Vec::default(),
            ref_node: Node::None.into(),
            position,
            ty,
        })
        .into()
    }

    pub fn set_ref_node(&mut self, ref_node: Link<Node>) {
        self.ref_node = ref_node;
    }
}

impl Child for Parameter {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
    }
}
