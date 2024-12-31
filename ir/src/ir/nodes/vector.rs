use std::marker::PhantomData;

use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Vector {
    pub parent: BackLink<Owner>,
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
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct VectorBuilder<State> {
    _state: PhantomData<State>,
    parent: BackLink<Owner>,
    size: Option<usize>,
    elements: Link<Vec<Link<Op>>>,
}

impl Builder for Vector {
    type BuilderEmpty = VectorBuilder<(BackLink<Owner>, NotSet, Link<Vec<Link<Op>>>)>;
    type BuilderFull = VectorBuilder<(BackLink<Owner>, NotSet, Link<Vec<Link<Op>>>)>;
    fn builder() -> Self::BuilderEmpty {
        VectorBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: PhantomData,
            parent: self.parent,
            size: Some(self.size),
            elements: self.elements,
        }
    }
}

impl Default for VectorBuilder<(BackLink<Owner>, NotSet, Link<Vec<Link<Op>>>)> {
    fn default() -> Self {
        Self {
            _state: PhantomData,
            parent: BackLink::default(),
            size: None,
            elements: Vec::new().into(),
        }
    }
}

impl VectorBuilder<(BackLink<Owner>, NotSet, Link<Vec<Link<Op>>>)> {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn size(
        mut self,
        size: usize,
    ) -> VectorBuilder<(BackLink<Owner>, usize, Link<Vec<Link<Op>>>)> {
        self.size = Some(size);
        unsafe { std::mem::transmute(self) }
    }
    pub fn elements(self, elements: Link<Op>) -> Self {
        self.elements.borrow_mut().push(elements);
        self
    }
}

impl VectorBuilder<(BackLink<Owner>, usize, Link<Vec<Link<Op>>>)> {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }
    pub fn elements(self, elements: Link<Op>) -> Self {
        self.elements.borrow_mut().push(elements);
        self
    }
    pub fn build(self) -> Link<Vector> {
        Vector {
            parent: self.parent,
            size: self.size.expect("size not set"),
            elements: self.elements,
        }
        .into()
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
            .parent(parent.clone())
            .size(2)
            .elements(a.clone())
            .elements(b.clone())
            .build();
        assert_eq!(
            vector.borrow().deref(),
            &Vector {
                parent: parent.into(),
                size: 2,
                elements: vec![a, b].into(),
            }
        );
    }
}
