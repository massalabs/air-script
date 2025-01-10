use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Vector {
    pub parents: Vec<BackLink<Owner>>,
    pub size: usize,
    pub elements: Link<Vec<Link<Op>>>,
}

impl Vector {
    pub fn create(elements: Vec<Link<Op>>) -> Link<Self> {
        let size = elements.len();
        Self {
            size,
            elements: Link::new(elements),
            ..Default::default()
        }
        .into()
    }
}

impl Link<Vector> {
    pub fn as_op(self) -> Link<Op> {
        Op::Vector(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Vector(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Vector(self).into()
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

#[cfg(test)]
mod tests {

    use std::ops::Deref;

    use super::*;

    #[test]
    fn test_vector_builder() {
        let parent = Link::new(Owner::default());
        let a = Link::new(Op::default());
        let b = Link::new(Op::default());
        let vector = VectorBuilder::default()
            .parents(parent.clone())
            .size(2)
            .elements(a.clone())
            .elements(b.clone())
            .build();
        assert_eq!(
            vector.borrow().deref(),
            &Vector {
                parents: vec![parent.into()],
                size: 2,
                elements: vec![a, b].into(),
            }
        );
    }
}
