mod evaluator;
mod function;
pub use evaluator::Evaluator;
pub use function::Function;

use super::MirType;
use crate::ir_fix::{BackLink, Child, Link, Op, Owner};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    parents: Vec<BackLink<Owner>>,
    pub position: usize,
    pub ty: MirType,
}

impl Parameter {
    pub fn create(position: usize, ty: MirType) -> Link<Op> {
        Op::Parameter(Self {
            parents: Vec::default(),
            position,
            ty,
        })
        .into()
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
