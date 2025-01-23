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
    pub ref_node: Option<usize>,
    pub position: usize,
    pub ty: MirType,
    pub _node: Option<Link<Node>>,
}

impl Parameter {
    pub fn create(position: usize, ty: MirType) -> Link<Op> {
        //println!("Create param : p:{:?} t:{:?} ", position, ty);
        Op::Parameter(Self {
            parents: Vec::default(),
            ref_node: None,
            position,
            ty,
            _node: None,
        })
        .into()
    }

    pub fn set_ref_node_ptr(&mut self, ref_node_ptr: usize) {
        //println!("set_ref_node_ptr for param: {:?}", self);
        //println!("  ref_node_ptr: {:?}", ref_node_ptr);
        self.ref_node = Some(ref_node_ptr);
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
