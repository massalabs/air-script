use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent, Singleton};
use miden_diagnostics::{SourceSpan, Spanned};

/// A MIR operation to represent a vector of MIR ops of a given size
///
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Vector {
    pub parents: Vec<BackLink<Owner>>,
    pub size: usize,
    pub elements: Link<Vec<Link<Op>>>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
}

impl Vector {
    pub fn create(elements: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let size = elements.len();
        let res = Link::new(Op::Vector(Self {
            size,
            elements: Link::new(elements),
            span,
            ..Default::default()
        }));
        let node = Node::Vector(BackLink::from(res.clone()));
        res.as_vector_mut().unwrap()._node = Singleton::from(node);
        let owner = Owner::Vector(BackLink::from(res.clone()));
        res.as_vector_mut().unwrap()._owner = Singleton::from(owner);
        res
    }
    pub fn get_element(&self, index: usize) -> Option<Link<Op>> {
        self.elements.borrow().get(index).cloned()
    }
}

impl Parent for Vector {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements.clone()
    }
}

impl Child for Vector {
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
