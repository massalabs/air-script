use std::marker::PhantomData;

use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent, Vector};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Matrix {
    pub parent: BackLink<Owner>,
    pub size: usize,
    pub elements: Link<Vec<Link<Vector>>>,
}

impl Matrix {
    pub fn create(elements: Vec<Link<Vector>>) -> Link<Self> {
        let size = elements.len();
        Self {
            size,
            elements: Link::new(elements),
            ..Default::default()
        }
        .into()
    }
}

impl Link<Matrix> {
    pub fn as_op(self) -> Link<Op> {
        Op::Matrix(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Matrix(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Matrix(self).into()
    }
}

impl Parent for Matrix {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements
            .clone()
            .borrow_mut()
            .iter()
            .map(|element| element.clone().as_op())
            .collect::<Vec<_>>()
            .into()
    }
}

impl Child for Matrix {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct MatrixBuilder<State> {
    _state: PhantomData<State>,
    parent: BackLink<Owner>,
    size: Option<usize>,
    elements: Vec<Link<Vector>>,
}

impl Builder for Matrix {
    type BuilderEmpty = MatrixBuilder<(BackLink<Owner>, NotSet, Vec<Link<Vector>>)>;
    type BuilderFull = MatrixBuilder<(BackLink<Owner>, NotSet, Vec<Link<Vector>>)>;
    fn builder() -> Self::BuilderEmpty {
        MatrixBuilder::default()
    }
}

impl Default for MatrixBuilder<(BackLink<Owner>, NotSet, Vec<Link<Vector>>)> {
    fn default() -> Self {
        Self {
            _state: PhantomData,
            parent: BackLink::default(),
            size: None,
            elements: Vec::new(),
        }
    }
}

impl MatrixBuilder<(BackLink<Owner>, NotSet, Vec<Link<Vector>>)> {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn size(
        mut self,
        size: usize,
    ) -> MatrixBuilder<(BackLink<Owner>, usize, Vec<Link<Vector>>)> {
        self.size = Some(size);
        unsafe { std::mem::transmute(self) }
    }
    pub fn elements(mut self, elements: Link<Vector>) -> Self {
        self.elements.push(elements);
        self
    }
}

impl MatrixBuilder<(BackLink<Owner>, usize, Vec<Link<Vector>>)> {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }
    pub fn elements(mut self, elements: Link<Vector>) -> Self {
        self.elements.push(elements);
        self
    }
    pub fn build(self) -> Link<Matrix> {
        Matrix {
            parent: self.parent,
            size: self.size.expect("size not set"),
            elements: Link::new(self.elements),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {

    use std::ops::Deref;

    use super::*;

    #[test]
    fn test_matrix_builder() {
        let parent = Link::new(Owner::default());
        let a = Link::new(Vector::default());
        let b = Link::new(Vector::default());
        let matrix = MatrixBuilder::default()
            .parent(parent.clone())
            .size(2)
            .elements(a.clone())
            .elements(b.clone())
            .build();
        assert_eq!(
            matrix.borrow().deref(),
            &Matrix {
                parent: parent.into(),
                size: 2,
                elements: Link::new(vec![a, b]),
            }
        );
    }
}
